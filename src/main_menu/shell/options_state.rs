//! Functional pre-game shell option state.

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::config::ctp2_options::{
    Ctp2GameplayOptions, Ctp2GraphicsOptions, Ctp2KeyBinding, Ctp2KeyboardCategory,
    Ctp2KeyboardOptions, Ctp2MusicOptions, Ctp2Options, Ctp2Resolution, Ctp2SoundOptions,
    Ctp2Toggle,
};

use super::{ShellOptionField, picker_data, pickers};

/// Bit for the music-enabled shell option.
const OPTION_FLAG_MUSIC_ENABLED: u32 = 0b0000_0000_0000_0001;
/// Bit for the auto-repeat shell option.
const OPTION_FLAG_AUTO_REPEAT: u32 = 0b0000_0000_0000_0010;
/// Bit for the random-order shell option.
const OPTION_FLAG_RANDOM_ORDER: u32 = 0b0000_0000_0000_0100;
/// Bit for the unit-animation graphics option.
const OPTION_FLAG_UNIT_ANIMATION: u32 = 0b0000_0000_0000_1000;
/// Bit for the animated-trade-routes graphics option.
const OPTION_FLAG_ANIMATED_TRADE_ROUTES: u32 = 0b0000_0000_0001_0000;
/// Bit for the wonder-movies graphics option.
const OPTION_FLAG_WONDER_MOVIES: u32 = 0b0000_0000_0010_0000;
/// Bit for the city-names graphics option.
const OPTION_FLAG_CITY_NAMES: u32 = 0b0000_0000_0100_0000;
/// Bit for the political-borders graphics option.
const OPTION_FLAG_POLITICAL_BORDERS: u32 = 0b0000_0000_1000_0000;
/// Bit for the trade-routes graphics option.
const OPTION_FLAG_TRADE_ROUTES: u32 = 0b0000_0001_0000_0000;
/// Bit for the city-influence graphics option.
const OPTION_FLAG_CITY_INFLUENCE: u32 = 0b0000_0010_0000_0000;
/// Bit for the grid graphics option.
const OPTION_FLAG_GRID: u32 = 0b0000_0100_0000_0000;
/// Bit for tutorial advice.
const OPTION_FLAG_TUTORIAL_ADVICE: u32 = 0b0000_1000_0000_0000;
/// Bit for automatic turn cycling.
const OPTION_FLAG_AUTO_CYCLE_TURNS: u32 = 0b0001_0000_0000_0000;
/// Bit for automatic unit cycling.
const OPTION_FLAG_AUTO_CYCLE_UNITS: u32 = 0b0010_0000_0000_0000;
/// Bit for zoomed battle view.
const OPTION_FLAG_BATTLE_VIEW: u32 = 0b0100_0000_0000_0000;
/// Bit for enemy movement display.
const OPTION_FLAG_ENEMY_MOVES: u32 = 0b1000_0000_0000_0000;
/// Bit for automatic map centering.
const OPTION_FLAG_AUTO_CENTER: u32 = 0b0001_0000_0000_0000_0000;
/// Bit for automatic tab selection.
const OPTION_FLAG_AUTO_TAB_SELECT: u32 = 0b0010_0000_0000_0000_0000;
/// Bit for always showing battles.
const OPTION_FLAG_SHOW_ALL_BATTLES: u32 = 0b0100_0000_0000_0000_0000;
/// Bit for autosave.
const OPTION_FLAG_AUTO_SAVE: u32 = 0b1000_0000_0000_0000_0000;
/// Bit for left-handed mouse.
const OPTION_FLAG_LEFT_HANDED_MOUSE: u32 = 0b0001_0000_0000_0000_0000_0000;
/// Original default enabled flags across music, graphics, and gameplay options.
const DEFAULT_OPTION_FLAGS: u32 = OPTION_FLAG_MUSIC_ENABLED
    | OPTION_FLAG_UNIT_ANIMATION
    | OPTION_FLAG_ANIMATED_TRADE_ROUTES
    | OPTION_FLAG_WONDER_MOVIES
    | OPTION_FLAG_CITY_NAMES
    | OPTION_FLAG_POLITICAL_BORDERS
    | OPTION_FLAG_TRADE_ROUTES
    | OPTION_FLAG_CITY_INFLUENCE
    | OPTION_FLAG_AUTO_CYCLE_TURNS
    | OPTION_FLAG_AUTO_CYCLE_UNITS
    | OPTION_FLAG_BATTLE_VIEW
    | OPTION_FLAG_ENEMY_MOVES
    | OPTION_FLAG_AUTO_CENTER
    | OPTION_FLAG_AUTO_TAB_SELECT
    | OPTION_FLAG_AUTO_SAVE;

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
    /// Mouse speed from 0 to 15.
    mouse_speed: i32,
    /// Selected resolution label.
    resolution: String,
    /// Bit-packed boolean option state for music, graphics, and gameplay toggles.
    enabled_option_flags: u32,
    /// Selected music track index.
    music_track: usize,
    /// First visible row in the resolution picker listbox.
    resolution_scroll_offset: usize,
    /// First visible row in the music-track picker listbox.
    music_track_scroll_offset: usize,
    /// Selected keyboard mapping category.
    keyboard_category: Ctp2KeyboardCategory,
    /// Persisted key-binding overrides.
    keyboard_bindings: Vec<Ctp2KeyBinding>,
}

