//! SDL runtime for the in-game single-player gameplay screen.

extern crate alloc;

use alloc::string::ToString;
use alloc::vec::Vec;
use core::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::time::Duration;

use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use sdl2::render::Canvas;
use sdl2::ttf::{Font, Hinting, Sdl2TtfContext};
use sdl2::video::Window;
use serde_json::{Value, json};

use super::model::{GameState, GameplayPopup, MapPoint, PopupKind, UnitId};
use super::server::ServerProjection;
use super::ui_model::{InGameAction, InGameUiModel};
use super::{GameplayError, GameplayOutcome, SinglePlayerLaunchConfig, runtime_error};

/// Runtime frame delay.
const FRAME_DELAY: Duration = Duration::from_millis(16);
/// Default SDL window width.
pub(super) const DEFAULT_WINDOW_WIDTH: u32 = 800;
/// Default SDL window height.
pub(super) const DEFAULT_WINDOW_HEIGHT: u32 = 600;
/// Candidate fonts used to draw gameplay labels over original-style chrome.
const FONT_CANDIDATES: [&str; 8] = [
    "C:/Windows/Fonts/timesbd.ttf",
    "/usr/share/fonts/truetype/msttcorefonts/timesbd.ttf",
    "/usr/share/fonts/truetype/liberation2/LiberationSerif-Bold.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSerif-Bold.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
    "/usr/share/fonts/truetype/liberation2/LiberationSans-Bold.ttf",
    "/usr/share/fonts/truetype/freefont/FreeSansBold.ttf",
    "C:/Windows/Fonts/arialbd.ttf",
];
/// Directional actor presentation with an injectable monotonic clock.
mod animation;
/// Authoritative server state alignment helpers.
mod authority;
/// Explicit source-aligned context selection and map information.
mod context_input;
/// Labeled controls shared by accessible automation and SDL pointer events.
pub mod controls;
/// Original software cursor art and edge-scrolling policy.
mod cursor;
/// Native-frame composition and uniform final scaling.
mod frame;
/// Original drawing/hit-test geometry.
mod layout;
/// Exclusive source-layout manager focus and authoritative interaction dispatch.
mod management_input;
/// CTP2-style movement input handling.
mod movement;
/// Gameplay frame rendering.
mod render;
/// Bootstrap and the live SDL presentation loop.
mod session;
/// Explicit opt-in framebuffer previews and capture.
pub mod snapshot;
/// Original CTP2 texture loading.
mod textures;
/// Camera and screen/tile coordinate conversion.
mod view;

use authority::{
    RuntimeAuthority, server_tile_ids_for_path, server_unit_id_for_unit,
    state_from_authority_projection,
};
use layout::{click_in_non_map_ui, end_turn_button_rect};
use movement::{MovementInputState, begin_or_activate_unit_drag};
use session::run_sdl_game_on_canvas;
use view::Camera;

/// Event-loop options used by production and tests.
struct RuntimeOptions {
    /// Initial SDL events to enqueue.
    initial_events: Vec<Event>,
    /// Maximum rendered frames before returning closed.
    frame_limit: Option<u32>,
}

impl RuntimeOptions {
    /// Interactive loop options.
    const fn interactive() -> Self {
        Self {
            initial_events: Vec::new(),
            frame_limit: None,
        }
    }

    /// Headless smoke options.
    #[cfg(test)]
    fn headless_smoke() -> Self {
        Self {
            initial_events: vec![Event::Quit { timestamp: 0 }],
            frame_limit: Some(3),
        }
    }
}

/// Runs the SDL gameplay event loop.
///
/// # Errors
///
/// Returns [`GameplayError`] if SDL initialization or drawing fails.
pub fn run_sdl_game(config: &SinglePlayerLaunchConfig) -> Result<GameplayOutcome, GameplayError> {
    run_sdl_game_with_options(config, RuntimeOptions::interactive())
}

