//! SDL runtime for the Call to Power II landing main menu.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect as SdlRect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};

use crate::config::GameInstallation;

use super::{
    ButtonStyleId, ButtonVisualStyle, MainMenuAction, MainMenuError, MainMenuModel, MenuRect,
};

/// Delay between menu frames.
const FRAME_DELAY: Duration = Duration::from_millis(16);
/// Texture format matching bytes emitted by the RIM RGBA conversion path.
const MENU_TEXTURE_FORMAT: PixelFormatEnum = PixelFormatEnum::RGBA32;
/// Magenta color used by original CTP2 shell button art as a chroma key.
const BUTTON_CHROMA_KEY: (u8, u8, u8) = (248, 0, 248);
/// Archive holding the original CTP2 shell screen art.
const SHELL_ART_ARCHIVE: &str = "ctp2_data/default/graphics/pictures/pic565.zfs";
/// Original CTP2 button label color from the shell button templates.
const BUTTON_LABEL_TEXT_COLOR: (u8, u8, u8) = (0, 0, 0);
/// Original CTP2 version label color from `spnewgame.ldl`.
const VERSION_TEXT_COLOR: (u8, u8, u8) = (100, 100, 255);
/// Font point size used by original CTP2 shell button templates.
const BUTTON_FONT_POINT_SIZE: u16 = 10;
/// Test-only guard serializing SDL initialization across parallel tests.
#[cfg(test)]
static SDL_TEST_LOCK: Mutex<()> = Mutex::new(());

/// Candidate fonts used only for drawing fallback text over original assets.
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

/// Original CTP2 RIM/ZFS asset loading.
mod assets;
/// SDL_mixer-backed menu audio helpers.
mod audio;
/// Original CTP2 button art loading and rendering.
mod button_art;
/// Immediate UI sound routing for SDL events.
mod event_sound;
/// Single-player shell runtime rendering and navigation.
mod shell_runtime;
/// Resolution-independent viewport mapping helpers.
mod viewport;

use assets::{create_texture_from_rgba, load_rim_rgba, load_rim_rgba_entry};
use audio::SdlMenuMusicBackend;
#[cfg(test)]
use audio::{music_track_relative_path, option_volume_to_mixer_volume};
use event_sound::{play_menu_sound, sound_before_menu_event};
use viewport::{output_point_to_menu_point, prepare_menu_canvas};

/// Event-loop settings used by production and headless smoke tests.
struct RuntimeLoopOptions {
    /// Events pushed into SDL's queue immediately after window creation.
    initial_events: Vec<Event>,
    /// Initial SDL window size matching persisted application options.
    initial_window_size: (u32, u32),
    /// Optional maximum number of frames rendered before returning an error.
    frame_limit: Option<u32>,
}

impl RuntimeLoopOptions {
    /// Creates unbounded interactive event-loop options.
    const fn interactive(initial_window_size: (u32, u32)) -> Self {
        Self {
            initial_events: Vec::new(),
            initial_window_size,
            frame_limit: None,
        }
    }

    /// Creates bounded options for smoke tests that must not hang.
    #[cfg(test)]
    fn headless_smoke() -> Self {
        Self {
            initial_events: vec![Event::Quit { timestamp: 0 }],
            initial_window_size: (super::MAIN_MENU_WIDTH, super::MAIN_MENU_HEIGHT),
            frame_limit: Some(3),
        }
    }
}

/// Texture cache for all original-art menu resources used in one frame.
struct MenuTextures<'texture> {
    /// Optional original landing-page background.
    background: Option<Texture<'texture>>,
    /// Optional blue/gold button textures.
    blue_large: Option<button_art::ButtonStateTextures<'texture>>,
    /// Optional stone button textures.
    stone_large: Option<button_art::ButtonStateTextures<'texture>>,
    /// Optional default small popup button textures.
    ctp2_small: Option<button_art::ButtonStateTextures<'texture>>,
}

