//! Rendering helpers for CTP2-style click-drag movement planning.

#![expect(
    clippy::arithmetic_side_effects,
    reason = "Movement feedback geometry mirrors fixed original CTP2 pixel offsets and turn-box sizes"
)]

extern crate alloc;

use alloc::string::ToString;
use alloc::vec::Vec;

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::ttf::Font;
use sdl2::video::Window;

use super::super::MovementInputState;
use super::super::view::Camera;
use super::text::draw_text;
use crate::gameplay::model::MapPoint;
use crate::gameplay::{GameplayError, runtime_error};

/// CTP2 `COLOR_GREEN` used for legal movement feedback.
const MOVEMENT_GO_COLOR: Color = Color::RGBA(32, 255, 64, 255);
/// CTP2 `COLOR_YELLOW` used for selected units that cannot currently move.
const MOVEMENT_WAIT_COLOR: Color = Color::RGBA(255, 225, 32, 255);
/// CTP2 `COLOR_RED` used for invalid movement feedback.
const MOVEMENT_STOP_COLOR: Color = Color::RGBA(230, 32, 32, 255);
/// CTP2 `COLOR_BLACK` used for turn-box frames/text.
const MOVEMENT_TEXT_COLOR: Color = Color::RGBA(0, 0, 0, 255);
/// Pixel offsets used to thicken movement-feedback lines.
const MOVEMENT_FEEDBACK_STROKE_OFFSETS: [i32; 3] = [-1, 0, 1];
/// Length of each selected-unit bracket arm in screen pixels.
const SELECTION_BRACKET_LENGTH: i32 = 14;
/// Outset around a unit sprite target used for selected-unit brackets.
const SELECTION_BRACKET_MARGIN: i32 = 4;
/// Half-size of the original CTP2 turn box in pixels.
const TURN_BOX_EDGE_SIZE: i32 = 8;
/// Minimum half-size enforced by the original CTP2 renderer.
const TURN_BOX_MINIMUM_EDGE_SIZE: i32 = 4;

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

/// One CTP2-style movement turn marker box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MovementTurnMarker {
    /// Screen rectangle for the marker.
    pub(super) rect: Rect,
    /// One-based turn label painted inside the marker.
    pub(super) label: u32,
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

/// Returns CTP2-style turn marker boxes for a computed path.
pub(super) fn planned_path_turn_markers(
    camera: &Camera,
    path: &[MapPoint],
    steps_per_turn: usize,
) -> Vec<MovementTurnMarker> {
    let normalized_steps = steps_per_turn.max(1_usize);
    let mut markers = Vec::new();
    for (index, point) in path.iter().enumerate() {
        let step_number = index.saturating_add(1_usize);
        if step_number % normalized_steps != 0_usize {
            continue;
        }
        markers.push(MovementTurnMarker {
            rect: turn_marker_rect(camera, *point),
            label: u32::try_from(step_number.checked_div(normalized_steps).unwrap_or(1_usize))
                .map_or(u32::MAX, |value| value),
        });
    }
    markers
}

/// Draws the currently planned movement path as original-style line segments and turn boxes.
pub(super) fn draw_planned_path(
    canvas: &mut Canvas<Window>,
    input: &MovementInputState,
    camera: &Camera,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    if let Some(origin) = input.planned_path_origin() {
        for segment in planned_path_segments(camera, origin, input.planned_path()) {
            draw_thick_line(canvas, segment, MOVEMENT_GO_COLOR)?;
        }
        for marker in planned_path_turn_markers(camera, input.planned_path(), 1_usize) {
            draw_turn_marker(canvas, font, marker, MOVEMENT_GO_COLOR)?;
        }
    }
    if let Some(origin) = input.planned_bad_path_origin() {
        for segment in planned_path_segments(camera, origin, input.planned_bad_path()) {
            draw_thick_line(canvas, segment, MOVEMENT_STOP_COLOR)?;
        }
    }
    Ok(())
}

/// Returns the bracket color used by the original unit actor selection overlay.
pub(super) const fn selection_feedback_color(can_move: bool) -> Color {
    if can_move {
        MOVEMENT_GO_COLOR
    } else {
        MOVEMENT_WAIT_COLOR
    }
}

/// Draws selected-unit brackets using CTP2-style corner segments.
pub(super) fn draw_selection_brackets(
    canvas: &mut Canvas<Window>,
    target: Rect,
    color: Color,
) -> Result<(), GameplayError> {
    canvas.set_draw_color(color);
    for segment in selection_bracket_segments(target) {
        draw_thick_line(canvas, segment, color)?;
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
    color: Color,
) -> Result<(), GameplayError> {
    canvas.set_draw_color(color);
    for offset in movement_feedback_stroke_offsets() {
        draw_line_with_offset(canvas, segment, *offset, 0_i32)?;
        if *offset != 0_i32 {
            draw_line_with_offset(canvas, segment, 0_i32, *offset)?;
        }
    }
    Ok(())
}

/// Draws a CTP2 movement turn marker box.
fn draw_turn_marker(
    canvas: &mut Canvas<Window>,
    font: Option<&Font<'_, '_>>,
    marker: MovementTurnMarker,
    color: Color,
) -> Result<(), GameplayError> {
    canvas.set_draw_color(color);
    canvas.fill_rect(marker.rect).map_err(runtime_error)?;
    canvas.set_draw_color(MOVEMENT_TEXT_COLOR);
    canvas.draw_rect(marker.rect).map_err(runtime_error)?;
    let label = marker.label.to_string();
    draw_text(
        canvas,
        font,
        &label,
        marker.rect.x().saturating_add(5_i32),
        marker.rect.y().saturating_add(4_i32),
        MOVEMENT_TEXT_COLOR,
    )
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

/// Returns a CTP2 turn-marker rectangle centered on a tile path anchor.
fn turn_marker_rect(camera: &Camera, point: MapPoint) -> Rect {
    let anchor = tile_line_anchor(*camera, point);
    let edge = TURN_BOX_EDGE_SIZE.max(TURN_BOX_MINIMUM_EDGE_SIZE);
    let size = u32::try_from(edge.saturating_mul(2_i32)).map_or(8_u32, |value| value);
    Rect::new(
        anchor.x.saturating_sub(edge),
        anchor.y.saturating_sub(edge),
        size,
        size,
    )
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
