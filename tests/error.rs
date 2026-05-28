#![allow(missing_docs)]

use envbind::{BindError, EnvironmentError, ValidationError};

#[test]
fn validation_error_keeps_message() {
    let error = ValidationError::new("bad value");

    assert_eq!(error.message(), "bad value");
}

#[test]
fn missing_error_has_stable_code() {
    let error = BindError::missing("PORT");

    assert_eq!(error.error_code(), "missing_variable");
}

#[test]
fn empty_error_has_stable_code() {
    let error = BindError::empty("PORT");

    assert_eq!(error.error_code(), "empty_variable");
}

#[test]
fn environment_error_has_stable_code() {
    let error = BindError::environment("PORT", EnvironmentError::not_unicode());

    assert_eq!(error.error_code(), "environment_error");
}

#[test]
fn invalid_boolean_error_has_stable_code() {
    let error = BindError::invalid_boolean("TLS");

    assert_eq!(error.error_code(), "invalid_boolean");
}

#[test]
fn parse_error_has_stable_code() {
    let error = BindError::parse("PORT", "u16");

    assert_eq!(error.error_code(), "parse_variable");
}

#[test]
fn validation_error_has_stable_code() {
    let error = BindError::validation("PORT", ValidationError::new("range"));

    assert_eq!(error.error_code(), "validation_failed");
}

#[test]
fn value_too_large_error_has_stable_code() {
    let error = BindError::value_too_large("TOKEN", 8);

    assert_eq!(error.error_code(), "value_too_large");
}

#[test]
fn bind_error_display_includes_variable_and_context() {
    let error = BindError::parse("PORT", "u16");

    assert_eq!(
        error.to_string(),
        "environment variable PORT must parse as u16"
    );
}

#[test]
fn bind_error_display_redacts_raw_values() {
    let error = BindError::invalid_boolean("TOKEN_MODE");

    assert!(!error.to_string().contains("secret-value"));
}

#[test]
fn invalid_boolean_display_is_safe() {
    let error = BindError::invalid_boolean("TOKEN_MODE");

    assert_eq!(
        error.to_string(),
        "environment variable TOKEN_MODE must be boolean-like"
    );
}

#[test]
fn environment_error_display_is_safe() {
    let error = BindError::environment("TOKEN", EnvironmentError::not_unicode());

    assert_eq!(
        error.to_string(),
        "environment variable TOKEN read failed: value is not valid Unicode"
    );
}

#[test]
fn environment_read_error_display_redacts_adapter_message() {
    let error = BindError::environment("TOKEN", EnvironmentError::read("secret-value"));

    assert_eq!(
        error.to_string(),
        "environment variable TOKEN read failed: adapter read failed"
    );
}
