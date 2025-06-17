//! Tests for the configuration module.

use super::*;
use std::env;
use std::fs;
use tempfile::TempDir;

/// Creates a temporary directory for testing configuration file operations.
fn setup_temp_config_dir() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir
        .path()
        .join("ctp-client")
        .join("ctp-client-config.toml");
    (temp_dir, config_path)
}

#[test]
fn test_app_config_new() {
    let config = AppConfig::new();
    assert_eq!(config.version, AppConfig::CONFIG_VERSION);
    assert!(config.installations.is_empty());
}

#[test]
fn test_app_config_default() {
    let config = AppConfig::default();
    assert_eq!(config.version, 0); // Default should be 0, not CONFIG_VERSION
    assert!(config.installations.is_empty());
}

#[test]
fn test_game_version_required_folders() {
    let (data, program) = GameVersion::CTP1.required_folders();
    assert_eq!(data, "ctp_data");
    assert_eq!(program, "ctp_program");

    let (data2, program2) = GameVersion::CTP2.required_folders();
    assert_eq!(data2, "ctp2_data");
    assert_eq!(program2, "ctp2_program");
}

#[test]
fn test_game_version_equality() {
    assert_eq!(GameVersion::CTP1, GameVersion::CTP1);
    assert_eq!(GameVersion::CTP2, GameVersion::CTP2);
    assert_ne!(GameVersion::CTP1, GameVersion::CTP2);
}

#[test]
fn test_game_installation_creation() {
    let path = PathBuf::from("/test/path");
    let installation = GameInstallation::new(
        path.clone(),
        GameVersion::CTP1,
        "Test Installation".to_string(),
        Some("2023-01-01 12:00:00 UTC".to_string()),
    );

    assert_eq!(installation.path, path);
    assert_eq!(installation.version, GameVersion::CTP1);
    assert_eq!(installation.display_name, "Test Installation");
    assert!(installation.validated_at.is_some());
}

#[test]
fn test_add_installation() {
    let mut config = AppConfig::new();
    let installation = GameInstallation::new(
        PathBuf::from("/test/path"),
        GameVersion::CTP1,
        "Test".to_string(),
        None,
    );

    config.add_installation("test_id".to_string(), installation.clone());
    assert_eq!(config.installations.len(), 1);
    assert!(config.installations.contains_key("test_id"));
    assert_eq!(config.installations["test_id"].display_name, "Test");
}

#[test]
fn test_remove_installation() {
    let mut config = AppConfig::new();
    let installation = GameInstallation::new(
        PathBuf::from("/test/path"),
        GameVersion::CTP1,
        "Test".to_string(),
        None,
    );

    config.add_installation("test_id".to_string(), installation);
    assert_eq!(config.installations.len(), 1);

    config.remove_installation("test_id");
    assert_eq!(config.installations.len(), 0);
    assert!(!config.installations.contains_key("test_id"));
}

#[test]
fn test_remove_nonexistent_installation() {
    let mut config = AppConfig::new();
    config.remove_installation("nonexistent");
    assert_eq!(config.installations.len(), 0);
}

#[test]
fn test_get_installations() {
    let mut config = AppConfig::new();
    let installation = GameInstallation::new(
        PathBuf::from("/test/path"),
        GameVersion::CTP2,
        "Test".to_string(),
        None,
    );

    config.add_installation("test_id".to_string(), installation);
    let installations = config.get_installations();
    assert_eq!(installations.len(), 1);
    assert!(installations.contains_key("test_id"));
}

#[test]
fn test_save_and_load_config() {
    let (temp_dir, config_path) = setup_temp_config_dir();

    // Set a temporary config directory
    let original_config_dir = env::var("XDG_CONFIG_HOME").ok();
    env::set_var("XDG_CONFIG_HOME", temp_dir.path());

    // Create config with data
    let mut config = AppConfig::new();
    let installation = GameInstallation::new(
        PathBuf::from("/test/path"),
        GameVersion::CTP1,
        "Test Installation".to_string(),
        Some("2023-01-01 12:00:00 UTC".to_string()),
    );
    config.add_installation("test_id".to_string(), installation);

    // Save config
    assert!(config.save().is_ok());
    assert!(config_path.exists());

    // Load config
    let loaded_config = AppConfig::load().unwrap();
    assert_eq!(loaded_config.version, AppConfig::CONFIG_VERSION);
    assert_eq!(loaded_config.installations.len(), 1);
    assert!(loaded_config.installations.contains_key("test_id"));
    assert_eq!(
        loaded_config.installations["test_id"].display_name,
        "Test Installation"
    );

    // Restore original config dir
    match original_config_dir {
        Some(dir) => env::set_var("XDG_CONFIG_HOME", dir),
        None => env::remove_var("XDG_CONFIG_HOME"),
    }
}

#[test]
fn test_load_nonexistent_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let original_config_dir = env::var("XDG_CONFIG_HOME").ok();
    env::set_var("XDG_CONFIG_HOME", temp_dir.path());

    let config = AppConfig::load().unwrap();
    assert_eq!(config.version, AppConfig::CONFIG_VERSION);
    assert!(config.installations.is_empty());

    // Restore original config dir
    match original_config_dir {
        Some(dir) => env::set_var("XDG_CONFIG_HOME", dir),
        None => env::remove_var("XDG_CONFIG_HOME"),
    }
}

