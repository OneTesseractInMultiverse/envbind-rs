#![allow(missing_docs)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use envbind::{
    BindError, Binder, BindingExt, BoolVar, EnumVar, FloatVar, IntVar, ListVar, MapEnvironment,
    U16Var, ValidationError,
};

fn context(error: BindError) -> (&'static str, String) {
    (error.error_code(), error.variable_name().to_owned())
}

#[test]
fn integer_accepts_both_bounds_and_rejects_overflow_through_optional() {
    let results = [
        "-9223372036854775808",
        "9223372036854775807",
        "-9223372036854775809",
        "9223372036854775808",
    ]
    .map(|raw| {
        Binder::new(MapEnvironment::from_pairs([("VALUE", raw)]))
            .bind(&IntVar::new("VALUE").default(0).optional())
            .map_err(context)
    });
    assert_eq!(
        results,
        [
            Ok(Some(i64::MIN)),
            Ok(Some(i64::MAX)),
            Err(("parse_variable", "VALUE".to_owned())),
            Err(("parse_variable", "VALUE".to_owned()))
        ]
    );
}

#[test]
fn u16_accepts_both_bounds_and_rejects_overflow_through_optional() {
    let results = ["0", "65535", "-1", "65536"].map(|raw| {
        Binder::new(MapEnvironment::from_pairs([("VALUE", raw)]))
            .bind(&U16Var::new("VALUE").default(1).optional())
            .map_err(context)
    });
    assert_eq!(
        results,
        [
            Ok(Some(0)),
            Ok(Some(u16::MAX)),
            Err(("parse_variable", "VALUE".to_owned())),
            Err(("parse_variable", "VALUE".to_owned()))
        ]
    );
}

#[test]
fn float_accepts_finite_scientific_notation_and_extremes() {
    let results = [
        "1.25e2",
        "-2.5e-1",
        "1.7976931348623157e308",
        "-1.7976931348623157e308",
    ]
    .map(|raw| {
        Binder::new(MapEnvironment::from_pairs([("VALUE", raw)])).bind(&FloatVar::new("VALUE"))
    });
    assert_eq!(results, [Ok(125.0), Ok(-0.25), Ok(f64::MAX), Ok(f64::MIN)]);
}

#[test]
fn booleans_accept_all_documented_tokens_with_case_and_whitespace() {
    let results = [
        "1", "TrUe", "yes", "ON", "y", "T", "0", "FaLsE", "no", "OFF", "n", "F",
    ]
    .map(|token| {
        Binder::new(MapEnvironment::from_pairs([(
            "VALUE",
            format!(" \t{token}\n"),
        )]))
        .bind(&BoolVar::new("VALUE"))
    });
    assert_eq!(
        results,
        [
            Ok(true),
            Ok(true),
            Ok(true),
            Ok(true),
            Ok(true),
            Ok(true),
            Ok(false),
            Ok(false),
            Ok(false),
            Ok(false),
            Ok(false),
            Ok(false)
        ]
    );
}

#[test]
fn float_list_parses_signed_and_scientific_notation() {
    let result = Binder::new(MapEnvironment::from_pairs([(
        "VALUE",
        "1.25, -2.5e-1, 3e2",
    )]))
    .bind(&ListVar::floats("VALUE"));
    assert_eq!(result, Ok(vec![1.25, -0.25, 300.0]));
}

macro_rules! invalid_list_items {
    ($name:ident, $spec:expr, $inputs:expr) => {
        #[test]
        fn $name() {
            let results = $inputs.map(|raw| {
                Binder::new(MapEnvironment::from_pairs([("VALUE", raw)]))
                    .bind(&$spec.optional())
                    .map_err(context)
            });
            assert_eq!(
                results,
                std::array::from_fn(|_| Err(("validation_failed", "VALUE".to_owned())))
            );
        }
    };
}

invalid_list_items!(
    integer_list_rejects_malformed_overflow_and_empty_items,
    ListVar::integers("VALUE"),
    [
        "1,bad",
        "1,9223372036854775808",
        "1,-9223372036854775809",
        "1,",
        "1,,2"
    ]
);
invalid_list_items!(
    u16_list_rejects_negative_and_overflow_items,
    ListVar::u16s("VALUE"),
    ["1,-1", "1,65536", "1,bad"]
);
invalid_list_items!(
    float_list_rejects_malformed_and_empty_items,
    ListVar::floats("VALUE"),
    ["1.0,bad", "1.0,", "1.0, "]
);
invalid_list_items!(
    boolean_list_rejects_malformed_and_empty_items,
    ListVar::booleans("VALUE"),
    ["true,maybe", "true,", "true, "]
);
invalid_list_items!(
    enum_list_rejects_unknown_labels,
    ListVar::enumeration("VALUE", [("ready", 1_u8)]),
    ["ready,unknown", "ready,"]
);
invalid_list_items!(
    enum_list_rejects_wrong_case_when_case_sensitive,
    ListVar::case_sensitive_enumeration("VALUE", [("ready", 1_u8)]),
    ["READY", "ready,Ready"]
);

