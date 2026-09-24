#![no_main]

use envbind_fuzz::{input, robustness};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Some((raw, selector)) = input(data) {
        let delimiters = [",", "::", "💡", "", "\0", "é"];
        let delimiter = delimiters[selector % delimiters.len()];
        assert_eq!(
            robustness::check_list(
                raw,
                delimiter,
                selector % (robustness::MAX_ITEMS + 1),
                selector & 1 == 0
            ),
            Ok(())
        );
        if selector == 0 {
            assert_eq!(
                robustness::check_list_raw_limit(data.get(2).copied().unwrap_or(0)),
                Ok(())
            );
        }
    }
});
