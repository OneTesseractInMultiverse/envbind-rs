//! String-oriented variable specs.

use crate::binder::Binding;
use crate::environment::Environment;
use crate::error::{BindError, ValidationError, VariableName};

use super::raw::{DEFAULT_MAX_RAW_BYTES, ensure_within_max_bytes};

type StringValidator = dyn Fn(&str) -> Result<(), ValidationError> + Send + Sync + 'static;

/// Bind one required string value.
pub struct StringVar {
    name: VariableName,
    default: Option<String>,
    allow_empty: bool,
    sensitive: bool,
    max_bytes: usize,
    validators: Vec<Box<StringValidator>>,
}

impl StringVar {
    /// Build a required string variable.
    #[must_use]
    pub fn new(name: impl Into<VariableName>) -> Self {
        Self {
            name: name.into(),
            default: None,
            allow_empty: false,
            sensitive: true,
            max_bytes: DEFAULT_MAX_RAW_BYTES,
            validators: Vec::new(),
        }
    }

    /// Provide a fallback value when the variable is missing.
    #[must_use]
    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.default = Some(value.into());
        self
    }

    /// Allow empty strings to pass through.
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

    /// Override the maximum accepted raw byte length.
    #[must_use]
    pub fn max_bytes(mut self, value: usize) -> Self {
        self.max_bytes = value;
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

impl Binding<String> for StringVar {
    fn bind<E: Environment>(&self, environment: &E) -> Result<String, BindError> {
        let name = self.name.as_ref();
        let resolved = required_string(
            environment,
            name,
            self.default.as_deref(),
            self.allow_empty,
            self.max_bytes,
        )?;
        let (value, used_default) = match resolved {
            ResolvedString::Parsed(value) => (value, false),
            ResolvedString::Defaulted(value) => (value, true),
        };
        if used_default {
            return Ok(value);
        }
        validate_string(
            name,
            &value,
            self.allow_empty,
            self.sensitive,
            self.validators.iter().map(Box::as_ref),
        )?;
        Ok(value)
    }
}

/// Bind one optional string value.
pub struct OptionalStringVar {
    name: VariableName,
    default: Option<String>,
    allow_empty: bool,
    sensitive: bool,
    max_bytes: usize,
    validators: Vec<Box<StringValidator>>,
}

impl OptionalStringVar {
    /// Build an optional string variable.
    #[must_use]
    pub fn new(name: impl Into<VariableName>) -> Self {
        Self {
            name: name.into(),
            default: None,
            allow_empty: false,
            sensitive: true,
            max_bytes: DEFAULT_MAX_RAW_BYTES,
            validators: Vec::new(),
        }
    }

    /// Provide a fallback value when the variable is missing or empty.
    #[must_use]
    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.default = Some(value.into());
        self
    }

    /// Allow empty strings to be returned as `Some("")`.
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

    /// Override the maximum accepted raw byte length.
    #[must_use]
    pub fn max_bytes(mut self, value: usize) -> Self {
        self.max_bytes = value;
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

impl Binding<Option<String>> for OptionalStringVar {
    fn bind<E: Environment>(&self, environment: &E) -> Result<Option<String>, BindError> {
        let name = self.name.as_ref();
        let value = optional_string(
            environment,
            name,
            self.default.as_deref(),
            self.allow_empty,
            self.max_bytes,
        )?;
        match value {
            OptionalResolvedString::Parsed(raw) => {
                validate_string(
                    name,
                    &raw,
                    self.allow_empty,
                    self.sensitive,
                    self.validators.iter().map(Box::as_ref),
                )?;
                Ok(Some(raw))
            }
            OptionalResolvedString::Defaulted(raw) => Ok(Some(raw)),
            OptionalResolvedString::Missing => Ok(None),
        }
    }
}

enum ResolvedString {
    Parsed(String),
    Defaulted(String),
}

enum OptionalResolvedString {
    Parsed(String),
    Defaulted(String),
    Missing,
}

fn required_string<E: Environment>(
    environment: &E,
    name: &str,
    default: Option<&str>,
    allow_empty: bool,
    max_bytes: usize,
) -> Result<ResolvedString, BindError> {
    match environment
        .get(name)
        .map_err(|source| BindError::environment(name.to_owned(), source))?
    {
        Some(value) if value.is_empty() && !allow_empty => match default {
            Some(value) => Ok(ResolvedString::Defaulted(value.to_owned())),
            None => Err(BindError::empty(name.to_owned())),
        },
        Some(value) => ensure_within_max_bytes(name, value, max_bytes).map(ResolvedString::Parsed),
        None => match default {
            Some(value) => Ok(ResolvedString::Defaulted(value.to_owned())),
            None => Err(BindError::missing(name.to_owned())),
        },
    }
}

fn optional_string<E: Environment>(
    environment: &E,
    name: &str,
    default: Option<&str>,
    allow_empty: bool,
    max_bytes: usize,
) -> Result<OptionalResolvedString, BindError> {
    Ok(
        match environment
            .get(name)
            .map_err(|source| BindError::environment(name.to_owned(), source))?
        {
            Some(value) if value.is_empty() && !allow_empty => match default {
                Some(value) => OptionalResolvedString::Defaulted(value.to_owned()),
                None => OptionalResolvedString::Missing,
            },
            Some(value) => {
                OptionalResolvedString::Parsed(ensure_within_max_bytes(name, value, max_bytes)?)
            }
            None => match default {
                Some(value) => OptionalResolvedString::Defaulted(value.to_owned()),
                None => OptionalResolvedString::Missing,
            },
        },
    )
}

fn validate_string<'a, I>(
    name: &str,
    value: &str,
    allow_empty: bool,
    sensitive: bool,
    validators: I,
) -> Result<(), BindError>
where
    I: IntoIterator<Item = &'a StringValidator>,
{
    if !allow_empty && value.is_empty() {
        return Err(BindError::empty(name.to_owned()));
    }

    for validator in validators {
        validator(value).map_err(|error| {
            BindError::validation_with_sensitivity(name.to_owned(), error, sensitive)
        })?;
    }

    Ok(())
}
