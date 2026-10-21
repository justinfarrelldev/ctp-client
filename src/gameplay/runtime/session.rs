//! Gameplay bootstrap and live SDL presentation loop, separate from input policy.

use std::thread;
use std::time::Instant;

use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::TextInputUtil;

/// Native text-input scope regressions using only isolated SDL memory state.
#[cfg(test)]
mod tests;
use sdl2::render::Canvas;
use sdl2::video::Window;

use super::authority::{RuntimeAuthority, state_from_authority_projection};
use super::cursor::{EdgeScroll, SoftwareCursorGuard, pointer_blocked};
use super::fonts::{ManagementFonts, RenderFonts};
use super::management_input::editor_input::{NativeClipboard, flush_clipboard};
use super::textures::GameplayTextures;
use super::{
    Camera, DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH, FRAME_DELAY, MovementInputState,
    configure_gameplay_canvas, frame, handle_event, load_gameplay_font, snapshot,
};
use crate::gameplay::model::GameState;
use crate::gameplay::server::ServerClient;
use crate::gameplay::ui_model::InGameUiModel;
use crate::gameplay::{GameplayError, GameplayOutcome, SinglePlayerLaunchConfig, runtime_error};

/// Restores the shell's text-input state after gameplay, including error exits.
struct TextInputScope {
    /// SDL's safe text-input handle keeps its video subsystem alive.
    input: TextInputUtil,
    /// Whether the calling shell already had text input active.
    was_active: bool,
}

impl TextInputScope {
    /// Enables SDL Unicode/IME events; only a focused local text field consumes them.
    fn new(video: &sdl2::VideoSubsystem) -> Self {
        let input = video.text_input();
        let was_active = input.is_active();
        input.start();
        Self { input, was_active }
    }
}

impl Drop for TextInputScope {
    fn drop(&mut self) {
        if !self.was_active {
            self.input.stop();
        }
    }
}

/// Creates only server-authoritative state in interactive play; preview/test mode
/// is explicitly selected by a disabled server configuration, never by a failure.
fn bootstrap(
    config: &SinglePlayerLaunchConfig,
) -> Result<(Option<RuntimeAuthority>, GameState), GameplayError> {
    if !config.server.required {
        return Ok((
            None,
            GameState::new(64, 48, config.empire_count, config.seed),
        ));
    }
    let client = ServerClient::from_config(&config.server)?;
    let session = client.bootstrap_local_singleplayer(&config.leader_name, config.empire_count)?;
    let authority = RuntimeAuthority {
        io: Some(super::network::Network::start(client.clone(), session.clone())),
        client,
        session,
        empire_count: config.empire_count,
        seed: config.seed,
    };
    let mut state = authority.session.projection.as_ref().map_or_else(
        || {
            GameState::from_server_projection(
                &crate::gameplay::server::ServerProjection::default(),
                config.empire_count,
                config.seed,
            )
        },
        |projection| state_from_authority_projection(&authority, projection),
    );
    state.popups.clear();
    log::info!(
        "Gameplay uses authoritative server game #{}",
        authority.session.game_id
    );
    Ok((Some(authority), state))
}

/// Loads source controls once and measures with the actual native gameplay font.
fn interface(
    config: &SinglePlayerLaunchConfig,
    font: Option<&sdl2::ttf::Font<'_, '_>>,
) -> Result<InGameUiModel, GameplayError> {
    let mut ui = InGameUiModel::default();
    if config.server.required {
        ui.management = crate::gameplay::ui_model::management::ManagementUi::new(
            crate::gameplay::assets::layout::LayoutBook::load(&config.installation_root)?,
        )?;
    }
    if let Some(font) = font {
        ui.measure_menu_text(|text| font.size_of(text).map_or(0_u32, |size| size.0));
        ui.management.line_height = u32::try_from(font.height()).unwrap_or(16_u32);
    }
    Ok(ui)
}

/// Capture is explicit and never consumes a game shortcut or command.
const fn is_capture_event(event: &Event) -> bool {
    matches!(
        *event,
        Event::KeyDown {
            keycode: Some(sdl2::keyboard::Keycode::F12),
            repeat: false,
            ..
        }
    )
}

