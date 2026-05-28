//! Floating-point variable spec.

use crate::binder::Binding;
use crate::environment::Environment;
use crate::error::{BindError, ValidationError, VariableName};

use super::raw::resolve_raw;

type F64Validator = dyn Fn(f64) -> Result<(), ValidationError> + Send + Sync + 'static;

/// Bind one `f64` value.
pub struct FloatVar {
    name: VariableName,
    default: Option<f64>,
    allow_empty: bool,
    sensitive: bool,
    validators: Vec<Box<F64Validator>>,
}

impl FloatVar {
    /// Build a floating-point variable.
    #[must_use]
    pub fn new(name: impl Into<VariableName>) -> Self {
        Self {
            name: name.into(),
            default: None,
            allow_empty: false,
            sensitive: true,
            validators: Vec::new(),
        }
    }

    /// Provide a fallback value when the variable is missing or empty.
    #[must_use]
    pub fn default(mut self, value: f64) -> Self {
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

    /// Attach a validation rule.
    #[must_use]
    pub fn validate<F>(mut self, validator: F) -> Self
    where
        F: Fn(f64) -> Result<(), ValidationError> + Send + Sync + 'static,
    {
        self.validators.push(Box::new(validator));
        self
    }
}

impl Binding<f64> for FloatVar {
    fn bind<E: Environment>(&self, environment: &E) -> Result<f64, BindError> {
        let name = self.name.as_ref();
        let resolved = resolve_raw(environment, name, self.default.is_some(), self.allow_empty)?;
        let (value, used_default) = match resolved {
            Some(raw) => (parse_f64(name, &raw)?, false),
            None => (
                self.default
                    .ok_or_else(|| BindError::missing(name.to_owned()))?,
                true,
            ),
        };
        if used_default {
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

fn parse_f64(name: &str, raw: &str) -> Result<f64, BindError> {
    raw.parse::<f64>()
        .map_err(|_| BindError::parse(name.to_owned(), "f64"))
}
