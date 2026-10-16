//! SDL runtime for the in-game single-player gameplay screen.

extern crate alloc;

use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use sdl2::render::Canvas;
use sdl2::video::Window;
use serde_json::{Value, json};

use super::model::{GameState, GameplayPopup, MapPoint, PopupKind, UnitId};
use super::server::{ServerClient, ServerProjection};
use super::ui_model::{
    CONTROL_PANEL_HEIGHT, END_TURN_BUTTON_HEIGHT, END_TURN_BUTTON_WIDTH, END_TURN_BUTTON_X,
    END_TURN_BUTTON_Y, InGameAction, InGameUiModel, MENU_BAR_HEIGHT, MenuRect,
};
use super::{GameplayError, GameplayOutcome, SinglePlayerLaunchConfig, runtime_error};

/// Runtime frame delay.
const FRAME_DELAY: Duration = Duration::from_millis(16);
/// Default SDL window width.
pub(super) const DEFAULT_WINDOW_WIDTH: u32 = 1024;
/// Default SDL window height.
pub(super) const DEFAULT_WINDOW_HEIGHT: u32 = 768;
/// Right-side panel width.
pub(super) const SIDE_PANEL_WIDTH: u32 = 160;
/// Candidate fonts used to draw gameplay labels over original-style chrome.
const FONT_CANDIDATES: [&str; 8] = [
    "C:/Windows/Fonts/timesbd.ttf",
    "/usr/share/fonts/truetype/msttcorefonts/timesbd.ttf",
    "/usr/share/fonts/truetype/liberation2/LiberationSerif-Bold.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSerif-Bold.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
    "/usr/share/fonts/truetype/liberation2/LiberationSans-Bold.ttf",
    "/usr/share/fonts/truetype/freefont/FreeSansBold.ttf",
    "C:/Windows/Fonts/arialbd.ttf",
];
/// Authoritative server state alignment helpers.
mod authority;
/// CTP2-style movement input handling.
mod movement;
/// Gameplay frame rendering.
mod render;
/// Original CTP2 texture loading.
mod textures;
/// Camera and screen/tile coordinate conversion.
mod view;

use authority::{
    RuntimeAuthority, server_tile_ids_for_path, server_unit_id_for_position,
    state_from_authority_projection,
};
use movement::{MovementInputState, begin_or_activate_unit_drag};
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

/// Runs gameplay on the provided SDL canvas and event pump.
fn run_sdl_game_on_canvas(
    config: &SinglePlayerLaunchConfig,
    canvas: &mut Canvas<Window>,
    event_pump: &mut EventPump,
    frame_limit: Option<u32>,
) -> Result<GameplayOutcome, GameplayError> {
    configure_gameplay_canvas(canvas)?;
    let texture_creator = canvas.texture_creator();
    let mut textures = textures::GameplayTextures::load(
        &config.installation_root,
        &texture_creator,
        config.server.required,
    )?;
    let ttf_context = sdl2::ttf::init().ok();
    let font_path = find_existing_font();
    let font = match (ttf_context.as_ref(), font_path.as_ref()) {
        (Some(context), Some(path)) => context.load_font(path, 13).ok(),
        _ => None,
    };
    let mut authority = if config.server.required {
        let client = ServerClient::from_config(&config.server)?;
        let session =
            client.bootstrap_local_singleplayer(&config.leader_name, config.empire_count)?;
        Some(RuntimeAuthority {
            client,
            session,
            empire_count: config.empire_count,
            seed: config.seed,
        })
    } else {
        None
    };
    let mut state = authority
        .as_ref()
        .and_then(|authority| {
            authority
                .session
                .projection
                .as_ref()
                .map(|projection| state_from_authority_projection(authority, projection))
        })
        .unwrap_or_else(|| GameState::new(64, 48, config.empire_count, config.seed));
    if let Some(active_authority) = authority.as_ref() {
        state.queue_server_connected_popup(active_authority.session.game_id);
    }
    let mut ui = InGameUiModel::default();
    let mut movement = MovementInputState::default();
    let mut camera = Camera::centered_on_selected(&state);
    let mut outcome = None;
    let mut rendered_frames = 0_u32;

    while outcome.is_none() {
        for event in event_pump.poll_iter() {
            if let Some(next) = handle_event(
                &event,
                &mut state,
                &mut ui,
                &mut camera,
                &mut movement,
                authority.as_mut(),
            ) {
                outcome = Some(next);
            }
        }
        render::draw_frame(
            canvas,
            &mut textures,
            &state,
            &ui,
            &camera,
            &movement,
            font.as_ref(),
        )?;
        rendered_frames = rendered_frames.saturating_add(1);
        if frame_limit.is_some_and(|limit| rendered_frames >= limit) {
            outcome = Some(GameplayOutcome::Closed);
        }
        thread::sleep(FRAME_DELAY);
    }

    Ok(outcome.unwrap_or(GameplayOutcome::Closed))
}

