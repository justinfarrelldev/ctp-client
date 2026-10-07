//! Functional pre-game shell option state.

extern crate alloc;

use alloc::string::String;

use super::{ShellOptionField, pickers};

/// Bit for the music-enabled shell option.
const OPTION_FLAG_MUSIC_ENABLED: u16 = 0b0000_0000_0001;
/// Bit for the auto-repeat shell option.
const OPTION_FLAG_AUTO_REPEAT: u16 = 0b0000_0000_0010;
/// Bit for the random-order shell option.
const OPTION_FLAG_RANDOM_ORDER: u16 = 0b0000_0000_0100;
/// Bit for the unit-animation graphics option.
const OPTION_FLAG_UNIT_ANIMATION: u16 = 0b0000_0000_1000;
/// Bit for the animated-trade-routes graphics option.
const OPTION_FLAG_ANIMATED_TRADE_ROUTES: u16 = 0b0000_0001_0000;
/// Bit for the wonder-movies graphics option.
const OPTION_FLAG_WONDER_MOVIES: u16 = 0b0000_0010_0000;
/// Bit for the city-names graphics option.
const OPTION_FLAG_CITY_NAMES: u16 = 0b0000_0100_0000;
/// Bit for the political-borders graphics option.
const OPTION_FLAG_POLITICAL_BORDERS: u16 = 0b0000_1000_0000;
/// Bit for the trade-routes graphics option.
const OPTION_FLAG_TRADE_ROUTES: u16 = 0b0001_0000_0000;
/// Bit for the city-influence graphics option.
const OPTION_FLAG_CITY_INFLUENCE: u16 = 0b0010_0000_0000;
/// Bit for the grid graphics option.
const OPTION_FLAG_GRID: u16 = 0b0100_0000_0000;
/// Original default enabled flags across music and graphics options.
const DEFAULT_OPTION_FLAGS: u16 = OPTION_FLAG_MUSIC_ENABLED
    | OPTION_FLAG_UNIT_ANIMATION
    | OPTION_FLAG_ANIMATED_TRADE_ROUTES
    | OPTION_FLAG_WONDER_MOVIES
    | OPTION_FLAG_CITY_NAMES
    | OPTION_FLAG_POLITICAL_BORDERS
    | OPTION_FLAG_TRADE_ROUTES
    | OPTION_FLAG_CITY_INFLUENCE;

/// Functional pre-game option state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::main_menu) struct ShellOptionsState {
    /// SFX volume from 0 to 10.
    sfx_volume: i32,
    /// Voice volume from 0 to 10.
    voice_volume: i32,
    /// Music volume from 0 to 10.
    music_volume: i32,
    /// Unit speed from 0 to 9.
    unit_speed: i32,
    /// Selected resolution label.
    resolution: String,
    /// Bit-packed boolean option state for music and graphics toggles.
    enabled_option_flags: u16,
    /// Selected music track index.
    music_track: usize,
}

impl Default for ShellOptionsState {
    fn default() -> Self {
        Self {
            sfx_volume: 5,
            voice_volume: 5,
            music_volume: 5,
            unit_speed: 5,
            resolution: "800 x 600".to_owned(),
            enabled_option_flags: DEFAULT_OPTION_FLAGS,
            music_track: 1,
        }
    }
}

impl ShellOptionsState {
    /// Returns the packed bit for a boolean option field.
    const fn boolean_option_flag(field: ShellOptionField) -> Option<u16> {
        match field {
            ShellOptionField::MusicEnabled => Some(OPTION_FLAG_MUSIC_ENABLED),
            ShellOptionField::AutoRepeat => Some(OPTION_FLAG_AUTO_REPEAT),
            ShellOptionField::RandomOrder => Some(OPTION_FLAG_RANDOM_ORDER),
            ShellOptionField::UnitAnimation => Some(OPTION_FLAG_UNIT_ANIMATION),
            ShellOptionField::AnimatedTradeRoutes => Some(OPTION_FLAG_ANIMATED_TRADE_ROUTES),
            ShellOptionField::WonderMovies => Some(OPTION_FLAG_WONDER_MOVIES),
            ShellOptionField::CityNames => Some(OPTION_FLAG_CITY_NAMES),
            ShellOptionField::PoliticalBorders => Some(OPTION_FLAG_POLITICAL_BORDERS),
            ShellOptionField::TradeRoutes => Some(OPTION_FLAG_TRADE_ROUTES),
            ShellOptionField::CityInfluence => Some(OPTION_FLAG_CITY_INFLUENCE),
            ShellOptionField::Grid => Some(OPTION_FLAG_GRID),
            ShellOptionField::SfxVolume
            | ShellOptionField::VoiceVolume
            | ShellOptionField::MusicVolume
            | ShellOptionField::UnitSpeed
            | ShellOptionField::Resolution
            | ShellOptionField::MusicTrack => None,
        }
    }

    /// Sets one packed boolean option.
    const fn set_bool_option(&mut self, field: ShellOptionField, enabled: bool) -> bool {
        let Some(flag) = Self::boolean_option_flag(field) else {
            return false;
        };
        if enabled {
            self.enabled_option_flags |= flag;
        } else {
            self.enabled_option_flags &= !flag;
        }
        true
    }

