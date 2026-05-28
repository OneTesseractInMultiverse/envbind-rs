//! Delimiter-separated list variable spec.

use crate::binder::Binding;
use crate::environment::Environment;
use crate::error::{BindError, ValidationError, VariableName};

use super::raw::resolve_raw;

type ElementParser<T> = dyn Fn(&str) -> Result<T, ValidationError> + Send + Sync + 'static;
type ListValidator<T> = dyn Fn(&[T]) -> Result<(), ValidationError> + Send + Sync + 'static;

/// Default maximum number of list items accepted from one variable.
pub const DEFAULT_MAX_LIST_ITEMS: usize = 1024;

/// Bind one delimiter-separated list value.
pub struct ListVar<T> {
    name: VariableName,
    delimiter: String,
    strip: bool,
    parser: Box<ElementParser<T>>,
    default: Option<Vec<T>>,
    allow_empty: bool,
    sensitive: bool,
    max_items: usize,
    validators: Vec<Box<ListValidator<T>>>,
}

impl ListVar<String> {
    /// Build a string list variable.
    #[must_use]
    pub fn strings(name: impl Into<VariableName>) -> Self {
        Self::new(name, |value| Ok(value.to_owned()))
    }
}

impl ListVar<i64> {
    /// Build an integer list variable.
    #[must_use]
    pub fn integers(name: impl Into<VariableName>) -> Self {
        Self::new(name, |value| {
            value
                .parse::<i64>()
                .map_err(|_| ValidationError::new("list item must be an integer"))
        })
    }
}

impl ListVar<f64> {
    /// Build a floating-point list variable.
    #[must_use]
    pub fn floats(name: impl Into<VariableName>) -> Self {
        Self::new(name, |value| {
            value
                .parse::<f64>()
                .map_err(|_| ValidationError::new("list item must be a float"))
        })
    }
}

impl ListVar<bool> {
    /// Build a boolean list variable.
    #[must_use]
    pub fn booleans(name: impl Into<VariableName>) -> Self {
        Self::new(name, parse_bool_like)
    }
}

impl ListVar<u16> {
    /// Build an unsigned 16-bit integer list variable.
    #[must_use]
    pub fn u16s(name: impl Into<VariableName>) -> Self {
        Self::new(name, |value| {
            value
                .parse::<u16>()
                .map_err(|_| ValidationError::new("list item must be a u16"))
        })
    }
}

impl<T> ListVar<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Build an enum-like list variable from accepted labels and values.
    #[must_use]
    pub fn enumeration<I, L>(name: impl Into<VariableName>, options: I) -> Self
    where
        I: IntoIterator<Item = (L, T)>,
        L: Into<String>,
    {
        let options = options
            .into_iter()
            .map(|(label, value)| (label.into(), value))
            .collect::<Vec<_>>();
        Self::new(name, move |value| {
            parse_enum_like(value, &options, false)
                .ok_or_else(|| ValidationError::new("list item must match an enum label"))
        })
    }

    /// Build an enum-like list variable from enum names, enum values, and targets.
    ///
    /// Both the name and value labels resolve to the same target value.
    #[must_use]
    pub fn enumeration_from_names_and_values<I, N, V>(
        name: impl Into<VariableName>,
        options: I,
    ) -> Self
    where
        I: IntoIterator<Item = (N, V, T)>,
        N: Into<String>,
        V: Into<String>,
    {
        let options = expand_names_and_values(options);
        Self::enumeration(name, options)
    }

    /// Build an enum-like list variable with exact-case label matching.
    #[must_use]
    pub fn case_sensitive_enumeration<I, L>(name: impl Into<VariableName>, options: I) -> Self
    where
        I: IntoIterator<Item = (L, T)>,
        L: Into<String>,
    {
        let options = options
            .into_iter()
            .map(|(label, value)| (label.into(), value))
            .collect::<Vec<_>>();
        Self::new(name, move |value| {
            parse_enum_like(value, &options, true)
                .ok_or_else(|| ValidationError::new("list item must match an enum label"))
        })
    }

    /// Build a list variable with a custom item parser.
    #[must_use]
    pub fn new<F>(name: impl Into<VariableName>, parser: F) -> Self
    where
        F: Fn(&str) -> Result<T, ValidationError> + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            delimiter: ",".to_owned(),
            strip: true,
            parser: Box::new(parser),
            default: None,
            allow_empty: false,
            sensitive: true,
            max_items: DEFAULT_MAX_LIST_ITEMS,
            validators: Vec::new(),
        }
    }

    /// Use a custom delimiter.
    ///
    /// Empty delimiters are reported as validation failures during binding.
    #[must_use]
    pub fn delimiter(mut self, delimiter: impl Into<String>) -> Self {
        self.delimiter = delimiter.into();
        self
    }

    /// Keep surrounding whitespace on list items.
    #[must_use]
    pub fn keep_whitespace(mut self) -> Self {
        self.strip = false;
        self
    }

    /// Provide a fallback value when the variable is missing or empty.
    #[must_use]
    pub fn default(mut self, value: Vec<T>) -> Self {
        self.default = Some(value);
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

    /// Override the maximum accepted item count.
    #[must_use]
    pub fn max_items(mut self, value: usize) -> Self {
        self.max_items = value;
        self
    }

    /// Attach a validation rule.
    #[must_use]
    pub fn validate<F>(mut self, validator: F) -> Self
    where
        F: Fn(&[T]) -> Result<(), ValidationError> + Send + Sync + 'static,
    {
        self.validators.push(Box::new(validator));
        self
    }
}

impl<T> Binding<Vec<T>> for ListVar<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn bind<E: Environment>(&self, environment: &E) -> Result<Vec<T>, BindError> {
        let name = self.name.as_ref();
        let resolved = resolve_raw(environment, name, self.default.is_some(), self.allow_empty)?;
        let (value, used_default) = match resolved {
            Some(raw) => (self.parse_list(name, &raw)?, false),
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

impl<T> ListVar<T> {
    fn parse_list(&self, name: &str, raw: &str) -> Result<Vec<T>, BindError> {
        if self.delimiter.is_empty() {
            return Err(BindError::validation_with_sensitivity(
                name.to_owned(),
                ValidationError::new("delimiter must not be empty"),
                self.sensitive,
            ));
        }

        let mut values = Vec::new();
        for item in raw.split(&self.delimiter) {
            if values.len() >= self.max_items {
                return Err(BindError::validation_with_sensitivity(
                    name.to_owned(),
                    ValidationError::new("list exceeds maximum item count"),
                    self.sensitive,
                ));
            }

            let item = if self.strip { item.trim() } else { item };
            let parsed = (self.parser)(item).map_err(|error| {
                BindError::validation_with_sensitivity(name.to_owned(), error, self.sensitive)
            })?;
            values.push(parsed);
        }

        Ok(values)
    }
}

fn parse_bool_like(raw: &str) -> Result<bool, ValidationError> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" | "y" | "t" => Ok(true),
        "0" | "false" | "no" | "off" | "n" | "f" => Ok(false),
        _ => Err(ValidationError::new("list item must be boolean-like")),
    }
}

fn parse_enum_like<T>(raw: &str, options: &[(String, T)], case_sensitive: bool) -> Option<T>
where
    T: Clone,
{
    let candidate = raw.trim();
    if case_sensitive {
        return options
            .iter()
            .find(|(label, _)| label == candidate)
            .map(|(_, value)| value.clone());
    }

    let candidate = candidate.to_ascii_lowercase();
    options
        .iter()
        .find(|(label, _)| label.to_ascii_lowercase() == candidate)
        .map(|(_, value)| value.clone())
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
