#![allow(missing_docs)]

use envbind::{Binder, BoolVar, MapEnvironment, ValidationError};

#[test]
fn bool_var_reads_truthy_value() {
    let value =
        Binder::new(MapEnvironment::from_pairs([("TLS", "yes")])).bind(&BoolVar::new("TLS"));

    assert_eq!(value, Ok(true));
}

#[test]
fn bool_var_reads_falsy_value() {
    let value =
        Binder::new(MapEnvironment::from_pairs([("TLS", "off")])).bind(&BoolVar::new("TLS"));

    assert_eq!(value, Ok(false));
}

#[test]
fn bool_var_uses_default_when_missing() {
    let value = Binder::new(MapEnvironment::new()).bind(&BoolVar::new("TLS").default(true));

    assert_eq!(value, Ok(true));
}

#[test]
fn bool_var_rejects_missing_value_without_default() {
    let value = Binder::new(MapEnvironment::new())
        .bind(&BoolVar::new("TLS"))
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("missing_variable"));
}

#[test]
fn bool_var_rejects_empty_value() {
    let value = Binder::new(MapEnvironment::from_pairs([("TLS", "")]))
        .bind(&BoolVar::new("TLS"))
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("empty_variable"));
}

#[test]
fn bool_var_rejects_whitespace_as_invalid_text() {
    let value = Binder::new(MapEnvironment::from_pairs([("TLS", " ")]))
        .bind(&BoolVar::new("TLS"))
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("invalid_boolean"));
}

#[test]
fn bool_var_rejects_invalid_text() {
    let value = Binder::new(MapEnvironment::from_pairs([("TLS", "maybe")]))
        .bind(&BoolVar::new("TLS"))
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("invalid_boolean"));
}

#[test]
fn bool_var_runs_validator() {
    let value = Binder::new(MapEnvironment::from_pairs([("TLS", "false")]))
        .bind(&BoolVar::new("TLS").sensitive(false).validate(|enabled| {
            if enabled {
                Ok(())
            } else {
                Err(ValidationError::new("TLS must be enabled"))
            }
        }))
        .map_err(|error| {
            (
                error.error_code().to_owned(),
                error.to_string().contains("TLS must be enabled"),
            )
        });

    assert_eq!(value, Err(("validation_failed".to_owned(), true)));
}

#[test]
fn bool_var_skips_validation_for_default_value() {
    let value = Binder::new(MapEnvironment::new()).bind(
        &BoolVar::new("TLS").default(false).validate(|enabled| {
            if enabled {
                Ok(())
            } else {
                Err(ValidationError::new("TLS must be enabled"))
            }
        }),
    );

    assert_eq!(value, Ok(false));
}
