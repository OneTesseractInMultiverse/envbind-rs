//! Signed integer variable spec.

use crate::binder::Binding;
use crate::environment::Environment;
use crate::error::{BindError, ValidationError, VariableName};

use super::raw::resolve_raw;

type I64Validator = dyn Fn(i64) -> Result<(), ValidationError> + Send + Sync + 'static;

/// Bind one signed integer value.
pub struct IntVar {
    name: VariableName,
    default: Option<i64>,
    allow_empty: bool,
    validate_default: bool,
    sensitive: bool,
    validators: Vec<Box<I64Validator>>,
}

impl IntVar {
    /// Build an integer variable.
    #[must_use]
    pub fn new(name: impl Into<VariableName>) -> Self {
        Self {
            name: name.into(),
            default: None,
            allow_empty: false,
            validate_default: false,
            sensitive: true,
            validators: Vec::new(),
        }
    }

    /// Provide a fallback value for missing or empty input.
    ///
    /// Empty input selects this fallback unless [`Self::allow_empty`] is enabled.
    /// Skips validators unless [`Self::validate_default`] is enabled.
    #[must_use]
    pub fn default(mut self, value: i64) -> Self {
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

    /// Run attached typed validators on the fallback value during binding.
    ///
    /// Disabled by default. Only affects a selected fallback; input parsing and
    /// limits are unchanged. See the [fallback validation policy](crate::fields).
    #[must_use]
    pub fn validate_default(mut self) -> Self {
        self.validate_default = true;
        self
    }

    /// Attach a validation rule.
    #[must_use]
    pub fn validate<F>(mut self, validator: F) -> Self
    where
        F: Fn(i64) -> Result<(), ValidationError> + Send + Sync + 'static,
    {
        self.validators.push(Box::new(validator));
        self
    }
}

impl Binding<i64> for IntVar {
    fn bind<E: Environment>(&self, environment: &E) -> Result<i64, BindError> {
        let name = self.name.as_ref();
        let resolved = resolve_raw(environment, name, self.default.is_some(), self.allow_empty)?;
        let (value, used_default) = match resolved {
            Some(raw) => (parse_i64(name, &raw)?, false),
            None => (
                self.default
                    .ok_or_else(|| BindError::missing(name.to_owned()))?,
                true,
            ),
        };
        if used_default && !self.validate_default {
            return Ok(value);
        }
        for validator in &self.validators {
            validator(value).map_err(|error| {
                BindError::validation_with_sensitivity(name.to_owned(), error, self.sensitive)
            })?;
        }
        Ok(value)
    }
}

fn parse_i64(name: &str, raw: &str) -> Result<i64, BindError> {
    raw.parse::<i64>()
        .map_err(|_| BindError::parse(name.to_owned(), "i64"))
}
