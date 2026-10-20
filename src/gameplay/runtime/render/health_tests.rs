//! Native health pixels remain authoritative and leave unfilled space black.

use super::panel::draw_health;
use crate::gameplay::runtime::test_support::with_canvas;
use crate::gameplay::server::ServerUnitProjection;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;

#[test]
fn partial_dead_and_unknown_units_never_paint_a_full_green_bar() {
    let result = with_canvas((100_u32, 10_u32), |canvas| {
        for (health, width, expected) in [
            (25_u64, 24_usize, [255_u8, 0_u8, 0_u8]),
            (50, 48, [255, 255, 0]),
            (75, 72, [0, 255, 0]),
            (0, 0, [0, 0, 0]),
        ] {
            let source = ServerUnitProjection {
                facts: serde_json::json!({"health":health,"max_health":100_u64}),
                ..ServerUnitProjection::default()
            };
            canvas.set_draw_color(Color::RGB(255, 255, 255));
            canvas.clear();
            draw_health(canvas, Some(&source), Rect::new(0, 0, 96, 3), false)
                .map_err(|error| error.to_string())?;
            let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
            if pixels[..3] != expected {
                return Err(format!("incorrect source health color at {health}"));
            }
            let tail = width.saturating_mul(4);
            if pixels[tail..tail.saturating_add(3)] != [0_u8; 3] {
                return Err(format!("unfilled health tail is not black at {health}"));
            }
        }
        draw_health(canvas, None, Rect::new(0, 0, 96, 3), false)
            .map_err(|error| error.to_string())?;
        let unknown = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        if unknown[..3] != [0_u8; 3] {
            return Err("missing authoritative health became a full green bar".to_owned());
        }
        Ok(())
    });
    assert!(result.is_ok(), "health framebuffer regression: {result:?}");
}
