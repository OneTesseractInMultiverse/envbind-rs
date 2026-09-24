#![no_main]

use envbind_fuzz::{input, robustness};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Some((raw, selector)) = input(data) {
        assert_eq!(
            robustness::check_scalars(raw, selector % (robustness::MAX_INPUT_BYTES + 1)),
            Ok(())
        );
        assert_eq!(robustness::check_redaction(raw), Ok(()));
        let mut bytes = [0_u8; 8];
        for (slot, byte) in bytes.iter_mut().zip(data) {
            *slot = *byte;
        }
        let candidate = f64::from_le_bytes(bytes);
        let finite = if candidate.is_finite() {
            candidate
        } else {
            0.0
        };
        assert_eq!(
            robustness::check_scalar_roundtrip(
                i64::from_le_bytes(bytes),
                selector as u16,
                selector & 1 == 0,
                finite
            ),
            Ok(())
        );
    }
});
