//! Non-persisted new-game setup state for the CTP2 shell.

extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::string::String;

use super::pickers;

/// Configurable game-setup field shown on the new-game screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::main_menu) enum GameSetupField {
    /// Player nation/civilisation.
    Nation,
    /// Difficulty level.
    Difficulty,
    /// Barbarian risk level paired with difficulty.
    BarbarianRisk,
    /// Map size.
    MapSize,
    /// World shape.
    WorldShape,
    /// Number of computer opponents/empires displayed by the shell.
    Players,
    /// Rule preset/toggle group.
    Rules,
}

/// Configurable custom-map generation slider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::main_menu) enum CustomMapField {
    /// Wet-to-dry terrain mix.
    WetDry,
    /// Warm-to-cold temperature mix.
    WarmCold,
    /// Ocean-to-land percentage.
    OceanLand,
    /// Island-to-continent percentage.
    IslandContinent,
    /// Homogeneous-to-diverse landmass distribution.
    HomoDiverse,
    /// Trade-good density.
    GoodCount,
}

/// Base-game default wet/dry setting from `ctp2_data/default/gamedata/profile.txt`.
const DEFAULT_WET_DRY: i32 = 5;
/// Base-game default warm/cold setting from `profile.txt`.
const DEFAULT_WARM_COLD: i32 = 5;
/// Base-game default ocean/land setting from `profile.txt`.
const DEFAULT_OCEAN_LAND: i32 = 3;
/// Base-game default island/continent setting from `profile.txt`.
const DEFAULT_ISLAND_CONTINENT: i32 = 5;
/// Base-game default homogeneous/diverse setting from `profile.txt`.
const DEFAULT_HOMO_DIVERSE: i32 = 8;
/// Base-game default good-count setting from `profile.txt`.
const DEFAULT_GOOD_COUNT: i32 = 5;
/// Minimum custom-map slider value.
const CUSTOM_MAP_SLIDER_MIN: i32 = 0;
/// Maximum custom-map slider value.
const CUSTOM_MAP_SLIDER_MAX: i32 = 10;

/// Non-persisted custom-map settings for the game currently being built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::main_menu) struct CustomMapState {
    /// Wet-to-dry terrain mix.
    wet_dry: i32,
    /// Warm-to-cold temperature mix.
    warm_cold: i32,
    /// Ocean-to-land percentage.
    ocean_land: i32,
    /// Island-to-continent percentage.
    island_continent: i32,
    /// Homogeneous-to-diverse landmass distribution.
    homo_diverse: i32,
    /// Trade-good density.
    good_count: i32,
}

impl Default for CustomMapState {
    fn default() -> Self {
        Self {
            wet_dry: DEFAULT_WET_DRY,
            warm_cold: DEFAULT_WARM_COLD,
            ocean_land: DEFAULT_OCEAN_LAND,
            island_continent: DEFAULT_ISLAND_CONTINENT,
            homo_diverse: DEFAULT_HOMO_DIVERSE,
            good_count: DEFAULT_GOOD_COUNT,
        }
    }
}

impl CustomMapState {
    /// Returns the current value for one custom-map slider.
    pub(super) const fn value(&self, field: CustomMapField) -> i32 {
        match field {
            CustomMapField::WetDry => self.wet_dry,
            CustomMapField::WarmCold => self.warm_cold,
            CustomMapField::OceanLand => self.ocean_land,
            CustomMapField::IslandContinent => self.island_continent,
            CustomMapField::HomoDiverse => self.homo_diverse,
            CustomMapField::GoodCount => self.good_count,
        }
    }

    /// Sets one custom-map slider value, clamped to the original 0..10 range.
    pub(super) fn set_value(&mut self, field: CustomMapField, value: i32) -> bool {
        let clamped = value.clamp(CUSTOM_MAP_SLIDER_MIN, CUSTOM_MAP_SLIDER_MAX);
        match field {
            CustomMapField::WetDry => self.wet_dry = clamped,
            CustomMapField::WarmCold => self.warm_cold = clamped,
            CustomMapField::OceanLand => self.ocean_land = clamped,
            CustomMapField::IslandContinent => self.island_continent = clamped,
            CustomMapField::HomoDiverse => self.homo_diverse = clamped,
            CustomMapField::GoodCount => self.good_count = clamped,
        }
        true
    }
}

/// Independent game-rule toggles from `SPNewGameRulesScreen`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GameRulesState {
    /// Whether bloodlust is enabled.
    bloodlust_enabled: bool,
    /// Whether pollution is enabled.
    pollution_enabled: bool,
}

impl Default for GameRulesState {
    fn default() -> Self {
        Self {
            bloodlust_enabled: false,
            pollution_enabled: true,
        }
    }
}

/// Editable leader-name focus state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct LeaderNameEditState {
    /// Whether the leader-name field is focused for text input.
    focused: bool,
    /// Whether the next leader-name text input replaces the current contents.
    replace_on_input: bool,
}

/// Current game setup selections before pressing Launch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::main_menu) struct GameSetupState {
    /// Selected nation label.
    nation: String,
    /// Selected leader name.
    leader_name: String,
    /// Selected difficulty label.
    difficulty: String,
    /// Selected barbarian-risk label.
    barbarian_risk: String,
    /// Selected map size.
    map_size: String,
    /// Selected world shape.
    world_shape: String,
    /// Selected player count displayed by the shell.
    players: String,
    /// Independent game-rule toggles.
    rules: GameRulesState,
    /// Non-persisted custom-map slider values.
    custom_map: CustomMapState,
    /// Editable leader-name focus state.
    leader_name_edit: LeaderNameEditState,
}

