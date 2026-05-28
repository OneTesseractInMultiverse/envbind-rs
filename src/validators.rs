//! Reusable validation helpers for typed variable specs.

use std::fmt::Display;

use regex::Regex;

use crate::error::ValidationError;

/// Boxed string validator used by composed validator helpers.
pub type BoxedStringValidator =
    Box<dyn Fn(&str) -> Result<(), ValidationError> + Send + Sync + 'static>;

/// Boxed copy-value validator used by composed validator helpers.
pub type BoxedValueValidator<T> =
    Box<dyn Fn(T) -> Result<(), ValidationError> + Send + Sync + 'static>;

/// Require a minimum string length.
#[must_use = "pass validators to a variable spec"]
pub fn min_length(
    minimum: usize,
) -> impl Fn(&str) -> Result<(), ValidationError> + Send + Sync + 'static {
    move |value| {
        if value.chars().count() >= minimum {
            Ok(())
        } else {
            Err(ValidationError::new(format!(
                "length must be at least {minimum}"
            )))
        }
    }
}

/// Require a maximum string length.
#[must_use = "pass validators to a variable spec"]
pub fn max_length(
    maximum: usize,
) -> impl Fn(&str) -> Result<(), ValidationError> + Send + Sync + 'static {
    move |value| {
        let length = value.chars().count();
        if length <= maximum {
            Ok(())
        } else {
            Err(ValidationError::new(format!(
                "length must be at most {maximum}"
            )))
        }
    }
}

/// Require membership in a fixed allowed set.
#[must_use = "pass validators to a variable spec"]
pub fn one_of<const N: usize>(
    allowed: [&'static str; N],
) -> impl Fn(&str) -> Result<(), ValidationError> + Send + Sync + 'static {
    move |value| {
        if allowed.contains(&value) {
            Ok(())
        } else {
            Err(ValidationError::new("value must be in the allowed set"))
        }
    }
}

/// Require a copyable value to belong to a fixed allowed set.
#[must_use = "pass validators to a variable spec"]
pub fn one_of_values<T, const N: usize>(
    allowed: [T; N],
) -> impl Fn(T) -> Result<(), ValidationError> + Send + Sync + 'static
where
    T: PartialEq + Copy + Send + Sync + 'static,
{
    move |value| {
        if allowed.contains(&value) {
            Ok(())
        } else {
            Err(ValidationError::new("value must be in the allowed set"))
        }
    }
}

/// Require a numeric value within an inclusive range.
#[must_use = "pass validators to a variable spec"]
pub fn in_range<T>(
    minimum: T,
    maximum: T,
) -> impl Fn(T) -> Result<(), ValidationError> + Send + Sync + 'static
where
    T: PartialOrd + Copy + Display + Send + Sync + 'static,
{
    move |value| {
        if value >= minimum && value <= maximum {
            Ok(())
        } else {
            Err(ValidationError::new(format!(
                "value must be between {minimum} and {maximum}"
            )))
        }
    }
}

/// Require a numeric value greater than or equal to a minimum.
#[must_use = "pass validators to a variable spec"]
pub fn min_value<T>(minimum: T) -> impl Fn(T) -> Result<(), ValidationError> + Send + Sync + 'static
where
    T: PartialOrd + Copy + Display + Send + Sync + 'static,
{
    move |value| {
        if value >= minimum {
            Ok(())
        } else {
            Err(ValidationError::new(format!(
                "value must be at least {minimum}"
            )))
        }
    }
}

/// Require a numeric value less than or equal to a maximum.
#[must_use = "pass validators to a variable spec"]
pub fn max_value<T>(maximum: T) -> impl Fn(T) -> Result<(), ValidationError> + Send + Sync + 'static
where
    T: PartialOrd + Copy + Display + Send + Sync + 'static,
{
    move |value| {
        if value <= maximum {
            Ok(())
        } else {
            Err(ValidationError::new(format!(
                "value must be at most {maximum}"
            )))
        }
    }
}

