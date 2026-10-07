//! SDL runtime for the Call to Power II landing main menu.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use sdl2::AudioSubsystem;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mixer::{self, DEFAULT_CHANNELS, DEFAULT_FORMAT, InitFlag, Music, Sdl2MixerContext};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect as SdlRect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};
use unciv::ZfsFile;

use crate::config::GameInstallation;

use super::{
    ButtonStyleId, MainMenuAction, MainMenuError, MainMenuModel, MainMenuMusicBackend,
    MainMenuSession, MenuAssetReference, MenuRect,
};

/// Audio frequency requested from `SDL_mixer` for menu music.
const MIXER_FREQUENCY: i32 = 44_100;
/// Mixer chunk size balancing latency and compatibility.
const MIXER_CHUNK_SIZE: i32 = 1_024;
/// Delay between menu frames.
const FRAME_DELAY: Duration = Duration::from_millis(16);
/// Candidate fonts used only for drawing fallback text over original assets.
const FONT_CANDIDATES: [&str; 5] = [
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
    "/usr/share/fonts/truetype/liberation2/LiberationSans-Bold.ttf",
    "/usr/share/fonts/truetype/freefont/FreeSansBold.ttf",
    "C:/Windows/Fonts/arialbd.ttf",
    "C:/Windows/Fonts/Arial.ttf",
];

/// Event-loop settings used by production and headless smoke tests.
struct RuntimeLoopOptions {
    /// Events pushed into SDL's queue immediately after window creation.
    initial_events: Vec<Event>,
    /// Optional maximum number of frames rendered before returning an error.
    frame_limit: Option<u32>,
}

impl RuntimeLoopOptions {
    /// Creates unbounded interactive event-loop options.
    const fn interactive() -> Self {
        Self {
            initial_events: Vec::new(),
            frame_limit: None,
        }
    }

    /// Creates bounded options for smoke tests that must not hang.
    #[cfg(test)]
    fn headless_smoke() -> Self {
        Self {
            initial_events: vec![Event::Quit { timestamp: 0 }],
            frame_limit: Some(3),
        }
    }
}

/// Image decoded from a RIM entry into RGBA bytes.
struct RgbaImage {
    /// Width in pixels.
    width: u32,
    /// Height in pixels.
    height: u32,
    /// Bytes per image row.
    pitch: usize,
    /// Pixel data in RGBA order.
    data: Vec<u8>,
}

/// SDL_mixer-backed menu music player.
pub struct SdlMenuMusicBackend {
    /// SDL context kept alive for the audio subsystem.
    _sdl: sdl2::Sdl,
    /// Audio subsystem lifetime guard.
    _audio: AudioSubsystem,
    /// Mixer context lifetime guard.
    _mixer_context: Sdl2MixerContext,
    /// Currently loaded music, kept alive while playback continues.
    music: Option<Music<'static>>,
}

impl SdlMenuMusicBackend {
    /// Creates and initializes the `SDL_mixer` backend.
    ///
    /// # Errors
    ///
    /// Returns [`MainMenuError::Music`] when SDL audio or mixer setup fails.
    fn new() -> Result<Self, MainMenuError> {
        let sdl = sdl2::init().map_err(|message| MainMenuError::Music { message })?;
        let audio = sdl
            .audio()
            .map_err(|message| MainMenuError::Music { message })?;
        let mixer_context =
            mixer::init(InitFlag::OGG).map_err(|message| MainMenuError::Music { message })?;
        mixer::open_audio(
            MIXER_FREQUENCY,
            DEFAULT_FORMAT,
            DEFAULT_CHANNELS,
            MIXER_CHUNK_SIZE,
        )
        .map_err(|message| MainMenuError::Music { message })?;
        mixer::allocate_channels(4);

        Ok(Self {
            _sdl: sdl,
            _audio: audio,
            _mixer_context: mixer_context,
            music: None,
        })
    }
}

impl MainMenuMusicBackend for SdlMenuMusicBackend {
    fn play_music(
        &mut self,
        policy: &super::MenuMusicPolicy,
        absolute_path: PathBuf,
    ) -> Result<(), MainMenuError> {
        let music = Music::from_file(&absolute_path).map_err(|message| MainMenuError::Music {
            message: format!("failed to load {}: {message}", absolute_path.display()),
        })?;
        music
            .play(policy.loops)
            .map_err(|message| MainMenuError::Music { message })?;
        self.music = Some(music);
        Ok(())
    }
}

