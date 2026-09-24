#![no_main]

use envbind_fuzz::{input, robustness};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Some((raw, selector)) = input(data) {
        assert_eq!(
            robustness::check_base64(raw, selector % (robustness::MAX_INPUT_BYTES + 1)),
            Ok(())
        );
        assert_eq!(robustness::check_base64_roundtrip(raw), Ok(()));
    }
});
