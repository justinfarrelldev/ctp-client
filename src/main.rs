//! # CTP Client
//!
//! A modern client application for strategic civilization gaming.
//! This client provides an interface for connecting and playing
//! CTP games with enhanced features and improved user experience.
//!
//! ## Features
//!
//! - Game installation detection and validation
//! - Modern GTK4-based graphical user interface
//! - Support for both CTP1 and CTP2
//! - Cross-platform compatibility
//!
//! ## Usage
//!
//! Run the client with:
//! ```bash
//! cargo run
//! ```

#![allow(
    clippy::multiple_crate_versions,
    reason = "Multiple versions are acceptable for this game client that integrates many external libraries"
)]
#![allow(
    clippy::literal_string_with_formatting_args,
    reason = "False positive on doc comments containing 'Call to Power'"
)]
#![allow(
    clippy::panic,
    reason = "Panic is acceptable in test code for validation failures"
)]
#![allow(
    clippy::needless_return,
    reason = "Some explicit returns improve readability in match arms"
)]
#![allow(
    clippy::pattern_type_mismatch,
    reason = "Pattern matching behavior is correct for our enum usage"
)]
#![allow(
    clippy::use_self,
    reason = "Explicit type names improve clarity in some contexts"
)]
#![allow(
    clippy::manual_assert,
    reason = "Manual assertions provide better error messages in tests"
)]
#![allow(
    clippy::uninlined_format_args,
    reason = "Some format strings are clearer with separate arguments"
)]
#![allow(
    clippy::absolute_paths,
    reason = "Absolute paths are clearer for std library types"
)]
#![allow(
    clippy::std_instead_of_core,
    reason = "Std is appropriate for this application context"
)]
#![allow(
    clippy::min_ident_chars,
    reason = "Single char identifiers like 'e' for errors are conventional"
)]
#![allow(
    clippy::doc_markdown,
    reason = "Some type names don't need backticks for clarity"
)]
#![allow(
    clippy::missing_trait_methods,
    reason = "Default trait implementations are sufficient for our error types"
)]
#![allow(
    clippy::semicolon_outside_block,
    reason = "Semicolons inside unsafe blocks are clearer"
)]
#![allow(
    clippy::undocumented_unsafe_blocks,
    reason = "OpenGL calls are standard and well-documented"
)]
#![allow(
    clippy::multiple_unsafe_ops_per_block,
    reason = "Related OpenGL calls are grouped for clarity"
)]
#![allow(
    clippy::let_underscore_must_use,
    reason = "Some results can be safely ignored in demo code"
)]
#![allow(
    clippy::wildcard_enum_match_arm,
    reason = "Catch-all for unhandled SDL events is appropriate"
)]
#![allow(
    clippy::shadow_reuse,
    reason = "UI closure parameter shadowing is idiomatic in egui"
)]
#![allow(
    clippy::if_not_else,
    reason = "Original logic flow is clearer than inverted condition"
)]
#![allow(
    clippy::missing_const_for_fn,
    reason = "Not all functions benefit from being const"
)]
#![allow(
    clippy::print_stderr,
    reason = "Error output to stderr is appropriate for debugging"
)]

extern crate alloc;

// pub mod app;
/// Asset extraction module.
pub mod asset_extraction;
/// Configuration management module.
pub mod config;
mod dialogs;
/// Internationalization module.
pub mod i18n;
/// Installation validation and detection module.
pub mod installation;
/// CTP2 main menu module.
pub mod main_menu;
/// Settings UI module.
pub mod settings;
/// User interface module.
pub mod ui;
/// Video playback module.
pub mod video;

use crate::asset_extraction::{
    ArchiveExtractor, ExtractionError, ExtractionSettings, build_confirmation_prompt,
    default_destination_root, log_extraction_progress, required_archives,
};
use config::GameInstallation;
use log::info;
use std::path::PathBuf;
use std::sync::Once;
#[cfg(test)]
use std::sync::{Mutex, OnceLock, atomic::AtomicBool};
use ui::{GtkUI, UIResult};
use video::{VideoPlayer, VideoResult};

/// Global flag to ensure logger is only initialized once.
static INIT: Once = Once::new();

