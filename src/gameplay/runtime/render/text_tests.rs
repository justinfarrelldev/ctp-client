//! Native in-memory word-wrapping regression; no screenshot files are created.

use super::text::draw_wrapped_text;
use crate::gameplay::runtime::test_support::with_canvas;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;

#[test]
fn narrow_original_labels_wrap_instead_of_losing_discovery_and_spending_text() {
    let result = with_canvas((120_u32, 80_u32), |canvas| {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        draw_wrapped_text(
            canvas,
            None,
            "Turns To Discovery:",
            Rect::new(0, 0, 70, 40),
            Color::RGB(255, 255, 255),
        )
        .map_err(|error| error.to_string())?;
        let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        let second_line = 120_usize.saturating_mul(8).saturating_mul(4);
        if !pixels[second_line..]
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [255_u8; 3])
        {
            return Err(
                "source wordwrap flag was ignored; label truncated to first line".to_owned(),
            );
        }
        Ok(())
    });
    assert!(result.is_ok(), "wordwrap pixel regression: {result:?}");
}
