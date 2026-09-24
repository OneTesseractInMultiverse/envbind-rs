#![no_main]

use envbind_fuzz::{input, robustness};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Some((raw, selector)) = input(data) {
        assert_eq!(robustness::check_url(raw), Ok(()));
        let label: String = data
            .iter()
            .take(16)
            .map(|byte| char::from(b'a' + byte % 26))
            .collect();
        let host = format!("a{label}.example");
        assert_eq!(
            robustness::check_url_roundtrip(&host, selector as u16, "path"),
            Ok(())
        );
    }
});