/// Runs a headless smoke gameplay loop.
///
/// # Errors
///
/// Returns [`GameplayError`] when dummy-driver SDL initialization or rendering fails.
#[cfg(test)]
pub fn run_headless_smoke_for_test(
    config: &SinglePlayerLaunchConfig,
) -> Result<GameplayOutcome, GameplayError> {
    let guard = crate::sdl_test_lock::lock_sdl_for_test().map_err(runtime_error)?;
    let environment = HeadlessSdlEnvironment::apply();
    let result = run_sdl_game_with_options(config, RuntimeOptions::headless_smoke());
    environment.restore();
    drop(guard);
    result
}

/// Runs the SDL gameplay loop with injectable options.
fn run_sdl_game_with_options(
    config: &SinglePlayerLaunchConfig,
    options: RuntimeOptions,
) -> Result<GameplayOutcome, GameplayError> {
    let sdl = sdl2::init().map_err(runtime_error)?;
    let video = sdl.video().map_err(runtime_error)?;
    let window = video
        .window(
            "Call to Power II - Game",
            DEFAULT_WINDOW_WIDTH,
            DEFAULT_WINDOW_HEIGHT,
        )
        .position_centered()
        .build()
        .map_err(|error| runtime_error(error.to_string()))?;
    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|error| runtime_error(error.to_string()))?;
    let event_subsystem = sdl.event().map_err(runtime_error)?;
    for event in options.initial_events {
        event_subsystem.push_event(event).map_err(runtime_error)?;
    }
    let mut event_pump = sdl.event_pump().map_err(runtime_error)?;
    run_sdl_game_on_canvas(config, &mut canvas, &mut event_pump, options.frame_limit)
}

/// Runs gameplay on an existing SDL window/canvas owned by the menu runtime.
///
/// # Errors
///
/// Returns [`GameplayError`] when original assets, server bootstrap, or rendering fail.
pub(crate) fn run_sdl_game_on_existing_window(
    config: &SinglePlayerLaunchConfig,
    canvas: &mut Canvas<Window>,
    event_pump: &mut EventPump,
) -> Result<GameplayOutcome, GameplayError> {
    canvas
        .window_mut()
        .set_title("Call to Power II - Game")
        .map_err(|error| runtime_error(error.to_string()))?;
    run_sdl_game_on_canvas(config, canvas, event_pump, None)
}

/// Uniformly scales the reference game frame with minimal aspect-ratio borders.
fn configure_gameplay_canvas(canvas: &mut Canvas<Window>) -> Result<(), GameplayError> {
    canvas.set_viewport(None);
    canvas
        .set_logical_size(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
        .map_err(|error| runtime_error(error.to_string()))
}

/// Handles one SDL event.
fn handle_event(
    event: &Event,
    state: &mut GameState,
    ui: &mut InGameUiModel,
    camera: &mut Camera,
    movement: &mut MovementInputState,
    mut authority: Option<&mut RuntimeAuthority>,
) -> Option<GameplayOutcome> {
    cursor::track_pointer(event, ui, movement);
    if management_input::consume(event, state, ui, movement, authority.as_deref_mut()) {
        return None;
    }
    match event.clone() {
        Event::Quit { .. } => Some(GameplayOutcome::Closed),
        Event::KeyDown {
            keycode: Some(key),
            repeat: false,
            ..
        } if ui.context.is_some() || ui.map_info.is_some() => {
            context_input::handle_key(key, state, ui, movement, authority);
            None
        }
        Event::KeyDown { repeat: false, .. } if !state.popups.is_empty() => {
            state.dismiss_popup();
            None
        }
        Event::KeyDown {
            keycode: Some(keycode),
            keymod,
            repeat: false,
            ..
        } => apply_ui_action(
            InGameUiModel::action_for_key(keycode, keymod),
            state,
            ui,
            camera,
            movement,
            authority,
        ),
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } => handle_left_click(x, y, state, ui, camera, movement, authority),
        Event::MouseButtonUp {
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } => handle_left_release(x, y, state, ui, camera, movement, authority),
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Right,
            x,
            y,
            ..
        } => handle_right_click(x, y, state, ui, camera, movement, authority),
        Event::MouseMotion {
            x, y, mousestate, ..
        } => {
            ui.hovered_tile = if click_in_non_map_ui(x, y) {
                None
            } else {
                camera.screen_to_tile(x, y)
            };
            if ui.context.is_some() || ui.map_info.is_some() {
                if let Some(menu) = ui.context.as_mut() {
                    menu.keyboard_focus = false;
                }
                movement.cancel_drag();
                return None;
            }
            if ui.open_menu.is_some() {
                if let Some(category) = ui.menu_button_at(x, y) {
                    ui.open_menu = Some(category);
                }
                movement.cancel_drag();
                return None;
            }
            if mousestate.left() {
                handle_mouse_motion(x, y, state, camera, movement);
            } else {
                handle_mouse_hover(x, y, state, camera, movement);
            }
            None
        }
        Event::MouseWheel { y, .. } => {
            if ui.context.is_some() || ui.map_info.is_some() {
                return None;
            }
            match y.cmp(&0_i32) {
                Ordering::Greater => camera.zoom_in(),
                Ordering::Less => camera.zoom_out(),
                Ordering::Equal => {}
            }
            None
        }
        _ => None,
    }
}