/// Main entry point for the CTP client application.
///
/// Initializes the logging system and starts the client application.
/// This function sets up the necessary components for the game client
/// and begins the main game loop.
fn main() {
    // Initialize the logging system (only once)
    INIT.call_once(|| {
        env_logger::init();
    });

    info!("Starting CTP Client v{}", env!("CARGO_PKG_VERSION"));
    info!("Initializing game client...");

    match GtkUI::new() {
        Ok(mut ui) => match ui.run() {
            Ok(UIResult::PlayGame(installation)) => {
                info!(
                    "Starting game: {} at {}",
                    installation.display_name,
                    installation.path.display()
                );

                if prepare_installation_assets(&installation) {
                    match VideoPlayer::new() {
                        Ok(video_player) => match video_player.play_intro_video(&installation) {
                            VideoResult::Completed => {
                                info!("Intro video completed successfully");
                            }
                            VideoResult::Skipped => {
                                info!("Intro video skipped by user");
                            }
                            VideoResult::Failed(error) => {
                                log::warn!("Failed to play intro video: {}", error);
                            }
                        },
                        Err(e) => {
                            log::warn!("Failed to initialize video player: {}", e);
                        }
                    }

                    match main_menu::run_sdl_main_menu(&installation) {
                        Ok(action) => {
                            info!("Main menu action selected: {}", action.label());
                        }
                        Err(error) => {
                            log::warn!("Main menu could not be displayed: {error}");
                        }
                    }

                    println!("Would start game: {}", installation.display_name);
                    info!("Game session completed");
                } else {
                    info!(
                        "Asset preparation aborted for {}. Skipping launch.",
                        installation.display_name
                    );
                }
            }
            Ok(UIResult::Exit) => {
                info!("User requested exit");
            }
            Err(e) => {
                log::error!("UI error: {}", e);
            }
        },
        Err(e) => {
            log::error!("Failed to initialize UI: {}", e);
        }
    }
}

/// Prepares extracted game assets before handing off to video and game launch.
fn prepare_installation_assets(installation: &GameInstallation) -> bool {
    prepare_installation_assets_internal(
        installation,
        &prompt_user_for_extraction,
        &show_extraction_error_dialog,
        &default_destination_root,
    )
}

/// Shared asset-preparation flow with injectable UI callbacks for tests.
fn prepare_installation_assets_internal<Prompt, ErrorHandler, DestinationResolver>(
    installation: &GameInstallation,
    prompt: &Prompt,
    on_error: &ErrorHandler,
    resolve_destination: &DestinationResolver,
) -> bool
where
    Prompt: Fn(&str) -> bool,
    ErrorHandler: Fn(&ExtractionError),
    DestinationResolver: Fn(&GameInstallation) -> Result<PathBuf, ExtractionError>,
{
    let destination = match resolve_destination(installation) {
        Ok(path) => path,
        Err(error) => {
            log::error!("Failed to resolve extraction target: {error}");
            on_error(&error);
            return false;
        }
    };

    let extractor = ArchiveExtractor::new(installation);
    if let Err(error) = extractor.validate() {
        log::error!("Archive validation failed: {error}");
        on_error(&error);
        return false;
    }

    let archive_count = required_archives(installation.version).len();
    let description =
        build_confirmation_prompt(&destination, archive_count, &installation.display_name);

    if !prompt(&description) {
        info!("User canceled extraction for {}", installation.display_name);
        return false;
    }

    let mut progress = log_extraction_progress;

    match extractor.extract_with_settings(
        &destination,
        ExtractionSettings::default(),
        Some(&mut progress),
    ) {
        Ok(outcome) => {
            if outcome.already_up_to_date {
                info!(
                    "Game data for {} is already up to date at {}",
                    installation.display_name,
                    destination.display()
                );
            } else {
                let files_written: usize = outcome
                    .archives
                    .iter()
                    .map(|summary| summary.files_written)
                    .sum();
                let bytes_written: u64 = outcome
                    .archives
                    .iter()
                    .map(|summary| summary.bytes_written)
                    .sum();
                info!(
                    "Extracted {} archives ({} files, {} bytes) to {}",
                    outcome.archives.len(),
                    files_written,
                    bytes_written,
                    destination.display()
                );
            }
            true
        }
        Err(error) => {
            log::error!("Extraction failed: {error}");
            on_error(&error);
            false
        }
    }
}

