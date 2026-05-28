//! Shared raw environment resolution for field specs.

use crate::environment::Environment;
use crate::error::BindError;

/// Default maximum accepted raw environment value size.
pub(super) const DEFAULT_MAX_RAW_BYTES: usize = 1024 * 1024;

/// Resolve raw text for field specs that use default-on-missing semantics.
///
/// Missing values return `Ok(None)`. Explicit empty strings return `Ok(None)`
/// with a default. Otherwise they fail without `allow_empty()`.
pub(super) fn resolve_raw<E: Environment>(
    environment: &E,
    name: &str,
    has_default: bool,
    allow_empty: bool,
) -> Result<Option<String>, BindError> {
    resolve_raw_with_max_bytes(
        environment,
        name,
        has_default,
        allow_empty,
        DEFAULT_MAX_RAW_BYTES,
    )
}

/// Resolve raw text and reject values above the configured byte limit.
pub(super) fn resolve_raw_with_max_bytes<E: Environment>(
    environment: &E,
    name: &str,
    has_default: bool,
    allow_empty: bool,
    max_bytes: usize,
) -> Result<Option<String>, BindError> {
    match environment
        .get(name)
        .map_err(|source| BindError::environment(name.to_owned(), source))?
    {
        Some(value) if value.is_empty() && !allow_empty && has_default => Ok(None),
        Some(value) if value.is_empty() && !allow_empty => Err(BindError::empty(name.to_owned())),
        Some(value) => ensure_within_max_bytes(name, value, max_bytes).map(Some),
        None => Ok(None),
    }
}

/// Reject a raw value that exceeds the configured byte limit.
pub(super) fn ensure_within_max_bytes(
    name: &str,
    value: String,
    max_bytes: usize,
) -> Result<String, BindError> {
    if value.len() > max_bytes {
        return Err(BindError::value_too_large(name.to_owned(), max_bytes));
    }

    Ok(value)
}
