#![allow(missing_docs)]

use envbind::{
    B64DecodedStringVar, Binder, BindingExt, BoolVar, EnumVar, FloatVar, IntVar, JsonVar, ListVar,
    MapEnvironment, StringVar, ValidationError, validators,
};
use serde_json::json;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Mode {
    Blue,
    Green,
}

#[test]
fn int_var_reads_integer_value() {
    let value =
        Binder::new(MapEnvironment::from_pairs([("PORT", "27017")])).bind(&IntVar::new("PORT"));

    assert_eq!(value, Ok(27_017));
}

#[test]
fn int_var_uses_default_for_empty_value() {
    let value = Binder::new(MapEnvironment::from_pairs([("PORT", "")]))
        .bind(&IntVar::new("PORT").default(5432));

    assert_eq!(value, Ok(5432));
}

#[test]
fn int_var_treats_whitespace_as_parse_input() {
    let value = Binder::new(MapEnvironment::from_pairs([("PORT", " ")]))
        .bind(&IntVar::new("PORT"))
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("parse_variable"));
}

#[test]
fn optional_wrapper_returns_none_for_missing_value() {
    let value = Binder::new(MapEnvironment::new()).bind(&IntVar::new("PORT").optional());

    assert_eq!(value, Ok(None));
}

#[test]
fn float_var_reads_decimal_value() {
    let value = Binder::new(MapEnvironment::from_pairs([("TIMEOUT", "1.25")]))
        .bind(&FloatVar::new("TIMEOUT"));

    assert_eq!(value, Ok(1.25));
}

#[test]
fn bool_var_accepts_short_truthy_token() {
    let value =
        Binder::new(MapEnvironment::from_pairs([("ENABLED", "t")])).bind(&BoolVar::new("ENABLED"));

    assert_eq!(value, Ok(true));
}

#[test]
fn bool_var_accepts_short_falsy_token() {
    let value =
        Binder::new(MapEnvironment::from_pairs([("ENABLED", "n")])).bind(&BoolVar::new("ENABLED"));

    assert_eq!(value, Ok(false));
}

#[test]
fn string_var_preserves_whitespace_input() {
    let value =
        Binder::new(MapEnvironment::from_pairs([("NAME", "   ")])).bind(&StringVar::new("NAME"));

    assert_eq!(value, Ok("   ".to_owned()));
}

#[test]
fn string_var_rejects_value_above_configured_limit() {
    let value = Binder::new(MapEnvironment::from_pairs([("TOKEN", "abcd")]))
        .bind(&StringVar::new("TOKEN").max_bytes(3))
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("value_too_large"));
}

#[test]
fn bool_var_redacts_sensitive_validator_details() {
    let value = Binder::new(MapEnvironment::from_pairs([("ENABLED", "false")]))
        .bind(&BoolVar::new("ENABLED").validate(|enabled| {
            if enabled {
                Ok(())
            } else {
                Err(ValidationError::new("secret validation detail"))
            }
        }))
        .map_err(|error| {
            (
                error.error_code().to_owned(),
                !error.to_string().contains("secret validation detail"),
            )
        });

    assert_eq!(value, Err(("validation_failed".to_owned(), true)));
}

#[test]
fn b64_decoded_string_var_decodes_utf8_text() {
    let value = Binder::new(MapEnvironment::from_pairs([("CERT", "Y2VydGlmaWNhdGU=")]))
        .bind(&B64DecodedStringVar::new("CERT"));

    assert_eq!(value, Ok("certificate".to_owned()));
}

#[test]
fn b64_decoded_string_var_rejects_invalid_text() {
    let value = Binder::new(MapEnvironment::from_pairs([("CERT", "bad:::")]))
        .bind(&B64DecodedStringVar::new("CERT"))
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("parse_variable"));
}

#[test]
fn b64_decoded_string_var_rejects_decoded_value_above_configured_limit() {
    let value = Binder::new(MapEnvironment::from_pairs([("CERT", "YWE=")]))
        .bind(&B64DecodedStringVar::new("CERT").max_decoded_bytes(1))
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("value_too_large"));
}

#[test]
fn json_var_reads_json_payload() {
    let value = Binder::new(MapEnvironment::from_pairs([(
        "PROFILE",
        r#"{"debug":true}"#,
    )]))
    .bind(&JsonVar::new("PROFILE"));

    assert_eq!(value, Ok(json!({ "debug": true })));
}

#[test]
fn json_var_rejects_payload_above_default_limit() {
    let raw = format!(r#""{}""#, "x".repeat(64 * 1024));
    let value = Binder::new(MapEnvironment::from_pairs([("PROFILE", raw)]))
        .bind(&JsonVar::new("PROFILE"))
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("value_too_large"));
}

