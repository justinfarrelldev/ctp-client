//! Gameplay camera and coordinate conversion helpers.

use super::super::model::{GameState, MapPoint};
use super::super::ui_model::MENU_BAR_HEIGHT;
/// Original terrain image width in screen pixels.
pub(super) const TILE_WIDTH: i32 = 94;
/// Original terrain image height in screen pixels, including headroom.
pub(super) const TILE_HEIGHT: i32 = 72;
/// Horizontal step between adjacent isometric tiles.
const TILE_STEP_X: i32 = 47;
/// Vertical step between adjacent isometric tiles.
const TILE_STEP_Y: i32 = 24;
/// Default viewport center X.
const VIEW_CENTER_X: i32 = 400;
/// Default viewport center Y.
const VIEW_CENTER_Y: i32 = 288;

/// Iterates each map cell once in back-to-front isometric screen-depth order.
pub(super) fn painter_order(width: u16, height: u16) -> impl Iterator<Item = MapPoint> {
    (0_u32..u32::from(width).saturating_add(u32::from(height))).flat_map(move |depth| {
        (0_u16..width).filter_map(move |x| {
            let y = depth
                .checked_sub(u32::from(x))
                .and_then(|value| u16::try_from(value).ok())
                .filter(|value| *value < height)?;
            Some(MapPoint::new(x, y))
        })
    })
}

/// Converts a signed coordinate pair into a map point.
fn map_point_from_signed(x: i32, y: i32) -> Option<MapPoint> {
    if x < 0_i32 || y < 0_i32 {
        return None;
    }
    Some(MapPoint::new(
        u16::try_from(x).ok()?,
        u16::try_from(y).ok()?,
    ))
}

/// Divides with rounding to the nearest integer.
fn rounded_div(numerator: i32, denominator: i32) -> Option<i32> {
    if denominator == 0_i32 {
        return None;
    }
    let half = denominator.unsigned_abs().checked_div(2_u32)?;
    let adjusted = if numerator >= 0_i32 {
        numerator.checked_add(i32::try_from(half).ok()?)?
    } else {
        numerator.checked_sub(i32::try_from(half).ok()?)?
    };
    adjusted.checked_div(denominator)
}

/// Camera state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Camera {
    /// Screen-space X offset applied after isometric conversion.
    offset_x: i32,
    /// Screen-space Y offset applied after isometric conversion.
    offset_y: i32,
    /// Integer zoom factor.
    zoom: i32,
}

impl Camera {
    /// Creates an identity-like camera for geometry-only tests.
    #[cfg(test)]
    pub(super) const fn for_test_identity() -> Self {
        Self {
            offset_x: 0,
            offset_y: 0,
            zoom: 1,
        }
    }

    /// Creates a camera centered around the selected unit.
    pub(super) fn centered_on_selected(state: &GameState) -> Self {
        let selected = state
            .selected_unit()
            .map_or(MapPoint::new(0, 0), |unit| unit.position);
        let zoom = 1_i32;
        let base_x = i32::from(selected.x)
            .saturating_sub(i32::from(selected.y))
            .saturating_mul(TILE_STEP_X.saturating_mul(zoom));
        let base_y = i32::from(selected.x)
            .saturating_add(i32::from(selected.y))
            .saturating_mul(TILE_STEP_Y.saturating_mul(zoom));
        let menu_height = i32::try_from(MENU_BAR_HEIGHT).map_or(24_i32, |value| value);
        Self {
            offset_x: VIEW_CENTER_X
                .saturating_sub(base_x)
                .saturating_sub(47_i32.saturating_mul(zoom)),
            offset_y: VIEW_CENTER_Y
                .saturating_sub(menu_height)
                .saturating_sub(base_y)
                .saturating_sub(36_i32.saturating_mul(zoom)),
            zoom,
        }
    }

    /// Converts tile to screen coordinates.
    pub(super) fn tile_to_screen(self, point: MapPoint) -> (i32, i32) {
        let step_x = TILE_STEP_X.saturating_mul(self.zoom);
        let step_y = TILE_STEP_Y.saturating_mul(self.zoom);
        (
            i32::from(point.x)
                .saturating_sub(i32::from(point.y))
                .saturating_mul(step_x)
                .saturating_add(self.offset_x),
            i32::from(point.x)
                .saturating_add(i32::from(point.y))
                .saturating_mul(step_y)
                .saturating_add(self.offset_y)
                .saturating_add(i32::try_from(MENU_BAR_HEIGHT).map_or(24, |value| value)),
        )
    }

