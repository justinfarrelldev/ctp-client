//! In-game sound-effect identifiers and CTP2 sound database helpers.

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use std::fs;
use std::path::Path;

/// Sound archive path relative to the CTP2 installation root.
pub const SOUND_ARCHIVE: &str = "ctp2_data/default/sound/sound.zfs";
/// Default sound database path relative to the CTP2 installation root.
pub const SOUND_DATABASE: &str = "ctp2_data/default/gamedata/sounds.txt";

/// Runtime sound effect requested by model/UI interactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GameplaySound {
    /// Standard interface click.
    UiClick,
    /// End-turn button/shortcut sound.
    EndTurn,
    /// Unit acknowledges a selection/order.
    UnitAcknowledge,
    /// Unit movement effect.
    UnitMove,
    /// Settler founds a city.
    Settle,
    /// Unit combat impact.
    Combat,
    /// Unit death effect.
    UnitDeath,
    /// Ruin/hut resolved.
    Ruin,
    /// Popup/message opened.
    Popup,
    /// Great Library opened.
    GreatLibrary,
}

impl GameplaySound {
    /// Returns the CTP2 sound database key preferred for this effect.
    #[must_use]
    pub const fn database_key(self) -> &'static str {
        match self {
            Self::UiClick | Self::GreatLibrary => "SOUND_BUTTON_CLICK",
            Self::EndTurn => "SOUND_ENDTURN",
            Self::UnitAcknowledge => "SOUND_ACKNOWLEDGE_SETTLER",
            Self::UnitMove => "SOUND_MOVE_SETTLER",
            Self::Settle => "SOUND_WORK_SETTLER",
            Self::Combat => "SOUND_ATTACK_SETTLER",
            Self::UnitDeath => "SOUND_DEATH_SETTLER",
            Self::Ruin | Self::Popup => "SOUND_MESSAGE",
        }
    }

    /// Returns a fallback WAV entry known to exist in base CTP2 installs.
    #[must_use]
    pub const fn fallback_entry(self) -> &'static str {
        match self {
            Self::UiClick | Self::GreatLibrary | Self::EndTurn | Self::Popup | Self::Ruin => {
                "CCLICK1.WAV"
            }
            Self::UnitAcknowledge => "GUV045.WAV",
            Self::UnitMove => "GUMX02.WAV",
            Self::Settle => "GUWX01.WAV",
            Self::Combat => "GUAX01.WAV",
            Self::UnitDeath => "GUDX01.WAV",
        }
    }
}

/// Sound database mapping from CTP2 symbolic identifiers to WAV entries.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SoundManifest {
    /// Parsed sound-key to WAV-entry mapping.
    pub entries: BTreeMap<String, String>,
}

impl SoundManifest {
    /// Loads `sounds.txt` from the installation root.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when the database cannot be read.
    pub fn load_from_installation(installation_root: &Path) -> std::io::Result<Self> {
        let text = fs::read_to_string(installation_root.join(SOUND_DATABASE))?;
        Ok(Self::parse(&text))
    }

    /// Parses CTP2 `sounds.txt` content.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let mut entries = BTreeMap::new();
        for line in text.lines() {
            if let Some((key, value)) = parse_sound_line(line) {
                entries.insert(key, value);
            }
        }
        Self { entries }
    }

    /// Resolves a gameplay sound to a WAV entry name.
    #[must_use]
    pub fn entry_for(&self, sound: GameplaySound) -> String {
        self.entries
            .get(sound.database_key())
            .cloned()
            .unwrap_or_else(|| sound.fallback_entry().to_owned())
    }

    /// Returns all sound entries requested by the current in-game implementation.
    #[must_use]
    pub fn required_entries(&self) -> Vec<String> {
        [
            GameplaySound::UiClick,
            GameplaySound::EndTurn,
            GameplaySound::UnitAcknowledge,
            GameplaySound::UnitMove,
            GameplaySound::Settle,
            GameplaySound::Combat,
            GameplaySound::UnitDeath,
            GameplaySound::Ruin,
            GameplaySound::Popup,
            GameplaySound::GreatLibrary,
        ]
        .into_iter()
        .map(|sound| self.entry_for(sound))
        .collect()
    }
}

/// Parses one `KEY "VALUE.WAV"` line from `sounds.txt`.
fn parse_sound_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    let mut quoted_parts = trimmed.split('"');
    let before_value = quoted_parts.next()?;
    let value = quoted_parts.next()?.to_owned();
    let _after_value = quoted_parts.next()?;
    let key = before_value.split_whitespace().next()?.to_owned();
    Some((key, value))
}

#[cfg(test)]
mod tests {
    use super::{GameplaySound, SoundManifest};

    #[test]
    fn parses_sound_database_lines_and_uses_fallbacks() {
        let manifest = SoundManifest::parse(
            "# comment\nSOUND_MOVE_SETTLER \"GUMX02.WAV\"\nSOUND_ACKNOWLEDGE_SETTLER \"ACK.WAV\"\n",
        );

        assert_eq!(
            manifest.entry_for(GameplaySound::UnitMove),
            "GUMX02.WAV",
            "known sound keys should resolve from sounds.txt"
        );
        assert_eq!(
            manifest.entry_for(GameplaySound::UiClick),
            "CCLICK1.WAV",
            "missing abstract UI keys should resolve to safe UI fallback"
        );
    }
}