/// Runs the SDL main-menu runtime for the supplied installation.
///
/// # Errors
///
/// Returns a [`MainMenuError`] when SDL rendering setup fails. Music errors are
/// logged and do not prevent the menu window from opening.
pub(super) fn run_sdl_main_menu(
    installation: &GameInstallation,
) -> Result<MainMenuAction, MainMenuError> {
    let model = MainMenuModel::ctp2_landing(installation.path.clone());

    let ctp2_options = load_ctp2_options_for_runtime();
    let mut music_backend = match SdlMenuMusicBackend::new(&installation.path) {
        Ok(mut audio) => {
            audio.apply_initial_options(&model, &ctp2_options);
            Some(audio)
        }
        Err(error) => {
            log::warn!("Main menu music backend is unavailable: {error}");
            None
        }
    };

    let initial_window_size = (
        ctp2_options.graphics.resolution.width,
        ctp2_options.graphics.resolution.height,
    );
    run_sdl_event_loop_with_audio(&model, music_backend.as_mut(), initial_window_size)
}

/// Loads persisted CTP2 options for runtime startup.
fn load_ctp2_options_for_runtime() -> crate::config::ctp2_options::Ctp2Options {
    crate::config::AppConfig::load().map_or_else(
        |error| {
            log::warn!("Could not load CTP2 options; using defaults: {error}");
            crate::config::ctp2_options::Ctp2Options::default()
        },
        |config| config.ctp2_options,
    )
}

/// Runs a real SDL menu frame in headless mode for tests.
#[cfg(test)]
pub(super) fn run_headless_smoke_for_test(
    model: &MainMenuModel,
) -> Result<MainMenuAction, MainMenuError> {
    with_sdl_test_lock(|| {
        run_sdl_event_loop_with_options(model, RuntimeLoopOptions::headless_smoke())
    })
}

/// Runs a headless SDL menu with a deterministic event sequence for tests.
#[cfg(test)]
pub(super) fn run_headless_events_for_test(
    model: &MainMenuModel,
    initial_events: Vec<Event>,
) -> Result<MainMenuAction, MainMenuError> {
    with_sdl_test_lock(|| {
        run_sdl_event_loop_with_options(
            model,
            RuntimeLoopOptions {
                initial_events,
                initial_window_size: (super::MAIN_MENU_WIDTH, super::MAIN_MENU_HEIGHT),
                frame_limit: Some(30_u32),
            },
        )
    })
}

/// Runs one SDL test action while holding the process-wide SDL test lock.
#[cfg(test)]
fn with_sdl_test_lock<T>(
    action: impl FnOnce() -> Result<T, MainMenuError>,
) -> Result<T, MainMenuError> {
    let guard = SDL_TEST_LOCK
        .lock()
        .map_err(|error| runtime_error(format!("SDL test lock was poisoned: {error}")))?;
    let result = action();
    drop(guard);
    result
}

/// Runs the SDL event loop with optional audio effects.
fn run_sdl_event_loop_with_audio(
    model: &MainMenuModel,
    audio_backend: Option<&mut SdlMenuMusicBackend>,
    initial_window_size: (u32, u32),
) -> Result<MainMenuAction, MainMenuError> {
    run_sdl_event_loop_with_options_and_audio(
        model,
        RuntimeLoopOptions::interactive(initial_window_size),
        audio_backend,
    )
}

/// Runs the SDL event loop with configurable test hooks.
#[cfg(test)]
fn run_sdl_event_loop_with_options(
    model: &MainMenuModel,
    options: RuntimeLoopOptions,
) -> Result<MainMenuAction, MainMenuError> {
    run_sdl_event_loop_with_options_and_audio(model, options, None)
}

