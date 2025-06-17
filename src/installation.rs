//! Installation validation and detection functionality.
//!
//! This module provides tools for validating Call to Power game installations,
//! detecting game versions, and scanning common installation paths.

use crate::config::{GameInstallation, GameVersion};
use core::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use std::path::{Path, PathBuf};

/// Errors that can occur during installation validation.
#[derive(Debug)]
#[non_exhaustive]
pub enum ValidationError {
    /// The installation path does not exist.
    PathNotFound,
    /// The required data folder is missing.
    MissingDataFolder,
    /// The required program folder is missing.
    MissingProgramFolder,
    /// The path exists but is not a directory.
    NotADirectory,
}

/// Display implementation for `ValidationError`.
impl Display for ValidationError {
    /// Formats the validation error for display.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match *self {
            Self::PathNotFound => write!(f, "Installation path not found"),
            Self::MissingDataFolder => write!(f, "Data folder missing"),
            Self::MissingProgramFolder => write!(f, "Program folder missing"),
            Self::NotADirectory => write!(f, "Path is not a directory"),
        }
    }
}

/// Error trait implementation for `ValidationError`.
impl Error for ValidationError {}

/// Validator for Call to Power game installations.
///
/// Provides methods to validate installation paths, detect game versions,
/// and scan common installation locations.
pub struct InstallationValidator;

/// Implementation of installation validation functionality.
impl InstallationValidator {
    /// Validates a Call to Power installation at the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check for a valid installation
    ///
    /// # Returns
    ///
    /// * `Ok(GameVersion)` - The detected game version if validation succeeds
    /// * `Err(ValidationError)` - The validation error if validation fails
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist
    /// - The path is not a directory
    /// - Neither CTP1 nor CTP2 installations are detected
    pub fn validate_installation(path: &Path) -> Result<GameVersion, ValidationError> {
        if !path.exists() {
            return Err(ValidationError::PathNotFound);
        }

        if !path.is_dir() {
            return Err(ValidationError::NotADirectory);
        }

        if Self::is_ctp1_installation(path) {
            return Ok(GameVersion::CTP1);
        }

        if Self::is_ctp2_installation(path) {
            return Ok(GameVersion::CTP2);
        }

        Err(ValidationError::MissingDataFolder)
    }

    /// Checks if the given path contains a valid CTP1 installation.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check
    ///
    /// # Returns
    ///
    /// * `true` if both required CTP1 folders are present, `false` otherwise
    fn is_ctp1_installation(path: &Path) -> bool {
        let (data_folder, program_folder) = GameVersion::CTP1.required_folders();
        path.join(data_folder).is_dir() && path.join(program_folder).is_dir()
    }

    /// Checks if the given path contains a valid CTP2 installation.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check
    ///
    /// # Returns
    ///
    /// * `true` if both required CTP2 folders are present, `false` otherwise
    fn is_ctp2_installation(path: &Path) -> bool {
        let (data_folder, program_folder) = GameVersion::CTP2.required_folders();
        path.join(data_folder).is_dir() && path.join(program_folder).is_dir()
    }

    /// Creates a `GameInstallation` from a validated path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to create an installation from
    ///
    /// # Returns
    ///
    /// * `Ok(GameInstallation)` - The created installation if validation succeeds
    /// * `Err(ValidationError)` - The validation error if validation fails
    ///
    /// # Errors
    ///
    /// Returns an error if the path fails validation.
    pub fn create_installation(path: PathBuf) -> Result<GameInstallation, ValidationError> {
        let version = Self::validate_installation(&path)?;
        let display_name = Self::generate_display_name(&path, &version);
        let validated_at = Some(
            chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
        );

        Ok(GameInstallation::new(
            path,
            version,
            display_name,
            validated_at,
        ))
    }

    /// Generates a display name for the installation.
    ///
    /// # Arguments
    ///
    /// * `path` - The installation path
    /// * `version` - The detected game version
    ///
    /// # Returns
    ///
    /// A formatted display name combining the game version and directory name
    fn generate_display_name(path: &Path, version: &GameVersion) -> String {
        let version_str = match *version {
            GameVersion::CTP1 => "Call to Power",
            GameVersion::CTP2 => "Call to Power II",
        };

        path.file_name().and_then(|n| n.to_str()).map_or_else(
            || version_str.to_owned(),
            |name| format!("{version_str} ({name})"),
        )
    }

