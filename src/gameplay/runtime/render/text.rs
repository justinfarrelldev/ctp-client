//! Text drawing helpers for gameplay rendering.

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::Font;
use sdl2::video::Window;

use crate::gameplay::{GameplayError, runtime_error};

/// Width of one fallback glyph in pixels.
const GLYPH_WIDTH: i32 = 5;
/// Space between fallback glyphs in pixels.
const GLYPH_SPACING: i32 = 1;
/// Fallback glyph scale.
const GLYPH_SCALE: u32 = 1;

/// Draws original-style CTP2 title text with a faint gold shadow.
pub(super) fn draw_shadowed_text(
    canvas: &mut Canvas<Window>,
    font: Option<&Font<'_, '_>>,
    text: &str,
    x: i32,
    y: i32,
    color: Color,
) -> Result<(), GameplayError> {
    draw_text(
        canvas,
        font,
        text,
        x.saturating_add(1),
        y.saturating_add(1),
        Color::RGB(20, 16, 10),
    )?;
    draw_text(canvas, font, text, x, y, color)
}

/// Draws text with `SDL_ttf` when available, otherwise with a built-in bitmap fallback.
pub(super) fn draw_text(
    canvas: &mut Canvas<Window>,
    font: Option<&Font<'_, '_>>,
    text: &str,
    x: i32,
    y: i32,
    color: Color,
) -> Result<(), GameplayError> {
    if let Some(font) = font {
        let surface = font
            .render(text)
            .blended(color)
            .map_err(|error| runtime_error(error.to_string()))?;
        let texture_creator = canvas.texture_creator();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|error| runtime_error(error.to_string()))?;
        let target = Rect::new(x, y, surface.width(), surface.height());
        return canvas.copy(&texture, None, target).map_err(runtime_error);
    }
    draw_bitmap_text(canvas, text, x, y, color)
}

/// Draws text with a tiny built-in ASCII bitmap font.
fn draw_bitmap_text(
    canvas: &mut Canvas<Window>,
    text: &str,
    x: i32,
    y: i32,
    color: Color,
) -> Result<(), GameplayError> {
    canvas.set_draw_color(color);
    let scale = i32::try_from(GLYPH_SCALE).map_or(1_i32, |value| value.max(1_i32));
    let mut cursor_x = x;
    for character in text.chars() {
        if character == ' ' {
            cursor_x = cursor_x.saturating_add((GLYPH_WIDTH + GLYPH_SPACING).saturating_mul(scale));
            continue;
        }
        let glyph = glyph_rows(character);
        for (row_index, row_bits) in glyph.iter().enumerate() {
            let row_y = y.saturating_add(
                i32::try_from(row_index).map_or(0_i32, |value| value.saturating_mul(scale)),
            );
            for column in 0_u8..5_u8 {
                let mask = 1_u8 << u32::from(4_u8.saturating_sub(column));
                if row_bits & mask == 0 {
                    continue;
                }
                let column_x = cursor_x.saturating_add(i32::from(column).saturating_mul(scale));
                canvas
                    .fill_rect(Rect::new(column_x, row_y, GLYPH_SCALE, GLYPH_SCALE))
                    .map_err(runtime_error)?;
            }
        }
        cursor_x = cursor_x.saturating_add((GLYPH_WIDTH + GLYPH_SPACING).saturating_mul(scale));
    }
    Ok(())
}

/// Returns fallback glyph rows as five-bit scanlines.
#[expect(
    clippy::too_many_lines,
    reason = "A compact built-in ASCII bitmap avoids invisible gameplay text when SDL_ttf is unavailable."
)]
const fn glyph_rows(character: char) -> [u8; 7] {
    match character.to_ascii_uppercase() {
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ],
        'J' => [
            0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010,
        ],
        'X' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ],
        'Y' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110,
        ],
        '6' => [
            0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100,
        ],
        '-' => [0, 0, 0, 0b11111, 0, 0, 0],
        '+' => [0, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0],
        '.' => [0, 0, 0, 0, 0, 0b01100, 0b01100],
        ':' => [0, 0b01100, 0b01100, 0, 0b01100, 0b01100, 0],
        '/' => [
            0b00001, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b10000,
        ],
        '!' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0, 0b00100],
        _ => [0b11111, 0b10001, 0b00010, 0b00100, 0b00100, 0, 0b00100],
    }
}

#[cfg(test)]
mod tests {
    use super::glyph_rows;

    #[test]
    fn fallback_glyphs_cover_dropdown_labels() {
        for character in "Empire Cities Units Diplomacy Stats Options Save Load Quit".chars() {
            if character == ' ' {
                continue;
            }
            assert!(
                glyph_rows(character).iter().any(|row| *row != 0),
                "fallback glyph should contain visible pixels for {character}"
            );
        }
    }
}
