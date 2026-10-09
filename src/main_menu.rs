//! Testable Call to Power II landing main menu model.
//!
//! The data in this module mirrors the original `InitPlayWindow` definitions
//! from `ctp2_data/english/uidata/layouts/spnewgame.ldl` and the button wiring
//! in `ui/interface/initialplaywindow.cpp`. Runtime rendering is intentionally
//! separated from the model so tests can find and activate controls by
//! accessible names instead of screen coordinates.

extern crate alloc;

/// SDL runtime for the landing and single-player shell menus.
mod runtime;
/// Testable single-player shell screen models.
mod shell;

use alloc::string::String;
use alloc::vec::Vec;
use core::error::Error;
use core::fmt::{Display, Formatter, Result as FmtResult};
use std::path::PathBuf;

use crate::config::GameInstallation;

/// Width of the original CTP2 shell screens at the supported resolution.
pub const MAIN_MENU_WIDTH: u32 = 800;
/// Height of the original CTP2 shell screens at the supported resolution.
pub const MAIN_MENU_HEIGHT: u32 = 600;

/// A rectangle in logical 800x600 menu coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuRect {
    /// Left edge in pixels.
    pub x: i32,
    /// Top edge in pixels.
    pub y: i32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl MenuRect {
    /// Creates a new menu rectangle.
    #[must_use]
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns whether the provided point is inside this rectangle.
    #[must_use]
    pub fn contains_point(&self, point_x: i32, point_y: i32) -> bool {
        let width = i32::try_from(self.width).map_or(i32::MAX, |value| value);
        let height = i32::try_from(self.height).map_or(i32::MAX, |value| value);
        let right = self.x.saturating_add(width);
        let bottom = self.y.saturating_add(height);

        point_x >= self.x && point_x < right && point_y >= self.y && point_y < bottom
    }
}

/// Three-piece image set used by the original stretchable CTP2 buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonAssetTriplet {
    /// Left cap image file from the original LDL style.
    pub left: &'static str,
    /// Center stretch image file from the original LDL style.
    pub middle: &'static str,
    /// Right cap image file from the original LDL style.
    pub right: &'static str,
}

impl ButtonAssetTriplet {
    /// Creates a new three-piece button asset set.
    #[must_use]
    pub const fn new(left: &'static str, middle: &'static str, right: &'static str) -> Self {
        Self {
            left,
            middle,
            right,
        }
    }
}

/// Asset groups for the original CTP2 button interaction states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonStateAssets {
    /// Original inactive/disabled triplet (`image00`-`image02`).
    pub inactive: ButtonAssetTriplet,
    /// Original up/default triplet (`image10`-`image12`).
    pub up: ButtonAssetTriplet,
    /// Original down/pressed triplet (`image20`-`image22`).
    pub down: ButtonAssetTriplet,
    /// Original hover triplet (`image30`-`image32`).
    pub hover: ButtonAssetTriplet,
}

/// Identifier for visual button styles copied from the original LDL templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyleId {
    /// Blue/gold large shell button, `SP_BUTTON_BIG` in `spnewgame.ldl`.
    BlueLarge,
    /// Stone large shell button, `SP_BUTTON_BIG_STONE` in `spnewgame.ldl`.
    StoneLarge,
    /// Default small popup button, `CTP2_BUTTON_SMALL` / `c3_PopupClose`.
    Ctp2Small,
}

impl ButtonStyleId {
    /// Returns complete visual metadata for this style identifier.
    #[must_use]
    pub const fn visual_style(self) -> ButtonVisualStyle {
        match self {
            Self::BlueLarge => ButtonVisualStyle::blue_large(),
            Self::StoneLarge => ButtonVisualStyle::stone_large(),
            Self::Ctp2Small => ButtonVisualStyle::ctp2_small(),
        }
    }
}

/// Complete visual style metadata for one menu button style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonVisualStyle {
    /// Stable style identifier.
    pub id: ButtonStyleId,
    /// LDL template name used by the original source.
    pub ldl_template: &'static str,
    /// Source asset names for each interaction state.
    pub assets: ButtonStateAssets,
    /// Original template width before LDL per-button overrides.
    pub template_width: u32,
    /// Original template height before LDL per-button overrides.
    pub template_height: u32,
}

