#![allow(missing_docs)]

use std::error::Error;

use envbind::{BindError, EnvironmentError, ValidationError};

#[test]
fn invalid_name_error_has_safe_display() {
    assert_eq!(
        EnvironmentError::invalid_name().to_string(),
        "invalid environment variable name",
    );
}

#[test]
fn invalid_name_error_has_safe_debug() {
    assert_eq!(
        format!("{:?}", EnvironmentError::invalid_name()),
        "InvalidName"
    );
}

#[test]
fn invalid_name_error_has_safe_alternate_debug() {
    assert_eq!(
        format!("{:#?}", EnvironmentError::invalid_name()),
        "InvalidName"
    );
}

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

#[test]
fn adapter_read_error_display_redacts_message() {
    let error = EnvironmentError::read("synthetic-secret-value");

    assert_eq!(error.to_string(), "adapter read failed");
}

#[test]
fn adapter_read_error_debug_redacts_message() {
    let error = EnvironmentError::read("synthetic-secret-value");

    assert_eq!(format!("{error:?}"), "Read { message: \"[redacted]\" }");
}

#[test]
fn adapter_read_error_pretty_debug_redacts_message() {
    let error = EnvironmentError::read("synthetic-secret-value");

    assert_eq!(
        format!("{error:#?}"),
        "Read {\n    message: \"[redacted]\",\n}"
    );
}

#[test]
fn not_unicode_debug_keeps_variant_name() {
    let error = EnvironmentError::not_unicode();

    assert_eq!(
        [format!("{error:?}"), format!("{error:#?}")],
        ["NotUnicode", "NotUnicode"]
    );
}

#[test]
fn bind_environment_error_formats_redact_adapter_message() {
    let error = BindError::environment("TOKEN", EnvironmentError::read("synthetic-secret-value"));
    let outputs = [
        error.to_string(),
        format!("{error:?}"),
        format!("{error:#?}"),
    ];

    assert!(
        outputs
            .iter()
            .all(|output| !output.contains("synthetic-secret-value"))
    );
}

#[test]
fn bind_environment_error_source_formats_redact_adapter_message() {
    let error = BindError::environment("TOKEN", EnvironmentError::read("synthetic-secret-value"));
    let redacted = error.source().map(|source| {
        [
            source.to_string(),
            format!("{source:?}"),
            format!("{source:#?}"),
        ]
        .iter()
        .all(|output| !output.contains("synthetic-secret-value"))
    });

    assert_eq!(redacted, Some(true));
}

#[test]
fn bind_environment_error_retains_structured_diagnostic() {
    let error = BindError::environment("TOKEN", EnvironmentError::read("synthetic-secret-value"));
    let message = error
        .source()
        .and_then(|source| source.downcast_ref::<EnvironmentError>())
        .and_then(|source| match source {
            EnvironmentError::Read { message } => Some(message.as_str()),
            _ => None,
        });

    assert_eq!(message, Some("synthetic-secret-value"));
}

#[test]
fn bind_environment_error_keeps_variable_context() {
    let error = BindError::environment("TOKEN", EnvironmentError::read("synthetic-secret-value"));

    assert_eq!(
        (error.error_code(), error.variable_name()),
        ("environment_error", "TOKEN")
    );
}
