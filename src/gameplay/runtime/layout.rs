//! Shared original-game geometry for drawing and accessible hit testing.
//!
//! The reference is an 800×600 frame resized to 640×480. UI art remains at
//! native LDL size; SDL scales the *whole* frame, not individual UI panels.

use sdl2::rect::Rect;

use super::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH};
use crate::gameplay::ui_model::{
    CONTROL_PANEL_HEIGHT, CONTROL_PANEL_WIDTH, END_TURN_BUTTON_HEIGHT, END_TURN_BUTTON_WIDTH,
    END_TURN_BUTTON_X, END_TURN_BUTTON_Y, MENU_BAR_HEIGHT, MenuRect,
};

/// Radar width in the original low-resolution layout.
pub(super) const RADAR_WIDTH: u32 = 198;
/// Height including the reference's two filter rows, square map and frame.
pub(super) const RADAR_HEIGHT: u32 = 240;
/// Toolbar height including the second row of map controls.
pub(super) const RADAR_TOOLBAR_HEIGHT: u32 = 54;

/// Original `ControlPanelWindow::AdjustToScreen` bottom-right anchoring.
pub(super) fn panel_origin() -> (i32, i32) {
    (
        i32::try_from(DEFAULT_WINDOW_WIDTH.saturating_sub(CONTROL_PANEL_WIDTH))
            .map_or(0_i32, |v| v),
        i32::try_from(DEFAULT_WINDOW_HEIGHT.saturating_sub(CONTROL_PANEL_HEIGHT))
            .map_or(0_i32, |v| v),
    )
}

/// Translates one panel-relative art/control rectangle into frame coordinates.
pub(super) fn panel_rect(x: i32, y: i32, width: u32, height: u32) -> Rect {
    let origin = panel_origin();
    Rect::new(
        origin.0.saturating_add(x),
        origin.1.saturating_add(y),
        width,
        height,
    )
}

/// Screen-space end-turn control, shared with event dispatch.
pub(super) fn end_turn_button_rect() -> MenuRect {
    let rect = panel_rect(
        END_TURN_BUTTON_X,
        END_TURN_BUTTON_Y,
        END_TURN_BUTTON_WIDTH,
        END_TURN_BUTTON_HEIGHT,
    );
    MenuRect {
        x: rect.x(),
        y: rect.y(),
        width: rect.width(),
        height: rect.height(),
    }
}

/// Bottom-left radar, not a full-height right-side debug panel.
pub(super) fn radar_rect() -> Rect {
    Rect::new(
        0_i32,
        i32::try_from(DEFAULT_WINDOW_HEIGHT.saturating_sub(RADAR_HEIGHT)).map_or(0_i32, |v| v),
        RADAR_WIDTH,
        RADAR_HEIGHT,
    )
}

/// Tests actual chrome regions; unused upper-right space remains interactive map.
pub(super) fn click_in_non_map_ui(x: i32, y: i32) -> bool {
    y < i32::try_from(MENU_BAR_HEIGHT).map_or(0_i32, |v| v)
        || panel_rect(0_i32, 0_i32, CONTROL_PANEL_WIDTH, CONTROL_PANEL_HEIGHT)
            .contains_point((x, y))
        || radar_rect().contains_point((x, y))
}