impl ButtonVisualStyle {
    /// Returns the original blue large button style.
    #[must_use]
    pub const fn blue_large() -> Self {
        Self {
            id: ButtonStyleId::BlueLarge,
            ldl_template: "SP_BUTTON_BIG",
            assets: ButtonStateAssets {
                inactive: ButtonAssetTriplet::new("upbt07aX.tga", "upbt07bX.tga", "upbt07cX.tga"),
                up: ButtonAssetTriplet::new("upbt07aU.tga", "upbt07bU.tga", "upbt07cU.tga"),
                down: ButtonAssetTriplet::new("upbt07aD.tga", "upbt07bD.tga", "upbt07cD.tga"),
                hover: ButtonAssetTriplet::new("upbt07aH.tga", "upbt07bH.tga", "upbt07cH.tga"),
            },
            template_width: 99,
            template_height: 20,
        }
    }

    /// Returns the original stone large button style.
    #[must_use]
    pub const fn stone_large() -> Self {
        Self {
            id: ButtonStyleId::StoneLarge,
            ldl_template: "SP_BUTTON_BIG_STONE",
            assets: ButtonStateAssets {
                inactive: ButtonAssetTriplet::new("upbt06aX.tga", "upbt06bX.tga", "upbt06cX.tga"),
                up: ButtonAssetTriplet::new("upbt06aU.tga", "upbt06bU.tga", "upbt06cU.tga"),
                down: ButtonAssetTriplet::new("upbt06aD.tga", "upbt06bD.tga", "upbt06cD.tga"),
                hover: ButtonAssetTriplet::new("upbt06aH.tga", "upbt06bH.tga", "upbt06cH.tga"),
            },
            template_width: 99,
            template_height: 20,
        }
    }

    /// Returns the original default small popup button style.
    #[must_use]
    pub const fn ctp2_small() -> Self {
        Self {
            id: ButtonStyleId::Ctp2Small,
            ldl_template: "CTP2_BUTTON_SMALL",
            assets: ButtonStateAssets {
                inactive: ButtonAssetTriplet::new("upbt01aX.tga", "upbt01bX.tga", "upbt01cX.tga"),
                up: ButtonAssetTriplet::new("upbt01aU.tga", "upbt01bU.tga", "upbt01cU.tga"),
                down: ButtonAssetTriplet::new("upbt01aD.tga", "upbt01bD.tga", "upbt01cD.tga"),
                hover: ButtonAssetTriplet::new("upbt01aH.tga", "upbt01bH.tga", "upbt01cH.tga"),
            },
            template_width: 70,
            template_height: 24,
        }
    }

    /// Returns the native rendered height of the original button art family.
    #[must_use]
    pub const fn rendered_height(self) -> u32 {
        match self.id {
            ButtonStyleId::BlueLarge | ButtonStyleId::StoneLarge => 39,
            ButtonStyleId::Ctp2Small => 24,
        }
    }

    /// Returns the rectangle covered by the rendered button art.
    #[must_use]
    pub const fn rendered_rect(self, rect: MenuRect) -> MenuRect {
        let rendered_height = self.rendered_height();
        let height = if rect.height > rendered_height {
            rect.height
        } else {
            rendered_height
        };
        MenuRect::new(rect.x, rect.y, rect.width, height)
    }
}

/// Stable identifier for each accessible landing-page button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuButtonId {
    /// `SPButton` in the original LDL and C++.
    SinglePlayer,
    /// `MPButton` in the original LDL and C++.
    Multiplayer,
    /// `CreditsButton` in the original LDL and C++.
    Credits,
    /// `QuitButton` in the original LDL and C++.
    Quit,
}

/// Resulting high-level action when a main-menu button is activated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuAction {
    /// Show the single-player shell flow.
    SinglePlayer,
    /// Launch single-player gameplay from the completed setup screen.
    LaunchSinglePlayer,
    /// Show the multiplayer shell flow.
    Multiplayer,
    /// Show the credits screen.
    Credits,
    /// Quit the client.
    Quit,
}

impl MainMenuAction {
    /// Returns a stable label suitable for logs and tests.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SinglePlayer => "single-player",
            Self::LaunchSinglePlayer => "launch-single-player",
            Self::Multiplayer => "multiplayer",
            Self::Credits => "credits",
            Self::Quit => "quit",
        }
    }
}