/// Configures gameplay to render into the original 1024x768 coordinate space.
fn configure_gameplay_canvas(canvas: &mut Canvas<Window>) -> Result<(), GameplayError> {
    ensure_native_gameplay_output_size(canvas)?;
    canvas.set_viewport(None);
    canvas
        .set_logical_size(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
        .map_err(|error| runtime_error(error.to_string()))
}

/// Avoids SDL downscaling the native CTP2 gameplay frame below 1024x768.
fn ensure_native_gameplay_output_size(canvas: &mut Canvas<Window>) -> Result<(), GameplayError> {
    let (width, height) = canvas.output_size().map_err(runtime_error)?;
    if width >= DEFAULT_WINDOW_WIDTH && height >= DEFAULT_WINDOW_HEIGHT {
        return Ok(());
    }
    canvas
        .window_mut()
        .set_size(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
        .map_err(|error| runtime_error(error.to_string()))
}

/// Handles one SDL event.
fn handle_event(
    event: &Event,
    state: &mut GameState,
    ui: &mut InGameUiModel,
    camera: &mut Camera,
    movement: &mut MovementInputState,
    authority: Option<&mut RuntimeAuthority>,
) -> Option<GameplayOutcome> {
    match event.clone() {
        Event::Quit { .. } => Some(GameplayOutcome::Closed),
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
            if mousestate.left() {
                handle_mouse_motion(x, y, state, camera, movement);
            } else {
                handle_mouse_hover(x, y, state, camera, movement);
            }
            None
        }
        Event::MouseWheel { y, .. } => {
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
    if click_in_non_map_ui(x, y) {
        movement.cancel_drag();
        return None;
    }
    if let Some(point) = camera.screen_to_tile(x, y) {
        if let Some((unit_id, path)) = movement.finish_preview_order_to(state, point) {
            apply_finalized_move_path(state, camera, movement, authority, unit_id, path);
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
    camera: &mut Camera,
    movement: &mut MovementInputState,
    authority: Option<&mut RuntimeAuthority>,
) -> Option<GameplayOutcome> {
    if !state.popups.is_empty() || ui.open_menu.is_some() || click_in_non_map_ui(x, y) {
        movement.cancel_drag();
        return None;
    }
    if let Some(point) = camera.screen_to_tile(x, y) {
        movement.extend_drag_to(state, point);
    }
    let (unit_id, path) = movement.finish_drag()?;
    apply_finalized_move_path(state, camera, movement, authority, unit_id, path);
    None
}

/// Applies or submits one finalized CTP2 movement path.
fn apply_finalized_move_path(
    state: &mut GameState,
    camera: &mut Camera,
    movement: &mut MovementInputState,
    authority: Option<&mut RuntimeAuthority>,
    unit_id: UnitId,
    path: Vec<MapPoint>,
) {
    if !state.select_unit(unit_id) {
        movement.retain_selectable(state);
        return;
    }
    if let Some(active_authority) = authority {
        submit_authoritative_move_path(active_authority, state, unit_id, &path);
    } else {
        state.order_selected_along_path(path);
    }
    movement.retain_selectable(state);
    *camera = Camera::centered_on_selected(state);
}

/// Handles a right click.
fn handle_right_click(
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
    if ui.open_menu.is_some() {
        movement.cancel_drag();
        ui.open_menu = None;
        return None;
    }
    if click_in_non_map_ui(x, y) {
        return None;
    }
    if let Some(point) = camera.screen_to_tile(x, y) {
        let menu = InGameUiModel::context_menu_for(state, point);
        if let Some(item) = menu.iter().find(|item| item.enabled) {
            return apply_ui_action(item.action, state, ui, camera, movement, authority);
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
        InGameAction::Settle => {
            let Some(unit) = state.selected_unit() else {
                return true;
            };
            let city_id = format!("city-{}-{}", unit.position.x, unit.position.y);
            let payload = json!({
                "unit_id": server_unit_id_for_position(authority, unit.position),
                "city_id": city_id,
                "name": "New City"
            });
            submit_authoritative_command(authority, state, "found-city", payload);
            true
        }
        InGameAction::Sentinel => {
            let Some(unit) = state.selected_unit() else {
                return true;
            };
            let payload = json!({"unit_id": server_unit_id_for_position(authority, unit.position)});
            submit_authoritative_command(authority, state, "fortify", payload);
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
        "unit_id": server_unit_id_for_position(authority, unit.position),
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
    let mut next = state_from_authority_projection(authority, projection);
    next.filters = filters;
    next.popups.clear();
    *state = next;
}

/// Returns the top edge of the bottom control panel.
fn control_panel_top() -> i32 {
    i32::try_from(DEFAULT_WINDOW_HEIGHT.saturating_sub(CONTROL_PANEL_HEIGHT))
        .map_or(0_i32, |value| value)
}

/// Returns the original end-turn button rectangle in screen coordinates.
fn end_turn_button_rect() -> MenuRect {
    MenuRect {
        x: END_TURN_BUTTON_X,
        y: control_panel_top().saturating_add(END_TURN_BUTTON_Y),
        width: END_TURN_BUTTON_WIDTH,
        height: END_TURN_BUTTON_HEIGHT,
    }
}

/// Returns whether a point hits the end-turn button.
fn click_in_end_turn_button(x: i32, y: i32) -> bool {
    end_turn_button_rect().contains(x, y)
}

/// Returns whether a point is inside non-map UI chrome.
fn click_in_non_map_ui(x: i32, y: i32) -> bool {
    let menu_height = i32::try_from(MENU_BAR_HEIGHT).map_or(0_i32, |height| height);
    let side_panel_x = i32::try_from(DEFAULT_WINDOW_WIDTH.saturating_sub(SIDE_PANEL_WIDTH))
        .map_or(i32::MAX, |value| value);
    y < menu_height || y >= control_panel_top() || x >= side_panel_x
}

/// Returns whether a top-menu/dropdown click was consumed and any selected action.
fn top_menu_click_action(x: i32, y: i32, ui: &mut InGameUiModel) -> (bool, Option<InGameAction>) {
    if let Some(category) = ui.menu_button_at(x, y) {
        ui.open_menu = Some(category);
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

/// Finds an installed font suitable for gameplay labels.
fn find_existing_font() -> Option<PathBuf> {
    FONT_CANDIDATES
        .iter()
        .map(Path::new)
        .find(|path| path.try_exists().unwrap_or(false))
        .map(Path::to_path_buf)
}

/// Restorable headless SDL environment.
#[cfg(test)]
struct HeadlessSdlEnvironment {
    /// Previous video driver.
    previous_video_driver: Option<std::ffi::OsString>,
    /// Previous audio driver.
    previous_audio_driver: Option<std::ffi::OsString>,
}

#[cfg(test)]
impl HeadlessSdlEnvironment {
    /// Applies dummy drivers.
    fn apply() -> Self {
        let environment = Self {
            previous_video_driver: std::env::var_os("SDL_VIDEODRIVER"),
            previous_audio_driver: std::env::var_os("SDL_AUDIODRIVER"),
        };
        // SAFETY: Gameplay smoke tests hold `SDL_GAMEPLAY_TEST_LOCK` while mutating
        // process-wide SDL environment variables and restore them before unlock.
        unsafe {
            std::env::set_var("SDL_VIDEODRIVER", "dummy");
            std::env::set_var("SDL_AUDIODRIVER", "dummy");
        }
        environment
    }

    /// Restores dummy drivers.
    fn restore(self) {
        restore_env_var("SDL_VIDEODRIVER", self.previous_video_driver);
        restore_env_var("SDL_AUDIODRIVER", self.previous_audio_driver);
    }
}

/// Restores one environment variable.
#[cfg(test)]
fn restore_env_var(key: &str, value: Option<std::ffi::OsString>) {
    // SAFETY: The same test lock used by `HeadlessSdlEnvironment::apply` is
    // still held, so no concurrent test observes a partial environment update.
    unsafe {
        match value {
            Some(previous) => std::env::set_var(key, previous),
            None => std::env::remove_var(key),
        }
    }
}

#[cfg(test)]
mod movement_input_tests;
#[cfg(test)]
mod tests;
