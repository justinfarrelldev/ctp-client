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

use crate::asset_extraction::{
    ArchiveExtractor, ExtractionError, ExtractionSettings, default_destination_root,
    log_extraction_progress, required_archives,
};
use crate::config::{AppConfig, GameInstallation, GameVersion};
use crate::dialogs;
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

/// Abstraction over intro video playback used by Play Game activation.
pub trait IntroVideoPlayback {
    /// Plays the intro video for the selected game installation.
    fn play_intro_for_installation(
        &self,
        installation: &GameInstallation,
    ) -> crate::video::VideoResult;
}

impl IntroVideoPlayback for crate::video::VideoPlayer {
    fn play_intro_for_installation(
        &self,
        installation: &GameInstallation,
    ) -> crate::video::VideoResult {
        Self::play_intro_video(self, installation)
    }
}

/// Handles the Play Game button's intro-video transition in a testable way.
///
/// # Returns
///
/// The result of the intro video playback request.
#[must_use]
pub fn activate_play_game_button<P>(
    installation: &GameInstallation,
    player: &P,
) -> crate::video::VideoResult
where
    P: IntroVideoPlayback,
{
    player.play_intro_for_installation(installation)
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
    /// CTP1 add installation button.
    ctp1_add_button: Button,
    /// CTP2 add installation button.
    ctp2_add_button: Button,
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

    /// Rebuilds the main UI window to reflect changes (e.g., after adding an installation).
    pub fn refresh_main_ui(&self) {
        // Remove the current window and rebuild UI
        if let Some(elements) = self.ui_elements.borrow().as_ref() {
            elements.window.close();
        }
        // Re-run build_ui with current state
        Self::build_ui(
            &self.app,
            self.config.clone(),
            self.translator.clone(),
            self.found_installations.clone(),
            self.ui_state.clone(),
            self.selected_installation.clone(),
            self.exit_requested.clone(),
            self.ui_elements.clone(),
        );
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
        ui_state: Rc<RefCell<UIState>>,
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
        settings_button.connect_clicked(glib::clone! {
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
        });
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

        // Create horizontal box for two game sections
        let games_box = GtkBox::new(Orientation::Horizontal, 20);
        games_box.set_halign(gtk4::Align::Fill);
        games_box.set_homogeneous(true);

        // Get installations for each game version
        let config_ref = config.borrow();
        let installations = config_ref.get_installations();
        let ctp1_installation = installations
            .values()
            .find(|inst| inst.version == GameVersion::CTP1);
        let ctp2_installation = installations
            .values()
            .find(|inst| inst.version == GameVersion::CTP2);

        // Clone the installations since we need them after dropping the borrow
        let ctp1_installation = ctp1_installation.cloned();
        let ctp2_installation = ctp2_installation.cloned();
        drop(config_ref);

        // CTP1 Section
        let ctp1_box = GtkBox::new(Orientation::Vertical, 12);
        ctp1_box.set_halign(gtk4::Align::Fill);

        let ctp1_title = Label::new(Some(
            &translator.borrow().translate(&TranslationKey::CallToPower1),
        ));
        ctp1_title.add_css_class("title-2");
        ctp1_box.append(&ctp1_title);

        let ctp1_status = ctp1_installation.as_ref().map_or_else(
            || Label::new(Some("Not installed")),
            |installation| {
                Label::new(Some(&format!(
                    "Installed at:\n{}",
                    installation.path.display()
                )))
            },
        );
        ctp1_status.set_wrap(true);
        ctp1_status.set_justify(gtk4::Justification::Center);
        ctp1_status.add_css_class("dim-label");
        ctp1_box.append(&ctp1_status);

        let ctp1_play_button = Button::with_label("Play CTP1");
        ctp1_play_button.add_css_class("suggested-action");
        ctp1_play_button.set_sensitive(ctp1_installation.is_some());

        ctp1_play_button.connect_clicked(glib::clone! {
            #[weak]
            window,
            #[strong]
            ctp1_installation,
            move |_| {
                if let Some(installation) = ctp1_installation.as_ref() {
                    Self::launch_game(&window, installation.clone());
                }
            }
        });
        ctp1_box.append(&ctp1_play_button);

        // Create a refresh_ui closure for dialogs
        let refresh_ui = {
            let app_instance = app.clone();
            let config_rc = Rc::clone(&config);
            let translator_rc = Rc::clone(&translator);
            let found_installations_snapshot = found_installations.clone();
            let ui_state_rc = Rc::clone(&ui_state);
            let selected_installation_rc = Rc::clone(&selected_installation);
            let exit_requested_rc = Rc::clone(&exit_requested);
            let ui_elements_rc = Rc::clone(&ui_elements);
            Rc::new(move || {
                Self::build_ui(
                    &app_instance,
                    Rc::clone(&config_rc),
                    Rc::clone(&translator_rc),
                    found_installations_snapshot.clone(),
                    Rc::clone(&ui_state_rc),
                    Rc::clone(&selected_installation_rc),
                    Rc::clone(&exit_requested_rc),
                    Rc::clone(&ui_elements_rc),
                );
            })
        };

        let ctp1_add_button = Button::with_label(
            &translator
                .borrow()
                .translate(&TranslationKey::BrowseForInstallation),
        );
        ctp1_add_button.connect_clicked(glib::clone! {
            #[weak] window,
            #[strong] config,
            #[strong] found_installations,
            #[strong] refresh_ui,
            move |_| {
                Self::show_add_installation_dialog_for_version(
                    &window,
                    config.clone(),
                    found_installations.clone(),
                    GameVersion::CTP1,
                    refresh_ui.clone(),
                );
            }
        });
        ctp1_box.append(&ctp1_add_button);

        games_box.append(&ctp1_box);

        // Add separator
        let separator = gtk4::Separator::new(Orientation::Vertical);
        games_box.append(&separator);

        // CTP2 Section
        let ctp2_box = GtkBox::new(Orientation::Vertical, 12);
        ctp2_box.set_halign(gtk4::Align::Fill);

        let ctp2_title = Label::new(Some(
            &translator.borrow().translate(&TranslationKey::CallToPower2),
        ));
        ctp2_title.add_css_class("title-2");
        ctp2_box.append(&ctp2_title);

        let ctp2_status = ctp2_installation.as_ref().map_or_else(
            || Label::new(Some("Not installed")),
            |installation| {
                Label::new(Some(&format!(
                    "Installed at:\n{}",
                    installation.path.display()
                )))
            },
        );
        ctp2_status.set_wrap(true);
        ctp2_status.set_justify(gtk4::Justification::Center);
        ctp2_status.add_css_class("dim-label");
        ctp2_box.append(&ctp2_status);

        let ctp2_play_button = Button::with_label("Play CTP2");
        ctp2_play_button.add_css_class("suggested-action");
        ctp2_play_button.set_sensitive(ctp2_installation.is_some());

        ctp2_play_button.connect_clicked(clone!(
            #[weak]
            window,
            #[strong]
            ctp2_installation,
            move |_| {
                if let Some(installation) = ctp2_installation.as_ref() {
                    Self::launch_game(&window, installation.clone());
                }
            }
        ));
        ctp2_box.append(&ctp2_play_button);

        let ctp2_add_button = Button::with_label(
            &translator
                .borrow()
                .translate(&TranslationKey::BrowseForInstallation),
        );
        ctp2_add_button.connect_clicked(clone!(
            #[weak]
            window,
            #[strong]
            config,
            #[strong]
            found_installations,
            #[strong]
            refresh_ui,
            move |_| {
                Self::show_add_installation_dialog_for_version(
                    &window,
                    config.clone(),
                    found_installations.clone(),
                    GameVersion::CTP2,
                    refresh_ui.clone(),
                );
            }
        ));
        ctp2_box.append(&ctp2_add_button);

        games_box.append(&ctp2_box);

        main_box.append(&games_box);

        // Exit button container
        let exit_box = GtkBox::new(Orientation::Horizontal, 12);
        exit_box.set_halign(gtk4::Align::Center);

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
        exit_box.append(&exit_button);

        main_box.append(&exit_box);

        window.set_child(Some(&main_box));

        // Store UI elements for language updates
        *ui_elements.borrow_mut() = Some(UIElements {
            window: window.clone(),
            header_label,
            settings_button,
            title_label,
            ctp1_add_button,
            ctp2_add_button,
            exit_button,
        });

        window.present();
    }

    /// Shows the add installation dialog with auto-detected and browse options.
    /// Now takes a `refresh_ui` closure to call after a successful add.
    fn show_add_installation_dialog(
        parent: &ApplicationWindow,
        config: Rc<RefCell<AppConfig>>,
        found_installations: Vec<PathBuf>,
        refresh_ui: Rc<dyn Fn()>,
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

        Self::append_installation_list(
            &main_box,
            dialog.clone(),
            config.clone(),
            refresh_ui.clone(),
            found_installations,
            "No installations auto-detected.",
            "Auto-detected installations:",
        );

        // Browse button
        let browse_button = Button::with_label("Browse for Installation...");
        browse_button.connect_clicked(glib::clone! {
            #[weak] dialog,
            #[strong] config,
            #[strong] refresh_ui,
            move |_button| {
                let (sender, receiver) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    log::info!("Opening rfd file dialog...");
                    let result = rfd::FileDialog::new()
                        .set_title("Select Call to Power Installation Directory")
                        .pick_folder();
                    if sender.send(result).is_err() {
                        log::error!("Failed to send file dialog result");
                    }
                });
                let config_for_timeout = config.clone();
                let refresh_ui_for_timeout = refresh_ui.clone();
                let dialog_for_timeout = dialog;
                glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                    receiver
                        .try_recv()
                        .map_or(glib::ControlFlow::Continue, |result| {
                            if let Some(path) = result {
                                Self::add_installation_static(config_for_timeout.clone(), path);
                                dialog_for_timeout.close();
                                (refresh_ui_for_timeout)();
                            }
                            glib::ControlFlow::Break
                        })
                });
            }
        });
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

    /// Shows the add installation dialog for a specific game version.
    /// Now takes a `refresh_ui` closure to call after a successful add.
    fn show_add_installation_dialog_for_version(
        parent: &ApplicationWindow,
        config: Rc<RefCell<AppConfig>>,
        found_installations: Vec<PathBuf>,
        version: GameVersion,
        refresh_ui: Rc<dyn Fn()>,
    ) {
        let version_name = match version {
            GameVersion::CTP1 => "Call to Power",
            GameVersion::CTP2 => "Call to Power II",
        };

        let dialog = gtk4::Dialog::builder()
            .title(format!("Add {version_name} Installation"))
            .modal(true)
            .transient_for(parent)
            .build();

        let content_area = dialog.content_area();
        let main_box = GtkBox::new(Orientation::Vertical, 12);
        main_box.set_margin_top(20);
        main_box.set_margin_bottom(20);
        main_box.set_margin_start(20);
        main_box.set_margin_end(20);

        // Filter found installations for this version
        let filtered_installations: Vec<PathBuf> = found_installations
            .into_iter()
            .filter(|path| {
                // Check if this path contains the required folders for this version
                let (data_folder, program_folder) = version.required_folders();
                path.join(data_folder).exists() && path.join(program_folder).exists()
            })
            .collect();

        Self::append_installation_list(
            &main_box,
            dialog.clone(),
            config.clone(),
            refresh_ui.clone(),
            filtered_installations,
            &format!("No {version_name} installations auto-detected."),
            &format!("Auto-detected {version_name} installations:"),
        );

        // Browse button
        let browse_button =
            Button::with_label(&format!("Browse for {version_name} Installation..."));
        browse_button.connect_clicked(glib::clone! {
            #[weak] dialog,
            #[strong] config,
            #[strong] refresh_ui,
            move |_button| {
                let (sender, receiver) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    log::info!("Opening rfd file dialog for {version_name}...");
                    let result = rfd::FileDialog::new()
                        .set_title(format!("Select {version_name} Installation Directory"))
                        .pick_folder();
                    if sender.send(result).is_err() {
                        log::error!("Failed to send file dialog result");
                    }
                });
                let config_for_timeout = config.clone();
                let refresh_ui_for_timeout = refresh_ui.clone();
                let dialog_for_timeout = dialog;
                glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                    receiver
                        .try_recv()
                        .map_or(glib::ControlFlow::Continue, |result| {
                            if let Some(path) = result {
                                Self::add_installation_static(config_for_timeout.clone(), path);
                                dialog_for_timeout.close();
                                (refresh_ui_for_timeout)();
                            }
                            glib::ControlFlow::Break
                        })
                });
            }
        });
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

    /// Adds auto-detected installation options to the dialog body, including the
    /// scrollable list that allows the user to pick a detected installation.
    fn append_installation_list(
        main_box: &GtkBox,
        dialog: gtk4::Dialog,
        config: Rc<RefCell<AppConfig>>,
        refresh_ui: Rc<dyn Fn()>,
        installations: Vec<PathBuf>,
        empty_message: &str,
        header_message: &str,
    ) {
        if installations.is_empty() {
            main_box.append(&Label::new(Some(empty_message)));
            return;
        }

        let header_label = Label::new(Some(header_message));
        main_box.append(&header_label);

        let list = ListBox::new();
        for path in installations {
            let row = ListBoxRow::new();
            let path_label = Label::new(Some(&path.display().to_string()));
            path_label.set_margin_top(6);
            path_label.set_margin_bottom(6);
            path_label.set_margin_start(12);
            path_label.set_margin_end(12);
            row.set_child(Some(&path_label));

            let path_for_activation = path.clone();
            row.connect_activate(glib::clone! {
                #[strong] config,
                #[weak] dialog,
                #[strong] refresh_ui,
                move |_| {
                    Self::add_installation_static(Rc::clone(&config), path_for_activation.clone());
                    dialog.close();
                    (refresh_ui)();
                }
            });

            list.append(&row);
        }

        let scrolled = ScrolledWindow::builder().height_request(200).build();
        scrolled.set_child(Some(&list));
        main_box.append(&scrolled);
    }

    /// Static method to add an installation (used in GTK callbacks).
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
            elements.ctp1_add_button.set_label(
                &translator
                    .borrow()
                    .translate(&TranslationKey::BrowseForInstallation),
            );
            elements.ctp2_add_button.set_label(
                &translator
                    .borrow()
                    .translate(&TranslationKey::BrowseForInstallation),
            );
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

    /// Handles game launching by directly starting the intro video.
    ///
    /// # Arguments
    ///
    /// * `parent` - The parent window
    /// * `installation` - The game installation to launch
    fn launch_game(parent: &ApplicationWindow, installation: GameInstallation) {
        log::info!(
            "Launching game: {} at {}",
            installation.display_name,
            installation.path.display()
        );

        let destination = match default_destination_root(&installation) {
            Ok(path) => path,
            Err(error) => {
                log::error!("Failed to resolve extraction target: {error}");
                Self::show_extraction_error_dialog(parent, &error);
                return;
            }
        };

        let extractor = ArchiveExtractor::new(&installation);
        if let Err(error) = extractor.validate() {
            log::error!("Archive validation failed: {error}");
            Self::show_extraction_error_dialog(parent, &error);
            return;
        }

        let archive_count = required_archives(installation.version).len();
        if !Self::prompt_extraction_confirmation(
            parent,
            &destination,
            archive_count,
            &installation.display_name,
        ) {
            log::info!("User canceled extraction for {}", installation.display_name);
            return;
        }

        let mut progress = log_extraction_progress;

        match extractor.extract_with_settings(
            &destination,
            ExtractionSettings::default(),
            Some(&mut progress),
        ) {
            Ok(outcome) => {
                if outcome.already_up_to_date {
                    log::info!(
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
                    log::info!(
                        "Extracted {} archives ({} files, {} bytes) to {}",
                        outcome.archives.len(),
                        files_written,
                        bytes_written,
                        destination.display()
                    );
                }
            }
            Err(error) => {
                log::error!("Extraction failed: {error}");
                Self::show_extraction_error_dialog(parent, &error);
                return;
            }
        }

        // Play intro video directly
        match crate::video::VideoPlayer::new() {
            Ok(video_player) => match activate_play_game_button(&installation, &video_player) {
                crate::video::VideoResult::Completed => {
                    log::info!("Intro video completed successfully");
                }
                crate::video::VideoResult::Skipped => {
                    log::info!("Intro video skipped by user");
                }
                crate::video::VideoResult::Failed(error) => {
                    log::warn!("Failed to play intro video: {error}");
                }
            },
            Err(e) => {
                log::warn!("Failed to initialize video player: {e}");
            }
        }

        // Close the application window after video playback
        parent.close();
    }

    /// Prompts the user before copying installation archives to managed storage.
    fn prompt_extraction_confirmation(
        parent: &ApplicationWindow,
        destination: &std::path::Path,
        archive_count: usize,
        installation_name: &str,
    ) -> bool {
        let text =
            dialogs::extraction_confirmation_text(destination, archive_count, installation_name);
        dialogs::run_confirmation_dialog(Some(parent.upcast_ref::<gtk4::Window>()), &text)
    }

    /// Displays a user-facing extraction error message.
    fn show_extraction_error_dialog(parent: &ApplicationWindow, error: &ExtractionError) {
        let text = dialogs::extraction_error_text(error);
        dialogs::run_message_dialog(Some(parent.upcast_ref::<gtk4::Window>()), &text);
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
                // Don't emit_activate() here as it can cause focus issues before window is ready
                // Instead, just set the selection data directly
                if let Some(first_installation) =
                    config.borrow().get_installations().values().next()
                {
                    *selected_installation.borrow_mut() = Some(first_installation.clone());
                }
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

    /// Mock video player used to verify Play Game activation wiring.
    struct RecordingVideoPlayer {
        /// Installations submitted for intro playback.
        calls: RefCell<Vec<GameInstallation>>,
        /// Result returned to the activation path.
        result: crate::video::VideoResult,
    }

    impl RecordingVideoPlayer {
        /// Creates a mock player with the supplied playback result.
        fn new(result: crate::video::VideoResult) -> Self {
            Self {
                calls: RefCell::new(Vec::new()),
                result,
            }
        }
    }

    impl IntroVideoPlayback for RecordingVideoPlayer {
        fn play_intro_for_installation(
            &self,
            installation: &GameInstallation,
        ) -> crate::video::VideoResult {
            self.calls.borrow_mut().push(installation.clone());
            self.result.clone()
        }
    }

    #[test]
    fn play_game_button_activation_invokes_intro_video() {
        let installation = GameInstallation::new(
            PathBuf::from("/test/ctp2"),
            GameVersion::CTP2,
            "Test CTP2".to_owned(),
            None,
        );
        let player = RecordingVideoPlayer::new(crate::video::VideoResult::Skipped);

        let result = activate_play_game_button(&installation, &player);

        assert_eq!(result, crate::video::VideoResult::Skipped);
        let calls = player.calls.borrow();
        assert_eq!(
            calls.len(),
            1,
            "Play Game activation must call intro video exactly once"
        );
        assert_eq!(calls[0].display_name, "Test CTP2");
        assert_eq!(calls[0].version, GameVersion::CTP2);
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
