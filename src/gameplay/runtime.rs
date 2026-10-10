//! SDL runtime for the in-game single-player gameplay screen.

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::render::Canvas;
use sdl2::video::Window;
use serde_json::{Value, json};

use super::model::{GameState, GameplayPopup, MapPoint, PopupKind};
use super::server::{ServerClient, ServerGameplaySession, ServerProjection};
use super::ui_model::{InGameAction, InGameUiModel};
use super::{GameplayError, GameplayOutcome, SinglePlayerLaunchConfig, runtime_error};

/// Runtime frame delay.
const FRAME_DELAY: Duration = Duration::from_millis(16);
/// Default SDL window width.
pub(super) const DEFAULT_WINDOW_WIDTH: u32 = 1024;
/// Default SDL window height.
pub(super) const DEFAULT_WINDOW_HEIGHT: u32 = 768;
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
/// Gameplay frame rendering.
mod render;
/// Original CTP2 texture loading.
mod textures;
/// Camera and screen/tile coordinate conversion.
mod view;

use view::Camera;

/// Event-loop options used by production and tests.
struct RuntimeOptions {
    /// Initial SDL events to enqueue.
    initial_events: Vec<Event>,
    /// Maximum rendered frames before returning closed.
    frame_limit: Option<u32>,
}

/// Live authoritative server state for an interactive runtime session.
struct RuntimeAuthority {
    /// Blocking command client.
    client: ServerClient,
    /// Current authoritative session metadata.
    session: ServerGameplaySession,
    /// Configured empire count for rebuilding projection state.
    empire_count: u8,
    /// Configured deterministic fallback seed.
    seed: u64,
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
    let texture_creator = canvas.texture_creator();
    let textures = textures::GameplayTextures::load(
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
        .and_then(|authority| authority.session.projection.as_ref())
        .map_or_else(
            || GameState::new(64, 48, config.empire_count, config.seed),
            |projection| {
                GameState::from_server_projection(projection, config.empire_count, config.seed)
            },
        );
    if let Some(active_authority) = authority.as_ref() {
        state.queue_server_connected_popup(active_authority.session.game_id);
    }
    let mut ui = InGameUiModel::default();
    let mut camera = Camera::centered_on_selected(&state);
    let mut outcome = None;
    let mut rendered_frames = 0_u32;

    while outcome.is_none() {
        for event in event_pump.poll_iter() {
            if let Some(next) =
                handle_event(&event, &mut state, &mut ui, &mut camera, authority.as_mut())
            {
                outcome = Some(next);
            }
        }
        render::draw_frame(canvas, &textures, &state, &ui, &camera, font.as_ref())?;
        rendered_frames = rendered_frames.saturating_add(1);
        if frame_limit.is_some_and(|limit| rendered_frames >= limit) {
            outcome = Some(GameplayOutcome::Closed);
        }
        thread::sleep(FRAME_DELAY);
    }

    Ok(outcome.unwrap_or(GameplayOutcome::Closed))
}

/// Handles one SDL event.
fn handle_event(
    event: &Event,
    state: &mut GameState,
    ui: &mut InGameUiModel,
    camera: &mut Camera,
    authority: Option<&mut RuntimeAuthority>,
) -> Option<GameplayOutcome> {
    match event.clone() {
        Event::Quit { .. } => Some(GameplayOutcome::Closed),
        Event::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
        } if !state.popups.is_empty() => {
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
            authority,
        ),
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } => handle_left_click(x, y, state, ui, camera, authority),
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Right,
            x,
            y,
            ..
        } => {
            if let Some(point) = camera.screen_to_tile(x, y) {
                let menu = InGameUiModel::context_menu_for(state, point);
                if let Some(item) = menu.iter().find(|item| item.enabled) {
                    return apply_ui_action(item.action, state, ui, camera, authority);
                }
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
    authority: Option<&mut RuntimeAuthority>,
) -> Option<GameplayOutcome> {
    if !state.popups.is_empty() {
        state.dismiss_popup();
        return None;
    }
    if ui.open_menu.is_some() {
        let (menu_consumed, menu_action) = top_menu_click_action(x, y, ui);
        if menu_consumed {
            if let Some(action) = menu_action {
                return apply_ui_action(action, state, ui, camera, authority);
            }
            return None;
        }
    }
    if click_in_end_turn_button(x, y) {
        return apply_ui_action(InGameAction::EndTurn, state, ui, camera, authority);
    }
    let (menu_consumed, menu_action) = top_menu_click_action(x, y, ui);
    if menu_consumed {
        if let Some(action) = menu_action {
            return apply_ui_action(action, state, ui, camera, authority);
        }
        return None;
    }
    if let Some(point) = camera.screen_to_tile(x, y) {
        if let Some(active_authority) = authority {
            submit_authoritative_move(active_authority, state, point);
        } else {
            state.order_selected_to(point);
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
                    return None;
                }
            }
            ui.apply_action(state, other);
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
    let payload = json!({
        "unit_id": server_unit_id_for_position(authority, unit.position),
        "path": [server_tile_id(authority.session.projection.as_ref(), target)]
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
    let mut next =
        GameState::from_server_projection(projection, authority.empire_count, authority.seed);
    next.filters = filters;
    next.popups.clear();
    *state = next;
}

/// Returns the server unit id for a local selected unit position.
fn server_unit_id_for_position(authority: &RuntimeAuthority, position: MapPoint) -> String {
    let Some(projection) = authority.session.projection.as_ref() else {
        return String::new();
    };
    projection
        .units
        .iter()
        .find(|unit| {
            projection
                .tiles
                .iter()
                .find(|tile| tile.id == unit.tile_id)
                .is_some_and(|tile| tile.x == position.x && tile.y == position.y)
        })
        .map_or_else(String::new, |unit| unit.id.clone())
}

/// Returns the server tile id for a local map point.
fn server_tile_id(projection: Option<&ServerProjection>, point: MapPoint) -> String {
    projection
        .and_then(|projection| {
            projection
                .tiles
                .iter()
                .find(|tile| tile.x == point.x && tile.y == point.y)
        })
        .map_or_else(String::new, |tile| tile.id.clone())
}

/// Returns whether a point hits the end-turn button.
const fn click_in_end_turn_button(x: i32, y: i32) -> bool {
    x >= 940_i32 && y >= 590_i32 && x < 1_010_i32 && y < 740_i32
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
mod tests {
    use std::path::PathBuf;

    use sdl2::event::Event;

    use super::{
        HeadlessSdlEnvironment, run_headless_smoke_for_test, run_sdl_game_on_existing_window,
        top_menu_click_action,
    };
    use crate::gameplay::ui_model::{InGameAction, InGameUiModel, MenuCategory};
    use crate::gameplay::{GameplayOutcome, SinglePlayerLaunchConfig};

    #[test]
    fn dropdown_hit_testing_preserves_disabled_row_indexes() {
        let mut ui = InGameUiModel {
            open_menu: Some(MenuCategory::Options),
            ..InGameUiModel::default()
        };
        let save_row = ui
            .dropdown_row_rect(MenuCategory::Options, 7)
            .expect("Save Game row should be present");
        let x = save_row.x.saturating_add(4);
        let y = save_row.y.saturating_add(4);

        let (consumed, action) = top_menu_click_action(x, y, &mut ui);

        assert!(
            consumed,
            "disabled Save Game row should still consume the click"
        );
        assert_eq!(
            action, None,
            "disabled Save Game must not fall through to Restart/New Game because enabled rows were filtered"
        );
        assert_eq!(
            ui.open_menu, None,
            "selecting any row should close the menu"
        );
    }

    #[test]
    fn open_dropdown_consumes_outside_end_turn_click_without_action() {
        let mut ui = InGameUiModel {
            open_menu: Some(MenuCategory::Options),
            ..InGameUiModel::default()
        };

        let (consumed, action) = top_menu_click_action(960, 648, &mut ui);

        assert!(
            consumed,
            "a weakly-modal CTP2 dropdown should consume outside clicks instead of activating end-turn/terrain"
        );
        assert_eq!(
            action, None,
            "outside click should only cancel the dropdown"
        );
        assert_eq!(
            ui.open_menu, None,
            "outside click should close the dropdown"
        );
    }

    #[test]
    fn top_menu_buttons_open_original_categories() {
        let mut ui = InGameUiModel::default();

        let (consumed, action) = top_menu_click_action(405, 12, &mut ui);

        assert!(
            consumed,
            "clicking the Options menu button should be consumed"
        );
        assert_eq!(action, None, "opening a menu should not fire a row action");
        assert_eq!(
            ui.open_menu,
            Some(MenuCategory::Options),
            "Options button should use original 80px LDL positioning"
        );
    }

    #[test]
    fn enabled_dropdown_row_returns_action_from_exact_row() {
        let mut ui = InGameUiModel {
            open_menu: Some(MenuCategory::Options),
            ..InGameUiModel::default()
        };
        let quit_index = ui
            .items_for(MenuCategory::Options)
            .iter()
            .position(|item| item.label == "Quit")
            .expect("Quit row should be present");
        let quit_row = ui
            .dropdown_row_rect(MenuCategory::Options, quit_index)
            .expect("Quit row should have a rectangle");

        let (consumed, action) = top_menu_click_action(
            quit_row.x.saturating_add(4),
            quit_row.y.saturating_add(4),
            &mut ui,
        );

        assert!(consumed, "enabled dropdown row should consume the click");
        assert_eq!(
            action,
            Some(InGameAction::Quit),
            "Quit row should return Quit"
        );
        assert_eq!(
            ui.open_menu, None,
            "row selection should close the dropdown"
        );
    }

    #[test]
    fn headless_gameplay_smoke_opens_and_closes() {
        let outcome = run_headless_smoke_for_test(&SinglePlayerLaunchConfig {
            installation_root: PathBuf::from("/tmp/ctp2"),
            empire_count: 2,
            leader_name: "Tester".to_owned(),
            seed: 42,
            server: crate::gameplay::server::ServerConfig::disabled_for_test(),
        })
        .expect("headless gameplay should run");

        assert_eq!(outcome, GameplayOutcome::Closed);
    }

    #[test]
    fn headless_gameplay_reuses_existing_sdl_window() {
        let guard =
            crate::sdl_test_lock::lock_sdl_for_test().expect("SDL test lock should be available");
        let environment = HeadlessSdlEnvironment::apply();
        {
            let sdl = sdl2::init().expect("SDL should initialize with dummy driver");
            let video = sdl.video().expect("video subsystem should initialize");
            let window = video
                .window("menu-owned", 800, 600)
                .position_centered()
                .build()
                .expect("dummy window should be created");
            let mut canvas = window
                .into_canvas()
                .software()
                .build()
                .expect("dummy canvas should be created");
            let existing_window_id = canvas.window().id();
            let events = sdl.event().expect("event subsystem should initialize");
            events
                .push_event(Event::Quit { timestamp: 0 })
                .expect("quit event should be queued");
            let mut event_pump = sdl.event_pump().expect("event pump should initialize");

            let outcome = run_sdl_game_on_existing_window(
                &SinglePlayerLaunchConfig {
                    installation_root: PathBuf::from("/tmp/ctp2"),
                    empire_count: 2,
                    leader_name: "Tester".to_owned(),
                    seed: 42,
                    server: crate::gameplay::server::ServerConfig::disabled_for_test(),
                },
                &mut canvas,
                &mut event_pump,
            )
            .expect("gameplay should run on a menu-owned window");

            assert_eq!(outcome, GameplayOutcome::Closed);
            assert_eq!(
                canvas.window().id(),
                existing_window_id,
                "gameplay must reuse the menu-owned SDL window"
            );
        }
        environment.restore();
        drop(guard);
    }
}