#[test]
fn string_list_custom_delimiters_preserve_trailing_and_consecutive_empty_items() {
    let results = [("::", "a::::b::"), ("💡", "a💡💡b💡")].map(|(delimiter, raw)| {
        Binder::new(MapEnvironment::from_pairs([("VALUE", raw)]))
            .bind(&ListVar::strings("VALUE").delimiter(delimiter))
    });
    assert_eq!(
        results,
        std::array::from_fn(|_| Ok(vec![
            "a".to_owned(),
            String::new(),
            "b".to_owned(),
            String::new()
        ]))
    );
}

#[test]
fn allowed_empty_string_list_parses_one_empty_item() {
    let result = Binder::new(MapEnvironment::from_pairs([("VALUE", "")])).bind(
        &ListVar::strings("VALUE")
            .default(vec!["fallback".to_owned()])
            .allow_empty(),
    );
    assert_eq!(result, Ok(vec![String::new()]));
}

#[test]
fn string_list_whitespace_policy_preserves_or_trims_unicode_space() {
    let environment = MapEnvironment::from_pairs([("VALUE", "\u{2003}one\t, two\n")]);
    let binder = Binder::new(environment);
    let results = [
        binder.bind(&ListVar::strings("VALUE")),
        binder.bind(&ListVar::strings("VALUE").keep_whitespace()),
    ];
    assert_eq!(
        results,
        [
            Ok(vec!["one".to_owned(), "two".to_owned()]),
            Ok(vec!["\u{2003}one\t".to_owned(), " two\n".to_owned()])
        ]
    );
}

#[test]
fn keeping_whitespace_exposes_spaces_to_integer_parser() {
    let result = Binder::new(MapEnvironment::from_pairs([("VALUE", "1, 2")]))
        .bind(&ListVar::integers("VALUE").keep_whitespace())
        .map_err(context);
    assert_eq!(result, Err(("validation_failed", "VALUE".to_owned())));
}

#[test]
fn boolean_item_parser_trims_even_when_split_whitespace_is_kept() {
    let result = Binder::new(MapEnvironment::from_pairs([("VALUE", " true , NO ")]))
        .bind(&ListVar::booleans("VALUE").keep_whitespace());
    assert_eq!(result, Ok(vec![true, false]));
}

#[test]
fn enum_item_parser_trims_even_when_split_whitespace_is_kept() {
    let result = Binder::new(MapEnvironment::from_pairs([("VALUE", " ready , READY ")]))
        .bind(&ListVar::enumeration("VALUE", [("ready", 1_u8)]).keep_whitespace());
    assert_eq!(result, Ok(vec![1, 1]));
}

#[test]
fn list_parser_failure_stops_later_items_and_skips_list_validation() {
    let count = Arc::new(AtomicUsize::new(0));
    let parse_calls = Arc::clone(&count);
    let validation_calls = Arc::clone(&count);
    let spec = ListVar::new("VALUE", move |item| {
        parse_calls.fetch_add(1, Ordering::SeqCst);
        if item == "bad" {
            Err(ValidationError::new("invalid item"))
        } else {
            Ok(())
        }
    })
    .validate(move |_| {
        validation_calls.fetch_add(100, Ordering::SeqCst);
        Ok(())
    });
    let result = Binder::new(MapEnvironment::from_pairs([(
        "VALUE",
        "good,bad,unvisited",
    )]))
    .bind(&spec)
    .map_err(context);
    assert_eq!(
        (result, count.load(Ordering::SeqCst)),
        (Err(("validation_failed", "VALUE".to_owned())), 2)
    );
}

#[test]
fn custom_list_parser_diagnostics_follow_sensitivity_through_optional() {
    let results = [true, false].map(|sensitive| {
        let spec = ListVar::<()>::new("VALUE", |item| {
            Err(ValidationError::new(format!("bad item: {item}")))
        })
        .sensitive(sensitive)
        .optional();
        Binder::new(MapEnvironment::from_pairs([(
            "VALUE",
            "synthetic-item-secret",
        )]))
        .bind(&spec)
        .map_err(|error| {
            let display = error.to_string().contains("synthetic-item-secret");
            let debug = format!("{error:?} {error:#?}").contains("synthetic-item-secret");
            (context(error), display, debug)
        })
    });
    assert_eq!(
        results,
        [
            Err((("validation_failed", "VALUE".to_owned()), false, false)),
            Err((("validation_failed", "VALUE".to_owned()), true, true))
        ]
    );
}

#[test]
fn enum_rejects_wrong_case_when_case_sensitive() {
    let result = Binder::new(MapEnvironment::from_pairs([("VALUE", "READY")]))
        .bind(
            &EnumVar::new("VALUE", [("ready", 1_u8)])
                .case_sensitive()
                .optional(),
        )
        .map_err(context);
    assert_eq!(result, Err(("parse_variable", "VALUE".to_owned())));
}

#[test]
fn enum_can_parse_an_explicit_empty_label_when_empty_is_allowed() {
    let result = Binder::new(MapEnvironment::from_pairs([("VALUE", "")]))
        .bind(&EnumVar::new("VALUE", [("", 1_u8)]).default(2).allow_empty());
    assert_eq!(result, Ok(1));
}