/// Runs the SDL event loop with configurable test hooks and optional audio backend.
fn run_sdl_event_loop_with_options_and_audio(
    model: &MainMenuModel,
    options: RuntimeLoopOptions,
    mut audio_backend: Option<&mut SdlMenuMusicBackend>,
) -> Result<MainMenuAction, MainMenuError> {
    let sdl = sdl2::init().map_err(runtime_error)?;
    let video = sdl.video().map_err(runtime_error)?;
    video.text_input().start();
    let window = video
        .window(
            "Call to Power II",
            options.initial_window_size.0,
            options.initial_window_size.1,
        )
        .position_centered()
        .build()
        .map_err(|error| runtime_error(error.to_string()))?;
    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|error| runtime_error(error.to_string()))?;
    let texture_creator = canvas.texture_creator();
    let textures = load_menu_textures(&texture_creator, model);
    let ttf_context = sdl2::ttf::init().ok();
    let font_path = find_existing_font();
    let font = ttf_context.as_ref().and_then(|context| {
        font_path
            .as_ref()
            .and_then(|path| context.load_font(path, BUTTON_FONT_POINT_SIZE).ok())
    });

    let event_subsystem = sdl.event().map_err(runtime_error)?;
    for event in options.initial_events {
        event_subsystem.push_event(event).map_err(runtime_error)?;
    }

    let mut event_pump = sdl.event_pump().map_err(runtime_error)?;
    let mut shell_state = shell_runtime::ShellRuntimeState::new();
    let mut selected_action = None;
    let mut rendered_frames = 0_u32;

    while selected_action.is_none() {
        let event_output_size = canvas
            .output_size()
            .map_or((super::MAIN_MENU_WIDTH, super::MAIN_MENU_HEIGHT), |size| {
                size
            });
        for event in event_pump.poll_iter() {
            if let Some(sound) =
                sound_before_menu_event(&event, model, &shell_state, event_output_size)
            {
                if let Some(backend) = audio_backend.as_deref_mut() {
                    play_menu_sound(backend, sound);
                }
            }
            if let Some(sound) = handle_menu_event(
                &event,
                model,
                &mut shell_state,
                &mut selected_action,
                event_output_size,
            ) {
                if let Some(backend) = audio_backend.as_deref_mut() {
                    play_menu_sound(backend, sound);
                }
            }
        }

        apply_shell_runtime_effects(
            &mut canvas,
            model,
            audio_backend.as_deref_mut(),
            &shell_state.take_effects(),
        )?;

        let mouse_state = event_pump.mouse_state();
        let draw_output_size = canvas
            .output_size()
            .map_or((super::MAIN_MENU_WIDTH, super::MAIN_MENU_HEIGHT), |size| {
                size
            });
        let (mouse_x, mouse_y) = output_point_to_menu_point(
            draw_output_size.0,
            draw_output_size.1,
            mouse_state.x(),
            mouse_state.y(),
        )
        .map_or((-1_i32, -1_i32), |point| point);
        draw_current_menu(
            &mut canvas,
            &texture_creator,
            &textures,
            model,
            &shell_state,
            mouse_x,
            mouse_y,
            font.as_ref(),
        )?;
        rendered_frames = rendered_frames.saturating_add(1);
        if options
            .frame_limit
            .is_some_and(|frame_limit| rendered_frames >= frame_limit)
        {
            break;
        }

        thread::sleep(FRAME_DELAY);
    }

    selected_action.ok_or_else(|| runtime_error("menu loop ended without an action".to_owned()))
}

/// Applies any queued shell runtime side effects.
fn apply_shell_runtime_effects(
    canvas: &mut Canvas<Window>,
    model: &MainMenuModel,
    audio_backend: Option<&mut SdlMenuMusicBackend>,
    effects: &shell_runtime::ShellRuntimeEffects,
) -> Result<(), MainMenuError> {
    if let Some((width, height)) = effects.resolution {
        apply_resolution(canvas, width, height)?;
    }

    if let Some(backend) = audio_backend {
        apply_audio_effects(backend, model, effects);
    }

    if let Some(options) = effects.persisted_options.as_ref() {
        persist_ctp2_options(options);
    }

    Ok(())
}

