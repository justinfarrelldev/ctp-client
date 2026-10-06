//! Configuration management for the CTP client.
//!
//! This module handles application configuration, including game installation
//! registration, persistence, and management.

use crate::i18n::{Language, Translator};
use core::error::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Represents a Call to Power game installation.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct GameInstallation {
    /// The filesystem path to the installation directory.
    pub path: PathBuf,
    /// The version of Call to Power (CTP1 or CTP2).
    pub version: GameVersion,
    /// Human-readable name for this installation.
    pub display_name: String,
    /// Timestamp when this installation was last validated.
    pub validated_at: Option<String>,
}

/// Supported Call to Power game versions.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum GameVersion {
    /// Call to Power (original).
    CTP1,
    /// Call to Power II.
    CTP2,
}

/// Implementation of `GameInstallation` functionality.
impl GameInstallation {
    /// Creates a new `GameInstallation`.
    ///
    /// # Arguments
    ///
    /// * `path` - The filesystem path to the installation directory
    /// * `version` - The version of Call to Power (CTP1 or CTP2)
    /// * `display_name` - Human-readable name for this installation
    /// * `validated_at` - Timestamp when this installation was last validated
    ///
    /// # Returns
    ///
    /// A new `GameInstallation` instance
    #[must_use]
    pub const fn new(
        path: PathBuf,
        version: GameVersion,
        display_name: String,
        validated_at: Option<String>,
    ) -> Self {
        Self {
            path,
            version,
            display_name,
            validated_at,
        }
    }
}

/// Implementation of `GameVersion` functionality.
impl GameVersion {
    /// Returns the required folder names for this game version.
    ///
    /// # Returns
    ///
    /// A tuple of (`data_folder_name`, `program_folder_name`) required for this version
    #[must_use]
    pub const fn required_folders(&self) -> (&'static str, &'static str) {
        match *self {
            Self::CTP1 => ("ctp_data", "ctp_program"),
            Self::CTP2 => ("ctp2_data", "ctp2_program"),
        }
    }
}

/// Main application configuration structure.
#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AppConfig {
    /// Configuration version for migration purposes.
    pub version: u32,
    /// Map of installation ID to `GameInstallation`.
    #[serde(default)]
    pub installations: HashMap<String, GameInstallation>,
    /// User's preferred language setting.
    pub language: Language,
}

/// Implementation of `AppConfig` functionality.
impl AppConfig {
    /// Current configuration version.
    pub const CONFIG_VERSION: u32 = 1;
    /// Name of the configuration file.
    pub const CONFIG_FILE_NAME: &'static str = "ctp-client-config.toml";

    /// Creates a new default configuration.
    ///
    /// # Returns
    ///
    /// A new `AppConfig` with default values
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: Self::CONFIG_VERSION,
            installations: HashMap::new(),
            language: Translator::detect_system_language(),
        }
    }

    /// Loads configuration from disk.
    ///
    /// If the configuration file doesn't exist, returns a new default configuration.
    /// If the configuration version is outdated, returns a new default configuration.
    ///
    /// # Returns
    ///
    /// * `Ok(AppConfig)` - The loaded or default configuration
    /// * `Err(Box<dyn std::error::Error>)` - Error if loading fails
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The config directory cannot be determined
    /// - The config file cannot be read
    /// - The config file contains invalid TOML
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&config_path)?;
        let mut config: Self = toml::from_str(&content)?;

        if config.version != Self::CONFIG_VERSION {
            log::warn!(
                "Config version mismatch. Expected {}, found {}. Using default config.",
                Self::CONFIG_VERSION,
                config.version
            );
            config = Self::new();
        }

        Ok(config)
    }

    /// Saves the configuration to disk.
    ///
    /// Creates the configuration directory if it doesn't exist.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Configuration saved successfully
    /// * `Err(Box<dyn std::error::Error>)` - Error if saving fails
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The config directory cannot be determined
    /// - The config directory cannot be created
    /// - The config file cannot be written
    /// - The configuration cannot be serialized to TOML
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let config_path = Self::get_config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;

        Ok(())
    }

    /// Adds a new installation to the configuration.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this installation
    /// * `installation` - The installation data to add
    pub fn add_installation(&mut self, id: String, installation: GameInstallation) {
        self.installations.insert(id, installation);
    }

    /// Remove an installation by ID. Currently unused but kept for future functionality.
    /// Removes an installation from the configuration.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the installation to remove
    pub fn remove_installation(&mut self, id: &str) {
        self.installations.remove(id);
    }

    /// Gets a reference to all registered installations.
    ///
    /// # Returns
    ///
    /// A reference to the installations `HashMap`
    #[must_use]
    pub const fn get_installations(&self) -> &HashMap<String, GameInstallation> {
        &self.installations
    }

    /// Gets the path where the configuration file should be stored.
    ///
    /// # Returns
    ///
    /// * `Ok(PathBuf)` - The configuration file path
    /// * `Err(Box<dyn std::error::Error>)` - Error if config directory cannot be determined
    ///
    /// # Errors
    ///
    /// Returns an error if the system configuration directory cannot be determined.
    fn get_config_path() -> Result<PathBuf, Box<dyn Error>> {
        let config_dir = dirs::config_dir().ok_or("Could not find config directory")?;

        Ok(config_dir.join("ctp-client").join(Self::CONFIG_FILE_NAME))
    }
}