/// One accessible button on the CTP2 landing menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MainMenuButton {
    /// Stable test and runtime identifier.
    pub id: MainMenuButtonId,
    /// Original LDL object name.
    pub ldl_name: &'static str,
    /// Text displayed on the button.
    pub label: &'static str,
    /// Accessible name used by tests and assistive activation paths.
    pub accessible_name: &'static str,
    /// Logical bounds copied from `spnewgame.ldl`.
    pub rect: MenuRect,
    /// Original visual style metadata.
    pub style: ButtonVisualStyle,
    /// Action emitted when this button is activated.
    pub action: MainMenuAction,
}

/// Reference to one asset inside a CTP2 ZFS archive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuAssetReference {
    /// Archive path relative to the installation root.
    pub archive_relative_path: &'static str,
    /// Entry name inside the archive.
    pub entry_name: &'static str,
}

/// Music autoplay configuration for the landing menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuMusicPolicy {
    /// Whether the menu should start music as soon as it is displayed.
    pub autoplay: bool,
    /// Music file path relative to the CTP2 installation root.
    pub relative_path: PathBuf,
    /// `SDL_mixer` loop count; `-1` means loop indefinitely.
    pub loops: i32,
}

/// Testable landing-menu model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MainMenuModel {
    /// Root directory of the CTP2 installation currently being launched.
    pub installation_root: PathBuf,
    /// Logical menu width.
    pub width: u32,
    /// Logical menu height.
    pub height: u32,
    /// Background image reference from the original LDL.
    pub background: MenuAssetReference,
    /// Landing-page buttons.
    pub buttons: Vec<MainMenuButton>,
    /// Menu music policy.
    pub music: MenuMusicPolicy,
    /// Version string shown by the original LDL landing page.
    pub version_text: &'static str,
    /// Version string bounds from the original LDL.
    pub version_rect: MenuRect,
}

impl MainMenuModel {
    /// Builds the Call to Power II landing page model for an installation root.
    #[must_use]
    pub fn ctp2_landing(installation_root: PathBuf) -> Self {
        Self {
            installation_root,
            width: MAIN_MENU_WIDTH,
            height: MAIN_MENU_HEIGHT,
            background: MenuAssetReference {
                archive_relative_path: "ctp2_data/default/graphics/pictures/pic565.zfs",
                entry_name: "upsg001.rim",
            },
            buttons: original_landing_buttons(),
            music: MenuMusicPolicy {
                autoplay: true,
                relative_path: PathBuf::from("ctp2_program/ctp/music/Track02.ogg"),
                loops: -1,
            },
            version_text: "1.1",
            version_rect: MenuRect::new(720, 515, 60, 24),
        }
    }

    /// Returns the absolute music path for the current installation.
    #[must_use]
    pub fn music_path(&self) -> PathBuf {
        self.installation_root.join(&self.music.relative_path)
    }

    /// Finds a visible button by accessible name, case-insensitively.
    #[must_use]
    pub fn find_button_by_accessible_name(&self, accessible_name: &str) -> Option<&MainMenuButton> {
        self.buttons
            .iter()
            .find(|button| button.accessible_name.eq_ignore_ascii_case(accessible_name))
    }

    /// Finds a visible button by stable identifier.
    #[must_use]
    pub fn button_by_id(&self, id: MainMenuButtonId) -> Option<&MainMenuButton> {
        self.buttons.iter().find(|button| button.id == id)
    }

    /// Activates a button by accessible name.
    ///
    /// # Errors
    ///
    /// Returns [`MainMenuError::ButtonNotFound`] when no button has that accessible name.
    pub fn activate_accessible(
        &self,
        accessible_name: &str,
    ) -> Result<MainMenuAction, MainMenuError> {
        self.find_button_by_accessible_name(accessible_name)
            .map(|button| button.action)
            .ok_or_else(|| MainMenuError::ButtonNotFound {
                accessible_name: accessible_name.to_owned(),
            })
    }

    /// Activates the button containing the supplied point, if any.
    #[must_use]
    pub fn activate_at_point(&self, point_x: i32, point_y: i32) -> Option<MainMenuAction> {
        self.buttons
            .iter()
            .find(|button| {
                button
                    .style
                    .rendered_rect(button.rect)
                    .contains_point(point_x, point_y)
            })
            .map(|button| button.action)
    }

