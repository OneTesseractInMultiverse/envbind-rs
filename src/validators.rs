//! Reusable validation helpers for typed variable specs.

use std::fmt::Display;

use regex::Regex;
use url::{ParseError, Url};

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

/// Require an absolute HTTP or HTTPS URL with a host and a valid optional port.
///
/// Raw whitespace, control characters, and backslashes are rejected. Validation
/// uses the URL parser's syntax rules without changing the original value.
#[must_use = "pass validators to a variable spec"]
pub fn is_url() -> impl Fn(&str) -> Result<(), ValidationError> + Send + Sync + 'static {
    is_url_with_options(true, ["http", "https"])
}

/// Require a string to be a URL with configurable scheme rules.
///
/// Explicit schemes are always checked against `allowed_schemes`, ignoring
/// ASCII case, even when the scheme is optional. Every URL must have a host.
///
/// When `require_scheme` is false, a bare hostname or IPv4 address may include
/// a path, query, or fragment. Prefix an authority with `//` to include a port,
/// credentials, or a bracketed IPv6 address: for example, `//localhost:8080`.
/// A syntactically valid scheme prefix is treated as an explicit scheme even
/// without `://`; use `//` to mark an authority containing a port.
///
/// Scheme-less input is parsed with an internal HTTPS prefix solely to validate
/// its host and port. That prefix is not subject to the scheme allowlist and
/// does not change the bound string. Relative paths, raw whitespace, control
/// characters, and backslashes are rejected.
///
/// ```
/// use envbind::validators;
///
/// let validate = validators::is_url_with_options(false, ["https"]);
/// assert_eq!(validate("//localhost:8443/path"), Ok(()));
/// ```
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
    if value.contains('\\')
        || value
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
    {
        return Err(ValidationError::new("value must be a valid URL"));
    }

    let parsed = match Url::parse(value) {
        Ok(parsed) => {
            if !allowed_schemes
                .iter()
                .any(|scheme| scheme == parsed.scheme())
            {
                return Err(ValidationError::new("URL scheme is not allowed"));
            }
            parsed
        }
        Err(ParseError::RelativeUrlWithoutBase) if require_scheme => {
            return Err(ValidationError::new("URL must include a scheme"));
        }
        Err(ParseError::RelativeUrlWithoutBase) => parse_schemeless_url(value)?,
        Err(_) => return Err(ValidationError::new("value must be a valid URL")),
    };

    if parsed.host_str().is_none_or(str::is_empty) {
        return Err(ValidationError::new("URL must include a host"));
    }
    Ok(())
}

fn parse_schemeless_url(value: &str) -> Result<Url, ValidationError> {
    let authority = if let Some(authority) = value.strip_prefix("//") {
        authority
    } else {
        let host = value.split(['/', '?', '#']).next().unwrap_or_default();
        if host.contains([':', '@', '[', ']']) {
            return Err(ValidationError::new(
                "scheme-less ports, credentials, and IPv6 hosts must use a // prefix",
            ));
        }
        value
    };

    // Reject relative references instead of letting the URL parser repair them
    // into an authority when the synthetic scheme is added.
    if authority.is_empty()
        || authority.starts_with(['/', '?', '#'])
        || authority.starts_with("./")
        || authority.starts_with("../")
    {
        return Err(ValidationError::new("URL must include a host"));
    }

    Url::parse(&format!("https://{authority}"))
        .map_err(|_| ValidationError::new("value must be a valid URL"))
}
