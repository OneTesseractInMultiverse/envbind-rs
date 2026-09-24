#[path = "../../tests/support/robustness.rs"]
pub mod robustness;

// Corpus files are plain UTF-8 payloads, with no binary header. Derive varied
// limits from the first two bytes while also testing the full-size parser path.
pub fn input(data: &[u8]) -> Option<(&str, usize)> {
    if data.len() > robustness::MAX_INPUT_BYTES {
        return None;
    }
    let raw = std::str::from_utf8(data).ok()?;
    let selector = usize::from(u16::from_le_bytes([
        data.first().copied().unwrap_or(0),
        data.get(1).copied().unwrap_or(0),
    ]));
    Some((raw, selector))
}