    /// Returns style/layout mismatches compared with the original LDL source.
    #[must_use]
    pub fn original_style_mismatches(&self) -> Vec<String> {
        let mut mismatches = Vec::new();
        let expected_buttons = original_landing_buttons();

        for expected in expected_buttons {
            match self.button_by_id(expected.id) {
                Some(actual) if actual == &expected => {}
                Some(actual) => mismatches.push(format!(
                    "{} differs from original style/layout: expected {} at {}x{}, got {} at {}x{}",
                    expected.ldl_name,
                    expected.style.ldl_template,
                    expected.rect.x,
                    expected.rect.y,
                    actual.style.ldl_template,
                    actual.rect.x,
                    actual.rect.y
                )),
                None => mismatches.push(format!(
                    "{} is missing from the landing menu",
                    expected.ldl_name
                )),
            }
        }

        mismatches
    }
}

/// Errors produced by the menu model or runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MainMenuError {
    /// No button could be found for an accessible activation request.
    ButtonNotFound {
        /// Accessible name that was requested.
        accessible_name: String,
    },
    /// Menu music could not be started.
    Music {
        /// Human-readable error details.
        message: String,
    },
    /// SDL/runtime rendering failed.
    Runtime {
        /// Human-readable error details.
        message: String,
    },
}

impl Display for MainMenuError {
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Borrowed enum matching keeps error strings available for Display formatting"
    )]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::ButtonNotFound { accessible_name } => {
                write!(f, "Main menu button not found: {accessible_name}")
            }
            Self::Music { message } => write!(f, "Main menu music failed: {message}"),
            Self::Runtime { message } => write!(f, "Main menu runtime failed: {message}"),
        }
    }
}

impl Error for MainMenuError {}

/// Audio backend used by menu sessions.
pub trait MainMenuMusicBackend {
    /// Starts menu music using the supplied policy and absolute path.
    ///
    /// # Errors
    ///
    /// Returns [`MainMenuError::Music`] or [`MainMenuError::Runtime`] when playback setup fails.
    fn play_music(
        &mut self,
        policy: &MenuMusicPolicy,
        absolute_path: PathBuf,
    ) -> Result<(), MainMenuError>;
}

/// Runtime session that keeps menu music alive while the landing menu is visible.
pub struct MainMenuSession<AudioBackend> {
    /// Testable menu model.
    model: MainMenuModel,
    /// Audio backend used for autoplay.
    audio_backend: AudioBackend,
    /// Whether autoplay has already been requested.
    music_started: bool,
}

impl<AudioBackend> MainMenuSession<AudioBackend>
where
    AudioBackend: MainMenuMusicBackend,
{
    /// Creates a new menu session.
    #[must_use]
    pub const fn new(model: MainMenuModel, audio_backend: AudioBackend) -> Self {
        Self {
            model,
            audio_backend,
            music_started: false,
        }
    }

    /// Starts autoplay music once when configured by the model.
    ///
    /// # Errors
    ///
    /// Returns a [`MainMenuError`] if the audio backend fails.
    pub fn start_music(&mut self) -> Result<(), MainMenuError> {
        if self.music_started || !self.model.music.autoplay {
            return Ok(());
        }

        let music_path = self.model.music_path();
        self.audio_backend
            .play_music(&self.model.music, music_path)?;
        self.music_started = true;
        Ok(())
    }

    /// Returns the model rendered by this session.
    #[must_use]
    pub const fn model(&self) -> &MainMenuModel {
        &self.model
    }

    /// Returns whether music has started.
    #[must_use]
    pub const fn music_started(&self) -> bool {
        self.music_started
    }
}

/// Runs the SDL-backed CTP2 main menu for an installation.
///
/// # Errors
///
/// Returns a [`MainMenuError`] when SDL, audio, asset loading, or rendering fails.
pub fn run_sdl_main_menu(installation: &GameInstallation) -> Result<MainMenuAction, MainMenuError> {
    runtime::run_sdl_main_menu(installation)
}

/// Runs a real SDL render/event-loop smoke test with headless SDL drivers.
///
/// # Errors
///
/// Returns a [`MainMenuError`] when the SDL runtime cannot create, render, or exit the menu.
#[cfg(test)]
pub fn run_headless_sdl_smoke_for_test(
    installation_root: PathBuf,
) -> Result<MainMenuAction, MainMenuError> {
    run_with_headless_sdl_environment(installation_root, runtime::run_headless_smoke_for_test)
}

/// Renders one real headless SDL menu frame and returns its average RGB color.
///
/// # Errors
///
/// Returns a [`MainMenuError`] when the SDL runtime cannot render or read back the menu frame.
#[cfg(test)]
pub fn render_headless_menu_average_color_for_test(
    installation_root: PathBuf,
) -> Result<(u8, u8, u8), MainMenuError> {
    run_with_headless_sdl_environment(
        installation_root,
        runtime::render_headless_average_color_for_test,
    )
}

