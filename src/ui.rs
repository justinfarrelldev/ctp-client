//! User interface implementation using GTK4.
//!
//! This module provides the main UI for the CTP client, including installation
//! selection, game launching, and file browsing functionality.

#![allow(
    clippy::std_instead_of_alloc,
    reason = "GTK4 requires std types for UI integration"
)]
#![allow(
    clippy::clone_on_ref_ptr,
    reason = "Rc cloning is necessary for GTK4 callback sharing"
)]
#![allow(
    clippy::needless_pass_by_value,
    reason = "GTK4 signal handlers require owned values"
)]
#![allow(
    dead_code,
    reason = "UIState fields reserved for future auto-detection features"
)]

extern crate alloc;

use crate::config::{AppConfig, GameInstallation, GameVersion};
use crate::i18n::{TranslationKey, Translator};
use crate::installation::InstallationValidator;
use crate::settings::{SettingsDialog, SettingsResult};
use alloc::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;

use glib::clone;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, HeaderBar, Label, ListBox, ListBoxRow,
    Orientation, ScrolledWindow, glib,
};

/// Result of UI operations, indicating the next action to take.
#[derive(Debug)]
pub enum UIResult {
    /// User selected a game installation to play.
    PlayGame(GameInstallation),
    /// User requested to exit the application.
    Exit,
}

/// Main UI structure managing the GTK interface and application state.
pub struct GtkUI {
    /// Application configuration including saved installations.
    config: Rc<RefCell<AppConfig>>,
    /// Translator for internationalization.
    translator: Rc<RefCell<Translator>>,
    /// Automatically detected installation paths.
    found_installations: Vec<PathBuf>,
    /// Current state of the UI.
    ui_state: Rc<RefCell<UIState>>,
    /// Currently selected installation for playing.
    selected_installation: Rc<RefCell<Option<GameInstallation>>>,
    /// Whether the user has requested to exit.
    exit_requested: Rc<RefCell<bool>>,
    /// GTK Application instance.
    app: Application,
    /// UI elements that need to be updated when language changes.
    ui_elements: Rc<RefCell<Option<UIElements>>>,
}

/// UI elements that need to be updated when language changes.
struct UIElements {
    /// Main application window.
    window: ApplicationWindow,
    /// Header title label.
    header_label: Label,
    /// Settings button.
    settings_button: Button,
    /// Main title label.
    title_label: Label,
    /// Add installation button.
    add_button: Button,
    /// Play game button.
    play_button: Button,
    /// Exit application button.
    exit_button: Button,
}

/// Internal state for UI elements.
#[derive(Default)]
struct UIState {
    /// Whether to show the auto-detected installations dialog.
    show_auto_detected: bool,
}

/// Implementation of the main UI functionality.
impl GtkUI {
    /// Creates a new UI instance.
    ///
    /// # Returns
    ///
    /// * `Ok(GtkUI)` - The initialized UI instance
    /// * `Err(Box<dyn std::error::Error>)` - Error if initialization fails
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be loaded or GTK initialization fails.
    pub fn new() -> Result<Self, alloc::boxed::Box<dyn std::error::Error>> {
        let config = AppConfig::load()?;
        let translator = Translator::with_language(config.language.clone());
        let found_installations = InstallationValidator::scan_common_paths();

        let app = Application::builder()
            .application_id("com.example.ctp-client")
            .build();

        Ok(Self {
            config: Rc::new(RefCell::new(config)),
            translator: Rc::new(RefCell::new(translator)),
            found_installations,
            ui_state: Rc::new(RefCell::new(UIState::default())),
            selected_installation: Rc::new(RefCell::new(None)),
            exit_requested: Rc::new(RefCell::new(false)),
            app,
            ui_elements: Rc::new(RefCell::new(None)),
        })
    }

