//! Settings UI module for the CTP client.
//!
//! This module provides a settings dialog for configuring user preferences
//! including language selection.

extern crate alloc;

use crate::config::AppConfig;
use crate::i18n::{Language, TranslationKey, Translator};
use alloc::rc::Rc;
use core::cell::RefCell;

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, ComboBoxText, Dialog, Frame, Grid, Label, Orientation, SearchEntry, Window,
};

/// Settings dialog for configuring application preferences.
pub struct SettingsDialog {
    /// Reference to the main application config.
    config: Rc<RefCell<AppConfig>>,
    /// Current translator instance.
    translator: Rc<RefCell<Translator>>,
    /// The GTK dialog widget.
    dialog: Dialog,
    /// Language selection combo box.
    language_combo: ComboBoxText,
    /// Language search entry.
    language_search: SearchEntry,
}

/// Result of the settings dialog.
#[derive(Debug, PartialEq, Eq)]
pub enum SettingsResult {
    /// Settings were saved with language change.
    Saved(Language),
    /// Dialog was cancelled.
    Cancelled,
}

impl SettingsDialog {
    /// Creates a new settings dialog.
    ///
    /// # Arguments
    ///
    /// * `parent` - The parent window
    /// * `config` - Application configuration
    /// * `translator` - Current translator instance
    ///
    /// # Returns
    ///
    /// A new `SettingsDialog` instance
    #[must_use]
    pub fn new(
        parent: &Window,
        config: Rc<RefCell<AppConfig>>,
        translator: Rc<RefCell<Translator>>,
    ) -> Self {
        let dialog = Dialog::builder()
            .title(
                translator
                    .borrow()
                    .translate(&TranslationKey::SettingsTitle),
            )
            .transient_for(parent)
            .modal(true)
            .default_width(400)
            .default_height(300)
            .build();

        let language_combo = ComboBoxText::new();
        let language_search = SearchEntry::builder()
            .placeholder_text(translator.borrow().translate(&TranslationKey::Language))
            .build();

        let settings = Self {
            config,
            translator,
            dialog,
            language_combo,
            language_search,
        };

        settings.setup_ui();
        settings
    }

    /// Sets up the UI elements and layout.
    fn setup_ui(&self) {
        let content_area = self.dialog.content_area();

        // Main container
        let main_box = GtkBox::new(Orientation::Vertical, 12);
        main_box.set_margin_top(12);
        main_box.set_margin_bottom(12);
        main_box.set_margin_start(12);
        main_box.set_margin_end(12);

        // Language section
        let language_frame = Frame::builder()
            .label(
                self.translator
                    .borrow()
                    .translate(&TranslationKey::Language),
            )
            .build();

        let language_grid = Grid::new();
        language_grid.set_row_spacing(6);
        language_grid.set_column_spacing(12);
        language_grid.set_margin_top(12);
        language_grid.set_margin_bottom(12);
        language_grid.set_margin_start(12);
        language_grid.set_margin_end(12);

        // Language description
        let lang_desc = Label::new(Some(
            &self
                .translator
                .borrow()
                .translate(&TranslationKey::LanguageDescription),
        ));
        lang_desc.set_halign(gtk4::Align::Start);
        language_grid.attach(&lang_desc, 0, 0, 2, 1);

        // Language search
        let search_label = Label::new(Some("Search:"));
        search_label.set_halign(gtk4::Align::Start);
        language_grid.attach(&search_label, 0, 1, 1, 1);
        language_grid.attach(&self.language_search, 1, 1, 1, 1);

        // Language combo
        let combo_label = Label::new(Some(&format!(
            "{}:",
            self.translator
                .borrow()
                .translate(&TranslationKey::Language)
        )));
        combo_label.set_halign(gtk4::Align::Start);
        language_grid.attach(&combo_label, 0, 2, 1, 1);
        language_grid.attach(&self.language_combo, 1, 2, 1, 1);

        language_frame.set_child(Some(&language_grid));
        main_box.append(&language_frame);

        content_area.append(&main_box);

        // Populate language combo
        self.populate_languages();

        // Set current language
        let current_lang = &self.config.borrow().language;
        self.set_selected_language(current_lang);

        // Setup search functionality
        self.setup_language_search();

        // Add action buttons
        self.add_action_buttons();
    }

    /// Populates the language combo box with available languages.
    fn populate_languages(&self) {
        self.language_combo.remove_all();

        for language in Language::all() {
            let display_text = format!("{} ({})", language.display_name(), language.code());
            self.language_combo
                .append(Some(language.code()), &display_text);
        }
    }

    /// Sets the selected language in the combo box.
    fn set_selected_language(&self, language: &Language) {
        self.language_combo.set_active_id(Some(language.code()));
    }

    /// Sets up language search functionality.
    fn setup_language_search(&self) {
        let combo = self.language_combo.clone();

        self.language_search
            .connect_search_changed(move |search_entry| {
                let search_text = search_entry.text().to_lowercase();
                combo.remove_all();

                for language in Language::all() {
                    let display_name = language.display_name().to_lowercase();
                    let code = language.code().to_lowercase();

                    if search_text.is_empty()
                        || display_name.contains(&search_text)
                        || code.contains(&search_text)
                    {
                        let display_text =
                            format!("{} ({})", language.display_name(), language.code());
                        combo.append(Some(language.code()), &display_text);
                    }
                }
            });
    }

