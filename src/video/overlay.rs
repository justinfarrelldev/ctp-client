extern crate alloc;

/// Presentation and accessibility metadata for the legacy skip overlay window.
pub(super) struct SkipOverlayLayout;

impl SkipOverlayLayout {
    /// Returns the overlay window title string used when the auxiliary window is displayed.
    pub(super) const fn window_title() -> &'static str {
        "Intro Video — Press Space/Esc, Click, or Controller Start to Skip"
    }

    /// Returns per-line instructional text presented on the overlay.
    pub(super) const fn instruction_lines() -> [&'static str; 3] {
        [
            "KEYBOARD: SPACE / ENTER / ESC",
            "MOUSE: CLICK TO SKIP",
            "CONTROLLER: A / B / START / GUIDE",
        ]
    }

    /// Indicates whether the overlay requests global input grabs.
    pub(super) const fn should_grab_input() -> bool {
        false
    }

    /// Height of each glyph expressed as `usize` for bitmap definitions, available in tests.
    #[cfg(test)]
    const GLYPH_HEIGHT_USIZE: usize = 7;

    /// Returns the 5x7 bitmap for supported glyphs. Test-only helper exposed for snapshotting.
    #[cfg(test)]
    pub(super) const fn glyph_bitmap_for_test(ch: char) -> Option<[u8; Self::GLYPH_HEIGHT_USIZE]> {
        Self::glyph_pattern(ch)
    }

    /// bitmap patterns for capital glyphs used in overlay messaging.
    #[cfg(test)]
    const fn glyph_pattern(ch: char) -> Option<[u8; Self::GLYPH_HEIGHT_USIZE]> {
        match ch {
            'A' => Some([
                0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
            ]),
            'B' => Some([
                0b11110, 0b10001, 0b11110, 0b10001, 0b10001, 0b10001, 0b11110,
            ]),
            'C' => Some([
                0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
            ]),
            'D' => Some([
                0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
            ]),
            'E' => Some([
                0b11111, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000, 0b11111,
            ]),
            'I' => Some([
                0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
            ]),
            'K' => Some([
                0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
            ]),
            'L' => Some([
                0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
            ]),
            'M' => Some([
                0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
            ]),
            'N' => Some([
                0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
            ]),
            'O' => Some([
                0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
            ]),
            'P' => Some([
                0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
            ]),
            'R' => Some([
                0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
            ]),
            'S' => Some([
                0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
            ]),
            'T' => Some([
                0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
            ]),
            'U' => Some([
                0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
            ]),
            'V' => Some([
                0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100,
            ]),
            'Y' => Some([
                0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
            ]),
            ':' => Some([
                0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000,
            ]),
            '/' => Some([
                0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b10000, 0b00000,
            ]),
            _ => None,
        }
    }
}
