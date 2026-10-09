//! SDL runtime for the in-game single-player gameplay screen.

extern crate alloc;

use alloc::vec::Vec;
use core::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

use super::model::GameState;
use super::ui_model::{InGameAction, InGameUiModel, MENU_BAR_HEIGHT};
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
    let ttf_context = sdl2::ttf::init().ok();
    let font_path = find_existing_font();
    let font = match (ttf_context.as_ref(), font_path.as_ref()) {
        (Some(context), Some(path)) => context.load_font(path, 13).ok(),
        _ => None,
    };
    let mut event_pump = sdl.event_pump().map_err(runtime_error)?;
    let mut state = GameState::new(64, 48, config.empire_count, config.seed);
    let mut ui = InGameUiModel::default();
    let mut camera = Camera::centered_on_selected(&state);
    let mut outcome = None;
    let mut rendered_frames = 0_u32;

    while outcome.is_none() {
        for event in event_pump.poll_iter() {
            if let Some(next) = handle_event(&event, &mut state, &mut ui, &mut camera) {
                outcome = Some(next);
            }
        }
        render::draw_frame(&mut canvas, &state, &ui, &camera, font.as_ref())?;
        rendered_frames = rendered_frames.saturating_add(1);
        if options
            .frame_limit
            .is_some_and(|frame_limit| rendered_frames >= frame_limit)
        {
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
        ),
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } => handle_left_click(x, y, state, ui, camera),
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Right,
            x,
            y,
            ..
        } => {
            if let Some(point) = camera.screen_to_tile(x, y) {
                let menu = InGameUiModel::context_menu_for(state, point);
                if let Some(item) = menu.iter().find(|item| item.enabled) {
                    return apply_ui_action(item.action, state, ui, camera);
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
) -> Option<GameplayOutcome> {
    if !state.popups.is_empty() {
        state.dismiss_popup();
        return None;
    }
    if click_in_end_turn_button(x, y) {
        return apply_ui_action(InGameAction::EndTurn, state, ui, camera);
    }
    let (menu_consumed, menu_action) = top_menu_click_action(x, y, ui);
    if menu_consumed {
        if let Some(action) = menu_action {
            return apply_ui_action(action, state, ui, camera);
        }
        return None;
    }
    if let Some(point) = camera.screen_to_tile(x, y) {
        state.order_selected_to(point);
    }
    None
}

/// Applies a model/UI action and returns loop outcome if terminal.
fn apply_ui_action(
    action: InGameAction,
    state: &mut GameState,
    ui: &mut InGameUiModel,
    camera: &mut Camera,
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
            ui.apply_action(state, other);
            if matches!(other, InGameAction::MoveBy(_, _) | InGameAction::EndTurn) {
                *camera = Camera::centered_on_selected(state);
            }
            None
        }
    }
}

/// Returns whether a point hits the end-turn button.
const fn click_in_end_turn_button(x: i32, y: i32) -> bool {
    x >= 940_i32 && y >= 590_i32 && x < 1_010_i32 && y < 740_i32
}

/// Returns whether a top-menu/dropdown click was consumed and any selected action.
fn top_menu_click_action(x: i32, y: i32, ui: &mut InGameUiModel) -> (bool, Option<InGameAction>) {
    let menu_height = i32::try_from(MENU_BAR_HEIGHT).map_or(0_i32, |height| height);
    if y >= 0_i32 && y < menu_height {
        if let Some(category) = ui
            .menu_buttons
            .iter()
            .find(|button| {
                x >= button.x
                    && x < button
                        .x
                        .saturating_add(i32::try_from(button.width).map_or(0_i32, |width| width))
            })
            .map(|button| button.category)
        {
            ui.open_menu = Some(category);
            return (true, None);
        }
    }
    let Some(category) = ui.open_menu else {
        return (false, None);
    };
    if y < menu_height || !(0_i32..260_i32).contains(&x) {
        ui.open_menu = None;
        return (false, None);
    }
    let row = usize::try_from((y.saturating_sub(menu_height)).saturating_div(18)).ok();
    let action = row.and_then(|index| {
        ui.items_for(category)
            .into_iter()
            .filter(|item| item.enabled)
            .nth(index)
            .map(|item| item.action)
    });
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

    use super::run_headless_smoke_for_test;
    use crate::gameplay::{GameplayOutcome, SinglePlayerLaunchConfig};

    #[test]
    fn headless_gameplay_smoke_opens_and_closes() {
        let outcome = run_headless_smoke_for_test(&SinglePlayerLaunchConfig {
            installation_root: PathBuf::from("/tmp/ctp2"),
            empire_count: 2,
            leader_name: "Tester".to_owned(),
            seed: 42,
        })
        .expect("headless gameplay should run");

        assert_eq!(outcome, GameplayOutcome::Closed);
    }
}
