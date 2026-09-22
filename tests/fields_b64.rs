#![allow(missing_docs)]

use envbind::{B64DecodedStringVar, Binder, MapEnvironment};

#[test]
fn base64_requires_canonical_padding() {
    let results = ["Zg", "Zg=", "Zg===", "Z=g=", "Zm8"].map(|raw| {
        Binder::new(MapEnvironment::from_pairs([("CERT", raw)]))
            .bind(&B64DecodedStringVar::new("CERT"))
            .map(|_| ())
            .map_err(|error| error.error_code())
    });

    assert_eq!(results, [Err("parse_variable"); 5]);
}

#[test]
fn base64_rejects_noncanonical_trailing_bits_without_exposing_input() {
    let results = [true, false].map(|sensitive| {
        Binder::new(MapEnvironment::from_pairs([("CERT", "c2VjcmV0IR==")]))
            .bind(&B64DecodedStringVar::new("CERT").sensitive(sensitive))
            .map(|_| ())
            .map_err(|error| {
                let formatted = format!("{error} {error:?} {error:#?}");
                (
                    error.error_code(),
                    formatted.contains("c2VjcmV0IR=="),
                    formatted.contains("secret!"),
                )
            })
    });

    assert_eq!(results, [Err(("parse_variable", false, false)); 2]);
}

#[test]
fn base64_requires_decoded_utf8() {
    let result = Binder::new(MapEnvironment::from_pairs([("CERT", "/w==")]))
        .bind(&B64DecodedStringVar::new("CERT"))
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("parse_variable"));
}

#[test]
fn base64_accepts_exact_decoded_byte_limits_for_each_padding_length() {
    let results = [("Zg==", 1), ("Zm8=", 2), ("Zm9v", 3)].map(|(raw, limit)| {
        Binder::new(MapEnvironment::from_pairs([("CERT", raw)]))
            .bind(&B64DecodedStringVar::new("CERT").max_decoded_bytes(limit))
    });

    assert_eq!(
        results,
        [
            Ok("f".to_owned()),
            Ok("fo".to_owned()),
            Ok("foo".to_owned())
        ]
    );
}

#[test]
fn base64_keeps_the_standard_alphabet() {
    let results = ["--__", "Zm9v\n"].map(|raw| {
        Binder::new(MapEnvironment::from_pairs([("CERT", raw)]))
            .bind(&B64DecodedStringVar::new("CERT"))
            .map(|_| ())
            .map_err(|error| error.error_code())
    });

    assert_eq!(results, [Err("parse_variable"); 2]);
}

#[test]
fn base64_decodes_multiple_full_blocks() {
    let result = Binder::new(MapEnvironment::from_pairs([("CERT", "Zm9v".repeat(64))]))
        .bind(&B64DecodedStringVar::new("CERT"));

    assert_eq!(result, Ok("foo".repeat(64)));
}