/// Runs a callback with dummy SDL drivers applied and restored.
#[cfg(test)]
fn run_with_headless_sdl_environment<ResultValue, Callback>(
    installation_root: PathBuf,
    callback: Callback,
) -> Result<ResultValue, MainMenuError>
where
    Callback: FnOnce(&MainMenuModel) -> Result<ResultValue, MainMenuError>,
{
    let _guard = headless_sdl_env_lock()
        .lock()
        .map_err(|error| MainMenuError::Runtime {
            message: format!("headless SDL environment lock is poisoned: {error}"),
        })?;
    let environment = HeadlessSdlEnvironment::apply();
    let model = MainMenuModel::ctp2_landing(installation_root);
    let result = callback(&model);
    environment.restore();
    result
}

/// Stores previous SDL environment values while a headless smoke test runs.
#[cfg(test)]
struct HeadlessSdlEnvironment {
    /// Previous `SDL_VIDEODRIVER` value.
    previous_video_driver: Option<std::ffi::OsString>,
    /// Previous `SDL_AUDIODRIVER` value.
    previous_audio_driver: Option<std::ffi::OsString>,
}

#[cfg(test)]
impl HeadlessSdlEnvironment {
    /// Applies dummy SDL drivers and returns a restorable snapshot.
    fn apply() -> Self {
        let snapshot = Self {
            previous_video_driver: std::env::var_os("SDL_VIDEODRIVER"),
            previous_audio_driver: std::env::var_os("SDL_AUDIODRIVER"),
        };

        // SAFETY: A process-wide mutex serializes all test mutations of these
        // environment variables, and values are restored before the lock is released.
        unsafe {
            std::env::set_var("SDL_VIDEODRIVER", "dummy");
            std::env::set_var("SDL_AUDIODRIVER", "dummy");
        }

        snapshot
    }

    /// Restores SDL environment values captured by [`Self::apply`].
    fn restore(self) {
        restore_env_var("SDL_VIDEODRIVER", self.previous_video_driver);
        restore_env_var("SDL_AUDIODRIVER", self.previous_audio_driver);
    }
}

/// Restores or removes one environment variable.
#[cfg(test)]
fn restore_env_var(key: &str, previous: Option<std::ffi::OsString>) {
    // SAFETY: The same process-wide mutex held during `apply` is still held
    // during restoration, so no concurrent test observes a partial update.
    unsafe {
        match previous {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}

/// Returns the global mutex used to serialize headless SDL environment changes.
#[cfg(test)]
fn headless_sdl_env_lock() -> &'static std::sync::Mutex<()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
}

/// Returns the landing-page buttons copied from `InitPlayWindow` in `spnewgame.ldl`.
fn original_landing_buttons() -> Vec<MainMenuButton> {
    vec![
        MainMenuButton {
            id: MainMenuButtonId::Multiplayer,
            ldl_name: "MPButton",
            label: "MULTIPLAYER",
            accessible_name: "Multiplayer",
            rect: MenuRect::new(36, 455, 290, 32),
            style: ButtonVisualStyle::blue_large(),
            action: MainMenuAction::Multiplayer,
        },
        MainMenuButton {
            id: MainMenuButtonId::SinglePlayer,
            ldl_name: "SPButton",
            label: "SINGLE PLAYER",
            accessible_name: "Single Player",
            rect: MenuRect::new(478, 455, 290, 32),
            style: ButtonVisualStyle::blue_large(),
            action: MainMenuAction::SinglePlayer,
        },
        MainMenuButton {
            id: MainMenuButtonId::Credits,
            ldl_name: "CreditsButton",
            label: "CREDITS",
            accessible_name: "Credits",
            rect: MenuRect::new(36, 558, 140, 32),
            style: ButtonVisualStyle::stone_large(),
            action: MainMenuAction::Credits,
        },
        MainMenuButton {
            id: MainMenuButtonId::Quit,
            ldl_name: "QuitButton",
            label: "QUIT",
            accessible_name: "Quit",
            rect: MenuRect::new(624, 558, 140, 32),
            style: ButtonVisualStyle::stone_large(),
            action: MainMenuAction::Quit,
        },
    ]
}

#[cfg(test)]
mod tests;