/// Handles a left click.
fn handle_left_click(
    x: i32,
    y: i32,
    state: &mut GameState,
    ui: &mut InGameUiModel,
    camera: &mut Camera,
    movement: &mut MovementInputState,
    authority: Option<&mut RuntimeAuthority>,
) -> Option<GameplayOutcome> {
    if !state.popups.is_empty() {
        movement.cancel_drag();
        state.dismiss_popup();
        return None;
    }
    if let Some(context) = ui.context.take() {
        movement.cancel_drag();
        if let Some(intent) = context.intent_at(x, y) {
            context_input::apply_context_intent(intent, state, ui, movement, authority);
        }
        return None;
    }
    if ui.map_info.is_some() {
        if context_input::map_info_close_rect().contains(x, y) {
            ui.map_info = None;
        }
        return None;
    }
    if ui.open_menu.is_some() {
        let (menu_consumed, menu_action) = top_menu_click_action(x, y, ui);
        if menu_consumed {
            if let Some(action) = menu_action {
                return apply_ui_action(action, state, ui, camera, movement, authority);
            }
            return None;
        }
    }
    if click_in_end_turn_button(x, y) {
        return apply_ui_action(
            InGameAction::EndTurn,
            state,
            ui,
            camera,
            movement,
            authority,
        );
    }
    let (menu_consumed, menu_action) = top_menu_click_action(x, y, ui);
    if menu_consumed {
        if let Some(action) = menu_action {
            return apply_ui_action(action, state, ui, camera, movement, authority);
        }
        return None;
    }
    if ui.stack_expanded && ui.active_tab == crate::gameplay::ui_model::ControlPanelTab::Unit {
        if let Some(label) = crate::gameplay::ui_model::stack::entries(state)
            .iter()
            .find(|entry| entry.rect.contains(x, y))
            .map(|entry| entry.label.clone())
        {
            if crate::gameplay::ui_model::stack::activate(state, ui, &label) {
                movement.deactivate();
            }
            return None;
        }
    }
    if let Some(intent) = controls::intent_at(state, ui.active_tab, x, y) {
        movement.cancel_drag();
        return controls::apply_intent(intent, state, ui, camera, movement, authority);
    }
    if click_in_non_map_ui(x, y) {
        movement.cancel_drag();
        return None;
    }
    if let Some(point) = camera.screen_to_tile(x, y) {
        if let Some((unit_id, path)) = movement.finish_preview_order_to(state, point) {
            apply_finalized_move_path(state, movement, authority, unit_id, &path);
            return None;
        }
        if begin_or_activate_unit_drag(state, movement, point) {
            return None;
        }
    }
    None
}