impl Drop for SdlMenuMusicBackend {
    fn drop(&mut self) {
        Music::halt();
        mixer::close_audio();
    }
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

    let mut music_session = match SdlMenuMusicBackend::new() {
        Ok(audio) => Some(MainMenuSession::new(model.clone(), audio)),
        Err(error) => {
            log::warn!("Main menu music backend is unavailable: {error}");
            None
        }
    };

    if let Some(session) = music_session.as_mut() {
        if let Err(error) = session.start_music() {
            log::warn!("Main menu music could not be started: {error}");
        }
    }

    run_sdl_event_loop(&model)
}

/// Runs a real SDL menu frame in headless mode for tests.
#[cfg(test)]
pub(super) fn run_headless_smoke_for_test(
    model: &MainMenuModel,
) -> Result<MainMenuAction, MainMenuError> {
    run_sdl_event_loop_with_options(model, RuntimeLoopOptions::headless_smoke())
}

/// Runs the interactive SDL event loop until a menu action is selected.
fn run_sdl_event_loop(model: &MainMenuModel) -> Result<MainMenuAction, MainMenuError> {
    run_sdl_event_loop_with_options(model, RuntimeLoopOptions::interactive())
}

/// Runs the SDL event loop with configurable test hooks.
fn run_sdl_event_loop_with_options(
    model: &MainMenuModel,
    options: RuntimeLoopOptions,
) -> Result<MainMenuAction, MainMenuError> {
    let sdl = sdl2::init().map_err(runtime_error)?;
    let video = sdl.video().map_err(runtime_error)?;
    let window = video
        .window("Call to Power II", model.width, model.height)
        .position_centered()
        .build()
        .map_err(|error| runtime_error(error.to_string()))?;
    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|error| runtime_error(error.to_string()))?;
    let texture_creator = canvas.texture_creator();
    let background_texture = load_background_texture(&texture_creator, model).ok();
    let ttf_context = sdl2::ttf::init().ok();
    let font_path = find_existing_font();
    let font = ttf_context.as_ref().and_then(|context| {
        font_path
            .as_ref()
            .and_then(|path| context.load_font(path, 18).ok())
    });

    let event_subsystem = sdl.event().map_err(runtime_error)?;
    for event in options.initial_events {
        event_subsystem.push_event(event).map_err(runtime_error)?;
    }

    let mut event_pump = sdl.event_pump().map_err(runtime_error)?;
    let mut selected_action = None;
    let mut rendered_frames = 0_u32;

    while selected_action.is_none() {
        for event in event_pump.poll_iter() {
            handle_menu_event(&event, model, &mut selected_action);
        }

        let mouse_state = event_pump.mouse_state();
        draw_menu(
            &mut canvas,
            background_texture.as_ref(),
            model,
            mouse_state.x(),
            mouse_state.y(),
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

/// Applies one SDL event to the menu action state.
fn handle_menu_event(
    event: &Event,
    model: &MainMenuModel,
    selected_action: &mut Option<MainMenuAction>,
) {
    match event {
        &Event::Quit { .. }
        | &Event::KeyDown {
            keycode: Some(Keycode::Escape),
            repeat: false,
            ..
        } => *selected_action = Some(MainMenuAction::Quit),
        &Event::MouseButtonDown { x, y, .. } => {
            if let Some(action) = model.activate_at_point(x, y) {
                *selected_action = Some(action);
            }
        }
        &Event::KeyDown { .. }
        | &Event::KeyUp { .. }
        | &Event::MouseMotion { .. }
        | &Event::MouseButtonUp { .. }
        | &Event::MouseWheel { .. }
        | &Event::ControllerButtonDown { .. }
        | &Event::ControllerButtonUp { .. }
        | &Event::ControllerAxisMotion { .. }
        | &Event::Window { .. }
        | &Event::AppTerminating { .. }
        | &Event::AppLowMemory { .. }
        | &Event::AppWillEnterBackground { .. }
        | &Event::AppDidEnterBackground { .. }
        | &Event::AppWillEnterForeground { .. }
        | &Event::AppDidEnterForeground { .. }
        | &Event::Display { .. }
        | &Event::TextEditing { .. }
        | &Event::TextInput { .. }
        | &Event::JoyAxisMotion { .. }
        | &Event::JoyBallMotion { .. }
        | &Event::JoyHatMotion { .. }
        | &Event::JoyButtonDown { .. }
        | &Event::JoyButtonUp { .. }
        | &Event::JoyDeviceAdded { .. }
        | &Event::JoyDeviceRemoved { .. }
        | &Event::ControllerDeviceAdded { .. }
        | &Event::ControllerDeviceRemoved { .. }
        | &Event::ControllerDeviceRemapped { .. }
        | &Event::ControllerTouchpadDown { .. }
        | &Event::ControllerTouchpadMotion { .. }
        | &Event::ControllerTouchpadUp { .. }
        | &Event::FingerDown { .. }
        | &Event::FingerUp { .. }
        | &Event::FingerMotion { .. }
        | &Event::DollarGesture { .. }
        | &Event::DollarRecord { .. }
        | &Event::MultiGesture { .. }
        | &Event::ClipboardUpdate { .. }
        | &Event::DropFile { .. }
        | &Event::DropText { .. }
        | &Event::DropBegin { .. }
        | &Event::DropComplete { .. }
        | &Event::AudioDeviceAdded { .. }
        | &Event::AudioDeviceRemoved { .. }
        | &Event::RenderTargetsReset { .. }
        | &Event::RenderDeviceReset { .. }
        | &Event::User { .. }
        | &Event::Unknown { .. } => {}
    }
}

/// Draws one frame of the menu.
fn draw_menu(
    canvas: &mut Canvas<Window>,
    background_texture: Option<&Texture<'_>>,
    model: &MainMenuModel,
    mouse_x: i32,
    mouse_y: i32,
    font: Option<&Font<'_, '_>>,
) -> Result<(), MainMenuError> {
    match background_texture {
        Some(texture) => canvas.copy(texture, None, None).map_err(runtime_error)?,
        None => draw_fallback_background(canvas)?,
    }

    for button in &model.buttons {
        let hovered = button.rect.contains_point(mouse_x, mouse_y);
        draw_button(canvas, &button.rect, button.style.id, hovered)?;
        draw_centered_text(canvas, font, &button.rect, button.label)?;
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
fn draw_button(
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
    };
    let border = match style {
        ButtonStyleId::BlueLarge => Color::RGB(211, 181, 85),
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
        .blended(Color::RGB(240, 222, 154))
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
        .blended(Color::RGB(100, 100, 255))
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

/// Loads the original background texture from the CTP2 ZFS archive.
fn load_background_texture<'texture>(
    texture_creator: &'texture TextureCreator<WindowContext>,
    model: &MainMenuModel,
) -> Result<Texture<'texture>, MainMenuError> {
    let image = load_rim_rgba(&model.installation_root, &model.background)?;
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGBA8888, image.width, image.height)
        .map_err(|error| runtime_error(error.to_string()))?;
    texture
        .update(None, &image.data, image.pitch)
        .map_err(|error| runtime_error(error.to_string()))?;
    Ok(texture)
}

/// Loads and converts a RIM image from a ZFS archive without copying assets to the repository.
fn load_rim_rgba(
    installation_root: &Path,
    asset: &MenuAssetReference,
) -> Result<RgbaImage, MainMenuError> {
    let archive_path = installation_root.join(asset.archive_relative_path);
    let file = File::open(&archive_path).map_err(|error| {
        runtime_error(format!(
            "failed to open {}: {error}",
            archive_path.display()
        ))
    })?;
    let mut reader = BufReader::new(file);
    let zfs = ZfsFile::from_stream(&mut reader).map_err(|error| {
        runtime_error(format!(
            "failed to parse {}: {error}",
            archive_path.display()
        ))
    })?;
    let entry = zfs
        .files
        .iter()
        .find(|candidate| candidate.name.eq_ignore_ascii_case(asset.entry_name))
        .cloned()
        .ok_or_else(|| {
            runtime_error(format!(
                "{} was not found in {}",
                asset.entry_name,
                archive_path.display()
            ))
        })?;
    let rim = entry.read_rim_image(&mut reader).map_err(|error| {
        runtime_error(format!(
            "failed to decode {} from {}: {error}",
            asset.entry_name,
            archive_path.display()
        ))
    })?;
    let width = u32::from(rim.width);
    let height = u32::from(rim.height);
    let pitch = usize::from(rim.width)
        .checked_mul(4)
        .ok_or_else(|| runtime_error("RIM pitch overflow".to_owned()))?;

    Ok(RgbaImage {
        width,
        height,
        pitch,
        data: rim.to_rgba_bytes(),
    })
}

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