/// Applies shell effects to the `SDL_mixer` backend.
fn apply_audio_effects(
    backend: &mut SdlMenuMusicBackend,
    model: &MainMenuModel,
    effects: &shell_runtime::ShellRuntimeEffects,
) {
    if let Some(volume) = effects.sfx_volume.or(effects.voice_volume) {
        SdlMenuMusicBackend::set_channel_volume(volume);
    }
    if let Some(volume) = effects.music_volume {
        SdlMenuMusicBackend::set_music_volume(volume);
    }
    if let Some(track) = effects.music_track {
        if let Err(error) = backend.play_track_number(&model.installation_root, track) {
            log::warn!("Music track {track} could not be played: {error}");
        }
    }
    if let Some(enabled) = effects.music_enabled {
        SdlMenuMusicBackend::set_music_enabled(enabled);
    }
    if let Some(enabled) = effects.auto_repeat {
        backend.set_auto_repeat(enabled);
    }
    if let Some(enabled) = effects.random_order {
        backend.set_random_order(enabled);
    }
}

/// Persists CTP2 options after a shell interaction.
#[cfg(not(test))]
fn persist_ctp2_options(options: &crate::config::ctp2_options::Ctp2Options) {
    match crate::config::AppConfig::load() {
        Ok(mut config) => {
            config.ctp2_options = options.clone();
            if let Err(error) = config.save() {
                log::warn!("Could not save CTP2 options: {error}");
            }
        }
        Err(error) => log::warn!("Could not load config for CTP2 option save: {error}"),
    }
}

/// Test-only persistence shim that prevents unit tests from touching real config files.
#[cfg(test)]
const fn persist_ctp2_options(_options: &crate::config::ctp2_options::Ctp2Options) {}

/// Applies a requested window resolution without scaling original 800x600 shell art.
fn apply_resolution(
    canvas: &mut Canvas<Window>,
    width: u32,
    height: u32,
) -> Result<(), MainMenuError> {
    canvas
        .window_mut()
        .set_size(width, height)
        .map_err(|error| runtime_error(error.to_string()))?;
    canvas
        .set_logical_size(0, 0)
        .map_err(|error| runtime_error(error.to_string()))?;
    canvas.set_scale(1.0, 1.0).map_err(runtime_error)?;
    canvas.set_viewport(None);
    Ok(())
}

