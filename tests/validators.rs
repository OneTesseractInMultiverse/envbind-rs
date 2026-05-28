#![allow(missing_docs)]

use envbind::{ValidationError, validators};

#[test]
fn min_length_accepts_long_enough_value() {
    let result = validators::min_length(3)("abcd");

    assert_eq!(result, Ok(()));
}

#[test]
fn min_length_rejects_short_value() {
    let message = validators::min_length(3)("ab")
        .err()
        .map(|error| error.message().to_owned());

    assert_eq!(message, Some("length must be at least 3".to_owned()));
}

#[test]
fn one_of_accepts_allowed_value() {
    let result = validators::one_of(["prod", "stage"])("prod");

    assert_eq!(result, Ok(()));
}

#[test]
fn one_of_rejects_disallowed_value() {
    let message = validators::one_of(["prod", "stage"])("dev")
        .err()
        .map(|error| error.message().to_owned());

    assert_eq!(message, Some("value must be in the allowed set".to_owned()));
}

#[test]
fn u16_in_range_accepts_value_in_range() {
    let result = validators::u16_in_range(1, 10)(5);

    assert_eq!(result, Ok(()));
}

#[test]
fn u16_in_range_rejects_value_out_of_range() {
    let message = validators::u16_in_range(1, 10)(11)
        .err()
        .map(|error: ValidationError| error.message().to_owned());

    assert_eq!(message, Some("value must be between 1 and 10".to_owned()));
}

#[test]
fn url_validator_accepts_configured_scheme() {
    let result = validators::is_url_with_options(true, ["https", "postgres"])(
        "postgres://db.example.com/service",
    );

    assert_eq!(result, Ok(()));
}

#[test]
fn url_validator_rejects_unconfigured_scheme() {
    let message = validators::is_url_with_options(true, ["https"])("ftp://example.com")
        .err()
        .map(|error| error.message().to_owned());

    assert_eq!(message, Some("URL scheme is not allowed".to_owned()));
}

#[test]
fn all_of_accepts_value_when_all_copy_validators_pass() {
    let validator = validators::all_of::<i64>(vec![
        Box::new(validators::min_value(10_i64)) as validators::BoxedValueValidator<i64>,
        Box::new(validators::max_value(20_i64)),
    ]);

    assert_eq!(validator(15), Ok(()));
}

#[test]
fn all_of_rejects_value_when_any_copy_validator_fails() {
    let validator = validators::all_of::<i64>(vec![
        Box::new(validators::min_value(10_i64)) as validators::BoxedValueValidator<i64>,
        Box::new(validators::max_value(20_i64)),
    ]);

    assert_eq!(
        validator(21).err().map(|error| error.message().to_owned()),
        Some("value must be at most 20".to_owned())
    );
}