    /// Adds action buttons (Save, Cancel) to the dialog.
    fn add_action_buttons(&self) {
        // Add Save button
        let save_button = self.dialog.add_button(
            &self.translator.borrow().translate(&TranslationKey::Save),
            gtk4::ResponseType::Ok,
        );
        save_button.add_css_class("suggested-action");

        // Add Cancel button
        self.dialog.add_button(
            &self.translator.borrow().translate(&TranslationKey::Cancel),
            gtk4::ResponseType::Cancel,
        );

        // The response handling is now done in the run() method
    }

    /// Shows the settings dialog and handles the result via callback.
    ///
    /// # Arguments
    ///
    /// * `callback` - Function to call when dialog is closed with the result
    pub fn show<F>(&self, callback: F)
    where
        F: Fn(SettingsResult) + 'static,
    {
        let config = Rc::<RefCell<AppConfig>>::clone(&self.config);
        let combo = self.language_combo.clone();

        // Connect to response signal
        self.dialog.connect_response(move |dialog, response| {
            let result = match response {
                gtk4::ResponseType::Ok => {
                    // Handle language change
                    combo
                        .active_id()
                        .map_or(SettingsResult::Cancelled, |active_id| {
                            Language::from_locale(&active_id).map_or(
                                SettingsResult::Cancelled,
                                |selected_language| {
                                    // Update config
                                    config.borrow_mut().language = selected_language.clone();
                                    SettingsResult::Saved(selected_language)
                                },
                            )
                        })
                }
                _ => SettingsResult::Cancelled,
            };

            dialog.close();
            callback(result);
        });

        self.dialog.present();
    }

    /// Gets the current language setting.
    #[must_use]
    pub fn current_language(&self) -> Language {
        self.config.borrow().language.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::i18n::{Language, Translator};
    use alloc::rc::Rc;
    use std::cell::RefCell;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn ensure_gtk_init() {
        INIT.call_once(|| {
            gtk4::init().expect("Failed to initialize GTK");
        });
    }

    // Helper function to create test window
    fn create_test_window() -> Window {
        ensure_gtk_init();

        let app = gtk4::Application::builder()
            .application_id("com.test.ctp-client")
            .build();

        // Register the application to trigger startup signal
        app.register(gtk4::gio::Cancellable::NONE).ok();

        Window::builder()
            .application(&app)
            .title("Test Window")
            .build()
    }

    #[test]
    fn test_settings_dialog_creation() {
        // This test will be skipped in CI environments without display
        if std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err() {
            return;
        }

        let window = create_test_window();
        let config = Rc::new(RefCell::new(AppConfig::new()));
        let translator = Rc::new(RefCell::new(Translator::new()));

        let settings = SettingsDialog::new(&window, config, translator);

        // Test that dialog was created
        assert_eq!(settings.dialog.title().unwrap(), "Settings");
    }

    #[test]
    fn test_language_selection_logic() {
        // Test language selection without GTK (unit test)
        let languages = Language::all();
        assert!(!languages.is_empty());

        // Test that all languages have codes
        for lang in languages {
            assert!(!lang.code().is_empty());
            assert!(!lang.display_name().is_empty());
        }
    }

    #[test]
    fn test_settings_result() {
        // Test SettingsResult enum
        let saved_result = SettingsResult::Saved(Language::English);
        let cancelled_result = SettingsResult::Cancelled;

        assert!(matches!(saved_result, SettingsResult::Saved(_)));
        assert!(matches!(cancelled_result, SettingsResult::Cancelled));

        if let SettingsResult::Saved(lang) = saved_result {
            assert_eq!(lang, Language::English);
        }
    }

    #[test]
    fn test_language_search_logic() {
        // Test the search logic without GTK
        let languages = Language::all();
        let search_text = "en";

        let matching_languages: Vec<_> = languages
            .into_iter()
            .filter(|lang| {
                let display_name = lang.display_name().to_lowercase();
                let code = lang.code().to_lowercase();
                display_name.contains(search_text) || code.contains(search_text)
            })
            .collect();

        // Should find English at minimum
        assert!(!matching_languages.is_empty());
        assert!(matching_languages.contains(&Language::English));
    }

    #[test]
    fn test_config_language_update() {
        let config = Rc::new(RefCell::new(AppConfig::new()));
        let initial_language = config.borrow().language.clone();

        // Change language
        let new_language = if initial_language == Language::English {
            Language::Spanish
        } else {
            Language::English
        };

        config.borrow_mut().language = new_language.clone();
        assert_eq!(config.borrow().language, new_language);
    }

    #[test]
    fn test_translator_language_update() {
        let mut translator = Translator::new();

        // Change to Spanish
        translator.set_language(Language::Spanish);
        assert_eq!(translator.current_language(), &Language::Spanish);

        // Verify translation changed
        let play_game_text = translator.translate(&TranslationKey::PlayGame);
        assert_eq!(play_game_text, "Jugar");
    }

    #[test]
    fn test_settings_dialog_language_persistence() {
        // Test that language changes persist in config
        let mut config = AppConfig::new();
        let original_language = config.language;

        // Simulate changing language
        config.language = Language::French;
        assert_eq!(config.language, Language::French);
        assert_ne!(config.language, original_language);

        // Test that config can be serialized/deserialized with language
        let serialized = toml::to_string(&config).expect("Should serialize");
        assert!(serialized.contains("language"));

        let deserialized: AppConfig = toml::from_str(&serialized).expect("Should deserialize");
        assert_eq!(deserialized.language, Language::French);
    }
}
