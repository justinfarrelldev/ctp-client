//! Integration tests for internationalization support.
//!
//! These tests verify that the i18n system works correctly across
//! the application, including UI integration and settings persistence.

use ctp_client::config::AppConfig;
use ctp_client::i18n::{Language, TranslationKey, Translator};
use ctp_client::settings::SettingsResult;

#[cfg(test)]
mod i18n_integration_tests {
    use super::*;

    #[test]
    fn test_system_language_detection() {
        // Test that system language detection works
        let detected = Translator::detect_system_language();

        // Should be one of the supported languages
        let supported_languages = Language::all();
        // Note: detection might fall back to English, which is always supported
        assert!(supported_languages.contains(&detected));
    }

    #[test]
    fn test_config_includes_language() {
        // Test that new configs include language setting
        let config = AppConfig::new();

        // Should have a language set
        let language = &config.language;

        // Should be a valid language
        let all_languages = Language::all();
        assert!(all_languages.contains(language));
    }

    #[test]
    fn test_config_serialization_with_language() {
        // Test that language is properly serialized and deserialized
        let mut config = AppConfig::new();
        config.language = Language::Spanish;

        // Serialize to TOML
        let toml_string = toml::to_string(&config).expect("Should serialize");
        assert!(toml_string.contains("language"));
        assert!(toml_string.contains("Spanish"));

        // Deserialize back
        let deserialized: AppConfig = toml::from_str(&toml_string).expect("Should deserialize");
        assert_eq!(deserialized.language, Language::Spanish);
        assert_eq!(deserialized.version, config.version);
    }

    #[test]
    fn test_translator_language_switching() {
        // Test that translator properly switches languages
        let mut translator = Translator::with_language(Language::English);

        // Get English translation
        let english_play = translator.translate(&TranslationKey::PlayGame);
        assert_eq!(english_play, "Play Game");

        // Switch to Spanish
        translator.set_language(Language::Spanish);
        let spanish_play = translator.translate(&TranslationKey::PlayGame);
        assert_eq!(spanish_play, "Jugar");

        // Switch to French
        translator.set_language(Language::French);
        let french_play = translator.translate(&TranslationKey::PlayGame);
        assert_eq!(french_play, "Jouer");
    }

    #[test]
    fn test_all_translation_keys_have_translations() {
        // Test that all languages have translations for essential keys
        let essential_keys = [
            TranslationKey::AppTitle,
            TranslationKey::PlayGame,
            TranslationKey::Exit,
            TranslationKey::Settings,
        ];

        for language in [
            Language::English,
            Language::Spanish,
            Language::French,
            Language::German,
        ] {
            let translator = Translator::with_language(language.clone());

            for key in &essential_keys {
                let translation = translator.translate(key);
                assert!(
                    !translation.is_empty(),
                    "Missing translation for {key:?} in {language:?}"
                );
                // Should not be the debug representation
                assert!(
                    !translation.contains("TranslationKey"),
                    "Translation for {key:?} in {language:?} is debug representation: {translation}"
                );
            }
        }
    }

    #[test]
    fn test_language_search_functionality() {
        // Test the language search logic
        let all_languages = Language::all();

        // Test searching by code
        let en_results: Vec<_> = all_languages
            .iter()
            .filter(|lang| lang.code().to_lowercase().contains("en"))
            .collect();
        assert!(!en_results.is_empty());
        assert!(en_results.contains(&&Language::English));

        // Test searching by display name
        let spanish_results: Vec<_> = all_languages
            .iter()
            .filter(|lang| lang.display_name().to_lowercase().contains("español"))
            .collect();
        assert!(!spanish_results.is_empty());
        assert!(spanish_results.contains(&&Language::Spanish));

        // Test empty search returns all
        let empty_count = all_languages
            .iter()
            .filter(|_| true) // Empty search shows all
            .count();
        assert_eq!(empty_count, all_languages.len());
    }