/// Default implementation for `AppConfig` that uses system language detection.
impl Default for AppConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_app_config_new() {
        let config = AppConfig::new();
        assert_eq!(config.version, AppConfig::CONFIG_VERSION);
        assert!(config.installations.is_empty());
    }

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.version, AppConfig::CONFIG_VERSION);
        assert!(config.installations.is_empty());
        // Should have a language set (system default)
        // Can't test specific language since it depends on system
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
            "Test Installation".to_owned(),
            Some("2023-01-01 12:00:00 UTC".to_owned()),
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
            "Test".to_owned(),
            None,
        );

        config.add_installation("test_id".to_owned(), installation);
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
            "Test".to_owned(),
            None,
        );

        config.add_installation("test_id".to_owned(), installation);
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
            "Test".to_owned(),
            None,
        );

        config.add_installation("test_id".to_owned(), installation);
        let installations = config.get_installations();
        assert_eq!(installations.len(), 1);
        assert!(installations.contains_key("test_id"));
    }

    #[test]
    fn test_config_serialization_deserialization() {
        let mut config = AppConfig::new();
        let installation = GameInstallation::new(
            PathBuf::from("/test/path"),
            GameVersion::CTP2,
            "Test CTP2".to_owned(),
            Some("2023-12-25 10:30:45 UTC".to_owned()),
        );
        config.add_installation("ctp2_test".to_owned(), installation);

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
            "CTP1 Installation".to_owned(),
            Some("2023-01-01 12:00:00 UTC".to_owned()),
        );

        let installation2 = GameInstallation::new(
            PathBuf::from("/path/to/ctp2"),
            GameVersion::CTP2,
            "CTP2 Installation".to_owned(),
            Some("2023-01-02 12:00:00 UTC".to_owned()),
        );

        config.add_installation("ctp1_id".to_owned(), installation1);
        config.add_installation("ctp2_id".to_owned(), installation2);

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
            "Old Installation".to_owned(),
            None,
        );

        let installation2 = GameInstallation::new(
            PathBuf::from("/path/new"),
            GameVersion::CTP2,
            "New Installation".to_owned(),
            Some("2023-01-01 12:00:00 UTC".to_owned()),
        );

        config.add_installation("same_id".to_owned(), installation1);
        assert_eq!(config.installations.len(), 1);
        assert_eq!(
            config.installations["same_id"].display_name,
            "Old Installation"
        );

        config.add_installation("same_id".to_owned(), installation2);
        assert_eq!(config.installations.len(), 1);
        assert_eq!(
            config.installations["same_id"].display_name,
            "New Installation"
        );
        assert_eq!(config.installations["same_id"].version, GameVersion::CTP2);
    }

    #[test]
    #[expect(clippy::redundant_clone, reason = "Testing clone functionality")]
    fn test_game_installation_clone() {
        let installation = GameInstallation::new(
            PathBuf::from("/test/path"),
            GameVersion::CTP1,
            "Test Installation".to_owned(),
            Some("2023-01-01 12:00:00 UTC".to_owned()),
        );

        let cloned = installation.clone();
        assert_eq!(installation.path, cloned.path);
        assert_eq!(installation.version, cloned.version);
        assert_eq!(installation.display_name, cloned.display_name);
        assert_eq!(installation.validated_at, cloned.validated_at);
    }

    #[test]
    fn test_debug_formats() {
        let installation = GameInstallation::new(
            PathBuf::from("/test/path"),
            GameVersion::CTP1,
            "Test".to_owned(),
            None,
        );

        let debug_str = format!("{installation:?}");
        assert!(debug_str.contains("Test"));
        assert!(debug_str.contains("CTP1"));

        let version_debug = format!("{:?}", GameVersion::CTP2);
        assert!(version_debug.contains("CTP2"));
    }

    #[test]
    fn test_config_version_constant() {
        assert_eq!(AppConfig::CONFIG_VERSION, 1);
    }

    #[test]
    fn test_config_file_name_constant() {
        assert_eq!(AppConfig::CONFIG_FILE_NAME, "ctp-client-config.toml");
    }

    #[test]
    fn test_config_load_nonexistent_file() {
        // Test loading when config file doesn't exist
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Set a temporary config directory
        // SAFETY: This is safe because we're only temporarily setting an environment variable
        // for testing purposes and will restore it afterward
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        }

        let result = AppConfig::load();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.version, AppConfig::CONFIG_VERSION);
        assert!(config.get_installations().is_empty());

        // Clean up
        // SAFETY: Safe to remove the temporary environment variable
        unsafe {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
    }

    #[test]
    fn test_config_load_with_version_mismatch() {
        use std::fs;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_dir = temp_dir.path().join("ctp-client");
        fs::create_dir_all(&config_dir).unwrap();

        // Create a config file with wrong version
        let config_path = config_dir.join("ctp-client-config.toml");
        let invalid_config = "
version = 999
language = \"English\"
installations = {}
";
        fs::write(&config_path, invalid_config).unwrap();

        // Set temporary config directory
        // SAFETY: Safe temporary environment variable modification for testing
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        }

        let result = AppConfig::load();
        assert!(result.is_ok());

        let config = result.unwrap();
        // Should fallback to default config due to version mismatch
        assert_eq!(config.version, AppConfig::CONFIG_VERSION);
        assert!(config.get_installations().is_empty());

        // Clean up
        // SAFETY: Safe to remove temporary environment variable
        unsafe {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
    }
}