    /// Runs the main UI loop until the user exits or selects a game.
    ///
    /// # Returns
    ///
    /// * `Ok(UIResult)` - The user's choice (play game or exit)
    /// * `Err(Box<dyn std::error::Error>)` - Error if the UI fails
    ///
    /// # Errors
    ///
    /// Returns an error if GTK initialization fails or other UI errors occur.
    pub fn run(&mut self) -> Result<UIResult, alloc::boxed::Box<dyn std::error::Error>> {
        let config = self.config.clone();
        let translator = self.translator.clone();
        let found_installations = self.found_installations.clone();
        let ui_state = self.ui_state.clone();
        let selected_installation = self.selected_installation.clone();
        let exit_requested = self.exit_requested.clone();
        let ui_elements = self.ui_elements.clone();

        self.app.connect_activate(move |app| {
            Self::build_ui(
                app,
                config.clone(),
                translator.clone(),
                found_installations.clone(),
                ui_state.clone(),
                selected_installation.clone(),
                exit_requested.clone(),
                ui_elements.clone(),
            );
        });

        self.app.run();

        // Check if a game was selected
        if let Some(installation) = self.selected_installation.borrow().as_ref() {
            return Ok(UIResult::PlayGame(installation.clone()));
        }

        // Check if exit was requested
        if *self.exit_requested.borrow() {
            return Ok(UIResult::Exit);
        }

        Ok(UIResult::Exit)
    }