/// Applies one SDL event to the menu action state.
fn handle_menu_event(
    event: &Event,
    model: &MainMenuModel,
    shell_state: &mut shell_runtime::ShellRuntimeState,
    selected_action: &mut Option<MainMenuAction>,
    output_size: (u32, u32),
) -> Option<shell_runtime::sound::ShellRuntimeSound> {
    match event.clone() {
        Event::Quit { .. }
        | Event::KeyDown {
            keycode: Some(Keycode::Escape),
            repeat: false,
            ..
        } => {
            *selected_action = Some(MainMenuAction::Quit);
            None
        }
        Event::MouseButtonDown { x, y, .. } => {
            let (menu_x, menu_y) = output_point_to_menu_point(output_size.0, output_size.1, x, y)?;
            if shell_state.is_landing() {
                if let Some(action) = model.activate_at_point(menu_x, menu_y) {
                    if action == MainMenuAction::SinglePlayer {
                        shell_state.open_single_player();
                    } else {
                        *selected_action = Some(action);
                    }
                }
                None
            } else {
                let interaction = shell_state.click_shell(menu_x, menu_y);
                if let Some(action) = interaction.action {
                    *selected_action = Some(action);
                }
                interaction.sound
            }
        }
        Event::MouseMotion { x, y, .. } => {
            if let Some((menu_x, menu_y)) =
                output_point_to_menu_point(output_size.0, output_size.1, x, y)
            {
                shell_state.drag_shell(menu_x, menu_y)
            } else {
                None
            }
        }
        Event::MouseButtonUp { .. } => shell_state.release_pointer(),
        Event::MouseWheel { y, .. } => {
            shell_state.scroll_shell(y);
            None
        }
        Event::TextInput { text, .. } => {
            shell_state.handle_text_input(&text);
            None
        }
        Event::KeyDown {
            keycode: Some(Keycode::Backspace),
            repeat: false,
            ..
        } => {
            shell_state.handle_backspace();
            None
        }
        Event::KeyDown { .. }
        | Event::KeyUp { .. }
        | Event::ControllerButtonDown { .. }
        | Event::ControllerButtonUp { .. }
        | Event::ControllerAxisMotion { .. }
        | Event::Window { .. }
        | Event::AppTerminating { .. }
        | Event::AppLowMemory { .. }
        | Event::AppWillEnterBackground { .. }
        | Event::AppDidEnterBackground { .. }
        | Event::AppWillEnterForeground { .. }
        | Event::AppDidEnterForeground { .. }
        | Event::Display { .. }
        | Event::TextEditing { .. }
        | Event::JoyAxisMotion { .. }
        | Event::JoyBallMotion { .. }
        | Event::JoyHatMotion { .. }
        | Event::JoyButtonDown { .. }
        | Event::JoyButtonUp { .. }
        | Event::JoyDeviceAdded { .. }
        | Event::JoyDeviceRemoved { .. }
        | Event::ControllerDeviceAdded { .. }
        | Event::ControllerDeviceRemoved { .. }
        | Event::ControllerDeviceRemapped { .. }
        | Event::ControllerTouchpadDown { .. }
        | Event::ControllerTouchpadMotion { .. }
        | Event::ControllerTouchpadUp { .. }
        | Event::FingerDown { .. }
        | Event::FingerUp { .. }
        | Event::FingerMotion { .. }
        | Event::DollarGesture { .. }
        | Event::DollarRecord { .. }
        | Event::MultiGesture { .. }
        | Event::ClipboardUpdate { .. }
        | Event::DropFile { .. }
        | Event::DropText { .. }
        | Event::DropBegin { .. }
        | Event::DropComplete { .. }
        | Event::AudioDeviceAdded { .. }
        | Event::AudioDeviceRemoved { .. }
        | Event::RenderTargetsReset { .. }
        | Event::RenderDeviceReset { .. }
        | Event::User { .. }
        | Event::Unknown { .. } => None,
    }
}

/// Draws the current landing or shell screen.
#[expect(
    clippy::too_many_arguments,
    reason = "The menu frame renderer borrows SDL resources explicitly."
)]
fn draw_current_menu(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    textures: &MenuTextures<'_>,
    model: &MainMenuModel,
    shell_state: &shell_runtime::ShellRuntimeState,
    mouse_x: i32,
    mouse_y: i32,
    font: Option<&Font<'_, '_>>,
) -> Result<(), MainMenuError> {
    prepare_menu_canvas(canvas)?;
    if shell_state.is_landing() {
        draw_menu(canvas, textures, model, mouse_x, mouse_y, font)
    } else {
        shell_state.draw(
            canvas,
            texture_creator,
            textures,
            model,
            mouse_x,
            mouse_y,
            font,
        )
    }
}

/// Draws one frame of the landing menu.
fn draw_menu(
    canvas: &mut Canvas<Window>,
    textures: &MenuTextures<'_>,
    model: &MainMenuModel,
    mouse_x: i32,
    mouse_y: i32,
    font: Option<&Font<'_, '_>>,
) -> Result<(), MainMenuError> {
    match textures.background.as_ref() {
        Some(texture) => canvas.copy(texture, None, None).map_err(runtime_error)?,
        None => draw_fallback_background(canvas)?,
    }

    for button in &model.buttons {
        let interaction_rect = button.style.rendered_rect(button.rect);
        let hovered = interaction_rect.contains_point(mouse_x, mouse_y);
        let text_rect = if let Some(button_textures) = textures_for_style(textures, button.style.id)
        {
            button_art::draw_styled_button(canvas, &button.rect, button_textures, hovered)?;
            button_art::styled_button_text_rect(&button.rect, button_textures, hovered)
        } else {
            draw_fallback_button(canvas, &interaction_rect, button.style.id, hovered)?;
            interaction_rect
        };
        draw_centered_text(canvas, font, &text_rect, button.label)?;
    }

    draw_text_at(canvas, font, &model.version_rect, model.version_text)?;
    canvas.present();
    Ok(())
}