    /// Converts tile to the center of the visible CTP2 ground diamond.
    #[cfg(test)]
    pub(super) fn tile_click_center_to_screen(self, point: MapPoint) -> (i32, i32) {
        let (screen_x, screen_y) = self.tile_to_screen(point);
        (
            screen_x.saturating_add(TILE_STEP_X.saturating_mul(self.zoom)),
            screen_y.saturating_add(
                TILE_HEIGHT
                    .saturating_sub(TILE_STEP_Y)
                    .saturating_mul(self.zoom),
            ),
        )
    }

    /// Converts tile to a representative visible CTP2 unit-body click point.
    #[cfg(test)]
    pub(super) fn tile_unit_body_to_screen(self, point: MapPoint) -> (i32, i32) {
        let (screen_x, screen_y) = self.tile_to_screen(point);
        (
            screen_x.saturating_add(
                TILE_WIDTH
                    .checked_div(2_i32)
                    .unwrap_or(0_i32)
                    .saturating_mul(self.zoom),
            ),
            screen_y.saturating_add(
                TILE_HEIGHT
                    .checked_div(2_i32)
                    .unwrap_or(0_i32)
                    .saturating_mul(self.zoom),
            ),
        )
    }

    /// Converts screen coordinates to a tile.
    pub(super) fn screen_to_tile(self, x: i32, y: i32) -> Option<MapPoint> {
        self.screen_to_tile_from_visible_diamond(x, y)
            .or_else(|| self.screen_to_tile_from_grid_origin(x, y))
    }

    /// Converts visible CTP2 tile-diamond pixels to a tile.
    fn screen_to_tile_from_visible_diamond(self, x: i32, y: i32) -> Option<MapPoint> {
        let step_x = TILE_STEP_X.checked_mul(self.zoom)?;
        let step_y = TILE_STEP_Y.checked_mul(self.zoom)?;
        if step_x == 0_i32 || step_y == 0_i32 {
            return None;
        }
        let menu_height = i32::try_from(MENU_BAR_HEIGHT).ok()?;
        let half_width = TILE_STEP_X.checked_mul(self.zoom)?;
        let ground_center_y = TILE_HEIGHT
            .saturating_sub(TILE_STEP_Y)
            .checked_mul(self.zoom)?;
        let world_x = x.saturating_sub(self.offset_x).saturating_sub(half_width);
        let world_y = y
            .saturating_sub(menu_height)
            .saturating_sub(self.offset_y)
            .saturating_sub(ground_center_y);
        let diagonal_a = rounded_div(world_y, step_y)?;
        let diagonal_b = rounded_div(world_x, step_x)?;
        let estimated_x = diagonal_a.saturating_add(diagonal_b).checked_div(2)?;
        let estimated_y = diagonal_a.saturating_sub(diagonal_b).checked_div(2)?;
        self.nearest_visible_diamond_tile(x, y, estimated_x, estimated_y)
    }

    /// Converts legacy grid-origin coordinates to a tile for non-diamond callers.
    fn screen_to_tile_from_grid_origin(self, x: i32, y: i32) -> Option<MapPoint> {
        let step_x = TILE_STEP_X.checked_mul(self.zoom)?;
        let step_y = TILE_STEP_Y.checked_mul(self.zoom)?;
        if step_x == 0_i32 || step_y == 0_i32 {
            return None;
        }
        let menu_height = i32::try_from(MENU_BAR_HEIGHT).ok()?;
        let world_x = x.saturating_sub(self.offset_x);
        let world_y = y.saturating_sub(menu_height).saturating_sub(self.offset_y);
        let diagonal_a = world_y.checked_div(step_y)?;
        let diagonal_b = world_x.checked_div(step_x)?;
        map_point_from_signed(
            diagonal_a.saturating_add(diagonal_b).checked_div(2)?,
            diagonal_a.saturating_sub(diagonal_b).checked_div(2)?,
        )
    }

    /// Returns the nearest visible diamond tile around an estimated coordinate.
    fn nearest_visible_diamond_tile(
        self,
        x: i32,
        y: i32,
        estimated_x: i32,
        estimated_y: i32,
    ) -> Option<MapPoint> {
        let mut best: Option<(i32, MapPoint)> = None;
        for candidate_y in estimated_y.saturating_sub(2_i32)..=estimated_y.saturating_add(2_i32) {
            for candidate_x in estimated_x.saturating_sub(2_i32)..=estimated_x.saturating_add(2_i32)
            {
                let Some(point) = map_point_from_signed(candidate_x, candidate_y) else {
                    continue;
                };
                let Some(score) = self.visible_diamond_score(x, y, point) else {
                    continue;
                };
                if best.is_none_or(|(best_score, _point)| score < best_score) {
                    best = Some((score, point));
                }
            }
        }
        best.map(|(_score, point)| point)
    }

