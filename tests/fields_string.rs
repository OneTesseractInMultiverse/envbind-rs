#![allow(missing_docs)]

use envbind::{
    Binder, Environment, EnvironmentError, MapEnvironment, OptionalStringVar, StringVar, validators,
};

#[derive(Debug, Clone, Copy)]
struct FailingEnvironment;

impl Environment for FailingEnvironment {
    fn get(&self, _name: &str) -> Result<Option<String>, EnvironmentError> {
        Err(EnvironmentError::read("backend unavailable"))
    }
}

#[test]
fn required_string_reads_present_value() {
    let value = Binder::new(MapEnvironment::from_pairs([("HOST", "mongodb")]))
        .bind(&StringVar::new("HOST"));

    assert_eq!(value, Ok("mongodb".to_owned()));
}

#[test]
fn required_string_accepts_owned_variable_name() {
    let name = String::from("SERVICE_HOST");
    let value = Binder::new(MapEnvironment::from_pairs([("SERVICE_HOST", "api")]))
        .bind(&StringVar::new(name));

    assert_eq!(value, Ok("api".to_owned()));
}

#[test]
fn required_string_uses_default_when_missing() {
    let value =
        Binder::new(MapEnvironment::new()).bind(&StringVar::new("HOST").default("localhost"));

    assert_eq!(value, Ok("localhost".to_owned()));
}

#[test]
fn required_string_rejects_missing_value_without_default() {
    let value = Binder::new(MapEnvironment::new())
        .bind(&StringVar::new("HOST"))
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("missing_variable"));
}

#[test]
fn required_string_reports_environment_errors() {
    let value = Binder::new(FailingEnvironment)
        .bind(&StringVar::new("HOST"))
        .map_err(|error| {
            (
                error.error_code().to_owned(),
                error.variable_name().to_owned(),
            )
        });

    assert_eq!(
        value,
        Err(("environment_error".to_owned(), "HOST".to_owned()))
    );
}

#[test]
fn required_string_rejects_empty_value_by_default() {
    let value = Binder::new(MapEnvironment::from_pairs([("HOST", "")]))
        .bind(&StringVar::new("HOST"))
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("empty_variable"));
}

#[test]
fn required_string_accepts_empty_value_when_allowed() {
    let value = Binder::new(MapEnvironment::from_pairs([("HOST", "")]))
        .bind(&StringVar::new("HOST").allow_empty());

    assert_eq!(value, Ok(String::new()));
}

#[test]
fn required_string_preserves_whitespace_value() {
    let value =
        Binder::new(MapEnvironment::from_pairs([("HOST", "   ")])).bind(&StringVar::new("HOST"));

    assert_eq!(value, Ok("   ".to_owned()));
}

#[test]
fn required_string_runs_validator() {
    let value = Binder::new(MapEnvironment::from_pairs([("HOST", "ab")]))
        .bind(&StringVar::new("HOST").validate(validators::min_length(3)))
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("validation_failed"));
}

#[test]
fn required_string_skips_validation_for_default_value() {
    let value = Binder::new(MapEnvironment::new()).bind(
        &StringVar::new("HOST")
            .default("")
            .validate(validators::min_length(1)),
    );

    assert_eq!(value, Ok(String::new()));
}

#[test]
fn optional_string_returns_none_when_missing() {
    let value = Binder::new(MapEnvironment::new()).bind(&OptionalStringVar::new("HOST"));

    assert_eq!(value, Ok(None));
}

#[test]
fn optional_string_treats_empty_as_none_by_default() {
    let value = Binder::new(MapEnvironment::from_pairs([("HOST", "")]))
        .bind(&OptionalStringVar::new("HOST"));

    assert_eq!(value, Ok(None));
}

#[test]
fn optional_string_preserves_whitespace_value() {
    let value = Binder::new(MapEnvironment::from_pairs([("HOST", " ")]))
        .bind(&OptionalStringVar::new("HOST"));

    assert_eq!(value, Ok(Some(" ".to_owned())));
}

#[test]
fn optional_string_can_keep_empty_value() {
    let value = Binder::new(MapEnvironment::from_pairs([("HOST", "")]))
        .bind(&OptionalStringVar::new("HOST").allow_empty());

    assert_eq!(value, Ok(Some(String::new())));
}

#[test]
fn optional_string_runs_validator_when_value_present() {
    let value = Binder::new(MapEnvironment::from_pairs([("HOST", "test")]))
        .bind(&OptionalStringVar::new("HOST").validate(validators::one_of(["prod"])))
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("validation_failed"));
}
