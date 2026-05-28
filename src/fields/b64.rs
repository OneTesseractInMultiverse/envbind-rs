//! Base64-decoded string variable spec.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use crate::binder::Binding;
use crate::environment::Environment;
use crate::error::{BindError, ValidationError, VariableName};

use super::raw::resolve_raw_with_max_bytes;

type StringValidator = dyn Fn(&str) -> Result<(), ValidationError> + Send + Sync + 'static;

/// Default maximum decoded byte length for base64 values.
pub const DEFAULT_MAX_B64_DECODED_BYTES: usize = 1024 * 1024;

/// Bind one base64-decoded UTF-8 string value.
pub struct B64DecodedStringVar {
    name: VariableName,
    default: Option<String>,
    allow_empty: bool,
    sensitive: bool,
    max_decoded_bytes: usize,
    validators: Vec<Box<StringValidator>>,
}

impl B64DecodedStringVar {
    /// Build a base64-decoded string variable.
    #[must_use]
    pub fn new(name: impl Into<VariableName>) -> Self {
        Self {
            name: name.into(),
            default: None,
            allow_empty: false,
            sensitive: true,
            max_decoded_bytes: DEFAULT_MAX_B64_DECODED_BYTES,
            validators: Vec::new(),
        }
    }

    /// Provide a fallback decoded value when the variable is missing or empty.
    #[must_use]
    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.default = Some(value.into());
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

    /// Override the maximum accepted decoded byte length.
    #[must_use]
    pub fn max_decoded_bytes(mut self, value: usize) -> Self {
        self.max_decoded_bytes = value;
        self
    }

    /// Attach a validation rule.
    #[must_use]
    pub fn validate<F>(mut self, validator: F) -> Self
    where
        F: Fn(&str) -> Result<(), ValidationError> + Send + Sync + 'static,
    {
        self.validators.push(Box::new(validator));
        self
    }
}

impl Binding<String> for B64DecodedStringVar {
    fn bind<E: Environment>(&self, environment: &E) -> Result<String, BindError> {
        let name = self.name.as_ref();
        let resolved = resolve_raw_with_max_bytes(
            environment,
            name,
            self.default.is_some(),
            self.allow_empty,
            max_encoded_bytes(self.max_decoded_bytes),
        )?;
        let (value, used_default) = match resolved {
            Some(raw) => (decode_b64(name, &raw, self.max_decoded_bytes)?, false),
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

fn decode_b64(name: &str, raw: &str, max_decoded_bytes: usize) -> Result<String, BindError> {
    let bytes = STANDARD
        .decode(raw)
        .map_err(|_| BindError::parse(name.to_owned(), "base64"))?;
    if bytes.len() > max_decoded_bytes {
        return Err(BindError::value_too_large(
            name.to_owned(),
            max_decoded_bytes,
        ));
    }
    String::from_utf8(bytes).map_err(|_| BindError::parse(name.to_owned(), "utf-8 text"))
}

fn max_encoded_bytes(max_decoded_bytes: usize) -> usize {
    (max_decoded_bytes.saturating_add(2) / 3).saturating_mul(4)
}
