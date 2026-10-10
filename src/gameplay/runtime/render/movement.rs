//! Rendering helpers for CTP2-style click-drag movement planning.

extern crate alloc;

use alloc::vec::Vec;

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::video::Window;

use super::super::MovementInputState;
use super::super::view::Camera;
use crate::gameplay::model::MapPoint;
use crate::gameplay::{GameplayError, runtime_error};

/// CTP2-style green used for selected-unit and path movement feedback.
const MOVEMENT_FEEDBACK_COLOR: Color = Color::RGBA(32, 255, 64, 255);
/// Pixel offsets used to thicken movement-feedback lines.
const MOVEMENT_FEEDBACK_STROKE_OFFSETS: [i32; 3] = [-1, 0, 1];
/// Length of each selected-unit bracket arm in screen pixels.
const SELECTION_BRACKET_LENGTH: i32 = 14;
/// Outset around a unit sprite target used for selected-unit brackets.
const SELECTION_BRACKET_MARGIN: i32 = 4;

/// One screen-space point used by movement feedback geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MovementScreenPoint {
    /// Screen x coordinate.
    pub(super) x: i32,
    /// Screen y coordinate.
    pub(super) y: i32,
}

/// One screen-space line segment used by movement feedback geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MovementLineSegment {
    /// Segment start point.
    pub(super) start: MovementScreenPoint,
    /// Segment end point.
    pub(super) end: MovementScreenPoint,
}

/// Returns the stroke offsets used by movement-feedback lines.
pub(super) const fn movement_feedback_stroke_offsets() -> &'static [i32] {
    &MOVEMENT_FEEDBACK_STROKE_OFFSETS
}

/// Returns CTP2-style movement path line segments for an origin and computed path.
pub(super) fn planned_path_segments(
    camera: &Camera,
    origin: MapPoint,
    path: &[MapPoint],
) -> Vec<MovementLineSegment> {
    let mut segments = Vec::with_capacity(path.len());
    let mut previous = origin;
    for point in path {
        segments.push(MovementLineSegment {
            start: tile_line_anchor(*camera, previous),
            end: tile_line_anchor(*camera, *point),
        });
        previous = *point;
    }
    segments
}

/// Draws the currently dragged movement path as original-style line segments.
pub(super) fn draw_planned_path(
    canvas: &mut Canvas<Window>,
    input: &MovementInputState,
    camera: &Camera,
) -> Result<(), GameplayError> {
    let Some(origin) = input.planned_path_origin() else {
        return Ok(());
    };
    for segment in planned_path_segments(camera, origin, input.planned_path()) {
        draw_thick_line(canvas, segment)?;
    }
    Ok(())
}

/// Draws selected-unit brackets using CTP2-style corner segments.
pub(super) fn draw_selection_brackets(
    canvas: &mut Canvas<Window>,
    target: Rect,
) -> Result<(), GameplayError> {
    canvas.set_draw_color(MOVEMENT_FEEDBACK_COLOR);
    for segment in selection_bracket_segments(target) {
        draw_thick_line(canvas, segment)?;
    }
    Ok(())
}

/// Returns selected-unit bracket line segments for a sprite target rectangle.
pub(super) fn selection_bracket_segments(target: Rect) -> [MovementLineSegment; 8] {
    let width = i32::try_from(target.width()).map_or(0_i32, |value| value);
    let height = i32::try_from(target.height()).map_or(0_i32, |value| value);
    let left = target.x().saturating_sub(SELECTION_BRACKET_MARGIN);
    let top = target.y().saturating_sub(SELECTION_BRACKET_MARGIN);
    let right = target
        .x()
        .saturating_add(width)
        .saturating_add(SELECTION_BRACKET_MARGIN);
    let bottom = target
        .y()
        .saturating_add(height)
        .saturating_add(SELECTION_BRACKET_MARGIN);
    let horizontal = SELECTION_BRACKET_LENGTH.min(right.saturating_sub(left));
    let vertical = SELECTION_BRACKET_LENGTH.min(bottom.saturating_sub(top));
    [
        MovementLineSegment::new(left, top, left.saturating_add(horizontal), top),
        MovementLineSegment::new(left, top, left, top.saturating_add(vertical)),
        MovementLineSegment::new(right, top, right.saturating_sub(horizontal), top),
        MovementLineSegment::new(right, top, right, top.saturating_add(vertical)),
        MovementLineSegment::new(right, bottom, right.saturating_sub(horizontal), bottom),
        MovementLineSegment::new(right, bottom, right, bottom.saturating_sub(vertical)),
        MovementLineSegment::new(left, bottom, left.saturating_add(horizontal), bottom),
        MovementLineSegment::new(left, bottom, left, bottom.saturating_sub(vertical)),
    ]
}

impl MovementLineSegment {
    /// Creates a movement line segment from integer coordinates.
    const fn new(start_x: i32, start_y: i32, end_x: i32, end_y: i32) -> Self {
        Self {
            start: MovementScreenPoint {
                x: start_x,
                y: start_y,
            },
            end: MovementScreenPoint { x: end_x, y: end_y },
        }
    }
}

/// Draws a thick screen-space line segment.
fn draw_thick_line(
    canvas: &mut Canvas<Window>,
    segment: MovementLineSegment,
) -> Result<(), GameplayError> {
    canvas.set_draw_color(MOVEMENT_FEEDBACK_COLOR);
    for offset in movement_feedback_stroke_offsets() {
        draw_line_with_offset(canvas, segment, *offset, 0_i32)?;
        if *offset != 0_i32 {
            draw_line_with_offset(canvas, segment, 0_i32, *offset)?;
        }
    }
    Ok(())
}

/// Draws one offset copy of a line segment.
fn draw_line_with_offset(
    canvas: &mut Canvas<Window>,
    segment: MovementLineSegment,
    x_offset: i32,
    y_offset: i32,
) -> Result<(), GameplayError> {
    canvas
        .draw_line(
            Point::new(
                segment.start.x.saturating_add(x_offset),
                segment.start.y.saturating_add(y_offset),
            ),
            Point::new(
                segment.end.x.saturating_add(x_offset),
                segment.end.y.saturating_add(y_offset),
            ),
        )
        .map_err(runtime_error)
}

/// Returns the original CTP2 path-line anchor for a tile.
fn tile_line_anchor(camera: Camera, point: MapPoint) -> MovementScreenPoint {
    let (screen_x, screen_y) = camera.tile_to_screen(point);
    let half_width = i32::try_from(camera.tile_width())
        .map_or(0_i32, |value| value.checked_div(2_i32).unwrap_or(0_i32));
    let height = i32::try_from(camera.tile_height()).map_or(0_i32, |value| value);
    MovementScreenPoint {
        x: screen_x.saturating_add(half_width),
        y: screen_y.saturating_add(height),
    }
}