/// Require a string to match a regular expression.
#[must_use = "pass validators to a variable spec"]
pub fn matches_pattern(pattern: &str) -> Result<BoxedStringValidator, regex::Error> {
    let regex = Regex::new(pattern)?;
    let pattern = pattern.to_owned();
    Ok(Box::new(move |value| {
        if regex.is_match(value) {
            Ok(())
        } else {
            Err(ValidationError::new(format!(
                "value must match pattern {pattern}"
            )))
        }
    }))
}

/// Require a string to be a URL with an allowed scheme.
#[must_use = "pass validators to a variable spec"]
pub fn is_url() -> impl Fn(&str) -> Result<(), ValidationError> + Send + Sync + 'static {
    is_url_with_options(true, ["http", "https"])
}

/// Require a string to be a URL with configurable scheme rules.
#[must_use = "pass validators to a variable spec"]
pub fn is_url_with_options<I, S>(
    require_scheme: bool,
    allowed_schemes: I,
) -> impl Fn(&str) -> Result<(), ValidationError> + Send + Sync + 'static
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let allowed_schemes = allowed_schemes
        .into_iter()
        .map(|scheme| scheme.into().to_ascii_lowercase())
        .collect::<Vec<_>>();
    move |value| validate_url(value, require_scheme, &allowed_schemes)
}

/// Require a string to be a common email address shape.
#[must_use = "pass validators to a variable spec"]
pub fn is_email() -> impl Fn(&str) -> Result<(), ValidationError> + Send + Sync + 'static {
    move |value| {
        let regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .map_err(|_| ValidationError::new("email validator is unavailable"))?;
        if regex.is_match(value) {
            Ok(())
        } else {
            Err(ValidationError::new("value must be an email address"))
        }
    }
}

/// Combine several string validators.
#[must_use = "pass validators to a variable spec"]
pub fn all_of_str(
    validators: Vec<BoxedStringValidator>,
) -> impl Fn(&str) -> Result<(), ValidationError> + Send + Sync + 'static {
    move |value| {
        for validator in &validators {
            validator(value)?;
        }
        Ok(())
    }
}

/// Combine several copy-value validators.
#[must_use = "pass validators to a variable spec"]
pub fn all_of<T>(
    validators: Vec<BoxedValueValidator<T>>,
) -> impl Fn(T) -> Result<(), ValidationError> + Send + Sync + 'static
where
    T: Copy + Send + Sync + 'static,
{
    move |value| {
        for validator in &validators {
            validator(value)?;
        }
        Ok(())
    }
}

/// Require a `u16` value within an inclusive range.
#[must_use = "pass validators to a variable spec"]
pub fn u16_in_range(
    minimum: u16,
    maximum: u16,
) -> impl Fn(u16) -> Result<(), ValidationError> + Send + Sync + 'static {
    move |value| {
        if (minimum..=maximum).contains(&value) {
            Ok(())
        } else {
            Err(ValidationError::new(format!(
                "value must be between {minimum} and {maximum}"
            )))
        }
    }
}

fn validate_url(
    value: &str,
    require_scheme: bool,
    allowed_schemes: &[String],
) -> Result<(), ValidationError> {
    let (scheme, rest) = match value.split_once("://") {
        Some((scheme, rest)) => (Some(scheme.to_ascii_lowercase()), rest),
        None if require_scheme => {
            return Err(ValidationError::new("URL must include a scheme"));
        }
        None => (None, value.trim_start_matches("//")),
    };

    if let Some(scheme) = scheme {
        if !allowed_schemes.contains(&scheme) {
            return Err(ValidationError::new("URL scheme is not allowed"));
        }
    }

    let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
    if host.is_empty() {
        return Err(ValidationError::new("URL must include a host"));
    }
    if host.chars().any(char::is_whitespace) {
        return Err(ValidationError::new("URL host must not contain whitespace"));
    }
    Ok(())
}
