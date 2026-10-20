//! Text decoding for original retail databases (Windows-1252) and modern UTF-8 mods.

/// Original ANSI punctuation; undefined CP1252 slots retain their control code.
const WINDOWS_PUNCTUATION: [char; 32] = [
    '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8d}', 'Ž', '\u{8f}',
    '\u{90}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{9d}', 'ž', 'Ÿ',
];

/// Preserves retail text without replacing non-UTF8 punctuation with U+FFFD.
#[must_use]
pub fn decode(bytes: &[u8]) -> String {
    if let Ok(text) = core::str::from_utf8(bytes) {
        return text.to_owned();
    }
    bytes
        .iter()
        .copied()
        .map(|byte| {
            if (0x80_u8..=0x9f_u8).contains(&byte) {
                WINDOWS_PUNCTUATION[usize::from(byte.saturating_sub(0x80))]
            } else {
                char::from(byte)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::decode;
    #[test]
    fn retail_punctuation_and_utf8_mod_text_are_preserved() {
        assert_eq!(
            decode(b"Hoplite\x92s \x96 city"),
            "Hoplite’s – city",
            "retail Windows-1252 punctuation"
        );
        assert_eq!(
            decode("Hoplite’s – city".as_bytes()),
            "Hoplite’s – city",
            "UTF-8 mods remain intact"
        );
    }
}