/// Handles map mouse motion while a unit path is being dragged.
fn handle_mouse_motion(
    x: i32,
    y: i32,
    state: &GameState,
    camera: &Camera,
    movement: &mut MovementInputState,
) {
    if click_in_non_map_ui(x, y) {
        movement.cancel_drag();
        return;
    }
    if let Some(point) = camera.screen_to_tile(x, y) {
        movement.extend_drag_to(state, point);
    }
}

/// Handles map mouse motion when no button is pressed, matching CTP2 hover path drawing.
fn handle_mouse_hover(
    x: i32,
    y: i32,
    state: &GameState,
    camera: &Camera,
    movement: &mut MovementInputState,
) {
    if click_in_non_map_ui(x, y) {
        movement.cancel_drag();
        return;
    }
    if let Some(point) = camera.screen_to_tile(x, y) {
        movement.preview_path_to(state, point);
    }
}

/// Handles a left mouse-button release.
fn handle_left_release(
    x: i32,
    y: i32,
    state: &mut GameState,
    ui: &InGameUiModel,
    camera: &Camera,
    movement: &mut MovementInputState,
    authority: Option<&mut RuntimeAuthority>,
) -> Option<GameplayOutcome> {
    if !state.popups.is_empty()
        || ui.open_menu.is_some()
        || ui.context.is_some()
        || ui.map_info.is_some()
        || click_in_non_map_ui(x, y)
    {
        movement.cancel_drag();
        return None;
    }
    if let Some(point) = camera.screen_to_tile(x, y) {
        movement.extend_drag_to(state, point);
    }
    let (unit_id, path) = movement.finish_drag()?;
    apply_finalized_move_path(state, movement, authority, unit_id, &path);
    None
}

/// Applies or submits one finalized CTP2 movement path.
fn apply_finalized_move_path(
    state: &mut GameState,
    movement: &mut MovementInputState,
    authority: Option<&mut RuntimeAuthority>,
    unit_id: UnitId,
    path: &[MapPoint],
) {
    if !state.select_unit(unit_id) {
        movement.retain_selectable(state);
        return;
    }
    let before = state
        .units
        .iter()
        .filter(|unit| unit.id == unit_id)
        .cloned()
        .collect::<Vec<_>>();
    if let Some(active_authority) = authority {
        submit_authoritative_move_path(active_authority, state, unit_id, path);
    } else {
        state.order_selected_along_path(path.to_vec());
    }
    movement.actors.confirm(&before, state, path);
    movement.retain_selectable(state);
    // Keep the viewport stable while the accepted move is animated.
}

/// Handles a right click.
fn handle_right_click(
    x: i32,
    y: i32,
    state: &mut GameState,
    ui: &mut InGameUiModel,
    camera: &Camera,
    movement: &mut MovementInputState,
    _authority: Option<&mut RuntimeAuthority>,
) -> Option<GameplayOutcome> {
    if !state.popups.is_empty() {
        movement.cancel_drag();
        state.dismiss_popup();
        return None;
    }
    if ui.open_menu.is_some() {
        movement.cancel_drag();
        ui.open_menu = None;
        return None;
    }
    if click_in_non_map_ui(x, y) {
        return None;
    }
    if ui.context.take().is_some() || ui.map_info.take().is_some() {
        return None;
    }
    movement.cancel_drag();
    if let Some(point) = camera.screen_to_tile(x, y) {
        let rows = crate::gameplay::ui_model::context::context_at(state, point);
        if rows.len() == 1
            && rows.first().is_some_and(|row| {
                matches!(
                    row.intent,
                    crate::gameplay::ui_model::context::ContextIntent::MapInfo(_)
                )
            })
        {
            ui.map_info = Some(point);
        } else {
            ui.context = crate::gameplay::ui_model::context::ContextView::new(state, point, x, y);
        }
    }
    None
}