#[test]
fn test_load_invalid_toml() {
    let (temp_dir, config_path) = setup_temp_config_dir();
    let original_config_dir = env::var("XDG_CONFIG_HOME").ok();
    env::set_var("XDG_CONFIG_HOME", temp_dir.path());

    // Create invalid TOML file
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    fs::write(&config_path, "invalid toml content [[[").unwrap();

    let result = AppConfig::load();
    assert!(result.is_err());

    // Restore original config dir
    match original_config_dir {
        Some(dir) => env::set_var("XDG_CONFIG_HOME", dir),
        None => env::remove_var("XDG_CONFIG_HOME"),
    }
}

#[test]
fn test_load_outdated_config_version() {
    let (temp_dir, config_path) = setup_temp_config_dir();
    let original_config_dir = env::var("XDG_CONFIG_HOME").ok();
    env::set_var("XDG_CONFIG_HOME", temp_dir.path());

    // Create config with old version
    let old_config_content = r#"
version = 0
[installations]
"#;
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    fs::write(&config_path, old_config_content).unwrap();

    let config = AppConfig::load().unwrap();
    assert_eq!(config.version, AppConfig::CONFIG_VERSION); // Should be reset to current version
    assert!(config.installations.is_empty());

    // Restore original config dir
    match original_config_dir {
        Some(dir) => env::set_var("XDG_CONFIG_HOME", dir),
        None => env::remove_var("XDG_CONFIG_HOME"),
    }
}

#[test]
fn test_save_config_creates_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let original_config_dir = env::var("XDG_CONFIG_HOME").ok();
    env::set_var("XDG_CONFIG_HOME", temp_dir.path());

    let config = AppConfig::new();
    assert!(config.save().is_ok());

    let config_path = temp_dir
        .path()
        .join("ctp-client")
        .join("ctp-client-config.toml");
    assert!(config_path.exists());
    assert!(config_path.parent().unwrap().is_dir());

    // Restore original config dir
    match original_config_dir {
        Some(dir) => env::set_var("XDG_CONFIG_HOME", dir),
        None => env::remove_var("XDG_CONFIG_HOME"),
    }
}

#[test]
fn test_config_serialization_deserialization() {
    let mut config = AppConfig::new();
    let installation = GameInstallation::new(
        PathBuf::from("/test/path"),
        GameVersion::CTP2,
        "Test CTP2".to_string(),
        Some("2023-12-25 10:30:45 UTC".to_string()),
    );
    config.add_installation("ctp2_test".to_string(), installation);

    // Serialize to TOML
    let toml_string = toml::to_string(&config).unwrap();
    assert!(toml_string.contains("version = 1"));
    assert!(toml_string.contains("CTP2"));
    assert!(toml_string.contains("Test CTP2"));

    // Deserialize from TOML
    let deserialized: AppConfig = toml::from_str(&toml_string).unwrap();
    assert_eq!(deserialized.version, config.version);
    assert_eq!(deserialized.installations.len(), 1);
    assert_eq!(
        deserialized.installations["ctp2_test"].display_name,
        "Test CTP2"
    );
    assert_eq!(
        deserialized.installations["ctp2_test"].version,
        GameVersion::CTP2
    );
}

#[test]
fn test_multiple_installations() {
    let mut config = AppConfig::new();

    let installation1 = GameInstallation::new(
        PathBuf::from("/path/to/ctp1"),
        GameVersion::CTP1,
        "CTP1 Installation".to_string(),
        Some("2023-01-01 12:00:00 UTC".to_string()),
    );

    let installation2 = GameInstallation::new(
        PathBuf::from("/path/to/ctp2"),
        GameVersion::CTP2,
        "CTP2 Installation".to_string(),
        Some("2023-01-02 12:00:00 UTC".to_string()),
    );

    config.add_installation("ctp1_id".to_string(), installation1);
    config.add_installation("ctp2_id".to_string(), installation2);

    assert_eq!(config.installations.len(), 2);
    assert!(config.installations.contains_key("ctp1_id"));
    assert!(config.installations.contains_key("ctp2_id"));
    assert_eq!(config.installations["ctp1_id"].version, GameVersion::CTP1);
    assert_eq!(config.installations["ctp2_id"].version, GameVersion::CTP2);
}

#[test]
fn test_overwrite_installation() {
    let mut config = AppConfig::new();

    let installation1 = GameInstallation::new(
        PathBuf::from("/path/old"),
        GameVersion::CTP1,
        "Old Installation".to_string(),
        None,
    );

    let installation2 = GameInstallation::new(
        PathBuf::from("/path/new"),
        GameVersion::CTP2,
        "New Installation".to_string(),
        Some("2023-01-01 12:00:00 UTC".to_string()),
    );

    config.add_installation("same_id".to_string(), installation1);
    assert_eq!(config.installations.len(), 1);
    assert_eq!(
        config.installations["same_id"].display_name,
        "Old Installation"
    );

    config.add_installation("same_id".to_string(), installation2);
    assert_eq!(config.installations.len(), 1);
    assert_eq!(
        config.installations["same_id"].display_name,
        "New Installation"
    );
    assert_eq!(config.installations["same_id"].version, GameVersion::CTP2);
}
