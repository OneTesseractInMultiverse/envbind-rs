#![allow(missing_docs)]

use envbind::{Binder, MapEnvironment, U16Var, validators};

#[test]
fn u16_var_reads_present_value() {
    let value =
        Binder::new(MapEnvironment::from_pairs([("PORT", "8080")])).bind(&U16Var::new("PORT"));

    assert_eq!(value, Ok(8080));
}

#[test]
fn u16_var_uses_default_when_missing() {
    let value = Binder::new(MapEnvironment::new()).bind(&U16Var::new("PORT").default(27017));

    assert_eq!(value, Ok(27017));
}

#[test]
fn u16_var_rejects_missing_value_without_default() {
    let value = Binder::new(MapEnvironment::new())
        .bind(&U16Var::new("PORT"))
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("missing_variable"));
}

#[test]
fn u16_var_rejects_empty_value() {
    let value = Binder::new(MapEnvironment::from_pairs([("PORT", "")]))
        .bind(&U16Var::new("PORT"))
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("empty_variable"));
}

#[test]
fn u16_var_rejects_whitespace_as_parse_error() {
    let value = Binder::new(MapEnvironment::from_pairs([("PORT", " ")]))
        .bind(&U16Var::new("PORT"))
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("parse_variable"));
}

#[test]
fn u16_var_rejects_unparseable_value() {
    let value = Binder::new(MapEnvironment::from_pairs([("PORT", "abc")]))
        .bind(&U16Var::new("PORT"))
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("parse_variable"));
}

#[test]
fn u16_var_rejects_out_of_type_range_value_before_validation() {
    let value = Binder::new(MapEnvironment::from_pairs([("PORT", "70000")]))
        .bind(&U16Var::new("PORT"))
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("parse_variable"));
}

#[test]
fn u16_var_reports_validation_failure() {
    let value = Binder::new(MapEnvironment::from_pairs([("PORT", "80")]))
        .bind(&U16Var::new("PORT").validate(validators::u16_in_range(1024, 65_535)))
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("validation_failed"));
}
