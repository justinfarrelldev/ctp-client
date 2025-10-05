extern crate alloc;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// Presentation and accessibility layout for the skip overlay window.
pub(super) struct SkipOverlayLayout;

impl SkipOverlayLayout {
    /// Width of each glyph in pixels before scaling to screen coordinates.
    const GLYPH_WIDTH: i32 = 5;
    /// Height of each glyph in pixels before scaling to screen coordinates.
    const GLYPH_HEIGHT: i32 = 7;
    /// Height of each glyph expressed as `usize` for bitmap definitions.
    const GLYPH_HEIGHT_USIZE: usize = 7;
    /// Horizontal padding around the text block to prevent edge clipping.
    const H_PADDING: i32 = 24;
    /// Vertical padding around the text block to maintain comfortable spacing.
    const V_PADDING: i32 = 24;
    /// Multiplier applied to each glyph pixel for readability on modern displays.
    const SCALE_I32: i32 = 3;
    /// Unsigned representation of [`SCALE_I32`] for SDL drawing APIs.
    const SCALE_U32: u32 = 3;
    /// Additional spacing between lines of text to improve legibility.
    const LINE_SPACING: i32 = 8;

    /// Returns the overlay window title string.
    pub(super) const fn window_title() -> &'static str {
        "Intro Video — Press Space, Click, or Controller Start to Skip"
    }

    /// Returns per-line instructional text to render on the overlay.
    pub(super) const fn instruction_lines() -> [&'static str; 3] {
        [
            "KEYBOARD: SPACE / ENTER / ESC",
            "MOUSE: CLICK TO SKIP",
            "CONTROLLER: A / B / START",
        ]
    }

    /// Indicates whether the overlay requests global input grabs.
    pub(super) const fn should_grab_input() -> bool {
        false
    }

    /// Primary foreground color used for instruction text.
    pub(super) const fn text_color() -> Color {
        Color::RGBA(245, 245, 245, 255)
    }

    /// Background color for the overlay canvas.
    pub(super) const fn background_color() -> Color {
        Color::RGBA(16, 16, 16, 230)
    }

    /// Horizontal advance applied after drawing each glyph.
    const fn glyph_advance() -> i32 {
        (Self::GLYPH_WIDTH + 1_i32) * Self::SCALE_I32
    }

    /// Renders the instruction block to the supplied canvas.
    pub(super) fn render_instructions(canvas: &mut Canvas<Window>) -> Result<(), String> {
        canvas.set_draw_color(Self::text_color());

        let lines = Self::instruction_lines();
        let line_height =
            Self::safe_mul(Self::GLYPH_HEIGHT, Self::SCALE_I32, "overlay line height")?;
        let line_advance = Self::safe_add(line_height, Self::LINE_SPACING, "overlay line advance")?;

        let mut y = Self::V_PADDING;
        let mut iter = lines.iter().peekable();
        while let Some(line) = iter.next() {
            Self::render_line(canvas, line, Self::H_PADDING, y)?;

            if iter.peek().is_some() {
                y = Self::safe_add(y, line_advance, "overlay vertical layout")?;
            }
        }

        Ok(())
    }

    /// Draws a single line of glyphs to the canvas.
    fn render_line(
        canvas: &mut Canvas<Window>,
        text: &str,
        start_x: i32,
        start_y: i32,
    ) -> Result<(), String> {
        let glyph_advance = Self::glyph_advance();
        let mut x = start_x;
        for ch in text.chars() {
            if ch == ' ' {
                x = Self::safe_add(x, glyph_advance, "overlay horizontal layout")?;
                continue;
            }

            if let Some(pattern) = Self::glyph_pattern(ch) {
                Self::draw_glyph(canvas, x, start_y, pattern)?;
            }

            x = Self::safe_add(x, glyph_advance, "overlay horizontal layout")?;
        }

        Ok(())
    }

    /// Draws a single glyph pattern to the canvas.
    ///
    /// # Errors
    ///
    /// Returns an error if coordinate calculations overflow the valid range for SDL rectangles.
    fn draw_glyph(
        canvas: &mut Canvas<Window>,
        origin_x: i32,
        origin_y: i32,
        pattern: [u8; Self::GLYPH_HEIGHT_USIZE],
    ) -> Result<(), String> {
        let scale = Self::SCALE_I32;
        let scale_u32 = Self::SCALE_U32;
        if scale <= 0_i32 || scale_u32 == 0_u32 {
            return Err("overlay scale must be positive".to_owned());
        }

        let highest_column = Self::safe_sub(Self::GLYPH_WIDTH, 1_i32, "overlay glyph column max")?;

        for (row_index, row_bits) in pattern.into_iter().enumerate() {
            let row = i32::try_from(row_index)
                .map_err(|err| format!("overlay glyph rows must fit in i32: {err}"))?;
            for col in 0_i32..Self::GLYPH_WIDTH {
                let shift_base = Self::safe_sub(highest_column, col, "overlay glyph column shift")?;
                let shift = u32::try_from(shift_base)
                    .map_err(|err| format!("overlay glyph column shift must fit in u32: {err}"))?;
                if (row_bits >> shift) & 1 == 1 {
                    let x_offset = Self::safe_mul(col, scale, "overlay glyph column scaling")?;
                    let x = Self::safe_add(origin_x, x_offset, "overlay glyph x position")?;
                    let y_offset = Self::safe_mul(row, scale, "overlay glyph row scaling")?;
                    let y = Self::safe_add(origin_y, y_offset, "overlay glyph y position")?;
                    canvas.fill_rect(Rect::new(x, y, scale_u32, scale_u32))?;
                }
            }
        }
        Ok(())
    }

    /// Returns the 5x7 bitmap for supported glyphs.
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

    /// Safely adds two integers, returning a descriptive error on overflow.
    fn safe_add(lhs: i32, rhs: i32, context: &str) -> Result<i32, String> {
        lhs.checked_add(rhs)
            .ok_or_else(|| format!("{context} overflow"))
    }

    /// Safely subtracts two integers, returning a descriptive error on underflow.
    fn safe_sub(lhs: i32, rhs: i32, context: &str) -> Result<i32, String> {
        lhs.checked_sub(rhs)
            .ok_or_else(|| format!("{context} underflow"))
    }

    /// Safely multiplies two integers, returning a descriptive error on overflow.
    fn safe_mul(lhs: i32, rhs: i32, context: &str) -> Result<i32, String> {
        lhs.checked_mul(rhs)
            .ok_or_else(|| format!("{context} overflow"))
    }

    #[cfg(test)]
    pub(super) const fn glyph_bitmap_for_test(ch: char) -> Option<[u8; Self::GLYPH_HEIGHT_USIZE]> {
        Self::glyph_pattern(ch)
    }
}
