//! In-game Call to Power II gameplay model and SDL runtime.
//!
//! This module owns the post-Launch game experience.  The model is deliberately
//! independent from SDL so tests can exercise movement, fog of war, ruins,
//! menus, shortcuts, and turn processing without writing files or opening a
//! window.

extern crate alloc;

use alloc::string::String;
use core::error::Error;
use core::fmt::{Display, Formatter, Result as FmtResult};
use std::path::PathBuf;

use crate::config::GameInstallation;

/// Original-asset loading and sprite parsing helpers.
pub mod assets;
/// Gameplay sound-effect manifest and runtime identifiers.
pub mod audio;
/// Testable single-player game-state model.
pub mod model;
/// SDL renderer/event loop for the in-game state.
pub mod runtime;
/// Authoritative Open CTP server HTTP client.
pub mod server;
/// Testable in-game menu/control-panel model.
pub mod ui_model;

/// Errors raised while launching or running the in-game session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameplayError {
    /// SDL/video/audio runtime failed.
    Runtime {
        /// Human-readable error details.
        message: String,
    },
    /// A requested asset could not be loaded or decoded.
    Asset {
        /// Human-readable error details.
        message: String,
    },
}

impl Display for GameplayError {
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Display matches borrowed enum variants so messages are formatted without cloning"
    )]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Runtime { message } => write!(f, "gameplay runtime failed: {message}"),
            Self::Asset { message } => write!(f, "gameplay asset failed: {message}"),
        }
    }
}

impl Error for GameplayError {}

/// Creates a runtime error with a plain message.
#[must_use]
pub const fn runtime_error(message: String) -> GameplayError {
    GameplayError::Runtime { message }
}

/// Creates an asset error with a plain message.
#[must_use]
pub const fn asset_error(message: String) -> GameplayError {
    GameplayError::Asset { message }
}

/// Result of an in-game runtime session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameplayOutcome {
    /// The player requested a return to the single-player setup/new-game flow.
    NewGame,
    /// The player requested application quit.
    Quit,
    /// The SDL window was closed without a higher-level action.
    Closed,
}

/// Configuration copied from the single-player setup screen into gameplay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SinglePlayerLaunchConfig {
    /// Installation root containing CTP2 assets.
    pub installation_root: PathBuf,
    /// Number of non-barbarian empires requested by setup.
    pub empire_count: u8,
    /// Leader name selected before launch.
    pub leader_name: String,
    /// Deterministic world seed.
    pub seed: u64,
    /// Authoritative server connection settings.
    pub server: server::ServerConfig,
    /// Graphics options selected in the shell, including unit travel/animation speed.
    pub graphics: crate::config::ctp2_options::Ctp2GraphicsOptions,
}

impl SinglePlayerLaunchConfig {
    /// Builds a default launch configuration from an installation.
    #[must_use]
    pub fn from_installation(installation: &GameInstallation) -> Self {
        Self {
            installation_root: installation.path.clone(),
            empire_count: 4,
            leader_name: "Leader".to_owned(),
            seed: 0x4354_5032_u64,
            server: server::ServerConfig::interactive_default(),
            graphics: crate::config::ctp2_options::Ctp2GraphicsOptions::default(),
        }
    }
}

/// Runs the SDL-backed single-player game after the shell Launch button.
///
/// # Errors
///
/// Returns [`GameplayError`] when SDL setup or rendering fails.
pub fn run_sdl_singleplayer_game(
    installation: &GameInstallation,
) -> Result<GameplayOutcome, GameplayError> {
    let config = SinglePlayerLaunchConfig::from_installation(installation);
    runtime::run_sdl_game(&config)
}

/// Runs a real SDL gameplay smoke test with dummy drivers.
///
/// # Errors
///
/// Returns [`GameplayError`] when headless SDL setup or the frame loop fails.
#[cfg(test)]
pub fn run_headless_gameplay_smoke_for_test(
    installation_root: PathBuf,
) -> Result<GameplayOutcome, GameplayError> {
    runtime::run_headless_smoke_for_test(&SinglePlayerLaunchConfig {
        installation_root,
        empire_count: 2,
        leader_name: "Tester".to_owned(),
        seed: 1,
        server: server::ServerConfig::disabled_for_test(),
        graphics: crate::config::ctp2_options::Ctp2GraphicsOptions::default(),
    })
}