/// Applies a model/UI action and returns loop outcome if terminal.
fn apply_ui_action(
    action: InGameAction,
    state: &mut GameState,
    ui: &mut InGameUiModel,
    camera: &mut Camera,
    movement: &mut MovementInputState,
    authority: Option<&mut RuntimeAuthority>,
) -> Option<GameplayOutcome> {
    match action {
        InGameAction::Quit => Some(GameplayOutcome::Quit),
        InGameAction::NewGame => Some(GameplayOutcome::NewGame),
        InGameAction::ZoomIn => {
            camera.zoom_in();
            None
        }
        InGameAction::ZoomOut => {
            camera.zoom_out();
            None
        }
        InGameAction::None => None,
        InGameAction::MoveBy(dx, dy) => {
            if let Some(unit) = state.selected_unit() {
                if let Some(target) = unit.position.offset(dx, dy, state.width, state.height) {
                    apply_finalized_move_path(state, movement, authority, unit.id, &[target]);
                }
            }
            None
        }
        other => {
            if let Some(active_authority) = authority {
                if apply_authoritative_action(active_authority, state, other) {
                    if matches!(other, InGameAction::MoveBy(_, _) | InGameAction::EndTurn) {
                        *camera = Camera::centered_on_selected(state);
                    }
                    movement.retain_selectable(state);
                    return None;
                }
            }
            ui.apply_action(state, other);
            movement.retain_selectable(state);
            if matches!(other, InGameAction::MoveBy(_, _) | InGameAction::EndTurn) {
                *camera = Camera::centered_on_selected(state);
            }
            None
        }
    }
}

/// Applies an action through the server when the action mutates authoritative game state.
fn apply_authoritative_action(
    authority: &mut RuntimeAuthority,
    state: &mut GameState,
    action: InGameAction,
) -> bool {
    match action {
        InGameAction::EndTurn => {
            submit_authoritative_command(authority, state, "round-complete", json!({}));
            true
        }
        InGameAction::Settle | InGameAction::Fortify | InGameAction::Disband => {
            let Some(unit) = state.selected_unit() else {
                return true;
            };
            let route = match action {
                InGameAction::Settle => "found-city",
                InGameAction::Fortify => "fortify",
                _ => "disband",
            };
            let id = server_unit_id_for_unit(authority, unit.id);
            let legal = authority
                .session
                .projection
                .as_ref()
                .and_then(|viewer| viewer.interface.action(route, &id, None))
                .cloned();
            if let Some(legal) = legal {
                submit_authoritative_command(authority, state, route, legal.payload);
            } else {
                state.popups.push(GameplayPopup {
                    title: "Order unavailable".to_owned(),
                    body: "The server does not authorize this order for the selected unit"
                        .to_owned(),
                    kind: PopupKind::Info,
                });
            }
            true
        }
        InGameAction::Sentinel => {
            state.popups.push(GameplayPopup { title:"Sentinel unavailable".to_owned(), body:"The server does not expose a Sentinel/Sleep order; Fortify is a different operation".to_owned(),kind:PopupKind::Info });
            true
        }
        InGameAction::MoveBy(dx, dy) => {
            let Some(unit) = state.selected_unit() else {
                return true;
            };
            if let Some(target) = unit.position.offset(dx, dy, state.width, state.height) {
                submit_authoritative_move(authority, state, target);
            }
            true
        }
        _ => false,
    }
}

/// Submits an authoritative move command to a target tile.
fn submit_authoritative_move(
    authority: &mut RuntimeAuthority,
    state: &mut GameState,
    target: MapPoint,
) {
    let Some(unit) = state.selected_unit() else {
        return;
    };
    submit_authoritative_move_path(authority, state, unit.id, &[target]);
}