#[test]
fn json_var_accepts_payload_with_larger_configured_limit() {
    let payload = "x".repeat(64 * 1024);
    let raw = format!(r#""{payload}""#);
    let value = Binder::new(MapEnvironment::from_pairs([("PROFILE", raw.clone())]))
        .bind(&JsonVar::new("PROFILE").max_bytes(raw.len()));

    assert_eq!(value, Ok(json!(payload)));
}

#[test]
fn string_list_var_splits_and_strips_items() {
    let value = Binder::new(MapEnvironment::from_pairs([("HOSTS", "api, worker ,db")]))
        .bind(&ListVar::strings("HOSTS"));

    assert_eq!(
        value,
        Ok(vec!["api".to_owned(), "worker".to_owned(), "db".to_owned()])
    );
}

#[test]
fn list_var_rejects_value_above_configured_item_limit() {
    let value = Binder::new(MapEnvironment::from_pairs([("HOSTS", "api,worker")]))
        .bind(&ListVar::strings("HOSTS").max_items(1).sensitive(false))
        .map_err(|error| {
            (
                error.error_code().to_owned(),
                error.to_string().contains("maximum item count"),
            )
        });

    assert_eq!(value, Err(("validation_failed".to_owned(), true)));
}

#[test]
fn integer_list_var_parses_items() {
    let value = Binder::new(MapEnvironment::from_pairs([("PORTS", "80,443")]))
        .bind(&ListVar::integers("PORTS"));

    assert_eq!(value, Ok(vec![80, 443]));
}

#[test]
fn bool_list_var_parses_items() {
    let value = Binder::new(MapEnvironment::from_pairs([("FLAGS", "yes,n")]))
        .bind(&ListVar::booleans("FLAGS"));

    assert_eq!(value, Ok(vec![true, false]));
}

#[test]
fn u16_list_var_parses_items() {
    let value = Binder::new(MapEnvironment::from_pairs([("PORTS", "80,443")]))
        .bind(&ListVar::<u16>::u16s("PORTS"));

    assert_eq!(value, Ok(vec![80, 443]));
}

#[test]
fn enum_list_var_matches_names_and_values() {
    let value = Binder::new(MapEnvironment::from_pairs([("MODES", "blue,GREEN")])).bind(
        &ListVar::<Mode>::enumeration_from_names_and_values(
            "MODES",
            [
                ("BLUE", "blue", Mode::Blue),
                ("GREEN", "green", Mode::Green),
            ],
        ),
    );

    assert_eq!(value, Ok(vec![Mode::Blue, Mode::Green]));
}

#[test]
fn enum_list_var_can_match_case_sensitively() {
    let value = Binder::new(MapEnvironment::from_pairs([("MODES", "blue")]))
        .bind(&ListVar::<Mode>::case_sensitive_enumeration(
            "MODES",
            [("BLUE", Mode::Blue)],
        ))
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("validation_failed"));
}

#[test]
fn list_var_rejects_empty_delimiter() {
    let value = Binder::new(MapEnvironment::from_pairs([("HOSTS", "api,worker")]))
        .bind(&ListVar::strings("HOSTS").delimiter("").sensitive(false))
        .map_err(|error| {
            (
                error.error_code().to_owned(),
                error.to_string().contains("delimiter must not be empty"),
            )
        });

    assert_eq!(value, Err(("validation_failed".to_owned(), true)));
}

#[test]
fn enum_var_matches_case_insensitively_by_default() {
    let value = Binder::new(MapEnvironment::from_pairs([("MODE", "green")])).bind(&EnumVar::new(
        "MODE",
        [("BLUE", Mode::Blue), ("GREEN", Mode::Green)],
    ));

    assert_eq!(value, Ok(Mode::Green));
}

#[test]
fn enum_var_matches_names_and_values() {
    let value = Binder::new(MapEnvironment::from_pairs([("MODE", "blue")])).bind(
        &EnumVar::<Mode>::from_names_and_values(
            "MODE",
            [
                ("BLUE", "blue", Mode::Blue),
                ("GREEN", "green", Mode::Green),
            ],
        ),
    );

    assert_eq!(value, Ok(Mode::Blue));
}

#[test]
fn enum_var_matches_alias() {
    let value = Binder::new(MapEnvironment::from_pairs([("MODE", "azure")]))
        .bind(&EnumVar::new("MODE", [("BLUE", Mode::Blue)]).alias("azure", Mode::Blue));

    assert_eq!(value, Ok(Mode::Blue));
}

#[test]
fn enum_var_can_match_case_sensitively() {
    let value = Binder::new(MapEnvironment::from_pairs([("MODE", "green")]))
        .bind(&EnumVar::new("MODE", [("GREEN", Mode::Green)]).case_sensitive())
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("parse_variable"));
}

#[test]
fn sensitive_validation_hides_custom_detail() {
    let value = Binder::new(MapEnvironment::from_pairs([("TOKEN", "secret")]))
        .bind(
            &StringVar::new("TOKEN")
                .validate(|value| Err(ValidationError::new(format!("invalid token: {value}")))),
        )
        .map_err(|error| !error.to_string().contains("secret"));

    assert_eq!(value, Err(true));
}

#[test]
fn non_sensitive_validation_keeps_custom_detail() {
    let value = Binder::new(MapEnvironment::from_pairs([("ENVIRONMENT", "qa")]))
        .bind(
            &StringVar::new("ENVIRONMENT")
                .sensitive(false)
                .validate(validators::one_of(["dev", "prod"])),
        )
        .map_err(|error| error.to_string().contains("allowed set"));

    assert_eq!(value, Err(true));
}

#[test]
fn url_validator_accepts_https_url() {
    let value = Binder::new(MapEnvironment::from_pairs([(
        "API_URL",
        "https://example.com",
    )]))
    .bind(&StringVar::new("API_URL").validate(validators::is_url()));

    assert_eq!(value, Ok("https://example.com".to_owned()));
}

#[test]
fn email_validator_rejects_invalid_email() {
    let value = Binder::new(MapEnvironment::from_pairs([("EMAIL", "not-email")]))
        .bind(&StringVar::new("EMAIL").validate(validators::is_email()))
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("validation_failed"));
}

#[test]
fn pattern_validator_accepts_matching_text() {
    let value = validators::matches_pattern(r"^[a-z]+-[0-9]+$")
        .map(|validator| {
            Binder::new(MapEnvironment::from_pairs([("NAME", "api-1")]))
                .bind(&StringVar::new("NAME").validate(validator))
        })
        .map_err(|_| "regex failed to compile");

    assert_eq!(value, Ok(Ok("api-1".to_owned())));
}
