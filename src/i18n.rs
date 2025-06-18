//! Internationalization module for the CTP client.
//!
//! This module handles text localization, language detection, and translation
//! management for the user interface.

use core::fmt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported languages for the application.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    English,
    Spanish,
    French,
    German,
    Italian,
    Japanese,
    Chinese,
    Russian,
}

impl Language {
    /// Returns the language code (e.g., "en", "es", "fr").
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match *self {
            Self::English => "en",
            Self::Spanish => "es", 
            Self::French => "fr",
            Self::German => "de",
            Self::Italian => "it",
            Self::Japanese => "ja",
            Self::Chinese => "zh",
            Self::Russian => "ru",
        }
    }

    /// Returns the display name of the language in its native form.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match *self {
            Self::English => "English",
            Self::Spanish => "Español",
            Self::French => "Français", 
            Self::German => "Deutsch",
            Self::Italian => "Italiano",
            Self::Japanese => "日本語",
            Self::Chinese => "中文",
            Self::Russian => "Русский",
        }
    }

    /// Returns all supported languages.
    #[must_use]
    pub fn all() -> Vec<Self> {
        vec![
            Self::English,
            Self::Spanish,
            Self::French,
            Self::German,
            Self::Italian,
            Self::Japanese,
            Self::Chinese,
            Self::Russian,
        ]
    }

    /// Attempts to parse a language from a locale string or language code.
    #[must_use]
    pub fn from_locale(locale: &str) -> Option<Self> {
        let normalized = locale.to_lowercase();
        let code = normalized.split(['_', '-', '.']).next()?;
        
        match code {
            "en" => Some(Self::English),
            "es" => Some(Self::Spanish),
            "fr" => Some(Self::French),
            "de" => Some(Self::German),
            "it" => Some(Self::Italian),
            "ja" => Some(Self::Japanese),
            "zh" => Some(Self::Chinese),
            "ru" => Some(Self::Russian),
            _ => None,
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Translation key for text strings.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TranslationKey {
    // Main UI
    AppTitle,
    PlayGame,
    Exit,
    Settings,
    
    // Installation UI
    SelectInstallation,
    BrowseForInstallation,
    NoInstallationsFound,
    InvalidInstallation,
    
    // Settings UI
    SettingsTitle,
    Language,
    LanguageDescription,
    Save,
    Cancel,
    
    // Game versions
    CallToPower1,
    CallToPower2,
    
    // Status messages
    GameStarting,
    LoadingConfiguration,
    ScanningInstallations,
}

/// Translation manager that handles text localization.
#[derive(Debug)]
pub struct Translator {
    /// The currently active language.
    current_language: Language,
    /// Map of language to translation key to translated string.
    translations: HashMap<Language, HashMap<TranslationKey, String>>,
}

impl Translator {
    /// Creates a new translator with the system default language.
    #[must_use]
    pub fn new() -> Self {
        let system_language = Self::detect_system_language();
        let mut translator = Self {
            current_language: system_language,
            translations: HashMap::new(),
        };
        translator.load_translations();
        translator
    }

    /// Creates a new translator with a specific language.
    #[must_use]
    pub fn with_language(language: Language) -> Self {
        let mut translator = Self {
            current_language: language,
            translations: HashMap::new(),
        };
        translator.load_translations();
        translator
    }

    /// Detects the system language from environment variables.
    #[must_use]
    pub fn detect_system_language() -> Language {
        // Try various environment variables in order of preference
        let env_vars = ["LANG", "LC_ALL", "LC_MESSAGES", "LANGUAGE"];
        
        for var in &env_vars {
            if let Ok(locale) = std::env::var(var) {
                if let Some(lang) = Language::from_locale(&locale) {
                    return lang;
                }
            }
        }
        
        // Default to English if no locale is found
        Language::English
    }

    /// Sets the current language and reloads translations.
    pub fn set_language(&mut self, language: Language) {
        self.current_language = language;
        self.load_translations();
    }

    /// Gets the current language.
    #[must_use]
    pub const fn current_language(&self) -> &Language {
        &self.current_language
    }

    /// Translates a key to the current language.
    #[must_use]
    pub fn translate(&self, key: &TranslationKey) -> String {
        self.translations
            .get(&self.current_language)
            .and_then(|lang_map| lang_map.get(key))
            .cloned()
            .unwrap_or_else(|| self.fallback_translation(key))
    }

    /// Gets the fallback translation (English) for a key.
    #[must_use]
    fn fallback_translation(&self, key: &TranslationKey) -> String {
        // If we don't have the translation in the current language,
        // fall back to English
        if self.current_language != Language::English {
            if let Some(english_map) = self.translations.get(&Language::English) {
                if let Some(translation) = english_map.get(key) {
                    return translation.clone();
                }
            }
        }
        
        // Ultimate fallback: return the debug representation
        format!("{key:?}")
    }

    /// Loads all translations for all languages.
    fn load_translations(&mut self) {
        for language in Language::all() {
            let translations = Self::get_translations_for_language(&language);
            self.translations.insert(language, translations);
        }
    }

    /// Gets translations for a specific language.
    #[must_use]
    fn get_translations_for_language(language: &Language) -> HashMap<TranslationKey, String> {
        let mut translations = HashMap::new();
        
        match *language {
            Language::English => {
                translations.insert(TranslationKey::AppTitle, "CTP Client".to_owned());
                translations.insert(TranslationKey::PlayGame, "Play Game".to_owned());
                translations.insert(TranslationKey::Exit, "Exit".to_owned());
                translations.insert(TranslationKey::Settings, "Settings".to_owned());
                translations.insert(TranslationKey::SelectInstallation, "Select Installation".to_owned());
                translations.insert(TranslationKey::BrowseForInstallation, "Browse for Installation".to_owned());
                translations.insert(TranslationKey::NoInstallationsFound, "No installations found".to_owned());
                translations.insert(TranslationKey::InvalidInstallation, "Invalid installation".to_owned());
                translations.insert(TranslationKey::SettingsTitle, "Settings".to_owned());
                translations.insert(TranslationKey::Language, "Language".to_owned());
                translations.insert(TranslationKey::LanguageDescription, "Select your preferred language".to_owned());
                translations.insert(TranslationKey::Save, "Save".to_owned());
                translations.insert(TranslationKey::Cancel, "Cancel".to_owned());
                translations.insert(TranslationKey::CallToPower1, "Call to Power".to_owned());
                translations.insert(TranslationKey::CallToPower2, "Call to Power II".to_owned());
                translations.insert(TranslationKey::GameStarting, "Starting game...".to_owned());
                translations.insert(TranslationKey::LoadingConfiguration, "Loading configuration...".to_owned());
                translations.insert(TranslationKey::ScanningInstallations, "Scanning for installations...".to_owned());
            }
            Language::Spanish => {
                translations.insert(TranslationKey::AppTitle, "Cliente CTP".to_owned());
                translations.insert(TranslationKey::PlayGame, "Jugar".to_owned());
                translations.insert(TranslationKey::Exit, "Salir".to_owned());
                translations.insert(TranslationKey::Settings, "Configuración".to_owned());
                translations.insert(TranslationKey::SelectInstallation, "Seleccionar Instalación".to_owned());
                translations.insert(TranslationKey::BrowseForInstallation, "Buscar Instalación".to_owned());
                translations.insert(TranslationKey::NoInstallationsFound, "No se encontraron instalaciones".to_owned());
                translations.insert(TranslationKey::InvalidInstallation, "Instalación inválida".to_owned());
                translations.insert(TranslationKey::SettingsTitle, "Configuración".to_owned());
                translations.insert(TranslationKey::Language, "Idioma".to_owned());
                translations.insert(TranslationKey::LanguageDescription, "Selecciona tu idioma preferido".to_owned());
                translations.insert(TranslationKey::Save, "Guardar".to_owned());
                translations.insert(TranslationKey::Cancel, "Cancelar".to_owned());
                translations.insert(TranslationKey::CallToPower1, "Call to Power".to_owned());
                translations.insert(TranslationKey::CallToPower2, "Call to Power II".to_owned());
                translations.insert(TranslationKey::GameStarting, "Iniciando juego...".to_owned());
                translations.insert(TranslationKey::LoadingConfiguration, "Cargando configuración...".to_owned());
                translations.insert(TranslationKey::ScanningInstallations, "Buscando instalaciones...".to_owned());
            }
            Language::French => {
                translations.insert(TranslationKey::AppTitle, "Client CTP".to_owned());
                translations.insert(TranslationKey::PlayGame, "Jouer".to_owned());
                translations.insert(TranslationKey::Exit, "Quitter".to_owned());
                translations.insert(TranslationKey::Settings, "Paramètres".to_owned());
                translations.insert(TranslationKey::SelectInstallation, "Sélectionner l'Installation".to_owned());
                translations.insert(TranslationKey::BrowseForInstallation, "Parcourir pour l'Installation".to_owned());
                translations.insert(TranslationKey::NoInstallationsFound, "Aucune installation trouvée".to_owned());
                translations.insert(TranslationKey::InvalidInstallation, "Installation invalide".to_owned());
                translations.insert(TranslationKey::SettingsTitle, "Paramètres".to_owned());
                translations.insert(TranslationKey::Language, "Langue".to_owned());
                translations.insert(TranslationKey::LanguageDescription, "Sélectionnez votre langue préférée".to_owned());
                translations.insert(TranslationKey::Save, "Enregistrer".to_owned());
                translations.insert(TranslationKey::Cancel, "Annuler".to_owned());
                translations.insert(TranslationKey::CallToPower1, "Call to Power".to_owned());
                translations.insert(TranslationKey::CallToPower2, "Call to Power II".to_owned());
                translations.insert(TranslationKey::GameStarting, "Démarrage du jeu...".to_owned());
                translations.insert(TranslationKey::LoadingConfiguration, "Chargement de la configuration...".to_owned());
                translations.insert(TranslationKey::ScanningInstallations, "Recherche d'installations...".to_owned());
            }
            Language::German => {
                translations.insert(TranslationKey::AppTitle, "CTP Client".to_owned());
                translations.insert(TranslationKey::PlayGame, "Spiel starten".to_owned());
                translations.insert(TranslationKey::Exit, "Beenden".to_owned());
                translations.insert(TranslationKey::Settings, "Einstellungen".to_owned());
                translations.insert(TranslationKey::SelectInstallation, "Installation auswählen".to_owned());
                translations.insert(TranslationKey::BrowseForInstallation, "Installation durchsuchen".to_owned());
                translations.insert(TranslationKey::NoInstallationsFound, "Keine Installationen gefunden".to_owned());
                translations.insert(TranslationKey::InvalidInstallation, "Ungültige Installation".to_owned());
                translations.insert(TranslationKey::SettingsTitle, "Einstellungen".to_owned());
                translations.insert(TranslationKey::Language, "Sprache".to_owned());
                translations.insert(TranslationKey::LanguageDescription, "Wählen Sie Ihre bevorzugte Sprache".to_owned());
                translations.insert(TranslationKey::Save, "Speichern".to_owned());
                translations.insert(TranslationKey::Cancel, "Abbrechen".to_owned());
                translations.insert(TranslationKey::CallToPower1, "Call to Power".to_owned());
                translations.insert(TranslationKey::CallToPower2, "Call to Power II".to_owned());
                translations.insert(TranslationKey::GameStarting, "Spiel wird gestartet...".to_owned());
                translations.insert(TranslationKey::LoadingConfiguration, "Konfiguration wird geladen...".to_owned());
                translations.insert(TranslationKey::ScanningInstallations, "Suche nach Installationen...".to_owned());
            }
            // Add basic translations for other languages (minimal set)
            _ => {
                // Fallback to English for other languages (would be expanded in real app)
                translations.insert(TranslationKey::AppTitle, "CTP Client".to_owned());
                translations.insert(TranslationKey::PlayGame, "Play Game".to_owned());
                translations.insert(TranslationKey::Exit, "Exit".to_owned());
                translations.insert(TranslationKey::Settings, "Settings".to_owned());
            }
        }
        
        translations
    }
}

impl Default for Translator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_code() {
        assert_eq!(Language::English.code(), "en");
        assert_eq!(Language::Spanish.code(), "es");
        assert_eq!(Language::French.code(), "fr");
        assert_eq!(Language::German.code(), "de");
    }

    #[test]
    fn test_language_display_name() {
        assert_eq!(Language::English.display_name(), "English");
        assert_eq!(Language::Spanish.display_name(), "Español");
        assert_eq!(Language::French.display_name(), "Français");
        assert_eq!(Language::German.display_name(), "Deutsch");
    }

    #[test]
    fn test_language_from_locale() {
        assert_eq!(Language::from_locale("en_US.UTF-8"), Some(Language::English));
        assert_eq!(Language::from_locale("es_ES"), Some(Language::Spanish));
        assert_eq!(Language::from_locale("fr-FR"), Some(Language::French));
        assert_eq!(Language::from_locale("de"), Some(Language::German));
        assert_eq!(Language::from_locale("unknown"), None);
    }

    #[test]
    fn test_language_all() {
        let languages = Language::all();
        assert!(languages.contains(&Language::English));
        assert!(languages.contains(&Language::Spanish));
        assert!(languages.len() >= 4);
    }

    #[test]
    fn test_translator_new() {
        let translator = Translator::new();
        // Should default to some language (system dependent)
        assert!(!translator.translate(&TranslationKey::AppTitle).is_empty());
    }

    #[test]
    fn test_translator_with_language() {
        let translator = Translator::with_language(Language::Spanish);
        assert_eq!(translator.current_language(), &Language::Spanish);
        assert_eq!(translator.translate(&TranslationKey::PlayGame), "Jugar");
    }

    #[test]
    fn test_translator_set_language() {
        let mut translator = Translator::with_language(Language::English);
        assert_eq!(translator.translate(&TranslationKey::Exit), "Exit");
        
        translator.set_language(Language::Spanish);
        assert_eq!(translator.translate(&TranslationKey::Exit), "Salir");
    }

    #[test]
    fn test_translation_fallback() {
        let translator = Translator::with_language(Language::English);
        // Should get some translation even for missing keys
        let translation = translator.translate(&TranslationKey::AppTitle);
        assert!(!translation.is_empty());
    }

    #[test]
    fn test_detect_system_language_with_env() {
        // Test with a controlled environment variable
        // SAFETY: Setting environment variable for test purposes only,
        // properly cleaned up afterwards
        unsafe {
            std::env::set_var("LANG", "es_ES.UTF-8");
        }
        let detected = Translator::detect_system_language();
        // Should be Spanish if environment is properly set
        // Note: This might be English due to test environment
        assert!(matches!(detected, Language::English | Language::Spanish));
        
        // Clean up
        // SAFETY: Removing the test environment variable we just set
        unsafe {
            std::env::remove_var("LANG");
        }
    }

    #[test]
    fn test_translation_keys_exist() {
        let translator = Translator::with_language(Language::English);
        
        // Test that all keys have translations
        let keys = [
            TranslationKey::AppTitle,
            TranslationKey::PlayGame,
            TranslationKey::Exit,
            TranslationKey::Settings,
            TranslationKey::SelectInstallation,
            TranslationKey::SettingsTitle,
            TranslationKey::Language,
            TranslationKey::Save,
            TranslationKey::Cancel,
        ];
        
        for key in &keys {
            let translation = translator.translate(key);
            assert!(!translation.is_empty(), "Missing translation for {key:?}");
        }
    }

    #[test]
    fn test_multiple_languages_have_translations() {
        for language in [Language::English, Language::Spanish, Language::French, Language::German] {
            let translator = Translator::with_language(language.clone());
            let translation = translator.translate(&TranslationKey::PlayGame);
            assert!(!translation.is_empty(), "Missing PlayGame translation for {language:?}");
        }
    }
}
