//! Boolean variable spec.

use crate::binder::Binding;
use crate::environment::Environment;
use crate::error::{BindError, ValidationError, VariableName};

use super::raw::resolve_raw;

type BoolValidator = dyn Fn(bool) -> Result<(), ValidationError> + Send + Sync + 'static;

/// Bind one boolean value.
pub struct BoolVar {
    name: VariableName,
    default: Option<bool>,
    allow_empty: bool,
    sensitive: bool,
    validators: Vec<Box<BoolValidator>>,
}

impl BoolVar {
    /// Build a boolean variable.
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

    /// Provide a fallback value when the variable is missing.
    #[must_use]
    pub fn default(mut self, value: bool) -> Self {
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
        F: Fn(bool) -> Result<(), ValidationError> + Send + Sync + 'static,
    {
        self.validators.push(Box::new(validator));
        self
    }
}

impl Binding<bool> for BoolVar {
    fn bind<E: Environment>(&self, environment: &E) -> Result<bool, BindError> {
        let name = self.name.as_ref();
        let raw = resolve_raw(environment, name, self.default.is_some(), self.allow_empty)?;
        let (value, used_default) = match raw {
            Some(value) => (parse_bool_like(name, &value)?, false),
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

fn parse_bool_like(name: &str, raw: &str) -> Result<bool, BindError> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" | "y" | "t" => Ok(true),
        "0" | "false" | "no" | "off" | "n" | "f" => Ok(false),
        _ => Err(BindError::invalid_boolean(name.to_owned())),
    }
}
