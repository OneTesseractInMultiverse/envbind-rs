#![allow(missing_docs)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use envbind::{
    B64DecodedStringVar, BindError, Binder, BindingExt, BoolVar, EnumVar, FloatVar, IntVar,
    JsonVar, ListVar, MapEnvironment, OptionalStringVar, StringVar, U16Var,
};
use serde_json::json;

// The fixed raw-input contract for scalar and list fields has no public setter.
const RAW_LIMIT: usize = 1024 * 1024;

fn context(error: BindError) -> (&'static str, String) {
    (error.error_code(), error.variable_name().to_owned())
}

fn size_context(error: BindError) -> (&'static str, String, Option<usize>) {
    let limit = match &error {
        BindError::ValueTooLarge { max_bytes, .. } => Some(*max_bytes),
        _ => None,
    };
    (error.error_code(), error.variable_name().to_owned(), limit)
}

macro_rules! byte_boundary {
    ($module:ident, $spec:expr, $raw:expr => $expected:expr, $limit:expr) => {
        mod $module {
            use super::*;

            #[test]
            fn exact_limit_is_accepted_and_validated_once() {
                let count = Arc::new(AtomicUsize::new(0));
                let calls = Arc::clone(&count);
                let spec = $spec.validate(move |_| {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                });
                let result = Binder::new(MapEnvironment::from_pairs([("VALUE", $raw)])).bind(&spec);
                assert_eq!((result, count.load(Ordering::SeqCst)), (Ok($expected), 1));
            }

            #[test]
            fn one_byte_over_limit_skips_validation_and_propagates_through_optional() {
                let count = Arc::new(AtomicUsize::new(0));
                let calls = Arc::clone(&count);
                let spec = $spec
                    .validate(move |_| {
                        calls.fetch_add(1, Ordering::SeqCst);
                        Ok(())
                    })
                    .optional();
                let raw = format!("{} ", $raw);
                let result = Binder::new(MapEnvironment::from_pairs([("VALUE", raw)]))
                    .bind(&spec)
                    .map_err(size_context);
                assert_eq!(
                    (result, count.load(Ordering::SeqCst)),
                    (
                        Err(("value_too_large", "VALUE".to_owned(), Some($limit))),
                        0
                    )
                );
            }
        }
    };
}

byte_boundary!(string_ascii, StringVar::new("VALUE").max_bytes(3), "abc" => "abc".to_owned(), 3);
byte_boundary!(string_utf8, StringVar::new("VALUE").max_bytes(4), "éé" => "éé".to_owned(), 4);
byte_boundary!(string_zero, StringVar::new("VALUE").max_bytes(0).allow_empty(), "" => String::new(), 0);
byte_boundary!(optional_string_ascii, OptionalStringVar::new("VALUE").max_bytes(3), "abc" => Some("abc".to_owned()), 3);
byte_boundary!(optional_string_utf8, OptionalStringVar::new("VALUE").max_bytes(4), "éé" => Some("éé".to_owned()), 4);
byte_boundary!(optional_string_zero, OptionalStringVar::new("VALUE").max_bytes(0).allow_empty(), "" => Some(String::new()), 0);
byte_boundary!(json_ascii, JsonVar::new("VALUE").max_bytes(2), "{}" => json!({}), 2);
byte_boundary!(json_utf8, JsonVar::new("VALUE").max_bytes(4), "\"é\"" => json!("é"), 4);
byte_boundary!(boolean_raw, BoolVar::new("VALUE"), format!("true{}", " ".repeat(RAW_LIMIT - 4)) => true, RAW_LIMIT);
byte_boundary!(integer_raw, IntVar::new("VALUE"), "0".repeat(RAW_LIMIT) => 0, RAW_LIMIT);
byte_boundary!(float_raw, FloatVar::new("VALUE"), "0".repeat(RAW_LIMIT) => 0.0, RAW_LIMIT);
byte_boundary!(u16_raw, U16Var::new("VALUE"), "0".repeat(RAW_LIMIT) => 0, RAW_LIMIT);
byte_boundary!(enum_raw, EnumVar::new("VALUE", [("ready", 1_u8)]), format!("ready{}", " ".repeat(RAW_LIMIT - 5)) => 1, RAW_LIMIT);

