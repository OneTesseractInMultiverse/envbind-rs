#![allow(missing_docs)]

use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use envbind::{
    B64DecodedStringVar, BindError, Binder, BindingExt, BoolVar, EnumVar, Environment,
    EnvironmentError, FloatVar, IntVar, JsonVar, ListVar, MapEnvironment, OptionalStringVar,
    StringVar, U16Var, ValidationError,
};
use serde_json::json;

fn context(error: BindError) -> (&'static str, String) {
    (error.error_code(), error.variable_name().to_owned())
}

fn diagnostic(error: BindError, marker: &str) -> (&'static str, String, bool, bool, bool) {
    (
        error.error_code(),
        error.variable_name().to_owned(),
        error.to_string().contains(marker),
        format!("{error:?} {error:#?}").contains(marker),
        error
            .source()
            .is_some_and(|source| format!("{source} {source:?} {source:#?}").contains(marker)),
    )
}

struct FailingEnvironment;

impl Environment for FailingEnvironment {
    fn get(&self, _name: &str) -> Result<Option<String>, EnvironmentError> {
        Err(EnvironmentError::read("synthetic-adapter-secret"))
    }
}

// Each invocation lists the field-specific expectations. Common test names
// make the documented contract matrix easy to filter with cargo test.
macro_rules! field_contracts {
    ($module:ident, $spec:expr,
     present: $raw:expr => $expected:expr,
     fallback: $fallback:expr => $defaulted:expr,
     absent: $missing:expr, empty: $empty:expr, optional_absent: $optional_absent:expr,
     whitespace: $whitespace:expr, allowed_empty: $allowed_empty:expr
     $(, malformed: $malformed:expr => $parse_code:expr)? $(,)?) => {
        mod $module {
            use super::*;

            #[test]
            fn present_input_parses() {
                let result = Binder::new(MapEnvironment::from_pairs([("VALUE", $raw)]))
                    .bind(&$spec);
                assert_eq!(result, Ok($expected));
            }

            #[test]
            fn missing_input_has_the_documented_result() {
                let result = Binder::new(MapEnvironment::new()).bind(&$spec).map_err(context);
                assert_eq!(result, $missing);
            }

            #[test]
            fn empty_input_has_the_documented_result() {
                let result = Binder::new(MapEnvironment::from_pairs([("VALUE", "")]))
                    .bind(&$spec).map_err(context);
                assert_eq!(result, $empty);
            }

            #[test]
            fn whitespace_is_input_not_absence() {
                let result = Binder::new(MapEnvironment::from_pairs([("VALUE", " \t ")]))
                    .bind(&$spec.default($fallback)).map_err(context);
                assert_eq!(result, $whitespace);
            }

            #[test]
            fn optional_wraps_successful_values() {
                let result = Binder::new(MapEnvironment::from_pairs([("VALUE", $raw)]))
                    .bind(&$spec.optional());
                assert_eq!(result, Ok(Some($expected)));
            }

            #[test]
            fn optional_handles_missing_and_empty_without_defaults() {
                let results = [MapEnvironment::new(), MapEnvironment::from_pairs([("VALUE", "")])]
                    .map(|environment| Binder::new(environment).bind(&$spec.optional()));
                assert_eq!(results, [$optional_absent, $optional_absent]);
            }

            #[test]
            fn optional_keeps_missing_and_empty_defaults() {
                let results = [MapEnvironment::new(), MapEnvironment::from_pairs([("VALUE", "")])]
                    .map(|environment| Binder::new(environment).bind(&$spec.default($fallback).optional()));
                assert_eq!(results, [Ok(Some($defaulted)), Ok(Some($defaulted))]);
            }

            #[test]
            fn allowed_empty_bypasses_default_and_reaches_the_parser() {
                let result = Binder::new(MapEnvironment::from_pairs([("VALUE", "")]))
                    .bind(&$spec.default($fallback).allow_empty().optional()).map_err(context);
                assert_eq!(result, $allowed_empty);
            }

            #[test]
            fn allowed_empty_does_not_replace_a_missing_default() {
                let result = Binder::new(MapEnvironment::new())
                    .bind(&$spec.default($fallback).allow_empty().optional());
                assert_eq!(result, Ok(Some($defaulted)));
            }

            #[test]
            fn adapter_failure_ignores_defaults_and_propagates_through_optional() {
                let result = Binder::new(FailingEnvironment)
                    .bind(&$spec.default($fallback).sensitive(false).optional())
                    .map_err(|error| diagnostic(error, "synthetic-adapter-secret"));
                assert_eq!(result, Err(("environment_error", "VALUE".to_owned(), false, false, false)));
            }

            #[test]
            fn successful_validator_runs_on_the_parsed_value() {
                let count = Arc::new(AtomicUsize::new(0));
                let calls = Arc::clone(&count);
                let spec = $spec.validate(move |_| { calls.fetch_add(1, Ordering::SeqCst); Ok(()) });
                let result = Binder::new(MapEnvironment::from_pairs([("VALUE", $raw)])).bind(&spec);
                assert_eq!((result, count.load(Ordering::SeqCst)), (Ok($expected), 1));
            }

            #[test]
            fn sensitive_validation_failure_propagates_without_diagnostics() {
                let spec = $spec.default($fallback)
                    .validate(|value| Err(ValidationError::new(format!("synthetic-validator-secret:{value:?}"))))
                    .optional();
                let result = Binder::new(MapEnvironment::from_pairs([("VALUE", $raw)]))
                    .bind(&spec).map_err(|error| diagnostic(error, "synthetic-validator-secret"));
                assert_eq!(result, Err(("validation_failed", "VALUE".to_owned(), false, false, false)));
            }

            #[test]
            fn nonsensitive_validation_failure_keeps_explicit_diagnostics() {
                let spec = $spec.sensitive(false)
                    .validate(|value| Err(ValidationError::new(format!("synthetic-public-detail:{value:?}"))))
                    .optional();
                let result = Binder::new(MapEnvironment::from_pairs([("VALUE", $raw)]))
                    .bind(&spec).map_err(|error| diagnostic(error, "synthetic-public-detail"));
                assert_eq!(result, Err(("validation_failed", "VALUE".to_owned(), true, true, false)));
            }

            #[test]
            fn validators_stop_in_registration_order_at_first_failure() {
                let count = Arc::new(AtomicUsize::new(0));
                let first = Arc::clone(&count);
                let second = Arc::clone(&count);
                let third = Arc::clone(&count);
                let spec = $spec
                    .validate(move |_| { first.fetch_add(1, Ordering::SeqCst); Ok(()) })
                    .validate(move |_| { second.fetch_add(10, Ordering::SeqCst); Err(ValidationError::new("stop")) })
                    .validate(move |_| { third.fetch_add(100, Ordering::SeqCst); Ok(()) });
                let result = Binder::new(MapEnvironment::from_pairs([("VALUE", $raw)]))
                    .bind(&spec).map_err(context);
                assert_eq!((result, count.load(Ordering::SeqCst)), (Err(("validation_failed", "VALUE".to_owned())), 11));
            }

            $(
                #[test]
                fn malformed_input_does_not_fall_back_or_become_optional_absence() {
                    let spec = $spec.default($fallback).optional();
                    let result = Binder::new(MapEnvironment::from_pairs([("VALUE", $malformed)]))
                        .bind(&spec).map_err(|error| diagnostic(error, "synthetic-parser-secret"));
                    assert_eq!(result, Err(($parse_code, "VALUE".to_owned(), false, false, false)));
                }
            )?
        }
    };
}