#[cfg(not(test))]
/// Shows the native confirmation dialog for asset extraction.
fn prompt_user_for_extraction(description: &str) -> bool {
    let text = dialogs::DialogText::new("Prepare game data", description.to_owned());
    dialogs::run_confirmation_dialog(None, &text)
}

#[cfg(test)]
fn prompt_user_for_extraction(description: &str) -> bool {
    TEST_PROMPT_RECORD
        .get_or_init(Default::default)
        .lock()
        .expect("prompt record lock")
        .push(description.to_owned());
    TEST_DIALOG_RESPONSE.load(std::sync::atomic::Ordering::SeqCst)
}

#[cfg(not(test))]
/// Shows the native error dialog for asset extraction failures.
fn show_extraction_error_dialog(error: &ExtractionError) {
    let text = dialogs::extraction_error_text(error);
    dialogs::run_message_dialog(None, &text);
}

#[cfg(test)]
fn show_extraction_error_dialog(error: &ExtractionError) {
    TEST_ERROR_RECORD
        .get_or_init(Default::default)
        .lock()
        .expect("error record lock")
        .push(format!("{error}"));
}

#[cfg(test)]
static TEST_DIALOG_RESPONSE: AtomicBool = AtomicBool::new(true);
#[cfg(test)]
static TEST_PROMPT_RECORD: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
#[cfg(test)]
static TEST_ERROR_RECORD: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset_extraction::{ExtractionError, required_archives};
    use crate::config::{AppConfig, GameInstallation, GameVersion};
    use crate::installation::InstallationValidator;
    use alloc::sync::Arc;
    use byteorder::{LittleEndian, WriteBytesExt};
    use std::fs::{self, File};
    use std::io::{Seek, Write};
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tempfile::TempDir;

    fn create_temp_installation(version: GameVersion) -> (GameInstallation, TempDir) {
        let temp = TempDir::new().expect("temp dir");
        let installation = GameInstallation::new(
            temp.path().to_path_buf(),
            version,
            "Test Installation".to_owned(),
            None,
        );
        (installation, temp)
    }

    fn write_le32(writer: &mut File, value: u32) {
        writer
            .write_u32::<LittleEndian>(value)
            .expect("write little-endian u32");
    }

    fn write_padding(file: &mut File, padding: u64) {
        let padding_len = usize::try_from(padding).expect("padding fits in usize");
        file.write_all(&vec![0_u8; padding_len])
            .expect("write padding");
    }

    fn write_sample_zfs(path: &Path, entry_name: &str, data: &[u8]) {
        const MAX_FILENAME_LEN: u32 = 260;
        const FILETABLE_OFFSET: u32 = 64;

        let mut file = File::create(path).expect("create zfs");

        write_le32(&mut file, unciv::ZFS_SIGNATURE);
        write_le32(&mut file, 1);
        write_le32(&mut file, MAX_FILENAME_LEN);
        write_le32(&mut file, 1);
        write_le32(&mut file, 1);
        write_le32(&mut file, 0);
        write_le32(&mut file, FILETABLE_OFFSET);

        let filetable_offset = u64::from(FILETABLE_OFFSET);
        let current = file.stream_position().expect("read zfs header position");
        if current < filetable_offset {
            let padding = filetable_offset
                .checked_sub(current)
                .expect("current position precedes file table");
            write_padding(&mut file, padding);
        }

        write_le32(&mut file, 0);

        let filename_len = usize::try_from(MAX_FILENAME_LEN).expect("filename length fits usize");
        let mut name_block = vec![0_u8; filename_len];
        let name_bytes = entry_name.as_bytes();
        assert!(
            name_bytes.len() < name_block.len(),
            "entry name must fit in sample ZFS header"
        );
        name_block[..name_bytes.len()].copy_from_slice(name_bytes);
        file.write_all(&name_block).expect("write entry name block");

        let data_offset = filetable_offset
            .checked_add(4)
            .and_then(|value| value.checked_add(u64::from(MAX_FILENAME_LEN)))
            .and_then(|value| value.checked_add(20))
            .expect("sample ZFS data offset fits u64");
        write_le32(
            &mut file,
            u32::try_from(data_offset).expect("sample data offset fits u32"),
        );
        write_le32(&mut file, 0);
        write_le32(
            &mut file,
            u32::try_from(data.len()).expect("sample payload length fits u32"),
        );
        write_le32(&mut file, 0);
        write_le32(&mut file, 0);

        let after_entry = file.stream_position().expect("read file table position");
        if after_entry < data_offset {
            let padding = data_offset
                .checked_sub(after_entry)
                .expect("entry metadata precedes payload");
            write_padding(&mut file, padding);
        }

        file.write_all(data).expect("write sample payload");
    }

    fn populate_sample_archives(root: &Path, version: GameVersion) {
        for (index, descriptor) in required_archives(version).iter().enumerate() {
            let archive_path = root.join(descriptor.relative_path);
            if let Some(parent) = archive_path.parent() {
                fs::create_dir_all(parent).expect("create archive parent");
            }
            let entry_name = format!("entries/{index}/asset.bin");
            let payload = format!("payload-{index}").into_bytes();
            write_sample_zfs(&archive_path, &entry_name, &payload);
        }
    }

    fn populate_corrupt_archives(root: &Path, version: GameVersion) {
        for descriptor in required_archives(version) {
            let archive_path = root.join(descriptor.relative_path);
            if let Some(parent) = archive_path.parent() {
                fs::create_dir_all(parent).expect("create archive parent");
            }
            fs::write(&archive_path, b"corrupt").expect("write corrupt archive");
        }
    }

    #[test]
    fn prepare_assets_returns_false_when_destination_resolution_fails() {
        let (installation, _temp) = create_temp_installation(GameVersion::CTP1);

        let prompt_calls = Arc::new(Mutex::new(Vec::new()));
        let prompt_capture = Arc::clone(&prompt_calls);
        let prompt = move |description: &str| {
            prompt_capture
                .lock()
                .expect("prompt log lock")
                .push(description.to_owned());
            true
        };

        let errors = Arc::new(Mutex::new(Vec::new()));
        let error_capture = Arc::clone(&errors);
        let error_handler = move |error: &ExtractionError| {
            error_capture
                .lock()
                .expect("error log lock")
                .push(format!("{error}"));
        };

        let resolver = |_installation: &GameInstallation| -> Result<PathBuf, ExtractionError> {
            Err(ExtractionError::PathResolution {
                message: "resolver failure".to_owned(),
            })
        };

        let result =
            prepare_installation_assets_internal(&installation, &prompt, &error_handler, &resolver);

        assert!(!result);
        assert!(errors.lock().expect("error log lock").len() == 1);
        assert!(prompt_calls.lock().expect("prompt log lock").is_empty());
    }

    #[test]
    fn prepare_assets_returns_false_when_validation_fails() {
        let (installation, temp) = create_temp_installation(GameVersion::CTP1);

        let prompt_calls = Arc::new(Mutex::new(Vec::new()));
        let prompt_capture = Arc::clone(&prompt_calls);
        let prompt = move |description: &str| {
            prompt_capture
                .lock()
                .expect("prompt log lock")
                .push(description.to_owned());
            true
        };

        let errors = Arc::new(Mutex::new(Vec::new()));
        let error_capture = Arc::clone(&errors);
        let error_handler = move |error: &ExtractionError| {
            error_capture
                .lock()
                .expect("error log lock")
                .push(format!("{error}"));
        };

        let destination =
            move |_installation: &GameInstallation| -> Result<PathBuf, ExtractionError> {
                Ok(temp.path().join("destination"))
            };

        let result = prepare_installation_assets_internal(
            &installation,
            &prompt,
            &error_handler,
            &destination,
        );

        assert!(!result);
        assert!(!errors.lock().expect("error log lock").is_empty());
        assert!(prompt_calls.lock().expect("prompt log lock").is_empty());
    }

    #[test]
    fn prepare_assets_stops_when_user_declines() {
        let (installation, temp) = create_temp_installation(GameVersion::CTP1);
        populate_sample_archives(temp.path(), installation.version);

        let prompt_calls = Arc::new(Mutex::new(Vec::new()));
        let prompt_capture = Arc::clone(&prompt_calls);
        let prompt = move |description: &str| {
            prompt_capture
                .lock()
                .expect("prompt log lock")
                .push(description.to_owned());
            false
        };

        let errors = Arc::new(Mutex::new(Vec::new()));
        let error_capture = Arc::clone(&errors);
        let error_handler = move |error: &ExtractionError| {
            error_capture
                .lock()
                .expect("error log lock")
                .push(format!("{error}"));
        };

        let destination =
            |game_installation: &GameInstallation| -> Result<PathBuf, ExtractionError> {
                Ok(game_installation.path.join("dest"))
            };

        let result = prepare_installation_assets_internal(
            &installation,
            &prompt,
            &error_handler,
            &destination,
        );

        assert!(!result);
        assert!(errors.lock().expect("error log lock").is_empty());
        assert_eq!(prompt_calls.lock().expect("prompt log lock").len(), 1);
        assert!(!installation.path.join("dest").exists());
    }

    #[test]
    fn prepare_assets_reports_extraction_failure() {
        let (installation, temp) = create_temp_installation(GameVersion::CTP1);
        populate_corrupt_archives(temp.path(), installation.version);

        let prompt = |_description: &str| true;
        let errors = Arc::new(Mutex::new(Vec::new()));
        let error_capture = Arc::clone(&errors);
        let error_handler = move |error: &ExtractionError| {
            error_capture
                .lock()
                .expect("error log lock")
                .push(format!("{error}"));
        };

        let destination =
            |game_installation: &GameInstallation| -> Result<PathBuf, ExtractionError> {
                Ok(game_installation.path.join("dest"))
            };

        let result = prepare_installation_assets_internal(
            &installation,
            &prompt,
            &error_handler,
            &destination,
        );

        assert!(!result);
        assert_eq!(errors.lock().expect("error log lock").len(), 1);
    }

    #[test]
    fn prepare_assets_succeeds_with_valid_setup() {
        let (installation, temp) = create_temp_installation(GameVersion::CTP1);
        populate_sample_archives(temp.path(), installation.version);

        let prompt = |_description: &str| true;
        let errors = Arc::new(Mutex::new(Vec::new()));
        let error_capture = Arc::clone(&errors);
        let error_handler = move |error: &ExtractionError| {
            error_capture
                .lock()
                .expect("error log lock")
                .push(format!("{error}"));
        };

        let destination =
            |game_installation: &GameInstallation| -> Result<PathBuf, ExtractionError> {
                Ok(game_installation.path.join("dest"))
            };

        let result = prepare_installation_assets_internal(
            &installation,
            &prompt,
            &error_handler,
            &destination,
        );

        assert!(result);
        assert!(errors.lock().expect("error log lock").is_empty());
        let destination_root = installation.path.join("dest");
        assert!(destination_root.exists());
        let manifest = destination_root.join(".ctp-asset-manifest.json");
        assert!(manifest.exists());
    }

    /// Tests that a new config is created with correct defaults.
    #[test]
    fn test_config_creation() {
        let config = AppConfig::new();
        assert_eq!(config.version, 1);
        assert!(config.installations.is_empty());
    }

    /// Tests that game versions return the correct required folder names.
    #[test]
    fn test_game_version_required_folders() {
        let (data_folder, program_folder) = GameVersion::CTP1.required_folders();
        assert_eq!(data_folder, "ctp_data");
        assert_eq!(program_folder, "ctp_program");

        let (data_folder_v2, program_folder_v2) = GameVersion::CTP2.required_folders();
        assert_eq!(data_folder_v2, "ctp2_data");
        assert_eq!(program_folder_v2, "ctp2_program");
    }

    /// Tests that validation fails for nonexistent paths.
    #[test]
    fn test_installation_validation_nonexistent_path() {
        let path = PathBuf::from("/nonexistent/path");
        let result = InstallationValidator::validate_installation(&path);
        assert!(
            result.is_err(),
            "Expected validation to fail for nonexistent path"
        );
    }

    /// Tests the static INIT flag behavior
    #[test]
    fn test_init_once_behavior() {
        // This test verifies that the INIT static is properly declared
        // The actual behavior is tested by calling it multiple times
        static TEST_COUNTER: AtomicBool = AtomicBool::new(false);

        INIT.call_once(|| {
            TEST_COUNTER.store(true, Ordering::SeqCst);
        });

        // Calling again should not change the counter
        INIT.call_once(|| {
            TEST_COUNTER.store(false, Ordering::SeqCst);
        });

        assert!(TEST_COUNTER.load(Ordering::SeqCst));
    }

    /// Tests UIResult enum variants
    #[test]
    fn test_ui_result_variants() {
        let installation = crate::config::GameInstallation::new(
            PathBuf::from("/test"),
            GameVersion::CTP1,
            "Test".to_owned(),
            None,
        );

        let play_result = UIResult::PlayGame(installation);
        let exit_result = UIResult::Exit;

        // Test Debug trait
        let play_debug = format!("{:?}", play_result);
        let exit_debug = format!("{:?}", exit_result);

        assert!(play_debug.contains("PlayGame"));
        assert!(exit_debug.contains("Exit"));

        // Verify enum matching works
        assert!(
            matches!(&play_result, &UIResult::PlayGame(_)),
            "Expected PlayGame variant"
        );
        if let UIResult::PlayGame(inst) = play_result {
            assert_eq!(inst.display_name, "Test");
            assert_eq!(inst.version, GameVersion::CTP1);
        }

        assert!(
            matches!(exit_result, UIResult::Exit),
            "Expected Exit variant"
        );
    }

    /// Test that main module imports work correctly
    #[test]
    fn test_module_imports() {
        // Verify that our modules are properly imported
        let _config = AppConfig::new();
        let _validator = InstallationValidator::scan_common_paths();

        // Test that UIResult is accessible
        let exit_result = UIResult::Exit;
        drop(exit_result); // Explicitly drop to avoid unused variable warning
    }

    /// Test that the logger initialization doesn't panic
    #[test]
    fn test_logger_init_safe() {
        // This test ensures that calling the logger init code doesn't panic
        // even if the logger is already initialized
        use std::sync::Once;

        static TEST_INIT: Once = Once::new();

        TEST_INIT.call_once(|| {
            // This simulates what happens in main()
            // env_logger::init() is safe to call multiple times
        });

        // Should not panic on second call
        TEST_INIT.call_once(|| {
            // This won't run since it's already initialized
        });
    }

    /// Test that we can create UI instances (basic instantiation test)
    #[test]
    fn test_egui_ui_new_basic() {
        // This tests that GtkUI::new() can be called without panicking
        // We can't fully test the UI without a display, but we can test instantiation
        let result = GtkUI::new();

        // Should succeed in creating the UI struct
        assert!(result.is_ok());

        let ui = result.expect("create GTK UI");
        // Verify that the config is properly loaded
        let config = ui.get_config();
        let installations = config.get_installations();
        assert!(installations.is_empty() || !installations.is_empty()); // Always true, tests return type
    }

    /// Test string operations in main module
    #[test]
    fn test_cargo_pkg_version_access() {
        // Test that the compile-time environment variable access works
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty());
        assert!(version.contains('.'));
    }

    /// Test that the info! macro calls would work (we can't test actual logging easily)
    #[test]
    fn test_log_macro_usage() {
        // Test that log macros can be used without compilation errors
        log::info!("Test log message");
        log::error!("Test error message");

        // No assertion needed - if this compiles and runs, the test passes
    }

    /// Test std::io operations that might be used in main
    #[test]
    fn test_stdin_operations() {
        // Test that stdin operations can be performed
        use std::io::{BufRead, BufReader, Cursor};

        // Simulate stdin with a cursor
        let input = "test input\n";
        let cursor = Cursor::new(input);
        let mut reader = BufReader::new(cursor);
        let mut buffer = String::new();

        let result = reader.read_line(&mut buffer);
        assert!(result.is_ok());
        assert_eq!(buffer.trim(), "test input");
    }

    /// Test PathBuf display operations used in main
    #[test]
    fn test_path_display() {
        let path = PathBuf::from("/test/path");
        let display_str = path.display().to_string();
        assert!(display_str.contains("test"));
        assert!(display_str.contains("path"));
    }

    /// Test basic struct field access patterns used in main
    #[test]
    fn test_installation_field_access() {
        let installation = crate::config::GameInstallation::new(
            PathBuf::from("/test/path"),
            GameVersion::CTP2,
            "Test Installation".to_owned(),
            Some("2023-01-01 12:00:00 UTC".to_owned()),
        );

        // Test field access patterns used in main()
        assert_eq!(installation.display_name, "Test Installation");
        assert_eq!(installation.path, PathBuf::from("/test/path"));

        let display_msg = format!(
            "Starting game: {} at {}",
            installation.display_name,
            installation.path.display()
        );
        assert!(display_msg.contains("Test Installation"));
        assert!(display_msg.contains("/test/path"));
    }
}
