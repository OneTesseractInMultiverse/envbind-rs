//! Error types for environment binding and validation.

use std::borrow::Cow;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// Environment variable name captured in public errors.
pub type VariableName = Cow<'static, str>;

/// Failure returned by an environment adapter while reading a variable.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvironmentError {
    /// The variable was present but could not be represented as valid Unicode.
    NotUnicode,
    /// Adapter-specific read failure.
    Read {
        /// Diagnostic message retained for structured handling.
        ///
        /// This message is not displayed by default because custom adapters may
        /// accidentally include raw environment values.
        message: String,
    },
}

impl EnvironmentError {
    /// Build a non-Unicode environment value error.
    #[must_use]
    pub fn not_unicode() -> Self {
        Self::NotUnicode
    }

    /// Build an adapter-specific read error.
    #[must_use]
    pub fn read(message: impl Into<String>) -> Self {
        Self::Read {
            message: message.into(),
        }
    }
}

impl Display for EnvironmentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotUnicode => formatter.write_str("value is not valid Unicode"),
            Self::Read { .. } => formatter.write_str("adapter read failed"),
        }
    }
}

impl Error for EnvironmentError {}

/// Validation failure returned by reusable validators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    message: String,
}

impl ValidationError {
    /// Build a validation error from a safe message.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Return the validation message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for ValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for ValidationError {}

/// Binding failure for one environment variable.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindError {
    /// Required variable missing.
    MissingVariable {
        /// Environment variable name.
        name: VariableName,
    },
    /// Required variable empty.
    EmptyVariable {
        /// Environment variable name.
        name: VariableName,
    },
    /// Environment adapter failed while reading the variable.
    Environment {
        /// Environment variable name.
        name: VariableName,
        /// Adapter failure.
        source: EnvironmentError,
    },
    /// Boolean value invalid.
    InvalidBoolean {
        /// Environment variable name.
        name: VariableName,
    },
    /// Parsed value invalid for the expected type.
    ParseVariable {
        /// Environment variable name.
        name: VariableName,
        /// Expected type text.
        expected: &'static str,
    },
    /// Value failed validation.
    Validation {
        /// Environment variable name.
        name: VariableName,
        /// Safe validation detail.
        message: String,
    },
    /// Raw value exceeded the configured byte limit.
    ValueTooLarge {
        /// Environment variable name.
        name: VariableName,
        /// Maximum accepted byte count.
        max_bytes: usize,
    },
}

impl BindError {
    /// Build a missing-variable error.
    #[must_use]
    pub fn missing(name: impl Into<VariableName>) -> Self {
        Self::MissingVariable { name: name.into() }
    }

    /// Build an empty-variable error.
    #[must_use]
    pub fn empty(name: impl Into<VariableName>) -> Self {
        Self::EmptyVariable { name: name.into() }
    }

    /// Build an environment adapter error.
    #[must_use]
    pub fn environment(name: impl Into<VariableName>, source: EnvironmentError) -> Self {
        Self::Environment {
            name: name.into(),
            source,
        }
    }

    /// Build a boolean parse error.
    #[must_use]
    pub fn invalid_boolean(name: impl Into<VariableName>) -> Self {
        Self::InvalidBoolean { name: name.into() }
    }

    /// Build a typed parse error.
    #[must_use]
    pub fn parse(name: impl Into<VariableName>, expected: &'static str) -> Self {
        Self::ParseVariable {
            name: name.into(),
            expected,
        }
    }

    /// Build a validation error.
    #[must_use]
    pub fn validation(name: impl Into<VariableName>, error: ValidationError) -> Self {
        Self::Validation {
            name: name.into(),
            message: error.message,
        }
    }

    /// Build a validation error, hiding custom details for sensitive values.
    #[must_use]
    pub fn validation_with_sensitivity(
        name: impl Into<VariableName>,
        error: ValidationError,
        sensitive: bool,
    ) -> Self {
        if sensitive {
            Self::Validation {
                name: name.into(),
                message: "validation failed for sensitive value".to_owned(),
            }
        } else {
            Self::validation(name, error)
        }
    }

    /// Build a value-size limit error.
    #[must_use]
    pub fn value_too_large(name: impl Into<VariableName>, max_bytes: usize) -> Self {
        Self::ValueTooLarge {
            name: name.into(),
            max_bytes,
        }
    }

    /// Return the environment variable name associated with the error.
    #[must_use]
    pub fn variable_name(&self) -> &str {
        match self {
            Self::MissingVariable { name }
            | Self::EmptyVariable { name }
            | Self::Environment { name, .. }
            | Self::InvalidBoolean { name }
            | Self::ParseVariable { name, .. }
            | Self::Validation { name, .. }
            | Self::ValueTooLarge { name, .. } => name,
        }
    }

    /// Return a stable error code.
    #[must_use]
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::MissingVariable { .. } => "missing_variable",
            Self::EmptyVariable { .. } => "empty_variable",
            Self::Environment { .. } => "environment_error",
            Self::InvalidBoolean { .. } => "invalid_boolean",
            Self::ParseVariable { .. } => "parse_variable",
            Self::Validation { .. } => "validation_failed",
            Self::ValueTooLarge { .. } => "value_too_large",
        }
    }
}

impl Display for BindError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingVariable { name } => {
                write!(formatter, "missing environment variable {name}")
            }
            Self::EmptyVariable { name } => {
                write!(formatter, "environment variable {name} must not be empty")
            }
            Self::Environment { name, source } => {
                write!(
                    formatter,
                    "environment variable {name} could not be read: {source}"
                )
            }
            Self::InvalidBoolean { name } => {
                write!(
                    formatter,
                    "environment variable {name} must be boolean-like"
                )
            }
            Self::ParseVariable { name, expected } => {
                write!(
                    formatter,
                    "environment variable {name} must parse as {expected}"
                )
            }
            Self::Validation { name, message } => {
                write!(
                    formatter,
                    "environment variable {name} failed validation: {message}"
                )
            }
            Self::ValueTooLarge { name, max_bytes } => {
                write!(
                    formatter,
                    "environment variable {name} exceeds maximum allowed size of {max_bytes} bytes"
                )
            }
        }
    }
}

impl Error for BindError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Environment { source, .. } => Some(source),
            _ => None,
        }
    }
}