impl Default for ShellOptionsState {
    fn default() -> Self {
        Self {
            sfx_volume: 5,
            voice_volume: 5,
            music_volume: 5,
            unit_speed: 5,
            mouse_speed: 5,
            resolution: "800 x 600".to_owned(),
            enabled_option_flags: DEFAULT_OPTION_FLAGS,
            music_track: 2,
            resolution_scroll_offset: 0,
            music_track_scroll_offset: 0,
            keyboard_category: Ctp2KeyboardCategory::Basic,
            keyboard_bindings: Vec::new(),
        }
    }
}

impl ShellOptionsState {
    /// Creates shell option state from persisted configuration.
    pub(in crate::main_menu) fn from_persisted(options: &Ctp2Options) -> Self {
        let mut state = Self {
            sfx_volume: options.sound.sound_volume.clamp(0_i32, 10_i32),
            voice_volume: options.sound.voice_volume.clamp(0_i32, 10_i32),
            music_volume: options.sound.music_volume.clamp(0_i32, 10_i32),
            unit_speed: options.graphics.unit_speed.clamp(0_i32, 9_i32),
            mouse_speed: options.gameplay.mouse_speed.clamp(0_i32, 15_i32),
            resolution: format!(
                "{} x {}",
                options.graphics.resolution.width, options.graphics.resolution.height
            ),
            music_track: options.music.selected_track.clamp(1_usize, 10_usize),
            keyboard_category: options.keyboard.selected_category,
            keyboard_bindings: options.keyboard.bindings.clone(),
            ..Self::default()
        };
        state.set_bool_option(ShellOptionField::MusicEnabled, options.music.music_enabled);
        state.set_bool_option(ShellOptionField::AutoRepeat, options.music.auto_repeat);
        state.set_bool_option(ShellOptionField::RandomOrder, options.music.random_order);
        state.set_bool_option(
            ShellOptionField::UnitAnimation,
            options.graphics.unit_animation.enabled(),
        );
        state.set_bool_option(
            ShellOptionField::AnimatedTradeRoutes,
            options.graphics.animated_trade_routes.enabled(),
        );
        state.set_bool_option(
            ShellOptionField::WonderMovies,
            options.graphics.wonder_movies.enabled(),
        );
        state.set_bool_option(
            ShellOptionField::CityNames,
            options.graphics.city_names.enabled(),
        );
        state.set_bool_option(
            ShellOptionField::PoliticalBorders,
            options.graphics.political_borders.enabled(),
        );
        state.set_bool_option(
            ShellOptionField::TradeRoutes,
            options.graphics.trade_routes.enabled(),
        );
        state.set_bool_option(
            ShellOptionField::CityInfluence,
            options.graphics.city_influence.enabled(),
        );
        state.set_bool_option(ShellOptionField::Grid, options.graphics.grid.enabled());
        state.set_bool_option(
            ShellOptionField::TutorialAdvice,
            options.gameplay.tutorial_advice.enabled(),
        );
        state.set_bool_option(
            ShellOptionField::AutoCycleTurns,
            options.gameplay.auto_cycle_turns.enabled(),
        );
        state.set_bool_option(
            ShellOptionField::AutoCycleUnits,
            options.gameplay.auto_cycle_units.enabled(),
        );
        state.set_bool_option(
            ShellOptionField::BattleView,
            options.gameplay.battle_view.enabled(),
        );
        state.set_bool_option(
            ShellOptionField::EnemyMoves,
            options.gameplay.enemy_moves.enabled(),
        );
        state.set_bool_option(
            ShellOptionField::AutoCenter,
            options.gameplay.auto_center.enabled(),
        );
        state.set_bool_option(
            ShellOptionField::AutoTabSelect,
            options.gameplay.auto_tab_select.enabled(),
        );
        state.set_bool_option(
            ShellOptionField::ShowAllBattles,
            options.gameplay.show_all_battles.enabled(),
        );
        state.set_bool_option(
            ShellOptionField::AutoSave,
            options.gameplay.auto_save.enabled(),
        );
        state.set_bool_option(
            ShellOptionField::LeftHandedMouse,
            options.gameplay.left_handed_mouse.enabled(),
        );
        state.reveal_selected_list_items();
        state
    }