field_contracts!(string, StringVar::new("VALUE"),
    present: "input" => "input".to_owned(), fallback: "fallback" => "fallback".to_owned(),
    absent: Err(("missing_variable", "VALUE".to_owned())), empty: Err(("empty_variable", "VALUE".to_owned())),
    optional_absent: Ok(None), whitespace: Ok(" \t ".to_owned()), allowed_empty: Ok(Some(String::new())));

field_contracts!(optional_string, OptionalStringVar::new("VALUE"),
    present: "input" => Some("input".to_owned()), fallback: "fallback" => Some("fallback".to_owned()),
    absent: Ok(None), empty: Ok(None), optional_absent: Ok(Some(None)),
    whitespace: Ok(Some(" \t ".to_owned())), allowed_empty: Ok(Some(Some(String::new()))));

field_contracts!(boolean, BoolVar::new("VALUE"),
    present: "true" => true, fallback: false => false,
    absent: Err(("missing_variable", "VALUE".to_owned())), empty: Err(("empty_variable", "VALUE".to_owned())),
    optional_absent: Ok(None), whitespace: Err(("invalid_boolean", "VALUE".to_owned())),
    allowed_empty: Err(("invalid_boolean", "VALUE".to_owned())), malformed: "synthetic-parser-secret" => "invalid_boolean");

