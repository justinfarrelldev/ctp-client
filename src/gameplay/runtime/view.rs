//! Gameplay camera and coordinate conversion helpers.

use super::super::model::{GameState, MapPoint};
use super::super::ui_model::MENU_BAR_HEIGHT;

/// Fallback tile width in screen pixels.
pub(super) const TILE_WIDTH: i32 = 32;
/// Fallback tile height in screen pixels.
pub(super) const TILE_HEIGHT: i32 = 24;

/// Camera state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Camera {
    /// Left world tile.
    origin_x: i32,
    /// Top world tile.
    origin_y: i32,
    /// Integer zoom factor.
    zoom: i32,
}

impl Camera {
    /// Creates a camera centered around the selected unit.
    pub(super) fn centered_on_selected(state: &GameState) -> Self {
        let selected = state
            .selected_unit()
            .map_or(MapPoint::new(0, 0), |unit| unit.position);
        Self {
            origin_x: i32::from(selected.x).saturating_sub(14),
            origin_y: i32::from(selected.y).saturating_sub(10),
            zoom: 1,
        }
    }

    /// Converts tile to screen coordinates.
    pub(super) fn tile_to_screen(self, point: MapPoint) -> (i32, i32) {
        (
            i32::from(point.x)
                .saturating_sub(self.origin_x)
                .saturating_mul(TILE_WIDTH.saturating_mul(self.zoom)),
            i32::from(point.y)
                .saturating_sub(self.origin_y)
                .saturating_mul(TILE_HEIGHT.saturating_mul(self.zoom))
                .saturating_add(i32::try_from(MENU_BAR_HEIGHT).map_or(24, |value| value)),
        )
    }

    /// Converts screen coordinates to a tile.
    pub(super) fn screen_to_tile(self, x: i32, y: i32) -> Option<MapPoint> {
        let tile_width = TILE_WIDTH.checked_mul(self.zoom)?;
        let tile_height = TILE_HEIGHT.checked_mul(self.zoom)?;
        if tile_width == 0_i32 || tile_height == 0_i32 {
            return None;
        }
        let menu_height = i32::try_from(MENU_BAR_HEIGHT).ok()?;
        let tile_x = x.checked_div(tile_width)?.saturating_add(self.origin_x);
        let tile_y = y
            .saturating_sub(menu_height)
            .checked_div(tile_height)?
            .saturating_add(self.origin_y);
        if tile_x < 0_i32 || tile_y < 0_i32 {
            return None;
        }
        Some(MapPoint::new(
            u16::try_from(tile_x).ok()?,
            u16::try_from(tile_y).ok()?,
        ))
    }

    /// Current tile width.
    pub(super) fn tile_width(self) -> u32 {
        u32::try_from(TILE_WIDTH.saturating_mul(self.zoom)).map_or(1, |value| value)
    }

    /// Current tile height.
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