/// Draws a simple fallback background when original ZFS assets are unavailable.
fn draw_fallback_background(canvas: &mut Canvas<Window>) -> Result<(), MainMenuError> {
    canvas.set_draw_color(Color::RGB(4, 9, 20));
    canvas.clear();
    canvas.set_draw_color(Color::RGB(18, 28, 44));
    canvas
        .fill_rect(SdlRect::new(0, 430, super::MAIN_MENU_WIDTH, 170))
        .map_err(runtime_error)
}

/// Draws a button using colors chosen to approximate the original asset family.
fn draw_fallback_button(
    canvas: &mut Canvas<Window>,
    rect: &MenuRect,
    style: ButtonStyleId,
    hovered: bool,
) -> Result<(), MainMenuError> {
    let color = match (style, hovered) {
        (ButtonStyleId::BlueLarge, false) => Color::RGB(33, 70, 112),
        (ButtonStyleId::BlueLarge, true) => Color::RGB(63, 105, 158),
        (ButtonStyleId::StoneLarge, false) => Color::RGB(89, 82, 70),
        (ButtonStyleId::StoneLarge, true) => Color::RGB(123, 112, 93),
        (ButtonStyleId::Ctp2Small, false) => Color::RGB(42, 78, 116),
        (ButtonStyleId::Ctp2Small, true) => Color::RGB(70, 112, 160),
    };
    let border = match style {
        ButtonStyleId::BlueLarge | ButtonStyleId::Ctp2Small => Color::RGB(211, 181, 85),
        ButtonStyleId::StoneLarge => Color::RGB(188, 174, 138),
    };
    let sdl_rect = to_sdl_rect(rect);

    canvas.set_draw_color(color);
    canvas.fill_rect(sdl_rect).map_err(runtime_error)?;
    canvas.set_draw_color(border);
    canvas.draw_rect(sdl_rect).map_err(runtime_error)
}

/// Draws centered text inside a rectangle when a system font is available.
fn draw_centered_text(
    canvas: &mut Canvas<Window>,
    font: Option<&Font<'_, '_>>,
    rect: &MenuRect,
    text: &str,
) -> Result<(), MainMenuError> {
    let Some(font) = font else {
        return Ok(());
    };

    let surface = font
        .render(text)
        .blended(rgb_color(BUTTON_LABEL_TEXT_COLOR))
        .map_err(|error| runtime_error(error.to_string()))?;
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|error| runtime_error(error.to_string()))?;
    let rect_width = i32::try_from(rect.width).map_or(0_i32, |value| value);
    let rect_height = i32::try_from(rect.height).map_or(0_i32, |value| value);
    let text_width = i32::try_from(surface.width()).map_or(0_i32, |value| value);
    let text_height = i32::try_from(surface.height()).map_or(0_i32, |value| value);
    let offset_x = rect_width
        .saturating_sub(text_width)
        .checked_div(2)
        .map_or(0_i32, |value| value);
    let offset_y = rect_height
        .saturating_sub(text_height)
        .checked_div(2)
        .map_or(0_i32, |value| value);
    let destination = SdlRect::new(
        rect.x.saturating_add(offset_x),
        rect.y.saturating_add(offset_y),
        surface.width(),
        surface.height(),
    );

    canvas
        .copy(&texture, None, destination)
        .map_err(runtime_error)
}

/// Draws text starting at the rectangle origin when a system font is available.
fn draw_text_at(
    canvas: &mut Canvas<Window>,
    font: Option<&Font<'_, '_>>,
    rect: &MenuRect,
    text: &str,
) -> Result<(), MainMenuError> {
    let Some(font) = font else {
        return Ok(());
    };

    let surface = font
        .render(text)
        .blended(rgb_color(VERSION_TEXT_COLOR))
        .map_err(|error| runtime_error(error.to_string()))?;
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|error| runtime_error(error.to_string()))?;
    let destination = SdlRect::new(rect.x, rect.y, surface.width(), surface.height());

    canvas
        .copy(&texture, None, destination)
        .map_err(runtime_error)
}