    /// Returns slider value for a functional option.
    pub(in crate::main_menu) const fn numeric(&self, field: ShellOptionField) -> Option<i32> {
        match field {
            ShellOptionField::SfxVolume => Some(self.sfx_volume),
            ShellOptionField::VoiceVolume => Some(self.voice_volume),
            ShellOptionField::MusicVolume => Some(self.music_volume),
            ShellOptionField::UnitSpeed => Some(self.unit_speed),
            ShellOptionField::UnitAnimation
            | ShellOptionField::AnimatedTradeRoutes
            | ShellOptionField::WonderMovies
            | ShellOptionField::CityNames
            | ShellOptionField::PoliticalBorders
            | ShellOptionField::TradeRoutes
            | ShellOptionField::CityInfluence
            | ShellOptionField::Grid
            | ShellOptionField::Resolution
            | ShellOptionField::MusicEnabled
            | ShellOptionField::AutoRepeat
            | ShellOptionField::RandomOrder
            | ShellOptionField::MusicTrack => None,
        }
    }

    /// Sets a numeric option, clamping to original bounds.
    pub(in crate::main_menu) fn set_numeric(
        &mut self,
        field: ShellOptionField,
        value: i32,
    ) -> bool {
        match field {
            ShellOptionField::SfxVolume => self.sfx_volume = value.clamp(0_i32, 10_i32),
            ShellOptionField::VoiceVolume => self.voice_volume = value.clamp(0_i32, 10_i32),
            ShellOptionField::MusicVolume => self.music_volume = value.clamp(0_i32, 10_i32),
            ShellOptionField::UnitSpeed => self.unit_speed = value.clamp(0_i32, 9_i32),
            ShellOptionField::UnitAnimation
            | ShellOptionField::AnimatedTradeRoutes
            | ShellOptionField::WonderMovies
            | ShellOptionField::CityNames
            | ShellOptionField::PoliticalBorders
            | ShellOptionField::TradeRoutes
            | ShellOptionField::CityInfluence
            | ShellOptionField::Grid
            | ShellOptionField::Resolution
            | ShellOptionField::MusicEnabled
            | ShellOptionField::AutoRepeat
            | ShellOptionField::RandomOrder
            | ShellOptionField::MusicTrack => return false,
        }
        true
    }

    /// Toggles a boolean option.
    pub(in crate::main_menu) const fn toggle(&mut self, field: ShellOptionField) -> bool {
        match Self::boolean_option_flag(field) {
            Some(flag) => {
                if self.enabled_option_flags & flag == 0 {
                    self.enabled_option_flags |= flag;
                } else {
                    self.enabled_option_flags &= !flag;
                }
                true
            }
            None => false,
        }
    }

    /// Selects a list/switch option.
    pub(in crate::main_menu) fn select(&mut self, field: ShellOptionField, index: usize) -> bool {
        match field {
            ShellOptionField::Resolution => {
                let Some(value) = pickers::RESOLUTIONS.get(index) else {
                    return false;
                };
                (*value).clone_into(&mut self.resolution);
            }
            ShellOptionField::MusicEnabled
            | ShellOptionField::AutoRepeat
            | ShellOptionField::RandomOrder
            | ShellOptionField::UnitAnimation
            | ShellOptionField::AnimatedTradeRoutes
            | ShellOptionField::WonderMovies
            | ShellOptionField::CityNames
            | ShellOptionField::PoliticalBorders
            | ShellOptionField::TradeRoutes
            | ShellOptionField::CityInfluence
            | ShellOptionField::Grid => return self.set_bool_option(field, index != 0),
            ShellOptionField::MusicTrack => {
                if index >= pickers::MUSIC_TRACKS.len() {
                    return false;
                }
                self.music_track = index.saturating_add(1);
            }
            ShellOptionField::SfxVolume
            | ShellOptionField::VoiceVolume
            | ShellOptionField::MusicVolume
            | ShellOptionField::UnitSpeed => return false,
        }
        true
    }

    /// Returns a boolean option state.
    pub(in crate::main_menu) const fn bool_option(&self, field: ShellOptionField) -> bool {
        match Self::boolean_option_flag(field) {
            Some(flag) => self.enabled_option_flags & flag != 0,
            None => false,
        }
    }

    /// Returns selected resolution label.
    pub(in crate::main_menu) fn resolution(&self) -> &str {
        self.resolution.as_str()
    }

    /// Returns whether music is enabled.
    pub(in crate::main_menu) const fn music_enabled(&self) -> bool {
        self.bool_option(ShellOptionField::MusicEnabled)
    }

    /// Returns whether auto-repeat is enabled.
    pub(in crate::main_menu) const fn auto_repeat(&self) -> bool {
        self.bool_option(ShellOptionField::AutoRepeat)
    }

    /// Returns whether random order is enabled.
    pub(in crate::main_menu) const fn random_order(&self) -> bool {
        self.bool_option(ShellOptionField::RandomOrder)
    }

    /// Returns selected music track number.
    pub(in crate::main_menu) const fn music_track(&self) -> usize {
        self.music_track
    }
}
