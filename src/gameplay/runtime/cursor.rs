//! CTP2 software cursors and focus-safe, clock-driven screen-edge scrolling.
//!
//! Artwork/hotspots: `civ3.ldl:CivMouse`. Thresholds and vertical corner
//! priority: `civ3_main.cpp:ui_CheckForScroll`. Software composition keeps the
//! cursor at the same scale as the game and makes F12/pixel tests inspect it.

use super::{Camera, MovementInputState, click_in_non_map_ui};
use crate::gameplay::model::{GameState, PlayerId};
use crate::gameplay::ui_model::InGameUiModel;

/// Original cursor variants supported by current gameplay orders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum CursorKind {
    /// Retail pointing hand, including over controls (never the OS arrow).
    Hand,
    /// Legal movement destination.
    Move,
    /// Unreachable destination.
    NoMove,
    /// Visible hostile occupant on a legal path.
    Assault,
    /// Scroll toward the top edge.
    ScrollUp,
    /// Scroll toward the bottom edge.
    ScrollDown,
    /// Scroll toward the left edge.
    ScrollLeft,
    /// Scroll toward the right edge.
    ScrollRight,
}

impl CursorKind {
    /// Installed art number and exact original hotspot, not a guessed center.
    pub(super) const fn art(self) -> (u16, (i32, i32)) {
        match self {
            Self::Hand => (1, (0, 0)),
            Self::Move => (2, (21, 21)),
            Self::NoMove => (3, (21, 21)),
            Self::Assault => (4, (21, 21)),
            Self::ScrollUp => (10, (23, 0)),
            Self::ScrollDown => (11, (23, 45)),
            Self::ScrollLeft => (12, (0, 23)),
            Self::ScrollRight => (13, (45, 22)),
        }
    }

    /// Camera direction; moving the viewport right translates world pixels left.
    const fn scroll_delta(self) -> (i32, i32) {
        match self {
            Self::ScrollUp => (0, -1),
            Self::ScrollDown => (0, 1),
            Self::ScrollLeft => (-1, 0),
            Self::ScrollRight => (1, 0),
            Self::Hand | Self::Move | Self::NoMove | Self::Assault => (0, 0),
        }
    }
}

/// Tracks pointer/focus events without changing authoritative world state.
pub(super) fn track_pointer(
    event: &sdl2::event::Event,
    ui: &mut InGameUiModel,
    movement: &mut MovementInputState,
) {
    use sdl2::event::{Event, WindowEvent};
    match *event {
        Event::MouseMotion { x, y, .. }
        | Event::MouseButtonDown { x, y, .. }
        | Event::MouseButtonUp { x, y, .. } => {
            ui.pointer = Some((x, y));
            ui.keyboard_status = false;
        }
        Event::Window {
            win_event: WindowEvent::Leave | WindowEvent::FocusLost,
            ..
        } => {
            ui.pointer = None;
            ui.keyboard_status = false;
            ui.hovered_tile = None;
            movement.cancel_drag();
        }
        _ => {}
    }
}

/// Returns the original edge cursor, with vertical edges taking corner priority.
pub(super) fn edge_direction(pointer: Option<(i32, i32)>) -> Option<CursorKind> {
    let (x, y) = pointer?;
    if !(0_i32..800_i32).contains(&x) || !(0_i32..600_i32).contains(&y) {
        return None;
    }
    if y <= 2 {
        Some(CursorKind::ScrollUp)
    } else if y >= 598 {
        Some(CursorKind::ScrollDown)
    } else if x < 4 {
        Some(CursorKind::ScrollLeft)
    } else if x > 796 {
        Some(CursorKind::ScrollRight)
    } else {
        None
    }
}

/// Whether a modal UI owns pointer input and prevents edge scrolling/path cursors.
pub(super) fn pointer_blocked(state: &GameState, ui: &InGameUiModel) -> bool {
    !state.popups.is_empty()
        || ui.open_menu.is_some()
        || ui.great_library.is_some()
        || ui.context.is_some()
        || ui.map_info.is_some()
        || ui.operation.is_some()
        || ui.messages.current().is_some()
        || ui.management.view.is_some()
}

/// Resolves feedback using the same legal/illegal preview that will be submitted.
pub(super) fn cursor_for_frame(
    state: &GameState,
    ui: &InGameUiModel,
    input: &MovementInputState,
) -> CursorKind {
    if pointer_blocked(state, ui) {
        return CursorKind::Hand;
    }
    if let Some(edge) = edge_direction(ui.pointer) {
        return edge;
    }
    let Some((x, y)) = ui.pointer else {
        return CursorKind::Hand;
    };
    if click_in_non_map_ui(x, y) || input.active_unit().is_none() {
        return CursorKind::Hand;
    }
    if !input.planned_bad_path().is_empty() {
        return CursorKind::NoMove;
    }
    if input.planned_path().is_empty() {
        return CursorKind::Hand;
    }
    if ui.hovered_tile.is_some_and(|point| {
        state
            .tile(point)
            .is_some_and(|tile| tile.fog == crate::gameplay::model::FogState::Visible)
            && state
                .units
                .iter()
                .any(|unit| unit.alive && unit.position == point && unit.owner != PlayerId::Human)
    }) {
        CursorKind::Assault
    } else {
        CursorKind::Move
    }
}

