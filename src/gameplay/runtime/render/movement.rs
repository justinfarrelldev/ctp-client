//! Rendering helpers for CTP2-style click-drag movement planning.

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use super::super::MovementInputState;
use super::super::view::Camera;
use super::draw_tile_diamond;
use crate::gameplay::{GameplayError, runtime_error};

/// CTP2-style green used for active-unit and path movement feedback.
const MOVEMENT_FEEDBACK_COLOR: Color = Color::RGBA(32, 255, 64, 255);
/// Pixel offsets used to thicken movement-feedback diamonds.
const MOVEMENT_FEEDBACK_STROKE_OFFSETS: [i32; 3] = [-1, 0, 1];

/// Draws a thick CTP2-green diamond for active movement feedback.
pub(super) fn draw_movement_diamond(
    canvas: &mut Canvas<Window>,
    camera: &Camera,
    screen_x: i32,
    screen_y: i32,
) -> Result<(), GameplayError> {
    for offset in movement_feedback_stroke_offsets() {
        draw_tile_diamond(
            canvas,
            camera,
            screen_x.saturating_add(*offset),
            screen_y,
            MOVEMENT_FEEDBACK_COLOR,
        )?;
        if *offset != 0 {
            draw_tile_diamond(
                canvas,
                camera,
                screen_x,
                screen_y.saturating_add(*offset),
                MOVEMENT_FEEDBACK_COLOR,
            )?;
        }
    }
    Ok(())
}

/// Returns the stroke offsets used by movement-feedback diamonds.
pub(super) const fn movement_feedback_stroke_offsets() -> &'static [i32] {
    &MOVEMENT_FEEDBACK_STROKE_OFFSETS
}

/// Draws the currently dragged movement path.
pub(super) fn draw_planned_path(
    canvas: &mut Canvas<Window>,
    input: &MovementInputState,
    camera: &Camera,
) -> Result<(), GameplayError> {
    for point in input.planned_path() {
        let (screen_x, screen_y) = camera.tile_to_screen(*point);
        draw_movement_diamond(canvas, camera, screen_x, screen_y)?;
        canvas.set_draw_color(MOVEMENT_FEEDBACK_COLOR);
        canvas
            .fill_rect(planned_path_marker_rect(screen_x, screen_y))
            .map_err(runtime_error)?;
    }
    Ok(())
}

/// Returns the center marker rectangle for one planned path tile.
pub(super) fn planned_path_marker_rect(screen_x: i32, screen_y: i32) -> Rect {
    Rect::new(
        screen_x.saturating_add(41),
        screen_y.saturating_add(30),
        12,
        12,
    )
}
