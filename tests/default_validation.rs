#![allow(missing_docs)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use envbind::{
    B64DecodedStringVar, Binder, BindingExt, BoolVar, EnumVar, FloatVar, IntVar, JsonVar, ListVar,
    MapEnvironment, OptionalStringVar, StringVar, U16Var, ValidationError, validators,
};
use serde_json::json;

// Exercise the same public fallback contract for every field's binding path.
macro_rules! default_validation_tests {
    ($module:ident, $spec:expr, $expected:expr, $raw:expr) => {
        mod $module {
            use super::*;

            #[test]
            fn defaults_skip_validators_unless_enabled() {
                let spec = $spec.validate(|_| Err(ValidationError::new("invalid fallback")));
                let results = [
                    MapEnvironment::new(),
                    MapEnvironment::from_pairs([("VALUE", "")]),
                ]
                .map(|environment| Binder::new(environment).bind(&spec));

                assert_eq!(results, [Ok($expected), Ok($expected)]);
            }

            #[test]
            fn missing_input_validates_default() {
                let spec = $spec
                    .validate_default()
                    .validate(|_| Err(ValidationError::new("invalid fallback")));
                let result = Binder::new(MapEnvironment::new())
                    .bind(&spec)
                    .map_err(|error| error.error_code());

                assert_eq!(result, Err("validation_failed"));
            }

            #[test]
            fn empty_input_validates_default() {
                let spec = $spec
                    .validate_default()
                    .validate(|_| Err(ValidationError::new("invalid fallback")));
                let result = Binder::new(MapEnvironment::from_pairs([("VALUE", "")]))
                    .bind(&spec)
                    .map_err(|error| error.error_code());

                assert_eq!(result, Err("validation_failed"));
            }

            #[test]
            fn each_validator_runs_once_per_binding() {
                let first = Arc::new(AtomicUsize::new(0));
                let second = Arc::new(AtomicUsize::new(0));
                let first_calls = Arc::clone(&first);
                let second_calls = Arc::clone(&second);
                let spec = $spec
                    .validate(move |_| {
                        first_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(())
                    })
                    .validate_default()
                    .validate_default()
                    .validate(move |_| {
                        second_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(())
                    });
                let results = [
                    MapEnvironment::new(),
                    MapEnvironment::new(),
                    MapEnvironment::from_pairs([("VALUE", "")]),
                    MapEnvironment::from_pairs([("VALUE", $raw)]),
                ]
                .map(|environment| Binder::new(environment).bind(&spec));

                assert_eq!(
                    (
                        results,
                        first.load(Ordering::SeqCst),
                        second.load(Ordering::SeqCst)
                    ),
                    (
                        [Ok($expected), Ok($expected), Ok($expected), Ok($expected)],
                        4,
                        4
                    )
                );
            }

            #[test]
            fn sensitive_default_errors_redact_custom_details() {
                let spec = $spec
                    .validate_default()
                    .validate(|_| Err(ValidationError::new("private fallback")));
                let result = Binder::new(MapEnvironment::new())
                    .bind(&spec)
                    .map_err(|error| {
                        (
                            error.error_code(),
                            error.variable_name().to_owned(),
                            error.to_string().contains("private fallback"),
                            format!("{error:?} {error:#?}").contains("private fallback"),
                        )
                    });

                assert_eq!(
                    result,
                    Err(("validation_failed", "VALUE".to_owned(), false, false))
                );
            }

            #[test]
            fn nonsensitive_default_errors_keep_custom_details() {
                let spec = $spec
                    .validate_default()
                    .sensitive(false)
                    .validate(|_| Err(ValidationError::new("safe fallback detail")));
                let result = Binder::new(MapEnvironment::new())
                    .bind(&spec)
                    .map_err(|error| {
                        (
                            error.error_code(),
                            error.to_string().contains("safe fallback detail"),
                        )
                    });

                assert_eq!(result, Err(("validation_failed", true)));
            }
        }
    };
}

default_validation_tests!(
    string,
    StringVar::new("VALUE").default("foo"),
    "foo".to_owned(),
    "foo"
);
default_validation_tests!(
    optional_string,
    OptionalStringVar::new("VALUE").default("foo"),
    Some("foo".to_owned()),
    "foo"
);
default_validation_tests!(boolean, BoolVar::new("VALUE").default(true), true, "true");
default_validation_tests!(integer, IntVar::new("VALUE").default(10), 10, "10");
default_validation_tests!(float, FloatVar::new("VALUE").default(1.5), 1.5, "1.5");
default_validation_tests!(u16, U16Var::new("VALUE").default(10), 10, "10");
default_validation_tests!(
    list,
    ListVar::integers("VALUE").default(vec![10]),
    vec![10],
    "10"
);
default_validation_tests!(
    json_value,
    JsonVar::new("VALUE").default(json!({"port": 10})),
    json!({"port": 10}),
    r#"{"port":10}"#
);
default_validation_tests!(
    enumeration,
    EnumVar::new("VALUE", [("ten", 10)]).default(10),
    10,
    "ten"
);
default_validation_tests!(
    base64,
    B64DecodedStringVar::new("VALUE").default("foo"),
    "foo".to_owned(),
    "Zm9v"
);

