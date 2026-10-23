//! Library-owned application startup, shared by the executable and its tests.
//!
//! Keeping startup here prevents the binary from compiling a second copy of
//! every library module and executing all shared unit tests twice. Native launch,
//! intro/menu handoff and extraction behavior are unchanged; no UI layout or
//! rendering/scaling rules are introduced by this relocation.

use crate::ui::{GtkUI, UIResult};
use crate::video::{VideoPlayer, VideoResult};
use std::sync::Once;

/// Asset preparation's native adapter and injectable orchestration.
mod assets;
/// The original non-extraction startup cases, now compiled only in the library.
#[cfg(test)]
mod tests;

/// Ensures logger initialization occurs only once in this process.
static INIT: Once = Once::new();

/// Runs the original native installation picker and game launch sequence.
///
/// Asset preparation, intro playback and existing-menu-window handoff retain
/// their error boundaries instead of starting a different or fallback game.
pub fn run() {
    INIT.call_once(|| {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    });
    log::info!("Starting CTP Client v{}", env!("CARGO_PKG_VERSION"));
    log::info!("Initializing game client...");
    match GtkUI::new() {
        Ok(mut ui) => match ui.run() {
            Ok(UIResult::PlayGame(installation)) => {
                log::info!(
                    "Starting game: {} at {}",
                    installation.display_name,
                    installation.path.display()
                );
                if assets::prepare(&installation) {
                    match VideoPlayer::new() {
                        Ok(video_player) => match video_player.play_intro_video(&installation) {
                            VideoResult::Completed => {
                                log::info!("Intro video completed successfully");
                            }
                            VideoResult::Skipped => {
                                log::info!("Intro video skipped by user");
                            }
                            VideoResult::Failed(error) => {
                                log::warn!("Failed to play intro video: {error}");
                            }
                        },
                        Err(error) => {
                            log::warn!("Failed to initialize video player: {error}");
                        }
                    }
                    match crate::main_menu::run_sdl_main_menu(&installation) {
                        Ok(action) => {
                            log::info!("Main menu action selected: {}", action.label());
                            if action == crate::main_menu::MainMenuAction::LaunchSinglePlayer {
                                log::info!(
                                    "Single-player gameplay was launched from the existing menu window"
                                );
                            }
                        }
                        Err(error) => {
                            log::warn!("Main menu could not be displayed: {error}");
                        }
                    }
                    log::info!("Game session completed");
                } else {
                    log::info!(
                        "Asset preparation aborted for {}. Skipping launch.",
                        installation.display_name
                    );
                }
            }
            Ok(UIResult::Exit) => {
                log::info!("User requested exit");
            }
            Err(error) => {
                log::error!("UI error: {error}");
            }
        },
        Err(error) => {
            log::error!("Failed to initialize UI: {error}");
        }
    }
}
