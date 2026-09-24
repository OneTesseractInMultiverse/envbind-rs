#![no_main]

use envbind_fuzz::{input, robustness};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Some((raw, selector)) = input(data) {
        assert_eq!(
            robustness::check_json(raw, selector % (robustness::MAX_INPUT_BYTES + 1)),
            Ok(())
        );
        let text: String = raw.chars().take(128).collect();
        let value = serde_json::json!({"text": text, "number": selector});
        assert_eq!(robustness::check_json_roundtrip(&value), Ok(()));
    }
});