#[test]
fn invalid_port_default_fails_the_same_rule_as_environment_input() {
    let spec = U16Var::new("PORT")
        .validate_default()
        .default(0)
        .validate(validators::u16_in_range(1, 65_535));
    let results = [
        MapEnvironment::new(),
        MapEnvironment::from_pairs([("PORT", "0")]),
    ]
    .map(|environment| {
        Binder::new(environment)
            .bind(&spec)
            .map_err(|error| error.error_code())
    });

    assert_eq!(
        results,
        [Err("validation_failed"), Err("validation_failed")]
    );
}

#[test]
fn fallback_validators_stop_at_first_failure_in_registration_order() {
    let calls = Arc::new(AtomicUsize::new(0));
    let first = Arc::clone(&calls);
    let second = Arc::clone(&calls);
    let third = Arc::clone(&calls);
    let spec = U16Var::new("PORT")
        .default(8080)
        .validate_default()
        .sensitive(false)
        .validate(move |_| {
            first.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .validate(move |_| {
            let previous = second.fetch_add(1, Ordering::SeqCst);
            Err(ValidationError::new(format!(
                "failed after {previous} validator"
            )))
        })
        .validate(move |_| {
            third.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });
    let result = Binder::new(MapEnvironment::new())
        .bind(&spec)
        .map_err(|error| {
            (
                error.error_code(),
                error.to_string().contains("failed after 1 validator"),
            )
        });

    assert_eq!(
        (result, calls.load(Ordering::SeqCst)),
        (Err(("validation_failed", true)), 2)
    );
}

#[test]
fn allow_empty_still_validates_fallback_when_missing() {
    let result = Binder::new(MapEnvironment::new())
        .bind(
            &StringVar::new("NAME")
                .default("")
                .allow_empty()
                .validate_default()
                .validate(validators::min_length(1)),
        )
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("validation_failed"));
}

#[test]
fn allow_empty_validates_explicit_empty_string_instead_of_fallback() {
    let result = Binder::new(MapEnvironment::from_pairs([("NAME", "")]))
        .bind(
            &StringVar::new("NAME")
                .default("worker")
                .allow_empty()
                .validate_default()
                .validate(validators::min_length(1)),
        )
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("validation_failed"));
}

#[test]
fn allow_empty_parses_explicit_empty_numeric_input_instead_of_fallback() {
    let result = Binder::new(MapEnvironment::from_pairs([("PORT", "")]))
        .bind(
            &U16Var::new("PORT")
                .default(8080)
                .allow_empty()
                .validate_default(),
        )
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("parse_variable"));
}

#[test]
fn optional_string_without_fallback_does_not_validate_absence() {
    let spec = OptionalStringVar::new("NAME")
        .validate_default()
        .validate(|_| Err(ValidationError::new("should not run")));
    let results = [
        MapEnvironment::new(),
        MapEnvironment::from_pairs([("NAME", "")]),
    ]
    .map(|environment| Binder::new(environment).bind(&spec));

    assert_eq!(results, [Ok(None), Ok(None)]);
}

#[test]
fn optional_string_allow_empty_validates_some_empty_string() {
    let result = Binder::new(MapEnvironment::from_pairs([("NAME", "")]))
        .bind(
            &OptionalStringVar::new("NAME")
                .default("worker")
                .allow_empty()
                .validate_default()
                .validate(validators::min_length(1)),
        )
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("validation_failed"));
}

#[test]
fn optional_wrapper_does_not_swallow_invalid_fallback() {
    let result = Binder::new(MapEnvironment::new())
        .bind(
            &U16Var::new("PORT")
                .default(0)
                .validate_default()
                .validate(validators::u16_in_range(1, 65_535))
                .optional(),
        )
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("validation_failed"));
}

#[test]
fn optional_wrapper_returns_validated_fallback() {
    let result = Binder::new(MapEnvironment::new()).bind(
        &U16Var::new("PORT")
            .default(8080)
            .validate_default()
            .validate(validators::u16_in_range(1, 65_535))
            .optional(),
    );

    assert_eq!(result, Ok(Some(8080)));
}

#[test]
fn optional_wrapper_without_fallback_does_not_validate_absence() {
    let spec = U16Var::new("PORT")
        .validate_default()
        .validate(|_| Err(ValidationError::new("should not run")))
        .optional();
    let results = [
        MapEnvironment::new(),
        MapEnvironment::from_pairs([("PORT", "")]),
    ]
    .map(|environment| Binder::new(environment).bind(&spec));

    assert_eq!(results, [Ok(None), Ok(None)]);
}

