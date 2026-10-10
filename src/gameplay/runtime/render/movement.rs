//! Rendering helpers for CTP2-style click-drag movement planning.

use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

use super::super::MovementInputState;
use super::super::view::Camera;
use super::draw_tile_diamond;
use crate::gameplay::{GameplayError, runtime_error};

/// Planned path marker color used by the original green movement reticle family.
const PATH_MARKER_COLOR: Color = Color::RGBA(32, 255, 64, 255);

/// Draws the currently dragged movement path.
pub(super) fn draw_planned_path(
    canvas: &mut Canvas<Window>,
    input: &MovementInputState,
    camera: &Camera,
) -> Result<(), GameplayError> {
    for point in input.planned_path() {
        let (screen_x, screen_y) = camera.tile_to_screen(*point);
        draw_tile_diamond(canvas, camera, screen_x, screen_y, PATH_MARKER_COLOR)?;
        canvas.set_draw_color(PATH_MARKER_COLOR);
        canvas
            .fill_rect(sdl2::rect::Rect::new(
                screen_x.saturating_add(44),
                screen_y.saturating_add(34),
                8,
                8,
            ))
            .map_err(runtime_error)?;
    }
    Ok(())
}