    /// Converts shell option state into persisted configuration.
    pub(in crate::main_menu) fn to_persisted(&self) -> Ctp2Options {
        let (width, height) = parse_resolution_label(self.resolution()).unwrap_or((800, 600));
        Ctp2Options {
            sound: Ctp2SoundOptions {
                sound_volume: self.sfx_volume,
                voice_volume: self.voice_volume,
                music_volume: self.music_volume,
            },
            graphics: Ctp2GraphicsOptions {
                unit_speed: self.unit_speed,
                unit_animation: Ctp2Toggle::new(self.bool_option(ShellOptionField::UnitAnimation)),
                animated_trade_routes: Ctp2Toggle::new(
                    self.bool_option(ShellOptionField::AnimatedTradeRoutes),
                ),
                wonder_movies: Ctp2Toggle::new(self.bool_option(ShellOptionField::WonderMovies)),
                city_names: Ctp2Toggle::new(self.bool_option(ShellOptionField::CityNames)),
                political_borders: Ctp2Toggle::new(
                    self.bool_option(ShellOptionField::PoliticalBorders),
                ),
                trade_routes: Ctp2Toggle::new(self.bool_option(ShellOptionField::TradeRoutes)),
                city_influence: Ctp2Toggle::new(self.bool_option(ShellOptionField::CityInfluence)),
                grid: Ctp2Toggle::new(self.bool_option(ShellOptionField::Grid)),
                resolution: Ctp2Resolution { width, height },
            },
            music: Ctp2MusicOptions {
                selected_track: self.music_track,
                music_enabled: self.music_enabled(),
                auto_repeat: self.auto_repeat(),
                random_order: self.random_order(),
            },
            gameplay: Ctp2GameplayOptions {
                tutorial_advice: Ctp2Toggle::new(
                    self.bool_option(ShellOptionField::TutorialAdvice),
                ),
                auto_cycle_turns: Ctp2Toggle::new(
                    self.bool_option(ShellOptionField::AutoCycleTurns),
                ),
                auto_cycle_units: Ctp2Toggle::new(
                    self.bool_option(ShellOptionField::AutoCycleUnits),
                ),
                auto_center: Ctp2Toggle::new(self.bool_option(ShellOptionField::AutoCenter)),
                auto_tab_select: Ctp2Toggle::new(self.bool_option(ShellOptionField::AutoTabSelect)),
                auto_save: Ctp2Toggle::new(self.bool_option(ShellOptionField::AutoSave)),
                battle_view: Ctp2Toggle::new(self.bool_option(ShellOptionField::BattleView)),
                show_all_battles: Ctp2Toggle::new(
                    self.bool_option(ShellOptionField::ShowAllBattles),
                ),
                enemy_moves: Ctp2Toggle::new(self.bool_option(ShellOptionField::EnemyMoves)),
                left_handed_mouse: Ctp2Toggle::new(
                    self.bool_option(ShellOptionField::LeftHandedMouse),
                ),
                mouse_speed: self.mouse_speed,
            },
            keyboard: Ctp2KeyboardOptions {
                selected_category: self.keyboard_category,
                bindings: self.keyboard_bindings.clone(),
            },
        }
    }

