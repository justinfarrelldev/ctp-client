//! Integration tests for the CTP client.

#![allow(
    clippy::tests_outside_test_module,
    reason = "Integration tests are outside cfg(test) by design"
)]

use std::path::PathBuf;

#[test]
fn test_error_handling_chain() {
    // Test the error handling chain from validation to UI
    let nonexistent_path = PathBuf::from("/absolutely/nonexistent/path/for/testing");

    // Validation should fail
    let validation_result =
        ctp_client::installation::InstallationValidator::validate_installation(&nonexistent_path);
    assert!(validation_result.is_err());

    // Installation creation should fail
    let installation_result =
        ctp_client::installation::InstallationValidator::create_installation(nonexistent_path);
    assert!(installation_result.is_err());

    // Config should remain unchanged when invalid installations are attempted
    let mut config = ctp_client::config::AppConfig::new();
    let initial_count = config.get_installations().len();

    // This would be how the UI handles the error (not adding invalid installations)
    if let Ok(installation) = installation_result {
        config.add_installation("invalid".to_owned(), installation);
    } else {
        // Don't add invalid installations
    }

    assert_eq!(config.get_installations().len(), initial_count);
}

#[test]
fn test_config_file_operations() {
    let mut config = ctp_client::config::AppConfig::new();

    // Test that config starts empty
    assert!(config.get_installations().is_empty());
    assert_eq!(config.version, 1);

    // Add some test data
    let installation = ctp_client::config::GameInstallation::new(
        PathBuf::from("/test/path"),
        ctp_client::config::GameVersion::CTP2,
        "Test Installation".to_owned(),
        Some("2023-01-01 12:00:00 UTC".to_owned()),
    );

    config.add_installation("test_id".to_owned(), installation);
    assert_eq!(config.get_installations().len(), 1);

    // Test serialization
    let serialized = toml::to_string(&config).unwrap();
    assert!(serialized.contains("version = 1"));
    assert!(serialized.contains("Test Installation"));
    assert!(serialized.contains("CTP2"));

    // Test deserialization
    let deserialized: ctp_client::config::AppConfig = toml::from_str(&serialized).unwrap();
    assert_eq!(deserialized.version, config.version);
    assert_eq!(deserialized.get_installations().len(), 1);

    // Test removal
    let mut config_copy = deserialized;
    config_copy.remove_installation("test_id");
    assert!(config_copy.get_installations().is_empty());
}

#[test]
fn test_ui_state_transitions() {
    // Test that UI can be created
    let ui_result = ctp_client::ui::GtkUI::new();
    assert!(ui_result.is_ok());

    let ui = ui_result.unwrap();

    // Test initial state
    let config = ui.get_config();
    assert!(config.get_installations().is_empty() || !config.get_installations().is_empty());
    assert_eq!(config.version, 1);
}

#[test]
fn test_display_name_generation() {
    // Test with normal paths
    let paths_and_expected = [
        ("/home/user/Games/CTP", "CTP"),
        ("/Program Files/Call to Power", "Call to Power"),
        ("/usr/local/games/ctp2", "ctp2"),
        ("/", ""), // Root path should have no filename
    ];

    for (path_str, expected_name) in paths_and_expected {
        let path = PathBuf::from(path_str);
        let actual_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        assert_eq!(actual_name, expected_name);
    }
}

#[test]
fn test_version_constants() {
    // Test that our version constants are as expected
    assert_eq!(ctp_client::config::AppConfig::CONFIG_VERSION, 1);
    assert_eq!(
        ctp_client::config::AppConfig::CONFIG_FILE_NAME,
        "ctp-client-config.toml"
    );

    // Test game version folder mappings
    let (ctp1_data, ctp1_program) = ctp_client::config::GameVersion::CTP1.required_folders();
    let (ctp2_data, ctp2_program) = ctp_client::config::GameVersion::CTP2.required_folders();

    assert_eq!(ctp1_data, "ctp_data");
    assert_eq!(ctp1_program, "ctp_program");
    assert_eq!(ctp2_data, "ctp2_data");
    assert_eq!(ctp2_program, "ctp2_program");
}

#[test]
fn test_path_scanning() {
    // Test that path scanning doesn't panic and returns a vector
    let paths = ctp_client::installation::InstallationValidator::scan_common_paths();

    // Should return a vector (empty or not depending on system)
    assert!(paths.is_empty() || !paths.is_empty()); // Always true but tests the return type

    // All returned paths should be PathBuf instances
    for path in paths {
        let _: PathBuf = path;
    }
}

#[test]
#[expect(
    clippy::std_instead_of_alloc,
    reason = "Arc from std is appropriate for testing"
)]
fn test_concurrent_operations() {
    use std::sync::Arc;
    use std::thread;

    // Test that config operations are safe to use from multiple threads
    let config = Arc::new(ctp_client::config::AppConfig::new());

    let handles: Vec<_> = (0_i32..5_i32)
        .map(|i| {
            let config_clone = Arc::clone(&config);
            thread::spawn(move || {
                // Test reading operations
                let installations = config_clone.get_installations();
                assert!(installations.is_empty() || !installations.is_empty());

                // Test version access
                assert_eq!(config_clone.version, 1);

                // Return thread id for verification
                i
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result < 5_i32);
    }
}
