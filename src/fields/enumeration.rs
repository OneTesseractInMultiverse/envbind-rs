//! Enum-like variable spec.

use crate::binder::Binding;
use crate::environment::Environment;
use crate::error::{BindError, ValidationError, VariableName};

use super::raw::resolve_raw;

type EnumValidator<T> = dyn Fn(&T) -> Result<(), ValidationError> + Send + Sync + 'static;

/// Bind one enum-like value by matching text labels.
///
/// Rust enums do not expose their variants at runtime, so callers provide the
/// accepted labels and target values explicitly.
pub struct EnumVar<T> {
    name: VariableName,
    options: Vec<(String, T)>,
    default: Option<T>,
    case_sensitive: bool,
    allow_empty: bool,
    sensitive: bool,
    validators: Vec<Box<EnumValidator<T>>>,
}

impl<T> EnumVar<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Build an enum-like variable from accepted labels and values.
    #[must_use]
    pub fn new<I, L>(name: impl Into<VariableName>, options: I) -> Self
    where
        I: IntoIterator<Item = (L, T)>,
        L: Into<String>,
    {
        Self {
            name: name.into(),
            options: options
                .into_iter()
                .map(|(label, value)| (label.into(), value))
                .collect(),
            default: None,
            case_sensitive: false,
            allow_empty: false,
            sensitive: true,
            validators: Vec::new(),
        }
    }

    /// Build an enum-like variable from enum names, enum values, and targets.
    ///
    /// This mirrors Python enum parsing more closely: callers provide the
    /// Rust-facing enum name and its external string value for each variant.
    /// Both labels resolve to the same target value.
    #[must_use]
    pub fn from_names_and_values<I, N, V>(name: impl Into<VariableName>, options: I) -> Self
    where
        I: IntoIterator<Item = (N, V, T)>,
        N: Into<String>,
        V: Into<String>,
    {
        let options = expand_names_and_values(options);
        Self::new(name, options)
    }

    /// Accept one additional label for a target value.
    #[must_use]
    pub fn alias(mut self, label: impl Into<String>, value: T) -> Self {
        self.options.push((label.into(), value));
        self
    }

    /// Provide a fallback value when the variable is missing or empty.
    #[must_use]
    pub fn default(mut self, value: T) -> Self {
        self.default = Some(value);
        self
    }

    /// Match labels with exact case.
    #[must_use]
    pub fn case_sensitive(mut self) -> Self {
        self.case_sensitive = true;
        self
    }

    /// Parse empty strings instead of treating them as missing.
    #[must_use]
    pub fn allow_empty(mut self) -> Self {
        self.allow_empty = true;
        self
    }

    /// Mark the value as safe to include in validation details.
    #[must_use]
    pub fn sensitive(mut self, value: bool) -> Self {
        self.sensitive = value;
        self
    }

    /// Attach a validation rule.
    #[must_use]
    pub fn validate<F>(mut self, validator: F) -> Self
    where
        F: Fn(&T) -> Result<(), ValidationError> + Send + Sync + 'static,
    {
        self.validators.push(Box::new(validator));
        self
    }
}

impl<T> Binding<T> for EnumVar<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn bind<E: Environment>(&self, environment: &E) -> Result<T, BindError> {
        let name = self.name.as_ref();
        let resolved = resolve_raw(environment, name, self.default.is_some(), self.allow_empty)?;
        let (value, used_default) = match resolved {
            Some(raw) => (self.parse(name, &raw)?, false),
            None => (
                self.default
                    .clone()
                    .ok_or_else(|| BindError::missing(name.to_owned()))?,
                true,
            ),
        };
        if used_default {
            return Ok(value);
        }
        for validator in &self.validators {
            validator(&value).map_err(|error| {
                BindError::validation_with_sensitivity(name.to_owned(), error, self.sensitive)
            })?;
        }
        Ok(value)
    }
}

impl<T> EnumVar<T>
where
    T: Clone,
{
    fn parse(&self, name: &str, raw: &str) -> Result<T, BindError> {
        let candidate = raw.trim();
        if self.case_sensitive {
            return self
                .options
                .iter()
                .find(|(label, _)| label == candidate)
                .map(|(_, value)| value.clone())
                .ok_or_else(|| BindError::parse(name.to_owned(), "enum label"));
        }

        let candidate = candidate.to_ascii_lowercase();
        self.options
            .iter()
            .find(|(label, _)| label.to_ascii_lowercase() == candidate)
            .map(|(_, value)| value.clone())
            .ok_or_else(|| BindError::parse(name.to_owned(), "enum label"))
    }
}

fn expand_names_and_values<I, N, V, T>(options: I) -> Vec<(String, T)>
where
    I: IntoIterator<Item = (N, V, T)>,
    N: Into<String>,
    V: Into<String>,
    T: Clone,
{
    options
        .into_iter()
        .flat_map(|(name, value, target)| [(name.into(), target.clone()), (value.into(), target)])
        .collect()
}