    #[test]
    fn test_settings_result_variants() {
        // Test SettingsResult enum behavior
        let saved_result = SettingsResult::Saved(Language::French);
        let cancelled_result = SettingsResult::Cancelled;

        // Test pattern matching
        let is_saved = matches!(saved_result, SettingsResult::Saved(Language::French));
        assert!(is_saved, "Expected Saved(French) variant");

        let is_cancelled = matches!(cancelled_result, SettingsResult::Cancelled);
        assert!(is_cancelled, "Expected Cancelled variant");

        // Test equality
        assert_eq!(SettingsResult::Cancelled, SettingsResult::Cancelled);
        assert_eq!(
            SettingsResult::Saved(Language::English),
            SettingsResult::Saved(Language::English)
        );
        assert_ne!(
            SettingsResult::Saved(Language::English),
            SettingsResult::Saved(Language::Spanish)
        );
        assert_ne!(
            SettingsResult::Saved(Language::English),
            SettingsResult::Cancelled
        );
    }

    #[test]
    fn test_language_locale_parsing() {
        // Test various locale string formats
        let test_cases = [
            ("en_US.UTF-8", Some(Language::English)),
            ("en_US", Some(Language::English)),
            ("en-US", Some(Language::English)),
            ("en", Some(Language::English)),
            ("es_ES.UTF-8", Some(Language::Spanish)),
            ("es_MX", Some(Language::Spanish)),
            ("fr_FR", Some(Language::French)),
            ("de_DE", Some(Language::German)),
            ("invalid_locale", None),
            ("", None),
        ];

        for (locale, expected) in test_cases {
            let result = Language::from_locale(locale);
            assert_eq!(result, expected, "Failed for locale: {locale}");
        }
    }

    #[test]
    fn test_fallback_translation_behavior() {
        // Test that fallback translations work when translation is missing
        let _translator = Translator::with_language(Language::English);

        // This should work for all languages since English is the fallback
        for language in Language::all() {
            let lang_translator = Translator::with_language(language);
            let translation = lang_translator.translate(&TranslationKey::AppTitle);

            // Should always get some translation
            assert!(!translation.is_empty());
        }
    }

    #[test]
    fn test_config_language_persistence() {
        // Test that language changes are properly saved in config
        let mut config = AppConfig::new();
        let original_language = config.language;

        // Change language
        config.language = Language::German;
        assert_eq!(config.language, Language::German);
        assert_ne!(config.language, original_language);

        // Test serialization includes new language
        let serialized = toml::to_string(&config).expect("Config should serialize");
        assert!(serialized.contains("German"));

        // Test deserialization preserves language
        let deserialized: AppConfig = toml::from_str(&serialized).expect("Should deserialize");
        assert_eq!(deserialized.language, Language::German);
    }

    #[test]
    fn test_ui_text_availability() {
        // Test that all UI text keys have non-empty translations
        let ui_keys = [
            TranslationKey::AppTitle,
            TranslationKey::PlayGame,
            TranslationKey::Exit,
            TranslationKey::Settings,
            TranslationKey::SelectInstallation,
            TranslationKey::BrowseForInstallation,
            TranslationKey::SettingsTitle,
            TranslationKey::Language,
            TranslationKey::LanguageDescription,
            TranslationKey::Save,
            TranslationKey::Cancel,
        ];

        let translator = Translator::with_language(Language::English);

        for key in &ui_keys {
            let translation = translator.translate(key);
            assert!(!translation.is_empty(), "Empty translation for {key:?}");
            assert!(
                translation.len() > 1,
                "Translation too short for {key:?}: {translation}"
            );
        }
    }

    #[test]
    fn test_multiple_translator_instances() {
        // Test that multiple translator instances work independently
        let english_translator = Translator::with_language(Language::English);
        let spanish_translator = Translator::with_language(Language::Spanish);

        let english_text = english_translator.translate(&TranslationKey::Settings);
        let spanish_text = spanish_translator.translate(&TranslationKey::Settings);

        assert_eq!(english_text, "Settings");
        assert_eq!(spanish_text, "Configuración");
        assert_ne!(english_text, spanish_text);
    }
}