impl Default for GameSetupState {
    fn default() -> Self {
        Self {
            nation: "ROMANS".to_owned(),
            leader_name: "Julius Caesar".to_owned(),
            difficulty: "Medium".to_owned(),
            barbarian_risk: "Bandits".to_owned(),
            map_size: "Regular".to_owned(),
            world_shape: "Earth World".to_owned(),
            players: "5".to_owned(),
            rules: GameRulesState::default(),
            custom_map: CustomMapState::default(),
            leader_name_edit: LeaderNameEditState::default(),
        }
    }
}

impl GameSetupState {
    /// Returns the selected value for a setup field.
    pub(super) fn selected(&self, field: GameSetupField) -> &str {
        match field {
            GameSetupField::Nation => self.nation.as_str(),
            GameSetupField::Difficulty => self.difficulty.as_str(),
            GameSetupField::BarbarianRisk => self.barbarian_risk.as_str(),
            GameSetupField::MapSize => self.map_size.as_str(),
            GameSetupField::WorldShape => self.world_shape.as_str(),
            GameSetupField::Players => self.players.as_str(),
            GameSetupField::Rules => self.rule_label(0),
        }
    }

    /// Applies a setup selection by field and index.
    pub(super) fn select(&mut self, field: GameSetupField, index: usize) -> bool {
        if matches!(field, GameSetupField::Rules) {
            return self.toggle_rule(index);
        }
        let Some(value) = pickers::setup_options(field).get(index) else {
            return false;
        };
        match field {
            GameSetupField::Nation => {
                (*value).clone_into(&mut self.nation);
                if let Some(leader_name) = pickers::leader_name_for_nation_index(index) {
                    leader_name.clone_into(&mut self.leader_name);
                }
            }
            GameSetupField::Difficulty => (*value).clone_into(&mut self.difficulty),
            GameSetupField::BarbarianRisk => (*value).clone_into(&mut self.barbarian_risk),
            GameSetupField::MapSize => (*value).clone_into(&mut self.map_size),
            GameSetupField::WorldShape => (*value).clone_into(&mut self.world_shape),
            GameSetupField::Players => (*value).clone_into(&mut self.players),
            GameSetupField::Rules => return false,
        }
        true
    }

    /// Returns selected leader name.
    pub(super) fn leader_name(&self) -> &str {
        self.leader_name.as_str()
    }

    /// Focuses or unfocuses the leader-name field.
    pub(super) const fn set_leader_name_focused(&mut self, focused: bool) {
        self.leader_name_edit.focused = focused;
        self.leader_name_edit.replace_on_input = focused;
    }

    /// Returns whether leader-name text input is focused.
    pub(super) const fn leader_name_focused(&self) -> bool {
        self.leader_name_edit.focused
    }

    /// Appends editable leader-name text, enforcing original `FIELD.maxfieldlen`.
    pub(super) fn input_leader_name(&mut self, text: &str) {
        const MAX_LEADER_NAME_LEN: usize = 23;
        if self.leader_name_edit.replace_on_input {
            self.leader_name.clear();
            self.leader_name_edit.replace_on_input = false;
        }
        let current_length = self.leader_name.chars().count();
        let remaining = MAX_LEADER_NAME_LEN.saturating_sub(current_length);
        self.leader_name.extend(text.chars().take(remaining));
    }

    /// Removes one character from the editable leader-name field.
    pub(super) fn backspace_leader_name(&mut self) {
        if self.leader_name_edit.replace_on_input {
            self.leader_name_edit.replace_on_input = false;
        }
        let removed = self.leader_name.pop();
        debug_assert!(
            removed.is_some() || self.leader_name.is_empty(),
            "backspace should either remove a character or leave an empty leader name"
        );
    }

    /// Returns one custom-map slider value.
    pub(super) const fn custom_map_value(&self, field: CustomMapField) -> i32 {
        self.custom_map.value(field)
    }

    /// Sets one custom-map slider value.
    pub(super) fn set_custom_map_value(&mut self, field: CustomMapField, value: i32) -> bool {
        self.custom_map.set_value(field, value)
    }

    /// Returns the current label for one rule switch by rule index.
    pub(super) const fn rule_label(&self, index: usize) -> &'static str {
        match index {
            0 => {
                if self.rules.bloodlust_enabled {
                    "Bloodlust - On"
                } else {
                    "Bloodlust - Off"
                }
            }
            1 => {
                if self.rules.pollution_enabled {
                    "Pollution - On"
                } else {
                    "Pollution - Off"
                }
            }
            _ => "Rule",
        }
    }

    /// Returns whether one rule switch is selected/on.
    pub(super) const fn rule_enabled(&self, index: usize) -> bool {
        match index {
            0 => self.rules.bloodlust_enabled,
            1 => self.rules.pollution_enabled,
            _ => false,
        }
    }

    /// Toggles one rule switch in place.
    const fn toggle_rule(&mut self, index: usize) -> bool {
        match index {
            0 => self.rules.bloodlust_enabled = !self.rules.bloodlust_enabled,
            1 => self.rules.pollution_enabled = !self.rules.pollution_enabled,
            _ => return false,
        }
        true
    }
}