/// Submits an authoritative move command with the exact path drawn by the user.
fn submit_authoritative_move_path(
    authority: &mut RuntimeAuthority,
    state: &mut GameState,
    unit_id: UnitId,
    path: &[MapPoint],
) {
    if path.is_empty() {
        return;
    }
    let Some(unit) = state
        .units
        .iter()
        .find(|unit| unit.id == unit_id && unit.alive)
    else {
        return;
    };
    let Some(path_ids) = server_tile_ids_for_path(authority.session.projection.as_ref(), path)
    else {
        state.popups.push(GameplayPopup {
            title: "Move Path Unavailable".to_owned(),
            body: "The dragged path includes a tile missing from the authoritative projection."
                .to_owned(),
            kind: PopupKind::Info,
        });
        return;
    };
    let payload = json!({
        "unit_id": server_unit_id_for_unit(authority, unit.id),
        "path": path_ids
    });
    submit_authoritative_command(authority, state, "move", payload);
}

/// Submits a command and rebuilds local state from the returned projection.
fn submit_authoritative_command(
    authority: &mut RuntimeAuthority,
    state: &mut GameState,
    command: &str,
    payload: Value,
) {
    match authority
        .client
        .submit_command(&authority.session, command, payload)
    {
        Ok(response) => {
            if let Some(projection) = response.viewer_projection {
                authority.session.apply_projection(projection.clone());
                replace_state_from_projection(authority, state, &projection);
            }
        }
        Err(error) => state.popups.push(GameplayPopup {
            title: "Server Command Failed".to_owned(),
            body: error.to_string(),
            kind: PopupKind::GameStart,
        }),
    }
}

/// Replaces local render state with a fresh authoritative projection.
fn replace_state_from_projection(
    authority: &RuntimeAuthority,
    state: &mut GameState,
    projection: &ServerProjection,
) {
    let filters = state.filters;
    let selected = state.selected_unit;
    let mut next = state_from_authority_projection(authority, projection);
    if selected.is_some_and(|id| {
        next.units
            .iter()
            .any(|unit| unit.id == id && unit.alive && unit.owner == super::model::PlayerId::Human)
    }) {
        next.selected_unit = selected;
    }
    next.city_build_menu = next
        .cities
        .iter()
        .find(|city| {
            city.owner == super::model::PlayerId::Human
                && !state.cities.iter().any(|previous| previous.id == city.id)
        })
        .map(|city| city.id);
    next.filters = filters;
    next.popups.clear();
    *state = next;
}

/// Returns whether a point hits the end-turn button.
fn click_in_end_turn_button(x: i32, y: i32) -> bool {
    end_turn_button_rect().contains(x, y)
}

/// Returns whether a top-menu/dropdown click was consumed and any selected action.
fn top_menu_click_action(x: i32, y: i32, ui: &mut InGameUiModel) -> (bool, Option<InGameAction>) {
    if let Some(category) = ui.menu_button_at(x, y) {
        ui.open_menu = (ui.open_menu != Some(category)).then_some(category);
        return (true, None);
    }
    let Some(category) = ui.open_menu else {
        return (false, None);
    };
    let action = ui
        .dropdown_item_at(category, x, y)
        .and_then(|item| item.enabled.then_some(item.action));
    ui.open_menu = None;
    (true, action)
}

/// Loads identical original-size font/hinting for live gameplay and previews.
fn load_gameplay_font(context: &Sdl2TtfContext) -> Option<Font<'_, 'static>> {
    let path = find_existing_font()?;
    let mut font = context.load_font(path, 13_u16).ok()?;
    font.set_hinting(Hinting::Mono);
    Some(font)
}

/// Finds an installed font suitable for gameplay labels.
fn find_existing_font() -> Option<PathBuf> {
    FONT_CANDIDATES
        .iter()
        .map(Path::new)
        .find(|path| path.try_exists().unwrap_or(false))
        .map(Path::to_path_buf)
}

/// Scoped dummy SDL drivers for renderer tests.
#[cfg(test)]
mod test_support;
#[cfg(test)]
use test_support::HeadlessSdlEnvironment;

#[cfg(test)]
mod fidelity_tests;
#[cfg(test)]
mod movement_input_tests;
#[cfg(test)]
mod presentation_tests;
#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod stack_tests;
#[cfg(test)]
mod tests;