field_contracts!(integer, IntVar::new("VALUE"),
    present: "42" => 42, fallback: -1 => -1,
    absent: Err(("missing_variable", "VALUE".to_owned())), empty: Err(("empty_variable", "VALUE".to_owned())),
    optional_absent: Ok(None), whitespace: Err(("parse_variable", "VALUE".to_owned())),
    allowed_empty: Err(("parse_variable", "VALUE".to_owned())), malformed: "synthetic-parser-secret" => "parse_variable");

field_contracts!(float, FloatVar::new("VALUE"),
    present: "1.25" => 1.25, fallback: -0.5 => -0.5,
    absent: Err(("missing_variable", "VALUE".to_owned())), empty: Err(("empty_variable", "VALUE".to_owned())),
    optional_absent: Ok(None), whitespace: Err(("parse_variable", "VALUE".to_owned())),
    allowed_empty: Err(("parse_variable", "VALUE".to_owned())), malformed: "synthetic-parser-secret" => "parse_variable");

field_contracts!(u16_value, U16Var::new("VALUE"),
    present: "8080" => 8080, fallback: 9090 => 9090,
    absent: Err(("missing_variable", "VALUE".to_owned())), empty: Err(("empty_variable", "VALUE".to_owned())),
    optional_absent: Ok(None), whitespace: Err(("parse_variable", "VALUE".to_owned())),
    allowed_empty: Err(("parse_variable", "VALUE".to_owned())), malformed: "synthetic-parser-secret" => "parse_variable");

field_contracts!(json_value, JsonVar::new("VALUE"),
    present: r#"{"port":8080}"# => json!({"port":8080}), fallback: json!(null) => json!(null),
    absent: Err(("missing_variable", "VALUE".to_owned())), empty: Err(("empty_variable", "VALUE".to_owned())),
    optional_absent: Ok(None), whitespace: Err(("parse_variable", "VALUE".to_owned())),
    allowed_empty: Err(("parse_variable", "VALUE".to_owned())), malformed: "synthetic-parser-secret" => "parse_variable");

field_contracts!(base64, B64DecodedStringVar::new("VALUE"),
    present: "aW5wdXQ=" => "input".to_owned(), fallback: "decoded fallback" => "decoded fallback".to_owned(),
    absent: Err(("missing_variable", "VALUE".to_owned())), empty: Err(("empty_variable", "VALUE".to_owned())),
    optional_absent: Ok(None), whitespace: Err(("parse_variable", "VALUE".to_owned())),
    allowed_empty: Ok(Some(String::new())), malformed: "synthetic-parser-secret" => "parse_variable");

field_contracts!(enumeration, EnumVar::new("VALUE", [("ready", 1_u8)]),
    present: "READY" => 1_u8, fallback: 2_u8 => 2_u8,
    absent: Err(("missing_variable", "VALUE".to_owned())), empty: Err(("empty_variable", "VALUE".to_owned())),
    optional_absent: Ok(None), whitespace: Err(("parse_variable", "VALUE".to_owned())),
    allowed_empty: Err(("parse_variable", "VALUE".to_owned())), malformed: "synthetic-parser-secret" => "parse_variable");

field_contracts!(list, ListVar::integers("VALUE"),
    present: "1, 2" => vec![1,2], fallback: vec![3] => vec![3],
    absent: Err(("missing_variable", "VALUE".to_owned())), empty: Err(("empty_variable", "VALUE".to_owned())),
    optional_absent: Ok(None), whitespace: Err(("validation_failed", "VALUE".to_owned())),
    allowed_empty: Err(("validation_failed", "VALUE".to_owned())), malformed: "synthetic-parser-secret" => "validation_failed");
