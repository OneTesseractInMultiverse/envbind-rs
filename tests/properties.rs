#![allow(missing_docs)]

#[path = "support/robustness.rs"]
mod robustness;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use envbind::{B64DecodedStringVar, Binder, MapEnvironment, StringVar, validators};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};
use serde_json::json;

fn config(cases: u32) -> Config {
    Config {
        cases,
        rng_seed: RngSeed::Fixed(0x7e1c_2026_0918),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        max_shrink_iters: 1024,
        max_shrink_time: 0,
        max_local_rejects: 256,
        max_global_rejects: 256,
        max_flat_map_regens: 1024,
        max_default_size_range: 64,
        verbose: 0,
        ..Config::default()
    }
}

fn text() -> impl Strategy<Value = String> {
    proptest::collection::vec(any::<char>(), 0..=128)
        .prop_map(|characters| characters.into_iter().collect())
}

fn delimiter() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        Just(",".to_owned()),
        Just("::".to_owned()),
        Just("💡".to_owned()),
        proptest::collection::vec(any::<char>(), 1..=2)
            .prop_map(|characters| characters.into_iter().collect())
    ]
}

proptest! {
    #![proptest_config(config(128))]

    #[test]
    fn arbitrary_json_obeys_limits_and_optional_contract(raw in text(), limit in 0..=512_usize) {
        prop_assert_eq!(robustness::check_json(&raw, limit), Ok(()));
    }

    #[test]
    fn generated_json_structures_roundtrip(
        message in text(), number in any::<i64>(), enabled in any::<bool>(),
        values in proptest::collection::vec(any::<i64>(), 0..=16),
    ) {
        let value = json!({"message": message, "number": number, "enabled": enabled, "values": values});
        prop_assert_eq!(robustness::check_json_roundtrip(&value), Ok(()));
    }

    #[test]
    fn generated_json_nesting_remains_bounded(depth in 0..=256_usize, complete in any::<bool>()) {
        let closing = if complete { "]".repeat(depth) } else { String::new() };
        let raw = format!("{}null{closing}", "[".repeat(depth));
        prop_assert_eq!(robustness::check_json(&raw, raw.len()), Ok(()));
    }

    #[test]
    fn arbitrary_base64_obeys_limits_and_optional_contract(raw in text(), limit in 0..=512_usize) {
        prop_assert_eq!(robustness::check_base64(&raw, limit), Ok(()));
    }

    #[test]
    fn generated_utf8_base64_roundtrips(raw in text()) {
        prop_assert_eq!(robustness::check_base64_roundtrip(&raw), Ok(()));
    }

    #[test]
    fn encoded_arbitrary_bytes_require_decoded_utf8(bytes in proptest::collection::vec(any::<u8>(), 0..=256)) {
        let raw = STANDARD.encode(&bytes);
        let actual = Binder::new(MapEnvironment::from_pairs([("VALUE", raw)]))
            .bind(&B64DecodedStringVar::new("VALUE").allow_empty().max_decoded_bytes(bytes.len()))
            .map_err(|error| error.error_code());
        let expected = String::from_utf8(bytes).map_err(|_| "parse_variable");
        prop_assert_eq!(actual, expected);
    }

    #[test]
    fn arbitrary_lists_bound_callbacks_and_preserve_optional_errors(
        raw in text(), separator in delimiter(), limit in 0..=robustness::MAX_ITEMS, keep in any::<bool>(),
    ) {
        prop_assert_eq!(robustness::check_list(&raw, &separator, limit, keep), Ok(()));
    }

    #[test]
    fn generated_lists_roundtrip_at_the_exact_item_limit(
        items in proptest::collection::vec("[a-zA-Z0-9é中]{0,16}", 1..=16),
        separator in prop_oneof![Just(","), Just("::"), Just("💡")],
    ) {
        prop_assert_eq!(robustness::check_list_roundtrip(&items, separator), Ok(()));
    }

    #[test]
    fn arbitrary_urls_are_safe_and_preserve_optional_errors(raw in text()) {
        prop_assert_eq!(robustness::check_url(&raw), Ok(()));
    }

    #[test]
    fn generated_valid_urls_retain_the_original_text(
        label in "[a-z][a-z0-9]{0,15}", port in any::<u16>(), path in "[a-zA-Z0-9/_-]{0,32}",
    ) {
        prop_assert_eq!(robustness::check_url_roundtrip(&format!("{label}.example"), port, &path), Ok(()));
    }

    #[test]
    fn url_validation_rejects_generated_forbidden_characters(
        suffix in text(), forbidden in prop_oneof![Just(' '), Just('\t'), Just('\n'), Just('\0'), Just('\\'), Just('\u{a0}')],
    ) {
        let raw = format!("https://example.test/{suffix}{forbidden}");
        let result = Binder::new(MapEnvironment::from_pairs([("VALUE", raw)]))
            .bind(&StringVar::new("VALUE").validate(validators::is_url()))
            .map_err(|error| error.error_code());
        prop_assert_eq!(result, Err("validation_failed"));
    }

    #[test]
    fn arbitrary_scalars_preserve_optional_errors(raw in text(), limit in 0..=512_usize) {
        prop_assert_eq!(robustness::check_scalars(&raw, limit), Ok(()));
    }

    #[test]
    fn generated_scalars_roundtrip(
        integer in any::<i64>(), port in any::<u16>(), boolean in any::<bool>(),
        float in any::<f64>().prop_filter("finite round trips only", |value| value.is_finite()),
    ) {
        prop_assert_eq!(robustness::check_scalar_roundtrip(integer, port, boolean, float), Ok(()));
    }

    #[test]
    fn synthetic_input_and_adapter_markers_stay_redacted(raw in text()) {
        prop_assert_eq!(robustness::check_redaction(&raw), Ok(()));
    }
}

proptest! {
    #![proptest_config(config(32))]

    #[test]
    fn oversized_raw_lists_never_reach_item_parsers(extra in any::<u8>()) {
        prop_assert_eq!(robustness::check_list_raw_limit(extra), Ok(()));
    }
}
