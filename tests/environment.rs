#![allow(missing_docs)]

use std::error::Error;

use envbind::{
    Binder, BindingExt, Environment, EnvironmentError, MapEnvironment, OptionalStringVar,
    ProcessEnvironment, StringVar,
};

#[test]
fn empty_map_environment_returns_none() {
    let value = MapEnvironment::new().get("MISSING");

    assert_eq!(value, Ok(None));
}

#[test]
fn map_environment_from_pairs_returns_value() {
    let value = MapEnvironment::from_pairs([("HOST", "localhost")]).get("HOST");

    assert_eq!(value, Ok(Some("localhost".to_owned())));
}

#[test]
fn map_environment_insert_replaces_value() {
    let mut environment = MapEnvironment::new();

    environment.insert("HOST", "mongodb");

    assert_eq!(environment.get("HOST"), Ok(Some("mongodb".to_owned())));
}

#[test]
fn process_environment_rejects_empty_equals_and_nul_names() {
    let results = ["", "INVALID=NAME", "INVALID\0NAME", "=C:"].map(|name| {
        ProcessEnvironment
            .get(name)
            .map_err(|error| error.to_string())
    });

    assert_eq!(
        results,
        std::array::from_fn(|_| Err("invalid environment variable name".to_owned()))
    );
}

#[test]
fn invalid_process_name_preserves_structured_binding_error() {
    let result = Binder::new(ProcessEnvironment)
        .bind(
            &StringVar::new("INVALID=NAME")
                .default("fallback")
                .optional(),
        )
        .map_err(|error| {
            (
                error.error_code(),
                error.variable_name().to_owned(),
                error
                    .source()
                    .and_then(|source| source.downcast_ref::<EnvironmentError>())
                    .cloned(),
            )
        });

    assert_eq!(
        result,
        Err((
            "environment_error",
            "INVALID=NAME".to_owned(),
            Some(EnvironmentError::InvalidName)
        )),
    );
}

#[test]
fn invalid_process_names_fail_required_binding() {
    let results = ["", "INVALID=NAME", "INVALID\0NAME"].map(|name| {
        Binder::new(ProcessEnvironment)
            .bind(&StringVar::new(name))
            .map_err(|error| error.error_code())
    });

    assert_eq!(results, std::array::from_fn(|_| Err("environment_error")));
}

#[test]
fn allow_empty_values_does_not_allow_invalid_process_names() {
    let result = Binder::new(ProcessEnvironment)
        .bind(&StringVar::new("").allow_empty())
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("environment_error"));
}

#[test]
fn invalid_process_names_do_not_select_defaults() {
    let results = ["", "INVALID=NAME", "INVALID\0NAME"].map(|name| {
        Binder::new(ProcessEnvironment)
            .bind(&StringVar::new(name).default("fallback"))
            .map_err(|error| error.error_code())
    });

    assert_eq!(results, std::array::from_fn(|_| Err("environment_error")));
}

#[test]
fn invalid_process_names_do_not_become_optional_absence() {
    let results = ["", "INVALID=NAME", "INVALID\0NAME"].map(|name| {
        Binder::new(ProcessEnvironment)
            .bind(&StringVar::new(name).optional())
            .map_err(|error| error.error_code())
    });

    assert_eq!(results, std::array::from_fn(|_| Err("environment_error")));
}

#[test]
fn invalid_process_names_do_not_select_wrapped_defaults() {
    let results = ["", "INVALID=NAME", "INVALID\0NAME"].map(|name| {
        Binder::new(ProcessEnvironment)
            .bind(&StringVar::new(name).default("fallback").optional())
            .map_err(|error| error.error_code())
    });

    assert_eq!(results, std::array::from_fn(|_| Err("environment_error")));
}

#[test]
fn invalid_process_names_fail_optional_string_binding() {
    let results = ["", "INVALID=NAME", "INVALID\0NAME"].map(|name| {
        Binder::new(ProcessEnvironment)
            .bind(&OptionalStringVar::new(name))
            .map_err(|error| error.error_code())
    });

    assert_eq!(results, std::array::from_fn(|_| Err("environment_error")));
}

#[test]
fn invalid_process_names_do_not_select_optional_string_defaults() {
    let results = ["", "INVALID=NAME", "INVALID\0NAME"].map(|name| {
        Binder::new(ProcessEnvironment)
            .bind(&OptionalStringVar::new(name).default("fallback"))
            .map_err(|error| error.error_code())
    });

    assert_eq!(results, std::array::from_fn(|_| Err("environment_error")));
}

#[test]
fn map_names_are_not_restricted_by_process_syntax() {
    let results = ["", "CUSTOM=NAME", "CUSTOM\0NAME"].map(|name| {
        Binder::new(MapEnvironment::from_pairs([(name, "value")])).bind(&StringVar::new(name))
    });

    assert_eq!(results, std::array::from_fn(|_| Ok("value".to_owned())));
}

#[test]
fn custom_adapter_names_are_not_restricted_by_process_syntax() {
    struct EchoName;

    impl Environment for EchoName {
        fn get(&self, name: &str) -> Result<Option<String>, EnvironmentError> {
            Ok(Some(name.to_owned()))
        }
    }

    let names = ["", "CUSTOM=NAME", "CUSTOM\0NAME"];
    let results = names.map(|name| Binder::new(EchoName).bind(&StringVar::new(name).allow_empty()));

    assert_eq!(results, names.map(|name| Ok(name.to_owned())));
}