#[test]
fn empty_typed_string_fallback_only_runs_attached_validators() {
    let result = Binder::new(MapEnvironment::new()).bind(
        &StringVar::new("NAME")
            .default("")
            .validate_default()
            .validate(validators::max_length(0)),
    );

    assert_eq!(result, Ok(String::new()));
}

#[test]
fn optional_wrapper_does_not_swallow_empty_fallback_validation_error() {
    let result = Binder::new(MapEnvironment::new())
        .bind(
            &StringVar::new("NAME")
                .default("")
                .validate_default()
                .validate(validators::min_length(1))
                .optional(),
        )
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("validation_failed"));
}

#[test]
fn optional_string_empty_fallback_is_some_and_runs_validators() {
    let result = Binder::new(MapEnvironment::new())
        .bind(
            &OptionalStringVar::new("NAME")
                .default("")
                .validate_default()
                .validate(validators::min_length(1)),
        )
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("validation_failed"));
}

#[test]
fn raw_byte_limit_does_not_restrict_validated_string_fallback() {
    let result = Binder::new(MapEnvironment::new()).bind(
        &StringVar::new("NAME")
            .default("worker")
            .max_bytes(1)
            .validate_default()
            .validate(validators::min_length(6)),
    );

    assert_eq!(result, Ok("worker".to_owned()));
}

#[test]
fn raw_byte_limit_does_not_restrict_validated_optional_string_fallback() {
    let result = Binder::new(MapEnvironment::new()).bind(
        &OptionalStringVar::new("NAME")
            .default("worker")
            .max_bytes(1)
            .validate_default()
            .validate(validators::min_length(6)),
    );

    assert_eq!(result, Ok(Some("worker".to_owned())));
}

#[test]
fn decoded_byte_limit_does_not_restrict_or_decode_base64_fallback() {
    let result = Binder::new(MapEnvironment::new()).bind(
        &B64DecodedStringVar::new("CERT")
            .default("plain text!")
            .max_decoded_bytes(1)
            .validate_default()
            .validate(validators::min_length(2)),
    );

    assert_eq!(result, Ok("plain text!".to_owned()));
}

#[test]
fn raw_json_byte_limit_does_not_restrict_validated_json_fallback() {
    let result = Binder::new(MapEnvironment::new()).bind(
        &JsonVar::new("CONFIG")
            .default(json!({"port": 8080}))
            .max_bytes(1)
            .validate_default()
            .validate(|value| {
                if value["port"] == 8080 {
                    Ok(())
                } else {
                    Err(ValidationError::new("port must be 8080"))
                }
            }),
    );

    assert_eq!(result, Ok(json!({"port": 8080})));
}

#[test]
fn json_null_is_a_typed_fallback_that_runs_validators() {
    let result = Binder::new(MapEnvironment::new())
        .bind(
            &JsonVar::new("CONFIG")
                .default(json!(null))
                .validate_default()
                .validate(|value| {
                    if value.is_object() {
                        Ok(())
                    } else {
                        Err(ValidationError::new("object required"))
                    }
                }),
        )
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("validation_failed"));
}

#[test]
fn typed_list_fallback_skips_parser_delimiter_and_item_limits() {
    #[derive(Clone, Debug, PartialEq)]
    struct Item(u16);

    let result = Binder::new(MapEnvironment::new()).bind(
        &ListVar::new("ITEMS", |_| {
            Err::<Item, _>(ValidationError::new("parser must not run"))
        })
        .default(vec![Item(10), Item(20)])
        .delimiter("")
        .max_items(0)
        .validate_default()
        .validate(|items| {
            if items == [Item(10), Item(20)] {
                Ok(())
            } else {
                Err(ValidationError::new("unexpected items"))
            }
        }),
    );

    assert_eq!(result, Ok(vec![Item(10), Item(20)]));
}

#[test]
fn enum_fallback_can_be_validated_without_an_input_label() {
    #[derive(Clone, Debug, PartialEq)]
    enum Mode {
        Active,
        Standby,
    }

    let result = Binder::new(MapEnvironment::new()).bind(
        &EnumVar::new("MODE", [("active", Mode::Active)])
            .default(Mode::Standby)
            .validate_default()
            .validate(|mode| {
                if mode == &Mode::Standby {
                    Ok(())
                } else {
                    Err(ValidationError::new("standby required"))
                }
            }),
    );

    assert_eq!(result, Ok(Mode::Standby));
}

#[test]
fn malformed_present_input_does_not_use_fallback() {
    let result = Binder::new(MapEnvironment::from_pairs([("PORT", "bad")]))
        .bind(&U16Var::new("PORT").default(8080).validate_default())
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("parse_variable"));
}

#[test]
fn raw_limits_still_apply_when_fallback_validation_is_enabled() {
    let result = Binder::new(MapEnvironment::from_pairs([("NAME", "worker")]))
        .bind(
            &StringVar::new("NAME")
                .default("ok")
                .max_bytes(2)
                .validate_default(),
        )
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("value_too_large"));
}