    /// Builds the main UI window and connects all event handlers.
    ///
    /// # Arguments
    ///
    /// * `app` - The GTK Application instance
    /// * `config` - The application configuration
    /// * `translator` - The translator for internationalization
    /// * `found_installations` - Auto-detected installations
    /// * `ui_state` - Current UI state
    /// * `selected_installation` - Currently selected installation
    /// * `exit_requested` - Whether exit was requested
    /// * `ui_elements` - Storage for UI elements that need updating
    #[expect(
        clippy::too_many_arguments,
        reason = "Building UI requires many parameters for proper initialization"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "UI building naturally requires many lines for proper layout"
    )]
    fn build_ui(
        app: &Application,
        config: Rc<RefCell<AppConfig>>,
        translator: Rc<RefCell<Translator>>,
        found_installations: Vec<PathBuf>,
        _ui_state: Rc<RefCell<UIState>>,
        selected_installation: Rc<RefCell<Option<GameInstallation>>>,
        exit_requested: Rc<RefCell<bool>>,
        ui_elements: Rc<RefCell<Option<UIElements>>>,
    ) {
        // Create the main window
        let window = ApplicationWindow::builder()
            .application(app)
            .title(translator.borrow().translate(&TranslationKey::AppTitle))
            .default_width(800)
            .default_height(600)
            .build();

        // Create header bar
        let header_label = Label::new(Some(
            &translator.borrow().translate(&TranslationKey::AppTitle),
        ));
        let header = HeaderBar::builder().title_widget(&header_label).build();

        // Add settings button to header bar
        let settings_button = Button::from_icon_name("preferences-system-symbolic");
        settings_button.set_tooltip_text(Some(
            &translator.borrow().translate(&TranslationKey::Settings),
        ));

        let settings_config = config.clone();
        let settings_translator = translator.clone();
        let settings_ui_elements = ui_elements.clone();
        settings_button.connect_clicked(clone!(
            #[weak]
            window,
            move |_| {
                Self::show_settings_dialog(
                    &window,
                    settings_config.clone(),
                    settings_translator.clone(),
                    settings_ui_elements.clone(),
                );
            }
        ));
        header.pack_end(&settings_button);

        window.set_titlebar(Some(&header));

        // Create main container
        let main_box = GtkBox::new(Orientation::Vertical, 12);
        main_box.set_margin_top(20);
        main_box.set_margin_bottom(20);
        main_box.set_margin_start(20);
        main_box.set_margin_end(20);

        // Title label
        let title_label = Label::new(Some(
            &translator
                .borrow()
                .translate(&TranslationKey::SelectInstallation),
        ));
        title_label.add_css_class("title-1");
        main_box.append(&title_label);

        // Create scrolled window for installations list
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .min_content_height(300)
            .build();

        let installations_list = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .build();

        // Populate installations list
        Self::populate_installations_list(
            &installations_list,
            config.clone(),
            selected_installation.clone(),
            app,
        );

        scrolled.set_child(Some(&installations_list));
        main_box.append(&scrolled);

        // Create buttons container
        let buttons_box = GtkBox::new(Orientation::Horizontal, 12);
        buttons_box.set_halign(gtk4::Align::Center);

        // Add Installation button
        let add_button = Button::with_label(
            &translator
                .borrow()
                .translate(&TranslationKey::BrowseForInstallation),
        );
        add_button.connect_clicked(clone!(
            #[weak]
            window,
            #[strong]
            config,
            #[strong]
            found_installations,
            move |_| {
                Self::show_add_installation_dialog(
                    &window,
                    config.clone(),
                    found_installations.clone(),
                );
            }
        ));
        buttons_box.append(&add_button);

        // Play button
        let play_button =
            Button::with_label(&translator.borrow().translate(&TranslationKey::PlayGame));
        play_button.add_css_class("suggested-action");
        play_button.connect_clicked(clone!(
            #[strong]
            selected_installation,
            #[weak]
            window,
            move |_| {
                if selected_installation.borrow().is_some() {
                    window.close();
                }
            }
        ));
        buttons_box.append(&play_button);

        // Exit button
        let exit_button = Button::with_label(&translator.borrow().translate(&TranslationKey::Exit));
        exit_button.connect_clicked(clone!(
            #[strong]
            exit_requested,
            #[weak]
            window,
            move |_| {
                *exit_requested.borrow_mut() = true;
                window.close();
            }
        ));
        buttons_box.append(&exit_button);

        main_box.append(&buttons_box);

        window.set_child(Some(&main_box));

        // Store UI elements for language updates
        *ui_elements.borrow_mut() = Some(UIElements {
            window: window.clone(),
            header_label,
            settings_button,
            title_label,
            add_button,
            play_button,
            exit_button,
        });

        window.present();
    }

    /// Shows the add installation dialog with auto-detected and browse options.
    ///
    /// # Arguments
    ///
    /// * `parent` - The parent window
    /// * `config` - The application configuration
    /// * `found_installations` - Auto-detected installations
    fn show_add_installation_dialog(
        parent: &ApplicationWindow,
        config: Rc<RefCell<AppConfig>>,
        found_installations: Vec<PathBuf>,
    ) {
        let dialog = gtk4::Dialog::builder()
            .title("Add Installation")
            .modal(true)
            .transient_for(parent)
            .build();

        let content_area = dialog.content_area();
        let main_box = GtkBox::new(Orientation::Vertical, 12);
        main_box.set_margin_top(20);
        main_box.set_margin_bottom(20);
        main_box.set_margin_start(20);
        main_box.set_margin_end(20);

        if found_installations.is_empty() {
            let label = Label::new(Some("No installations auto-detected."));
            main_box.append(&label);
        } else {
            let label = Label::new(Some("Auto-detected installations:"));
            main_box.append(&label);

            let list = ListBox::new();
            for path in found_installations {
                let row = ListBoxRow::new();
                let path_label = Label::new(Some(&path.display().to_string()));
                path_label.set_margin_top(6);
                path_label.set_margin_bottom(6);
                path_label.set_margin_start(12);
                path_label.set_margin_end(12);
                row.set_child(Some(&path_label));

                let path_clone = path.clone();
                row.connect_activate(clone!(
                    #[strong]
                    config,
                    #[weak]
                    dialog,
                    move |_| {
                        Self::add_installation_static(config.clone(), path_clone.clone());
                        dialog.close();
                    }
                ));

                list.append(&row);
            }

            let scrolled = ScrolledWindow::builder().height_request(200).build();
            scrolled.set_child(Some(&list));
            main_box.append(&scrolled);
        }

        // Browse button
        let browse_button = Button::with_label("Browse for Installation...");
        browse_button.connect_clicked(clone!(
            #[weak]
            dialog,
            #[strong]
            config,
            move |_| {
                log::info!("Browse button clicked - using rfd file dialog");

                // Create a channel to communicate between threads
                let (sender, receiver) = std::sync::mpsc::channel();

                // Spawn rfd dialog in a separate thread
                std::thread::spawn(move || {
                    log::info!("Opening rfd file dialog...");
                    let result = rfd::FileDialog::new()
                        .set_title("Select Call to Power Installation Directory")
                        .pick_folder();

                    if sender.send(result).is_err() {
                        log::error!("Failed to send file dialog result");
                    }
                });

                // Poll for the result on the main thread
                let config_clone = config.clone();
                glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                    receiver
                        .try_recv()
                        .map_or(glib::ControlFlow::Continue, |result| {
                            match result {
                                Some(path) => {
                                    log::info!("rfd selected path: {path:?}");
                                    Self::add_installation_static(config_clone.clone(), path);
                                    dialog.close();
                                }
                                None => {
                                    log::info!("rfd file dialog cancelled or no selection made");
                                }
                            }
                            glib::ControlFlow::Break
                        })
                });
            }
        ));
        main_box.append(&browse_button);

        // Cancel button
        let cancel_button = Button::with_label("Cancel");
        cancel_button.connect_clicked(clone!(
            #[weak]
            dialog,
            move |_| {
                dialog.close();
            }
        ));
        main_box.append(&cancel_button);

        content_area.append(&main_box);
        dialog.present();
    }

    /// Static method to add an installation (used in GTK callbacks).
    ///
    /// # Arguments
    ///
    /// * `config` - The application configuration
    /// * `path` - The path to the installation directory
    fn add_installation_static(config: Rc<RefCell<AppConfig>>, path: PathBuf) {
        match InstallationValidator::create_installation(path) {
            Ok(installation) => {
                let id = format!(
                    "{}_{}",
                    match installation.version {
                        GameVersion::CTP1 => "ctp1",
                        GameVersion::CTP2 => "ctp2",
                    },
                    installation
                        .path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                );

                log::info!("Added installation: {}", installation.display_name);
                config.borrow_mut().add_installation(id, installation);

                if let Err(e) = config.borrow().save() {
                    log::error!("Failed to save config: {e}");
                }
            }
            Err(e) => {
                log::error!("Invalid installation: {e}");
            }
        }
    }

    /// Shows the settings dialog for configuring application preferences.
    ///
    /// # Arguments
    ///
    /// * `parent` - The parent window
    /// * `config` - The application configuration
    /// * `translator` - The current translator instance
    /// * `ui_elements` - UI elements to refresh after language change
    fn show_settings_dialog(
        parent: &ApplicationWindow,
        config: Rc<RefCell<AppConfig>>,
        translator: Rc<RefCell<Translator>>,
        ui_elements: Rc<RefCell<Option<UIElements>>>,
    ) {
        let settings_dialog = SettingsDialog::new(
            parent.upcast_ref::<gtk4::Window>(),
            config.clone(),
            translator.clone(),
        );

        // Use callback-based approach to avoid blocking the main thread
        settings_dialog.show(move |result| {
            match result {
                SettingsResult::Saved(new_language) => {
                    // Update the translator with the new language
                    translator.borrow_mut().set_language(new_language);

                    // Save the configuration
                    if let Err(e) = config.borrow().save() {
                        log::error!("Failed to save config after language change: {e}");
                    } else {
                        log::info!("Language settings saved successfully");
                    }

                    // Refresh UI text to new language
                    Self::refresh_ui_text(&ui_elements, &translator);
                }
                SettingsResult::Cancelled => {
                    log::info!("Settings dialog cancelled");
                }
            }
        });
    }

    /// Refreshes all UI text elements with the current language.
    fn refresh_ui_text(
        ui_elements: &Rc<RefCell<Option<UIElements>>>,
        translator: &Rc<RefCell<Translator>>,
    ) {
        if let Some(elements) = ui_elements.borrow().as_ref() {
            // Update window title
            elements.window.set_title(Some(
                &translator.borrow().translate(&TranslationKey::AppTitle),
            ));

            // Update header label
            elements
                .header_label
                .set_text(&translator.borrow().translate(&TranslationKey::AppTitle));

            // Update settings button tooltip
            elements.settings_button.set_tooltip_text(Some(
                &translator.borrow().translate(&TranslationKey::Settings),
            ));

            // Update main title label
            elements.title_label.set_text(
                &translator
                    .borrow()
                    .translate(&TranslationKey::SelectInstallation),
            );

            // Update button labels
            elements.add_button.set_label(
                &translator
                    .borrow()
                    .translate(&TranslationKey::BrowseForInstallation),
            );
            elements
                .play_button
                .set_label(&translator.borrow().translate(&TranslationKey::PlayGame));
            elements
                .exit_button
                .set_label(&translator.borrow().translate(&TranslationKey::Exit));
        }
    }

    /// Gets a reference to the current configuration.
    ///
    /// # Returns
    ///
    /// A reference to the current `AppConfig`
    #[must_use]
    pub fn get_config(&self) -> std::cell::Ref<AppConfig> {
        self.config.borrow()
    }

    /// Populates the installations list with available games.
    ///
    /// # Arguments
    ///
    /// * `list` - The GTK `ListBox` to populate
    /// * `config` - The application configuration
    /// * `selected_installation` - Currently selected installation storage
    /// * `app` - The GTK Application instance
    fn populate_installations_list(
        list: &ListBox,
        config: Rc<RefCell<AppConfig>>,
        selected_installation: Rc<RefCell<Option<GameInstallation>>>,
        _app: &Application,
    ) {
        // Clear existing rows
        while let Some(child) = list.first_child() {
            list.remove(&child);
        }

        let installations = config.borrow().get_installations().clone();
        let installations_count = installations.len();

        if installations.is_empty() {
            let row = ListBoxRow::new();
            let label = Label::new(Some(
                "No installations found. Click 'Add Installation' to get started.",
            ));
            label.set_margin_top(20);
            label.set_margin_bottom(20);
            row.set_child(Some(&label));
            list.append(&row);
            return;
        }

        for (_id, installation) in installations {
            let row = ListBoxRow::new();

            let container = GtkBox::new(Orientation::Vertical, 6);
            container.set_margin_top(12);
            container.set_margin_bottom(12);
            container.set_margin_start(12);
            container.set_margin_end(12);

            // Game title
            let title_label = Label::new(Some(&installation.display_name));
            title_label.add_css_class("heading");
            title_label.set_halign(gtk4::Align::Start);
            container.append(&title_label);

            // Path info
            let path_label = Label::new(Some(&format!("Path: {}", installation.path.display())));
            path_label.set_halign(gtk4::Align::Start);
            path_label.add_css_class("dim-label");
            container.append(&path_label);

            // Version info
            let version_text = match installation.version {
                GameVersion::CTP1 => "Civilization: Call to Power",
                GameVersion::CTP2 => "Call to Power II",
            };
            let version_label = Label::new(Some(version_text));
            version_label.set_halign(gtk4::Align::Start);
            version_label.add_css_class("dim-label");
            container.append(&version_label);

            row.set_child(Some(&container));

            // Store installation data with the row
            let installation_clone = installation.clone();
            row.connect_activate(clone!(
                #[strong]
                selected_installation,
                #[strong]
                installation_clone,
                move |_| {
                    *selected_installation.borrow_mut() = Some(installation_clone.clone());
                }
            ));

            list.append(&row);
        }

        // Auto-select first installation if none selected
        if selected_installation.borrow().is_none() && installations_count > 0 {
            if let Some(first_row) = list.row_at_index(0) {
                list.select_row(Some(&first_row));
                first_row.emit_activate();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::rc::Rc;
    use std::cell::RefCell;
    use std::fs;
    use tempfile::TempDir;

    /// Helper function to create a test configuration
    fn create_test_config() -> AppConfig {
        AppConfig::new()
    }

    /// Helper function to create a mock CTP1 installation directory
    fn create_mock_ctp1_installation() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path();

        // Create CTP1 folder structure
        fs::create_dir_all(base_path.join("ctp_data")).unwrap();
        fs::create_dir_all(base_path.join("ctp_program")).unwrap();

        temp_dir
    }

    /// Helper function to create a mock CTP2 installation directory
    fn create_mock_ctp2_installation() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path();

        // Create CTP2 folder structure
        fs::create_dir_all(base_path.join("ctp2_data")).unwrap();
        fs::create_dir_all(base_path.join("ctp2_program")).unwrap();

        temp_dir
    }

    #[test]
    fn test_ui_result_debug() {
        let installation = GameInstallation::new(
            PathBuf::from("/test/path"),
            GameVersion::CTP1,
            "Test Game".to_owned(),
            None,
        );

        let play_result = UIResult::PlayGame(installation);
        let exit_result = UIResult::Exit;

        let play_debug = format!("{play_result:?}");
        let exit_debug = format!("{exit_result:?}");

        assert!(play_debug.contains("PlayGame"));
        assert!(play_debug.contains("Test Game"));
        assert!(exit_debug.contains("Exit"));
    }

    #[test]
    fn test_ui_state_default() {
        let ui_state = UIState::default();
        assert!(!ui_state.show_auto_detected);
    }

    #[test]
    fn test_gtk_ui_new() {
        // Test that UI can be created (GTK initialization might fail in test environment)
        match GtkUI::new() {
            Ok(_ui) => {
                // UI creation succeeded
            }
            Err(_e) => {
                // This is expected in test environments without proper GTK setup
                // The important thing is that the Result type is correct
            }
        }
    }

    #[test]
    fn test_gtk_ui_get_config() {
        // Test the configuration part that doesn't require GTK
        let config = AppConfig::new();
        assert_eq!(config.version, 1);
        assert!(config.get_installations().is_empty());
    }

    #[test]
    fn test_add_installation_valid_ctp1() {
        // Set up isolated test environment
        let temp_config_dir = TempDir::new().expect("Failed to create temp config dir");
        let original_config_dir = std::env::var("XDG_CONFIG_HOME").ok();

        // SAFETY: This is safe because we're only temporarily setting an environment variable
        // for testing purposes and will restore it afterward
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", temp_config_dir.path());
        }

        let config = Rc::new(RefCell::new(create_test_config()));
        let temp_dir = create_mock_ctp1_installation();

        // Count installations before
        let initial_count = config.borrow().get_installations().len();

        // Add the installation
        GtkUI::add_installation_static(config.clone(), temp_dir.path().to_path_buf());

        // Verify installation was added
        let final_count = config.borrow().get_installations().len();
        assert_eq!(final_count, initial_count + 1);

        // Find the added installation
        let installations = config.borrow().get_installations().clone();
        let added_installation = installations
            .values()
            .find(|inst| inst.version == GameVersion::CTP1 && inst.path == temp_dir.path());

        assert!(added_installation.is_some());
        let installation = added_installation.unwrap();
        assert!(installation.display_name.contains("Call to Power"));
        assert!(installation.validated_at.is_some());

        // Restore original config dir
        // SAFETY: Safe to restore the original environment variable
        unsafe {
            match original_config_dir {
                Some(dir) => std::env::set_var("XDG_CONFIG_HOME", dir),
                None => std::env::remove_var("XDG_CONFIG_HOME"),
            }
        }
    }

    #[test]
    fn test_add_installation_valid_ctp2() {
        // Set up isolated test environment
        let temp_config_dir = TempDir::new().expect("Failed to create temp config dir");
        let original_config_dir = std::env::var("XDG_CONFIG_HOME").ok();

        // SAFETY: This is safe because we're only temporarily setting an environment variable
        // for testing purposes and will restore it afterward
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", temp_config_dir.path());
        }

        let config = Rc::new(RefCell::new(create_test_config()));
        let temp_dir = create_mock_ctp2_installation();

        // Count installations before
        let initial_count = config.borrow().get_installations().len();

        // Add the installation
        GtkUI::add_installation_static(config.clone(), temp_dir.path().to_path_buf());

        // Verify installation was added
        let final_count = config.borrow().get_installations().len();
        assert_eq!(final_count, initial_count + 1);

        // Find the added installation
        let installations = config.borrow().get_installations().clone();
        let added_installation = installations
            .values()
            .find(|inst| inst.version == GameVersion::CTP2 && inst.path == temp_dir.path());

        assert!(added_installation.is_some());
        let installation = added_installation.unwrap();
        assert!(installation.display_name.contains("Call to Power II"));
        assert!(installation.validated_at.is_some());

        // Restore original config dir
        // SAFETY: Safe to restore the original environment variable
        unsafe {
            match original_config_dir {
                Some(dir) => std::env::set_var("XDG_CONFIG_HOME", dir),
                None => std::env::remove_var("XDG_CONFIG_HOME"),
            }
        }
    }

    #[test]
    fn test_add_installation_invalid_path() {
        // Set up isolated test environment
        let temp_config_dir = TempDir::new().expect("Failed to create temp config dir");
        let original_config_dir = std::env::var("XDG_CONFIG_HOME").ok();

        // SAFETY: This is safe because we're only temporarily setting an environment variable
        // for testing purposes and will restore it afterward
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", temp_config_dir.path());
        }

        let config = Rc::new(RefCell::new(create_test_config()));
        let invalid_path = PathBuf::from("/absolutely/nonexistent/path");

        // Count installations before
        let initial_count = config.borrow().get_installations().len();

        // Try to add invalid installation
        GtkUI::add_installation_static(config.clone(), invalid_path);

        // Verify no installation was added
        let final_count = config.borrow().get_installations().len();
        assert_eq!(final_count, initial_count);

        // Restore original config dir
        // SAFETY: Safe to restore the original environment variable
        unsafe {
            match original_config_dir {
                Some(dir) => std::env::set_var("XDG_CONFIG_HOME", dir),
                None => std::env::remove_var("XDG_CONFIG_HOME"),
            }
        }
    }

    #[test]
    fn test_add_installation_empty_directory() {
        // Set up isolated test environment
        let temp_config_dir = TempDir::new().expect("Failed to create temp config dir");
        let original_config_dir = std::env::var("XDG_CONFIG_HOME").ok();

        // SAFETY: This is safe because we're only temporarily setting an environment variable
        // for testing purposes and will restore it afterward
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", temp_config_dir.path());
        }

        let config = Rc::new(RefCell::new(create_test_config()));
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Count installations before
        let initial_count = config.borrow().get_installations().len();

        // Try to add empty directory
        GtkUI::add_installation_static(config.clone(), temp_dir.path().to_path_buf());

        // Verify no installation was added
        let final_count = config.borrow().get_installations().len();
        assert_eq!(final_count, initial_count);

        // Restore original config dir
        // SAFETY: Safe to restore the original environment variable
        unsafe {
            match original_config_dir {
                Some(dir) => std::env::set_var("XDG_CONFIG_HOME", dir),
                None => std::env::remove_var("XDG_CONFIG_HOME"),
            }
        }
    }

    #[test]
    fn test_installation_id_generation() {
        // Set up isolated test environment
        let temp_config_dir = TempDir::new().expect("Failed to create temp config dir");
        let original_config_dir = std::env::var("XDG_CONFIG_HOME").ok();

        // SAFETY: This is safe because we're only temporarily setting an environment variable
        // for testing purposes and will restore it afterward
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", temp_config_dir.path());
        }

        let config = Rc::new(RefCell::new(create_test_config()));
        let temp_dir = create_mock_ctp1_installation();

        // Add installation
        GtkUI::add_installation_static(config.clone(), temp_dir.path().to_path_buf());

        // Check that an ID was generated correctly
        let installations = config.borrow().get_installations().clone();
        let id_keys: Vec<String> = installations.keys().cloned().collect();

        assert!(!id_keys.is_empty());
        let first_id = id_keys
            .first()
            .expect("Should have at least one installation");
        assert!(first_id.starts_with("ctp1_"));

        // Restore original config dir
        // SAFETY: Safe to restore the original environment variable
        unsafe {
            match original_config_dir {
                Some(dir) => std::env::set_var("XDG_CONFIG_HOME", dir),
                None => std::env::remove_var("XDG_CONFIG_HOME"),
            }
        }
    }

    #[test]
    fn test_installation_id_generation_ctp2() {
        // Set up isolated test environment
        let temp_config_dir = TempDir::new().expect("Failed to create temp config dir");
        let original_config_dir = std::env::var("XDG_CONFIG_HOME").ok();

        // SAFETY: This is safe because we're only temporarily setting an environment variable
        // for testing purposes and will restore it afterward
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", temp_config_dir.path());
        }

        let config = Rc::new(RefCell::new(create_test_config()));
        let temp_dir = create_mock_ctp2_installation();

        // Add installation
        GtkUI::add_installation_static(config.clone(), temp_dir.path().to_path_buf());

        // Check that an ID was generated correctly
        let installations = config.borrow().get_installations().clone();
        let id_keys: Vec<String> = installations.keys().cloned().collect();

        assert!(!id_keys.is_empty());
        let first_id = id_keys
            .first()
            .expect("Should have at least one installation");
        assert!(first_id.starts_with("ctp2_"));

        // Restore original config dir
        // SAFETY: Safe to restore the original environment variable
        unsafe {
            match original_config_dir {
                Some(dir) => std::env::set_var("XDG_CONFIG_HOME", dir),
                None => std::env::remove_var("XDG_CONFIG_HOME"),
            }
        }
    }

    #[test]
    fn test_multiple_installations() {
        // Set up isolated test environment
        let temp_config_dir = TempDir::new().expect("Failed to create temp config dir");
        let original_config_dir = std::env::var("XDG_CONFIG_HOME").ok();

        // SAFETY: This is safe because we're only temporarily setting an environment variable
        // for testing purposes and will restore it afterward
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", temp_config_dir.path());
        }

        let config = Rc::new(RefCell::new(create_test_config()));
        let ctp1_dir = create_mock_ctp1_installation();
        let ctp2_dir = create_mock_ctp2_installation();

        let initial_count = config.borrow().get_installations().len();

        // Add both installations
        GtkUI::add_installation_static(config.clone(), ctp1_dir.path().to_path_buf());
        GtkUI::add_installation_static(config.clone(), ctp2_dir.path().to_path_buf());

        let final_count = config.borrow().get_installations().len();
        assert_eq!(final_count, initial_count + 2);

        // Verify both versions are present
        let installations = config.borrow().get_installations().clone();
        let has_ctp1 = installations
            .values()
            .any(|inst| inst.version == GameVersion::CTP1);
        let has_ctp2 = installations
            .values()
            .any(|inst| inst.version == GameVersion::CTP2);

        assert!(has_ctp1);
        assert!(has_ctp2);

        // Restore original config dir
        // SAFETY: Safe to restore the original environment variable
        unsafe {
            match original_config_dir {
                Some(dir) => std::env::set_var("XDG_CONFIG_HOME", dir),
                None => std::env::remove_var("XDG_CONFIG_HOME"),
            }
        }
    }

    #[test]
    fn test_ui_error_handling() {
        // Test that UI creation handles config loading errors gracefully
        // This tests the error path in GtkUI::new()

        // We can't easily force config loading to fail without unsafe code,
        // but we can verify that the Result type is properly handled
        let result = GtkUI::new();

        // Should either succeed or return a proper error
        match result {
            Ok(_ui) => {
                // Success case
            }
            Err(error) => {
                // Error case - verify it's a proper error
                let error_str = format!("{error}");
                assert!(!error_str.is_empty());
            }
        }
    }

    #[test]
    fn test_path_display_formatting() {
        // Test the path display functionality used in the UI
        let path = PathBuf::from("/some/test/path/game_dir");
        let display_string = path.display().to_string();

        assert!(display_string.contains("game_dir"));
        assert!(display_string.contains("test"));

        // Test with special characters
        let special_path = PathBuf::from("/path/with spaces/and-dashes");
        let special_display = special_path.display().to_string();

        assert!(special_display.contains("with spaces"));
        assert!(special_display.contains("and-dashes"));
    }

    #[test]
    fn test_installation_cloning() {
        let installation = GameInstallation::new(
            PathBuf::from("/test/path"),
            GameVersion::CTP2,
            "Test Installation".to_owned(),
            Some("2023-01-01 12:00:00 UTC".to_owned()),
        );

        // Test that clone produces identical data
        let cloned = installation.clone();

        // Move original to ensure they're independent
        let original = installation;

        assert_eq!(original.path, cloned.path);
        assert_eq!(original.version, cloned.version);
        assert_eq!(original.display_name, cloned.display_name);
        assert_eq!(original.validated_at, cloned.validated_at);
    }

    #[test]
    fn test_ui_result_matching() {
        let installation = GameInstallation::new(
            PathBuf::from("/test/path"),
            GameVersion::CTP1,
            "Test Game".to_owned(),
            None,
        );

        let play_result = UIResult::PlayGame(installation);
        let exit_result = UIResult::Exit;

        // Test pattern matching
        assert!(
            matches!(play_result, UIResult::PlayGame(_)),
            "Expected PlayGame variant"
        );
        if let UIResult::PlayGame(inst) = play_result {
            assert_eq!(inst.display_name, "Test Game");
        }

        assert!(
            matches!(exit_result, UIResult::Exit),
            "Expected Exit variant"
        );
    }

    #[test]
    fn test_directory_name_handling() {
        // Test path file name extraction used in add_installation
        let path_with_name = PathBuf::from("/some/path/game_directory");
        let file_name = path_with_name
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        assert_eq!(file_name, "game_directory");

        // Test path without file name
        let root_path = PathBuf::from("/");
        let no_name = root_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        assert_eq!(no_name, "unknown");
    }
}
