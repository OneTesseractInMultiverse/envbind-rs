//! JSON variable spec.

use serde_json::Value;

use crate::binder::Binding;
use crate::environment::Environment;
use crate::error::{BindError, ValidationError, VariableName};

use super::raw::resolve_raw_with_max_bytes;

type JsonValidator = dyn Fn(&Value) -> Result<(), ValidationError> + Send + Sync + 'static;

/// Default maximum accepted JSON payload size.
pub const DEFAULT_MAX_JSON_BYTES: usize = 64 * 1024;

/// Bind one JSON value as `serde_json::Value`.
pub struct JsonVar {
    name: VariableName,
    default: Option<Value>,
    allow_empty: bool,
    sensitive: bool,
    max_bytes: usize,
    validators: Vec<Box<JsonValidator>>,
}

impl JsonVar {
    /// Build a JSON variable.
    #[must_use]
    pub fn new(name: impl Into<VariableName>) -> Self {
        Self {
            name: name.into(),
            default: None,
            allow_empty: false,
            sensitive: true,
            max_bytes: DEFAULT_MAX_JSON_BYTES,
            validators: Vec::new(),
        }
    }

    /// Provide a fallback value when the variable is missing or empty.
    #[must_use]
    pub fn default(mut self, value: Value) -> Self {
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

    /// Override the maximum accepted raw JSON byte length.
    #[must_use]
    pub fn max_bytes(mut self, value: usize) -> Self {
        self.max_bytes = value;
        self
    }

    /// Attach a validation rule.
    #[must_use]
    pub fn validate<F>(mut self, validator: F) -> Self
    where
        F: Fn(&Value) -> Result<(), ValidationError> + Send + Sync + 'static,
    {
        self.validators.push(Box::new(validator));
        self
    }
}

impl Binding<Value> for JsonVar {
    fn bind<E: Environment>(&self, environment: &E) -> Result<Value, BindError> {
        let name = self.name.as_ref();
        let resolved = resolve_raw_with_max_bytes(
            environment,
            name,
            self.default.is_some(),
            self.allow_empty,
            self.max_bytes,
        )?;
        let (value, used_default) = match resolved {
            Some(raw) => (parse_json(name, &raw)?, false),
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

fn parse_json(name: &str, raw: &str) -> Result<Value, BindError> {
    serde_json::from_str::<Value>(raw).map_err(|_| BindError::parse(name.to_owned(), "JSON"))
}
