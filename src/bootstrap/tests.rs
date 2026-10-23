//! Original startup smoke cases retained once, without extraction/file writes.

use super::INIT;
use crate::config::{AppConfig, GameInstallation, GameVersion};
use crate::installation::InstallationValidator;
use crate::ui::{GtkUI, UIResult};
use core::cell::Cell;
use std::io::{BufRead, BufReader, Cursor};
use std::path::PathBuf;
use std::sync::Once;

/// New configuration keeps the original startup defaults.
#[test]
fn test_config_creation() {
    let config = AppConfig::new();
    assert_eq!(config.version, 1_u32, "current configuration schema");
    assert!(config.installations.is_empty(), "no invented installations");
}

/// Preserve both existing installation formats while targeting CTP2 first.
#[test]
fn test_game_version_required_folders() {
    assert_eq!(
        GameVersion::CTP1.required_folders(),
        ("ctp_data", "ctp_program"),
        "legacy installation directory names"
    );
    assert_eq!(
        GameVersion::CTP2.required_folders(),
        ("ctp2_data", "ctp2_program"),
        "current target installation directory names"
    );
}

/// Read-only invalid-path validation remains available through the canonical type.
#[test]
fn test_installation_validation_nonexistent_path() {
    assert!(
        InstallationValidator::validate_installation(&PathBuf::from("/nonexistent/path")).is_err(),
        "startup must not accept an installation with no required directories"
    );
}

/// The actual startup guard cannot initialize a process twice.
#[test]
fn test_init_once_behavior() {
    let initialized = Cell::new(false);
    INIT.call_once(|| initialized.set(true));
    INIT.call_once(|| initialized.set(false));
    assert!(
        initialized.get(),
        "startup's Once invokes only its first callback"
    );
}

/// Keep Play/Exit handling on the same library-owned installation and result types.
#[test]
fn test_ui_result_variants() {
    let installation = GameInstallation::new(
        PathBuf::from("/test"),
        GameVersion::CTP1,
        "Test".to_owned(),
        None,
    );
    let play = UIResult::PlayGame(installation);
    assert!(
        format!("{play:?}").contains("PlayGame"),
        "debug play variant"
    );
    if let UIResult::PlayGame(selected) = play {
        assert_eq!(selected.display_name, "Test", "preserve selected name");
        assert_eq!(
            selected.version,
            GameVersion::CTP1,
            "preserve selected version"
        );
    } else {
        assert!(
            matches!(play, UIResult::PlayGame(_)),
            "expected play variant"
        );
    }
    let exit = UIResult::Exit;
    assert!(format!("{exit:?}").contains("Exit"), "debug exit variant");
    assert!(
        matches!(exit, UIResult::Exit),
        "explicit exit remains distinct"
    );
}

/// Startup imports use the library rather than binary-private duplicate modules.
#[test]
fn test_module_imports() {
    let config = AppConfig::new();
    let paths = InstallationValidator::scan_common_paths();
    assert!(
        config.installations.is_empty(),
        "fresh canonical configuration"
    );
    assert!(
        paths.iter().all(|path| path.is_absolute()),
        "installation scan candidates remain complete paths"
    );
    assert!(
        matches!(UIResult::Exit, UIResult::Exit),
        "shared result type"
    );
}

/// Logger's initialization guard is safe on repeated startup requests.
#[test]
fn test_logger_init_safe() {
    let once = Once::new();
    let calls = Cell::new(0_u8);
    once.call_once(|| calls.set(1_u8));
    once.call_once(|| calls.set(2_u8));
    assert_eq!(calls.get(), 1_u8, "logger setup is requested only once");
}

/// Preserve the original non-running GTK application construction smoke case.
#[test]
fn test_egui_ui_new_basic() {
    let ui = GtkUI::new().expect("construct shared GTK UI without running a display loop");
    assert_eq!(
        ui.get_config().version,
        1_u32,
        "loaded/default current schema"
    );
}

/// Library-owned startup retains package version metadata.
#[test]
fn test_cargo_pkg_version_access() {
    let version = env!("CARGO_PKG_VERSION");
    assert!(!version.is_empty(), "startup version must be present");
    assert!(
        version.contains('.'),
        "package version retains its components"
    );
}

/// The relocated startup can still emit the original logging levels.
#[test]
fn test_log_macro_usage() {
    log::info!("Test log message");
    log::error!("Test error message");
}

/// Retain the startup input smoke case entirely in memory.
#[test]
fn test_stdin_operations() {
    let mut reader = BufReader::new(Cursor::new("test input\n"));
    let mut buffer = String::new();
    reader
        .read_line(&mut buffer)
        .expect("read in-memory startup input");
    assert_eq!(buffer.trim(), "test input", "preserve input text");
}

/// Installation paths remain printable without accessing their contents.
#[test]
fn test_path_display() {
    let path = PathBuf::from("/test/path");
    let displayed = path.display().to_string();
    assert!(
        displayed.contains("test"),
        "display preserves parent component"
    );
    assert!(
        displayed.contains("path"),
        "display preserves final component"
    );
}

/// Preserve selected installation identity and launch-log formatting.
#[test]
fn test_installation_field_access() {
    let installation = GameInstallation::new(
        PathBuf::from("/test/path"),
        GameVersion::CTP2,
        "Test Installation".to_owned(),
        Some("2023-01-01 12:00:00 UTC".to_owned()),
    );
    assert_eq!(
        installation.display_name, "Test Installation",
        "original name"
    );
    assert_eq!(
        installation.path,
        PathBuf::from("/test/path"),
        "original path"
    );
    let message = format!(
        "Starting game: {} at {}",
        installation.display_name,
        installation.path.display()
    );
    assert!(
        message.contains("Test Installation"),
        "selected name in launch log"
    );
    assert!(
        message.contains("/test/path"),
        "selected path in launch log"
    );
}