/// `profileDB.cpp` initializes ScrollDelay=3; `ui_CheckForScroll` uses 100ms units.
const SCROLL_DELAY_MS: u64 = 300;
/// Presentation ticks retain fractional time at varying rendering frame rates.
const SCROLL_TICK_MS: u64 = 16;

/// Re-evaluates the tile under a stationary pointer after a camera pan. Otherwise
/// edge scrolling would keep ordering toward the stale, pre-scroll destination.
pub(super) fn refresh_map_pointer(
    state: &GameState,
    ui: &mut InGameUiModel,
    camera: &Camera,
    input: &mut MovementInputState,
) {
    ui.hovered_tile = ui
        .pointer
        .filter(|&(x, y)| !click_in_non_map_ui(x, y))
        .and_then(|(x, y)| camera.screen_to_tile(x, y))
        .filter(|point| state.in_bounds(*point));
    if let Some(point) = ui.hovered_tile {
        input.preview_path_to(state, point);
    } else {
        input.cancel_drag();
    }
}

/// Edge-scroll timing is independent of mouse-event frequency and render FPS.
#[derive(Debug, Default)]
pub(super) struct EdgeScroll {
    /// Active edge and its uninterrupted entry timestamp.
    active: Option<(CursorKind, u64)>,
    /// Last simulated scroll tick, in milliseconds.
    last_tick: u64,
}

impl EdgeScroll {
    /// Applies original delay/acceleration on bounded 16ms presentation ticks.
    /// A long suspended frame does not fling the camera across the map.
    pub(super) fn update(
        &mut self,
        camera: &mut Camera,
        pointer: Option<(i32, i32)>,
        blocked: bool,
        now: u64,
    ) {
        let Some(direction) = edge_direction(pointer).filter(|_| !blocked) else {
            self.active = None;
            return;
        };
        if self.active.is_none_or(|(edge, _)| edge != direction) {
            self.active = Some((direction, now));
            self.last_tick = now.saturating_add(SCROLL_DELAY_MS.saturating_sub(SCROLL_TICK_MS));
        }
        let Some((_, entered)) = self.active else {
            return;
        };
        if now < entered.saturating_add(SCROLL_DELAY_MS) {
            return;
        }
        let available = now
            .saturating_sub(self.last_tick)
            .div_euclid(SCROLL_TICK_MS);
        let (dx, dy) = direction.scroll_delta();
        for _tick in 0..available.min(4) {
            self.last_tick = self.last_tick.saturating_add(SCROLL_TICK_MS);
            let acceleration = self
                .last_tick
                .saturating_sub(entered.saturating_add(SCROLL_DELAY_MS))
                .div_euclid(50)
                .saturating_add(1);
            let horizontal = acceleration.min(u64::from(camera.tile_width()));
            let vertical = acceleration.min(u64::from(camera.tile_height().div_euclid(3)));
            let amount_x = i32::try_from(horizontal).unwrap_or(i32::MAX);
            let amount_y = i32::try_from(vertical).unwrap_or(i32::MAX);
            camera.pan_pixels(dx.saturating_mul(amount_x), dy.saturating_mul(amount_y));
        }
        if available > 4 {
            self.last_tick = now;
        }
    }
}

/// Restores the caller's OS-cursor visibility on every exit path, including errors.
pub(super) struct SoftwareCursorGuard {
    /// SDL mouse handle retained until gameplay releases the window.
    mouse: sdl2::mouse::MouseUtil,
    /// Visibility state inherited from the menu/window owner.
    was_visible: bool,
}

impl SoftwareCursorGuard {
    /// Hides the OS cursor while the original cursor is composed in the framebuffer.
    pub(super) fn new() -> Result<Self, crate::gameplay::GameplayError> {
        let sdl = sdl2::init().map_err(crate::gameplay::runtime_error)?;
        let mouse = sdl.mouse();
        let was_visible = mouse.is_cursor_showing();
        mouse.show_cursor(false);
        Ok(Self { mouse, was_visible })
    }

    /// Focus/window handoffs may restore SDL's native visibility; keep the OS
    /// arrow hidden on every presented gameplay frame, not only at loop entry.
    pub(super) fn refresh(&self) {
        self.mouse.show_cursor(false);
    }
}

impl Drop for SoftwareCursorGuard {
    fn drop(&mut self) {
        self.mouse.show_cursor(self.was_visible);
    }
}

#[cfg(test)]
mod tests;