/// The explicit F12 sink is separate from fileless rendering and tests.
fn save_capture(canvas: &Canvas<Window>) {
    match snapshot::save_interactive_capture(canvas) {
        Ok(path) => log::info!("Saved gameplay frame to {}", path.display()),
        Err(error) => log::warn!("Gameplay screenshot failed: {error}"),
    }
}

/// Runs gameplay on the existing canvas, preserving the main menu's window.
pub(super) fn run_sdl_game_on_canvas(
    config: &SinglePlayerLaunchConfig,
    canvas: &mut Canvas<Window>,
    event_pump: &mut EventPump,
    frame_limit: Option<u32>,
) -> Result<GameplayOutcome, GameplayError> {
    configure_gameplay_canvas(canvas)?;
    let texture_creator = canvas.texture_creator();
    let mut textures = GameplayTextures::load(
        &config.installation_root,
        &texture_creator,
        config.server.required,
    )?;
    let mut frame_texture = texture_creator
        .create_texture_target(None, DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
        .map_err(|error| runtime_error(error.to_string()))?;
    let ttf_context = sdl2::ttf::init().ok();
    let font = ttf_context.as_ref().and_then(load_gameplay_font);
    let (mut authority, mut state) = bootstrap(config)?;
    let mut ui = interface(config, font.as_ref())?;
    let management_fonts = ttf_context
        .as_ref()
        .map(|context| ManagementFonts::load_measured(context, &mut ui.management))
        .transpose()?;
    let mut movement = MovementInputState::default();
    movement.actors.speed = config.graphics.unit_speed;
    movement.actors.enabled = config.graphics.unit_animation.enabled();
    let mut camera = Camera::centered_on_selected(&state);
    let cursor_guard = SoftwareCursorGuard::new()?;
    let _text_input = TextInputScope::new(canvas.window().subsystem());
    let mut clipboard = NativeClipboard::new(canvas.window().subsystem());
    let mut edge_scroll = EdgeScroll::default();
    let clock = Instant::now();
    let mut outcome = None;
    let mut rendered_frames = 0_u32;
    while outcome.is_none() {
        let mut capture_requested = false;
        for window_event in event_pump.poll_iter() {
            let Some(event) = super::viewport::map_event(canvas, &window_event)? else {
                ui.pointer = None;
                ui.hovered_tile = None;
                movement.cancel_drag();
                continue;
            };
            if is_capture_event(&event) {
                capture_requested = true;
                continue;
            }
            if let Some(fonts) = management_fonts.as_ref() {
                fonts.measure_query(&mut ui.management);
            }
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
            flush_clipboard(&mut ui, &mut clipboard);
        }
        if let Some(active) = authority.as_mut() {
            super::sync::poll(active, &mut state, &mut ui, &mut movement);
        }
        super::management_input::open_pending_build(&mut state, &mut ui);
        movement
            .actors
            .begin_frame(u64::try_from(clock.elapsed().as_millis()).unwrap_or(u64::MAX));
        let before_pan = camera;
        edge_scroll.update(
            &mut camera,
            ui.pointer,
            pointer_blocked(&state, &ui),
            movement.actors.now_ms,
        );
        if camera != before_pan {
            super::cursor::refresh_map_pointer(&state, &mut ui, &camera, &mut movement);
        }
        frame::draw_scaled_frame(
            canvas,
            &mut frame_texture,
            &mut textures,
            &state,
            &ui,
            &camera,
            &movement,
            RenderFonts {
                body: font.as_ref(),
                management: management_fonts.as_ref(),
            },
        )?;
        if capture_requested {
            save_capture(canvas);
        }
        cursor_guard.refresh();
        canvas.present();
        rendered_frames = rendered_frames.saturating_add(1);
        if frame_limit.is_some_and(|limit| rendered_frames >= limit) {
            outcome = Some(GameplayOutcome::Closed);
        }
        thread::sleep(FRAME_DELAY);
    }
    Ok(outcome.unwrap_or(GameplayOutcome::Closed))
}