    /// Returns an inside-diamond score for a candidate tile.
    fn visible_diamond_score(self, x: i32, y: i32, point: MapPoint) -> Option<i32> {
        let (screen_x, screen_y) = self.tile_to_screen(point);
        let half_width = TILE_STEP_X.checked_mul(self.zoom)?;
        let half_height = TILE_STEP_Y.checked_mul(self.zoom)?;
        let center_x = screen_x.saturating_add(half_width);
        let center_y = screen_y.saturating_add(
            TILE_HEIGHT
                .saturating_sub(TILE_STEP_Y)
                .checked_mul(self.zoom)?,
        );
        let dx = x.saturating_sub(center_x).unsigned_abs();
        let dy = y.saturating_sub(center_y).unsigned_abs();
        let width = u32::try_from(half_width.max(1_i32)).ok()?;
        let height = u32::try_from(half_height.max(1_i32)).ok()?;
        let normalized = dx
            .saturating_mul(height)
            .saturating_add(dy.saturating_mul(width));
        let limit = width.saturating_mul(height);
        if normalized > limit {
            return None;
        }
        i32::try_from(normalized).ok()
    }

    /// Current tile texture width.
    pub(super) fn tile_width(self) -> u32 {
        u32::try_from(TILE_WIDTH.saturating_mul(self.zoom)).map_or(1, |value| value)
    }

    /// Current tile texture height.
    pub(super) fn tile_height(self) -> u32 {
        u32::try_from(TILE_HEIGHT.saturating_mul(self.zoom)).map_or(1, |value| value)
    }

    /// Zooms in.
    pub(super) fn zoom_in(&mut self) {
        self.zoom = self.zoom.saturating_add(1_i32).min(3_i32);
    }

    /// Zooms out.
    pub(super) fn zoom_out(&mut self) {
        self.zoom = self.zoom.saturating_sub(1_i32).max(1_i32);
    }
}

#[cfg(test)]
mod tests {
    use super::{Camera, TILE_HEIGHT, TILE_STEP_X, TILE_STEP_Y, TILE_WIDTH};
    use crate::gameplay::assets::tile::{
        TILE_GRID_HEIGHT, TILE_PIXEL_HEADROOM, TILE_PIXEL_HEIGHT, TILE_PIXEL_WIDTH,
    };
    use crate::gameplay::model::GameState;

    #[test]
    fn tile_constants_match_ctp2_source_dimensions() {
        assert_eq!(
            TILE_WIDTH,
            i32::from(TILE_PIXEL_WIDTH),
            "tile width should match tileset.h"
        );
        assert_eq!(
            TILE_HEIGHT,
            i32::from(TILE_GRID_HEIGHT),
            "tile grid height should include original headroom"
        );
        assert_eq!(
            TILE_STEP_X,
            i32::from(TILE_PIXEL_WIDTH)
                .checked_div(2_i32)
                .unwrap_or(0_i32),
            "horizontal step should be half the original tile width"
        );
        assert_eq!(
            TILE_STEP_Y,
            i32::from(TILE_PIXEL_HEIGHT)
                .checked_div(2_i32)
                .unwrap_or(0_i32),
            "vertical step should be half the original terrain diamond height"
        );
        assert_eq!(
            TILE_PIXEL_HEADROOM, 24_u16,
            "source headroom should remain 24px"
        );
    }

    #[test]
    fn camera_round_trips_visible_tile_pixels_at_native_zoom() {
        let state = GameState::new(24, 18, 1, 88);
        let selected = state
            .selected_unit()
            .expect("selected unit should exist")
            .position;
        let camera = Camera::centered_on_selected(&state);
        let (screen_x, screen_y) = camera.tile_to_screen(selected);
        let (center_x, center_y) = camera.tile_click_center_to_screen(selected);

        assert_eq!(
            camera.screen_to_tile(center_x, center_y),
            Some(selected),
            "clicking the center of the visible CTP2 tile diamond should hit that tile"
        );
        assert_eq!(
            camera.screen_to_tile(
                screen_x.saturating_add(TILE_WIDTH.checked_div(2_i32).unwrap_or(0_i32)),
                screen_y.saturating_add(TILE_HEIGHT.checked_div(2_i32).unwrap_or(0_i32)),
            ),
            Some(selected),
            "clicking the visible unit body above the diamond center should hit the unit tile"
        );
    }
}