#[test]
fn oversized_malformed_json_is_rejected_by_size_before_parsing() {
    let result = Binder::new(MapEnvironment::from_pairs([("VALUE", "not-json")]))
        .bind(&JsonVar::new("VALUE").max_bytes(7))
        .map_err(size_context);
    assert_eq!(
        result,
        Err(("value_too_large", "VALUE".to_owned(), Some(7)))
    );
}

#[test]
fn list_raw_limit_accepts_exact_bytes_and_rejects_one_more_before_parser() {
    let results = [RAW_LIMIT, RAW_LIMIT + 1].map(|length| {
        let count = Arc::new(AtomicUsize::new(0));
        let calls = Arc::clone(&count);
        let spec = ListVar::new("VALUE", move |item| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(item.len())
        });
        let result = Binder::new(MapEnvironment::from_pairs([("VALUE", "x".repeat(length))]))
            .bind(&spec.optional())
            .map_err(size_context);
        (result, count.load(Ordering::SeqCst))
    });
    assert_eq!(
        results,
        [
            (Ok(Some(vec![RAW_LIMIT])), 1),
            (
                Err(("value_too_large", "VALUE".to_owned(), Some(RAW_LIMIT))),
                0
            ),
        ]
    );
}

#[test]
fn list_item_limit_never_calls_parser_for_an_excess_item() {
    let results = [
        (0, "one"),
        (1, "one"),
        (1, "one,two"),
        (2, "one,two"),
        (2, "one,two,three"),
    ]
    .map(|(limit, raw)| {
        let count = Arc::new(AtomicUsize::new(0));
        let calls = Arc::clone(&count);
        let spec = ListVar::new("VALUE", move |item| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(item.to_owned())
        })
        .max_items(limit);
        let result = Binder::new(MapEnvironment::from_pairs([("VALUE", raw)]))
            .bind(&spec.optional())
            .map_err(context);
        (result, count.load(Ordering::SeqCst))
    });
    assert_eq!(
        results,
        [
            (Err(("validation_failed", "VALUE".to_owned())), 0),
            (Ok(Some(vec!["one".to_owned()])), 1),
            (Err(("validation_failed", "VALUE".to_owned())), 1),
            (Ok(Some(vec!["one".to_owned(), "two".to_owned()])), 2),
            (Err(("validation_failed", "VALUE".to_owned())), 2),
        ]
    );
}

#[test]
fn default_list_item_limit_accepts_1024_and_stops_before_item_1025() {
    let results = [1024, 1025].map(|items| {
        let count = Arc::new(AtomicUsize::new(0));
        let calls = Arc::clone(&count);
        let spec = ListVar::new("VALUE", move |_| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });
        let raw = vec!["x"; items].join(",");
        let result = Binder::new(MapEnvironment::from_pairs([("VALUE", raw)]))
            .bind(&spec)
            .map(|values| values.len())
            .map_err(context);
        (result, count.load(Ordering::SeqCst))
    });
    assert_eq!(
        results,
        [
            (Ok(1024), 1024),
            (Err(("validation_failed", "VALUE".to_owned())), 1024)
        ]
    );
}

#[test]
fn empty_delimiter_is_rejected_before_any_item_parser_calls() {
    let count = Arc::new(AtomicUsize::new(0));
    let calls = Arc::clone(&count);
    let spec = ListVar::new("VALUE", move |_| {
        calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    })
    .delimiter("");
    let result = Binder::new(MapEnvironment::from_pairs([("VALUE", "one,two")]))
        .bind(&spec)
        .map_err(context);
    assert_eq!(
        (result, count.load(Ordering::SeqCst)),
        (Err(("validation_failed", "VALUE".to_owned())), 0)
    );
}

