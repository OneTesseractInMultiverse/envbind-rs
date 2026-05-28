//! Unsigned 16-bit integer variable spec.

use crate::binder::Binding;
use crate::environment::Environment;
use crate::error::{BindError, ValidationError, VariableName};

use super::raw::resolve_raw;

type U16Validator = dyn Fn(u16) -> Result<(), ValidationError> + Send + Sync + 'static;

/// Bind one `u16` value.
pub struct U16Var {
    name: VariableName,
    default: Option<u16>,
    allow_empty: bool,
    sensitive: bool,
    validators: Vec<Box<U16Validator>>,
}

impl U16Var {
    /// Build a `u16` variable.
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
    pub fn default(mut self, value: u16) -> Self {
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
        F: Fn(u16) -> Result<(), ValidationError> + Send + Sync + 'static,
    {
        self.validators.push(Box::new(validator));
        self
    }
}

impl Binding<u16> for U16Var {
    fn bind<E: Environment>(&self, environment: &E) -> Result<u16, BindError> {
        let name = self.name.as_ref();
        let resolved = resolve_raw(environment, name, self.default.is_some(), self.allow_empty)?;
        let (value, used_default) = match resolved {
            Some(text) => (parse_u16(name, &text)?, false),
            None => (
                self.default
                    .ok_or_else(|| BindError::missing(name.to_owned()))?,
                true,
            ),
        };
        if used_default {
            return Ok(value);
        }
        validate_u16(
            name,
            value,
            self.sensitive,
            self.validators.iter().map(Box::as_ref),
        )?;
        Ok(value)
    }
}

fn parse_u16(name: &str, raw: &str) -> Result<u16, BindError> {
    raw.parse::<u16>()
        .map_err(|_| BindError::parse(name.to_owned(), "u16"))
}

fn validate_u16<'a, I>(
    name: &str,
    value: u16,
    sensitive: bool,
    validators: I,
) -> Result<(), BindError>
where
    I: IntoIterator<Item = &'a U16Validator>,
{
    for validator in validators {
        validator(value).map_err(|error| {
            BindError::validation_with_sensitivity(name.to_owned(), error, sensitive)
        })?;
    }
    Ok(())
}