/// Converts a menu rectangle into an SDL rectangle.
fn to_sdl_rect(rect: &MenuRect) -> SdlRect {
    SdlRect::new(rect.x, rect.y, rect.width, rect.height)
}

/// Converts an RGB tuple into an SDL color.
const fn rgb_color((red, green, blue): (u8, u8, u8)) -> Color {
    Color::RGB(red, green, blue)
}

/// Parses an original resolution label such as `1024 x 768`.
fn parse_resolution_label(label: &str) -> Option<(u32, u32)> {
    let (width, height) = label.split_once('x')?;
    let width = width.trim().parse::<u32>().ok()?;
    let height = height.trim().parse::<u32>().ok()?;
    Some((width, height))
}

/// Loads all original-art textures available for a menu render pass.
fn load_menu_textures<'texture>(
    texture_creator: &'texture TextureCreator<WindowContext>,
    model: &MainMenuModel,
) -> MenuTextures<'texture> {
    MenuTextures {
        background: load_background_texture(texture_creator, model).ok(),
        blue_large: button_art::load_button_state_textures(
            texture_creator,
            model,
            ButtonVisualStyle::blue_large(),
        )
        .ok(),
        stone_large: button_art::load_button_state_textures(
            texture_creator,
            model,
            ButtonVisualStyle::stone_large(),
        )
        .ok(),
        ctp2_small: button_art::load_button_state_textures(
            texture_creator,
            model,
            ButtonVisualStyle::ctp2_small(),
        )
        .ok(),
    }
}

/// Selects cached textures for a button style.
const fn textures_for_style<'texture>(
    textures: &'texture MenuTextures<'texture>,
    style: ButtonStyleId,
) -> Option<&'texture button_art::ButtonStateTextures<'texture>> {
    match style {
        ButtonStyleId::BlueLarge => textures.blue_large.as_ref(),
        ButtonStyleId::StoneLarge => textures.stone_large.as_ref(),
        ButtonStyleId::Ctp2Small => textures.ctp2_small.as_ref(),
    }
}

/// Loads the original background texture from the CTP2 ZFS archive.
fn load_background_texture<'texture>(
    texture_creator: &'texture TextureCreator<WindowContext>,
    model: &MainMenuModel,
) -> Result<Texture<'texture>, MainMenuError> {
    let image = load_rim_rgba(&model.installation_root, &model.background)?;
    create_texture_from_rgba(texture_creator, &image)
}

/// Headless SDL runtime smoke-test helpers.
#[cfg(test)]
pub(super) mod testing;
#[cfg(test)]
pub(super) use testing::{
    button_asset_entry_name_for_test, button_click_sound_summary_for_test,
    button_label_text_color_for_test, button_layer_render_plan_for_test,
    button_style_asset_summaries_for_test, button_triplet_render_plan_for_test,
    disabled_button_layer_render_plan_for_test, initial_music_track_for_test,
    listbox_scrollbar_arrow_glyph_plan_for_test, menu_texture_format_for_test,
    mixer_allocated_channels_for_test, mixer_chunk_size_for_test,
    music_track_relative_path_for_test, option_volume_to_mixer_volume_for_test,
    parse_resolution_label_for_test, render_headless_average_color_for_test,
    shell_control_draw_order_for_test, slider_sound_summaries_for_test,
};

/// Finds an installed font suitable for button labels.
fn find_existing_font() -> Option<PathBuf> {
    FONT_CANDIDATES
        .iter()
        .map(PathBuf::from)
        .find(|path| path_is_regular_file(path))
}

/// Returns whether the path metadata identifies a regular file.
fn path_is_regular_file(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|metadata| !metadata.file_type().is_dir())
}

/// Wraps a runtime error string.
const fn runtime_error(message: String) -> MainMenuError {
    MainMenuError::Runtime { message }
}