#[test]
fn allowed_empty_list_still_has_one_item_when_limit_is_zero() {
    let result = Binder::new(MapEnvironment::from_pairs([("VALUE", "")]))
        .bind(&ListVar::strings("VALUE").allow_empty().max_items(0))
        .map_err(context);
    assert_eq!(result, Err(("validation_failed", "VALUE".to_owned())));
}

#[test]
fn base64_enforces_encoded_and_decoded_limits_before_validators() {
    // Each case supplies (decoded cap, input). Encoded bytes are checked first;
    // decoded length is checked before conversion to UTF-8.
    let results = [
        (1, "Zm8="),
        (3, "Zm9vYg=="),
        (1, "Zg== "),
        (1, "//8="),
        (1, "w6k="),
    ]
    .map(|(limit, raw)| {
        let count = Arc::new(AtomicUsize::new(0));
        let calls = Arc::clone(&count);
        let spec = B64DecodedStringVar::new("VALUE")
            .max_decoded_bytes(limit)
            .validate(move |_| {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .optional();
        let result = Binder::new(MapEnvironment::from_pairs([("VALUE", raw)]))
            .bind(&spec)
            .map_err(size_context);
        (result, count.load(Ordering::SeqCst))
    });
    assert_eq!(
        results,
        [1, 4, 4, 1, 1].map(|limit| (Err(("value_too_large", "VALUE".to_owned(), Some(limit))), 0))
    );
}

#[test]
fn base64_decoded_limit_counts_utf8_bytes() {
    let result = Binder::new(MapEnvironment::from_pairs([("VALUE", "w6k=")]))
        .bind(&B64DecodedStringVar::new("VALUE").max_decoded_bytes(2));
    assert_eq!(result, Ok("é".to_owned()));
}

#[test]
fn base64_zero_limit_accepts_allowed_empty_and_rejects_nonempty() {
    let results = ["", "Zg=="].map(|raw| {
        Binder::new(MapEnvironment::from_pairs([("VALUE", raw)]))
            .bind(
                &B64DecodedStringVar::new("VALUE")
                    .max_decoded_bytes(0)
                    .allow_empty(),
            )
            .map_err(size_context)
    });
    assert_eq!(
        results,
        [
            Ok(String::new()),
            Err(("value_too_large", "VALUE".to_owned(), Some(0)))
        ]
    );
}

#[test]
fn base64_maximum_configured_limit_does_not_overflow_encoded_bound() {
    let result = Binder::new(MapEnvironment::from_pairs([("VALUE", "Zm9v")]))
        .bind(&B64DecodedStringVar::new("VALUE").max_decoded_bytes(usize::MAX));
    assert_eq!(result, Ok("foo".to_owned()));
}

#[test]
fn json_rejects_malformed_or_trailing_data_without_exposing_input() {
    let results = [
        "[",
        "{\"synthetic-json-secret\":}",
        "null true",
        "{\"a\":1,}",
    ]
    .map(|raw| {
        Binder::new(MapEnvironment::from_pairs([("VALUE", raw)]))
            .bind(&JsonVar::new("VALUE").optional())
            .map_err(|error| {
                let exposed =
                    format!("{error} {error:?} {error:#?}").contains("synthetic-json-secret");
                (context(error), exposed)
            })
    });
    assert_eq!(
        results,
        std::array::from_fn(|_| Err((("parse_variable", "VALUE".to_owned()), false)))
    );
}

#[test]
fn json_accepts_moderate_nesting_and_rejects_excessive_nesting() {
    let results = [32, 256].map(|depth| {
        let raw = format!("{}null{}", "[".repeat(depth), "]".repeat(depth));
        Binder::new(MapEnvironment::from_pairs([("VALUE", raw)]))
            .bind(&JsonVar::new("VALUE"))
            .map(|_| ())
            .map_err(context)
    });
    assert_eq!(
        results,
        [Ok(()), Err(("parse_variable", "VALUE".to_owned()))]
    );
}