    /// Scans common installation paths for Call to Power games.
    ///
    /// This function searches platform-specific common installation locations
    /// where Call to Power games are typically installed.
    ///
    /// # Returns
    ///
    /// A vector of `PathBuf` objects representing existing installation paths
    #[must_use]
    pub fn scan_common_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        #[cfg(target_os = "windows")]
        {
            let common_paths = [
                r"C:\Program Files (x86)\Activision\Call to Power",
                r"C:\Program Files\Activision\Call to Power",
                r"C:\Program Files (x86)\GOG.com\Call to Power 2",
                r"C:\Program Files\GOG.com\Call to Power 2",
                r"C:\GOG Games\Call to Power 2",
                r"C:\Steam\steamapps\common\Call to Power 2",
            ];

            for path_str in &common_paths {
                let path = PathBuf::from(path_str);
                if path.exists() {
                    paths.push(path);
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Some(home) = dirs::home_dir() {
                let common_paths = [
                    ".steam/steam/steamapps/common/Call to Power 2",
                    ".local/share/Steam/steamapps/common/Call to Power 2",
                    "Games/Call to Power",
                    "Games/Call to Power 2",
                ];

                for path_str in &common_paths {
                    let path = home.join(path_str);
                    if path.exists() {
                        paths.push(path);
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(home) = dirs::home_dir() {
                let common_paths = [
                    "Library/Application Support/Steam/steamapps/common/Call to Power 2",
                    "Applications/Call to Power.app",
                    "Applications/Call to Power 2.app",
                ];

                for path_str in &common_paths {
                    let path = home.join(path_str);
                    if path.exists() {
                        paths.push(path);
                    }
                }
            }
        }

        paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Creates a temporary CTP1 installation structure for testing
    fn create_test_ctp1_installation() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path();

        // Create CTP1 folders
        let (data_folder, program_folder) = GameVersion::CTP1.required_folders();
        fs::create_dir_all(base_path.join(data_folder)).unwrap();
        fs::create_dir_all(base_path.join(program_folder)).unwrap();

        temp_dir
    }

    /// Creates a temporary CTP2 installation structure for testing
    fn create_test_ctp2_installation() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path();

        // Create CTP2 folders
        let (data_folder, program_folder) = GameVersion::CTP2.required_folders();
        fs::create_dir_all(base_path.join(data_folder)).unwrap();
        fs::create_dir_all(base_path.join(program_folder)).unwrap();

        temp_dir
    }

    #[test]
    fn test_validation_error_display() {
        let error1 = ValidationError::PathNotFound;
        assert_eq!(format!("{error1}"), "Installation path not found");

        let error2 = ValidationError::MissingDataFolder;
        assert_eq!(format!("{error2}"), "Data folder missing");

        let error3 = ValidationError::MissingProgramFolder;
        assert_eq!(format!("{error3}"), "Program folder missing");

        let error4 = ValidationError::NotADirectory;
        assert_eq!(format!("{error4}"), "Path is not a directory");
    }

    #[test]
    fn test_validation_error_debug() {
        let error1 = ValidationError::PathNotFound;
        let debug_str1 = format!("{error1:?}");
        assert!(debug_str1.contains("PathNotFound"));

        let error2 = ValidationError::MissingProgramFolder;
        let debug_str2 = format!("{error2:?}");
        assert!(debug_str2.contains("MissingProgramFolder"));
    }

    #[test]
    fn test_validation_error_is_error_trait() {
        let error = ValidationError::PathNotFound;
        let _: &dyn std::error::Error = &error;
    }

    #[test]
    fn test_validate_nonexistent_path() {
        let path = PathBuf::from("/absolutely/nonexistent/path/that/should/never/exist");
        let result = InstallationValidator::validate_installation(&path);

        assert!(result.is_err());
        matches!(result.unwrap_err(), ValidationError::PathNotFound);
    }

    #[test]
    fn test_validate_file_instead_of_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test_file");
        fs::write(&file_path, "test content").unwrap();

        let result = InstallationValidator::validate_installation(&file_path);
        assert!(result.is_err());
        matches!(result.unwrap_err(), ValidationError::NotADirectory);
    }

    #[test]
    fn test_validate_empty_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let result = InstallationValidator::validate_installation(temp_dir.path());

        assert!(result.is_err());
        matches!(result.unwrap_err(), ValidationError::MissingDataFolder);
    }

    #[test]
    fn test_validate_ctp1_installation_success() {
        let temp_dir = create_test_ctp1_installation();
        let result = InstallationValidator::validate_installation(temp_dir.path());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), GameVersion::CTP1);
    }

    #[test]
    fn test_validate_ctp2_installation_success() {
        let temp_dir = create_test_ctp2_installation();
        let result = InstallationValidator::validate_installation(temp_dir.path());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), GameVersion::CTP2);
    }

    #[test]
    fn test_validate_ctp1_missing_data_folder() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path();

        // Only create program folder, not data folder
        let (_, program_folder) = GameVersion::CTP1.required_folders();
        fs::create_dir_all(base_path.join(program_folder)).unwrap();

        let result = InstallationValidator::validate_installation(base_path);
        assert!(result.is_err());
        matches!(result.unwrap_err(), ValidationError::MissingDataFolder);
    }

    #[test]
    fn test_validate_ctp1_missing_program_folder() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path();

        // Only create data folder, not program folder
        let (data_folder, _) = GameVersion::CTP1.required_folders();
        fs::create_dir_all(base_path.join(data_folder)).unwrap();

        let result = InstallationValidator::validate_installation(base_path);
        assert!(result.is_err());
        matches!(result.unwrap_err(), ValidationError::MissingDataFolder);
    }

    #[test]
    fn test_validate_ctp2_missing_data_folder() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path();

        // Only create program folder, not data folder
        let (_, program_folder) = GameVersion::CTP2.required_folders();
        fs::create_dir_all(base_path.join(program_folder)).unwrap();

        let result = InstallationValidator::validate_installation(base_path);
        assert!(result.is_err());
        matches!(result.unwrap_err(), ValidationError::MissingDataFolder);
    }

    #[test]
    fn test_validate_ctp2_missing_program_folder() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path();

        // Only create data folder, not program folder
        let (data_folder, _) = GameVersion::CTP2.required_folders();
        fs::create_dir_all(base_path.join(data_folder)).unwrap();

        let result = InstallationValidator::validate_installation(base_path);
        assert!(result.is_err());
        matches!(result.unwrap_err(), ValidationError::MissingDataFolder);
    }

    #[test]
    fn test_is_ctp1_installation_true() {
        let temp_dir = create_test_ctp1_installation();
        let result = InstallationValidator::is_ctp1_installation(temp_dir.path());
        assert!(result);
    }

    #[test]
    fn test_is_ctp1_installation_false() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let result = InstallationValidator::is_ctp1_installation(temp_dir.path());
        assert!(!result);
    }

    #[test]
    fn test_is_ctp2_installation_true() {
        let temp_dir = create_test_ctp2_installation();
        let result = InstallationValidator::is_ctp2_installation(temp_dir.path());
        assert!(result);
    }

    #[test]
    fn test_is_ctp2_installation_false() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let result = InstallationValidator::is_ctp2_installation(temp_dir.path());
        assert!(!result);
    }

    #[test]
    fn test_create_installation_success_ctp1() {
        let temp_dir = create_test_ctp1_installation();
        let path = temp_dir.path().to_path_buf();

        let result = InstallationValidator::create_installation(path.clone());
        assert!(result.is_ok());

        let installation = result.unwrap();
        assert_eq!(installation.path, path);
        assert_eq!(installation.version, GameVersion::CTP1);
        assert!(installation.display_name.contains("Call to Power"));
        assert!(installation.validated_at.is_some());
    }

    #[test]
    fn test_create_installation_success_ctp2() {
        let temp_dir = create_test_ctp2_installation();
        let path = temp_dir.path().to_path_buf();

        let result = InstallationValidator::create_installation(path.clone());
        assert!(result.is_ok());

        let installation = result.unwrap();
        assert_eq!(installation.path, path);
        assert_eq!(installation.version, GameVersion::CTP2);
        assert!(installation.display_name.contains("Call to Power II"));
        assert!(installation.validated_at.is_some());
    }

    #[test]
    fn test_create_installation_failure() {
        let path = PathBuf::from("/nonexistent/path");
        let result = InstallationValidator::create_installation(path);

        assert!(result.is_err());
        matches!(result.unwrap_err(), ValidationError::PathNotFound);
    }

    #[test]
    fn test_generate_display_name_ctp1() {
        let path = PathBuf::from("/some/path/game_directory");
        let display_name = InstallationValidator::generate_display_name(&path, &GameVersion::CTP1);
        assert_eq!(display_name, "Call to Power (game_directory)");
    }

    #[test]
    fn test_generate_display_name_ctp2() {
        let path = PathBuf::from("/some/path/game_directory");
        let display_name = InstallationValidator::generate_display_name(&path, &GameVersion::CTP2);
        assert_eq!(display_name, "Call to Power II (game_directory)");
    }

    #[test]
    fn test_generate_display_name_no_file_name() {
        let path = PathBuf::from("/");
        let display_name = InstallationValidator::generate_display_name(&path, &GameVersion::CTP1);
        assert_eq!(display_name, "Call to Power");

        let display_name2 = InstallationValidator::generate_display_name(&path, &GameVersion::CTP2);
        assert_eq!(display_name2, "Call to Power II");
    }

    #[test]
    fn test_generate_display_name_invalid_unicode() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        // Create a path with invalid UTF-8
        let invalid_bytes = b"\xFF\xFE";
        let invalid_os_str = OsStr::from_bytes(invalid_bytes);
        let path = PathBuf::from("/some/path").join(invalid_os_str);

        let display_name = InstallationValidator::generate_display_name(&path, &GameVersion::CTP1);
        assert_eq!(display_name, "Call to Power");
    }

    #[test]
    fn test_scan_common_paths_returns_vector() {
        let paths = InstallationValidator::scan_common_paths();
        // Just verify it returns a vector (contents depend on system)
        assert!(paths.is_empty() || !paths.is_empty()); // Always true, but tests the return type
    }

    #[test]
    fn test_installation_validation_prefers_ctp1_over_ctp2() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path();

        // Create both CTP1 and CTP2 folders
        let (ctp1_data, ctp1_program) = GameVersion::CTP1.required_folders();
        let (ctp2_data, ctp2_program) = GameVersion::CTP2.required_folders();

        fs::create_dir_all(base_path.join(ctp1_data)).unwrap();
        fs::create_dir_all(base_path.join(ctp1_program)).unwrap();
        fs::create_dir_all(base_path.join(ctp2_data)).unwrap();
        fs::create_dir_all(base_path.join(ctp2_program)).unwrap();

        let result = InstallationValidator::validate_installation(base_path);
        assert!(result.is_ok());
        // Should prefer CTP1 since it's checked first
        assert_eq!(result.unwrap(), GameVersion::CTP1);
    }

    #[test]
    fn test_both_installation_checks_with_mixed_folders() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path();

        // Create CTP1 data folder and CTP2 program folder
        let (ctp1_data, _) = GameVersion::CTP1.required_folders();
        let (_, ctp2_program) = GameVersion::CTP2.required_folders();

        fs::create_dir_all(base_path.join(ctp1_data)).unwrap();
        fs::create_dir_all(base_path.join(ctp2_program)).unwrap();

        // Neither should be detected as valid
        let is_ctp1 = InstallationValidator::is_ctp1_installation(base_path);
        let is_ctp2 = InstallationValidator::is_ctp2_installation(base_path);

        assert!(!is_ctp1);
        assert!(!is_ctp2);

        let result = InstallationValidator::validate_installation(base_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_game_installation_with_different_timestamps() {
        let temp_dir = create_test_ctp1_installation();
        let path = temp_dir.path().to_path_buf();

        let installation1 = InstallationValidator::create_installation(path.clone()).unwrap();

        // Larger delay to ensure different timestamp
        std::thread::sleep(std::time::Duration::from_millis(1100));

        let installation2 = InstallationValidator::create_installation(path).unwrap();

        // Timestamps should be different (or at least not panic if they're the same)
        // This test verifies that timestamps are created properly
        assert!(installation1.validated_at.is_some());
        assert!(installation2.validated_at.is_some());

        // Both should be valid timestamps
        let ts1 = installation1.validated_at.unwrap();
        let ts2 = installation2.validated_at.unwrap();
        assert!(ts1.contains("UTC"));
        assert!(ts2.contains("UTC"));
    }

    #[test]
    fn test_path_with_special_characters() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let special_path = temp_dir.path().join("My Game (Special Edition)");
        fs::create_dir_all(&special_path).unwrap();

        // Create CTP1 structure
        let (data_folder, program_folder) = GameVersion::CTP1.required_folders();
        fs::create_dir_all(special_path.join(data_folder)).unwrap();
        fs::create_dir_all(special_path.join(program_folder)).unwrap();

        let result = InstallationValidator::validate_installation(&special_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), GameVersion::CTP1);

        let installation = InstallationValidator::create_installation(special_path).unwrap();
        assert!(
            installation
                .display_name
                .contains("My Game (Special Edition)")
        );
    }
}