    /// Returns the packed bit for a boolean option field.
    const fn boolean_option_flag(field: ShellOptionField) -> Option<u32> {
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
            ShellOptionField::TutorialAdvice => Some(OPTION_FLAG_TUTORIAL_ADVICE),
            ShellOptionField::AutoCycleTurns => Some(OPTION_FLAG_AUTO_CYCLE_TURNS),
            ShellOptionField::AutoCycleUnits => Some(OPTION_FLAG_AUTO_CYCLE_UNITS),
            ShellOptionField::BattleView => Some(OPTION_FLAG_BATTLE_VIEW),
            ShellOptionField::EnemyMoves => Some(OPTION_FLAG_ENEMY_MOVES),
            ShellOptionField::AutoCenter => Some(OPTION_FLAG_AUTO_CENTER),
            ShellOptionField::AutoTabSelect => Some(OPTION_FLAG_AUTO_TAB_SELECT),
            ShellOptionField::ShowAllBattles => Some(OPTION_FLAG_SHOW_ALL_BATTLES),
            ShellOptionField::AutoSave => Some(OPTION_FLAG_AUTO_SAVE),
            ShellOptionField::LeftHandedMouse => Some(OPTION_FLAG_LEFT_HANDED_MOUSE),
            ShellOptionField::SfxVolume
            | ShellOptionField::VoiceVolume
            | ShellOptionField::MusicVolume
            | ShellOptionField::UnitSpeed
            | ShellOptionField::MouseSpeed
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
            ShellOptionField::MouseSpeed => Some(self.mouse_speed),
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
            | ShellOptionField::MusicTrack
            | ShellOptionField::TutorialAdvice
            | ShellOptionField::AutoCycleTurns
            | ShellOptionField::AutoCycleUnits
            | ShellOptionField::BattleView
            | ShellOptionField::EnemyMoves
            | ShellOptionField::AutoCenter
            | ShellOptionField::AutoTabSelect
            | ShellOptionField::ShowAllBattles
            | ShellOptionField::AutoSave
            | ShellOptionField::LeftHandedMouse => None,
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
            ShellOptionField::MouseSpeed => self.mouse_speed = value.clamp(0_i32, 15_i32),
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
            | ShellOptionField::MusicTrack
            | ShellOptionField::TutorialAdvice
            | ShellOptionField::AutoCycleTurns
            | ShellOptionField::AutoCycleUnits
            | ShellOptionField::BattleView
            | ShellOptionField::EnemyMoves
            | ShellOptionField::AutoCenter
            | ShellOptionField::AutoTabSelect
            | ShellOptionField::ShowAllBattles
            | ShellOptionField::AutoSave
            | ShellOptionField::LeftHandedMouse => return false,
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
                let Some(value) = picker_data::RESOLUTIONS.get(index) else {
                    return false;
                };
                (*value).clone_into(&mut self.resolution);
                self.resolution_scroll_offset = scroll_offset_for_selected(
                    index,
                    pickers::RESOLUTION_VISIBLE_ROWS,
                    picker_data::RESOLUTIONS.len(),
                );
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
            | ShellOptionField::Grid
            | ShellOptionField::TutorialAdvice
            | ShellOptionField::AutoCycleTurns
            | ShellOptionField::AutoCycleUnits
            | ShellOptionField::BattleView
            | ShellOptionField::EnemyMoves
            | ShellOptionField::AutoCenter
            | ShellOptionField::AutoTabSelect
            | ShellOptionField::ShowAllBattles
            | ShellOptionField::AutoSave
            | ShellOptionField::LeftHandedMouse => return self.set_bool_option(field, index != 0),
            ShellOptionField::MusicTrack => {
                if index >= picker_data::MUSIC_TRACKS.len() {
                    return false;
                }
                self.music_track = index.saturating_add(1);
                self.music_track_scroll_offset = scroll_offset_for_selected(
                    index,
                    pickers::MUSIC_TRACK_VISIBLE_ROWS,
                    picker_data::MUSIC_TRACKS.len(),
                );
            }
            ShellOptionField::SfxVolume
            | ShellOptionField::VoiceVolume
            | ShellOptionField::MusicVolume
            | ShellOptionField::UnitSpeed
            | ShellOptionField::MouseSpeed => return false,
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

    /// Selects a keyboard category.
    pub(in crate::main_menu) const fn select_keyboard_category(
        &mut self,
        category: Ctp2KeyboardCategory,
    ) {
        self.keyboard_category = category;
    }

    /// Clears key-binding overrides so the default key map is restored.
    pub(in crate::main_menu) fn reset_keyboard_defaults(&mut self) {
        self.keyboard_bindings.clear();
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

    /// Returns the first visible row for an option list.
    pub(in crate::main_menu) const fn option_list_scroll_offset(
        &self,
        field: ShellOptionField,
    ) -> usize {
        match field {
            ShellOptionField::Resolution => self.resolution_scroll_offset,
            ShellOptionField::MusicTrack => self.music_track_scroll_offset,
            ShellOptionField::SfxVolume
            | ShellOptionField::VoiceVolume
            | ShellOptionField::MusicVolume
            | ShellOptionField::UnitSpeed
            | ShellOptionField::MouseSpeed
            | ShellOptionField::UnitAnimation
            | ShellOptionField::AnimatedTradeRoutes
            | ShellOptionField::WonderMovies
            | ShellOptionField::CityNames
            | ShellOptionField::PoliticalBorders
            | ShellOptionField::TradeRoutes
            | ShellOptionField::CityInfluence
            | ShellOptionField::Grid
            | ShellOptionField::MusicEnabled
            | ShellOptionField::AutoRepeat
            | ShellOptionField::RandomOrder
            | ShellOptionField::TutorialAdvice
            | ShellOptionField::AutoCycleTurns
            | ShellOptionField::AutoCycleUnits
            | ShellOptionField::BattleView
            | ShellOptionField::EnemyMoves
            | ShellOptionField::AutoCenter
            | ShellOptionField::AutoTabSelect
            | ShellOptionField::ShowAllBattles
            | ShellOptionField::AutoSave
            | ShellOptionField::LeftHandedMouse => 0,
        }
    }

    /// Scrolls an option list by signed rows and returns whether the offset changed.
    pub(in crate::main_menu) fn scroll_option_list(
        &mut self,
        field: ShellOptionField,
        delta: i32,
    ) -> bool {
        match field {
            ShellOptionField::Resolution => {
                let next = scrolled_offset(
                    self.resolution_scroll_offset,
                    delta,
                    picker_data::RESOLUTIONS.len(),
                    pickers::RESOLUTION_VISIBLE_ROWS,
                );
                let changed = next != self.resolution_scroll_offset;
                self.resolution_scroll_offset = next;
                changed
            }
            ShellOptionField::MusicTrack => {
                let next = scrolled_offset(
                    self.music_track_scroll_offset,
                    delta,
                    picker_data::MUSIC_TRACKS.len(),
                    pickers::MUSIC_TRACK_VISIBLE_ROWS,
                );
                let changed = next != self.music_track_scroll_offset;
                self.music_track_scroll_offset = next;
                changed
            }
            ShellOptionField::SfxVolume
            | ShellOptionField::VoiceVolume
            | ShellOptionField::MusicVolume
            | ShellOptionField::UnitSpeed
            | ShellOptionField::MouseSpeed
            | ShellOptionField::UnitAnimation
            | ShellOptionField::AnimatedTradeRoutes
            | ShellOptionField::WonderMovies
            | ShellOptionField::CityNames
            | ShellOptionField::PoliticalBorders
            | ShellOptionField::TradeRoutes
            | ShellOptionField::CityInfluence
            | ShellOptionField::Grid
            | ShellOptionField::MusicEnabled
            | ShellOptionField::AutoRepeat
            | ShellOptionField::RandomOrder
            | ShellOptionField::TutorialAdvice
            | ShellOptionField::AutoCycleTurns
            | ShellOptionField::AutoCycleUnits
            | ShellOptionField::BattleView
            | ShellOptionField::EnemyMoves
            | ShellOptionField::AutoCenter
            | ShellOptionField::AutoTabSelect
            | ShellOptionField::ShowAllBattles
            | ShellOptionField::AutoSave
            | ShellOptionField::LeftHandedMouse => false,
        }
    }

    /// Reveals persisted selected items when opening scrollable pickers.
    fn reveal_selected_list_items(&mut self) {
        if let Some(index) = picker_data::RESOLUTIONS
            .iter()
            .position(|value| *value == self.resolution())
        {
            self.resolution_scroll_offset = scroll_offset_for_selected(
                index,
                pickers::RESOLUTION_VISIBLE_ROWS,
                picker_data::RESOLUTIONS.len(),
            );
        }
        let music_index = self
            .music_track
            .checked_sub(1_usize)
            .map_or(0_usize, |value| value);
        self.music_track_scroll_offset = scroll_offset_for_selected(
            music_index,
            pickers::MUSIC_TRACK_VISIBLE_ROWS,
            picker_data::MUSIC_TRACKS.len(),
        );
    }

    /// Returns selected keyboard category.
    pub(in crate::main_menu) const fn keyboard_category(&self) -> Ctp2KeyboardCategory {
        self.keyboard_category
    }

    /// Returns persisted keyboard binding overrides.
    #[cfg(test)]
    pub(in crate::main_menu) fn keyboard_bindings(&self) -> &[Ctp2KeyBinding] {
        self.keyboard_bindings.as_slice()
    }
}

/// Returns a listbox offset that reveals the selected row.
fn scroll_offset_for_selected(
    selected_index: usize,
    visible_rows: usize,
    item_count: usize,
) -> usize {
    let max_offset = item_count.saturating_sub(visible_rows);
    if selected_index < visible_rows {
        0_usize
    } else {
        selected_index.min(max_offset)
    }
}

/// Returns a scroll offset after applying a signed row delta.
fn scrolled_offset(current: usize, delta: i32, item_count: usize, visible_rows: usize) -> usize {
    let max_offset = item_count.saturating_sub(visible_rows);
    if delta < 0_i32 {
        let distance = usize::try_from(delta.unsigned_abs()).map_or(usize::MAX, |value| value);
        current.saturating_sub(distance)
    } else {
        let distance = usize::try_from(delta).map_or(usize::MAX, |value| value);
        current.saturating_add(distance).min(max_offset)
    }
}

/// Parses an option resolution label into numeric dimensions.
fn parse_resolution_label(label: &str) -> Option<(u32, u32)> {
    let (width, height) = label.split_once('x')?;
    let width = width.trim().parse::<u32>().ok()?;
    let height = height.trim().parse::<u32>().ok()?;
    Some((width, height))
}
