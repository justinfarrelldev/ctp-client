//! Persisted CTP2 shell option structures.

use serde::{Deserialize, Serialize};

/// Boolean toggle persisted as a TOML boolean while avoiding large bool-heavy structs.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(transparent)]
#[non_exhaustive]
pub struct Ctp2Toggle {
    /// Whether the option is enabled.
    pub enabled: bool,
}

impl Ctp2Toggle {
    /// Creates a toggle from a boolean value.
    #[must_use]
    pub const fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Returns whether the option is enabled.
    #[must_use]
    pub const fn enabled(self) -> bool {
        self.enabled
    }
}

/// Screen resolution persisted for the CTP2 shell and intro movie.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Ctp2Resolution {
    /// Window width in pixels.
    pub width: u32,
    /// Window height in pixels.
    pub height: u32,
}

impl Default for Ctp2Resolution {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
        }
    }
}

/// Persisted CTP2 sound option values.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Ctp2SoundOptions {
    /// Sound-effect volume on the original 0-10 scale.
    pub sound_volume: i32,
    /// Voice volume on the original 0-10 scale.
    pub voice_volume: i32,
    /// Music volume on the original 0-10 scale.
    pub music_volume: i32,
}

impl Default for Ctp2SoundOptions {
    fn default() -> Self {
        Self {
            sound_volume: 5,
            voice_volume: 5,
            music_volume: 5,
        }
    }
}

/// Persisted CTP2 graphics option values.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Ctp2GraphicsOptions {
    /// Unit animation speed on the original 0-9 scale.
    pub unit_speed: i32,
    /// Whether unit animations are enabled.
    pub unit_animation: Ctp2Toggle,
    /// Whether animated trade routes are enabled.
    pub animated_trade_routes: Ctp2Toggle,
    /// Whether wonder movies are enabled.
    pub wonder_movies: Ctp2Toggle,
    /// Whether city names are shown.
    pub city_names: Ctp2Toggle,
    /// Whether political borders are shown.
    pub political_borders: Ctp2Toggle,
    /// Whether trade routes are shown.
    pub trade_routes: Ctp2Toggle,
    /// Whether city influence overlays are shown.
    pub city_influence: Ctp2Toggle,
    /// Whether map grid lines are shown.
    pub grid: Ctp2Toggle,
    /// Application/window resolution.
    pub resolution: Ctp2Resolution,
}

impl Default for Ctp2GraphicsOptions {
    fn default() -> Self {
        Self {
            unit_speed: 5,
            unit_animation: Ctp2Toggle::new(true),
            animated_trade_routes: Ctp2Toggle::new(true),
            wonder_movies: Ctp2Toggle::new(true),
            city_names: Ctp2Toggle::new(true),
            political_borders: Ctp2Toggle::new(true),
            trade_routes: Ctp2Toggle::new(true),
            city_influence: Ctp2Toggle::new(true),
            grid: Ctp2Toggle::new(false),
            resolution: Ctp2Resolution::default(),
        }
    }
}

/// Persisted CTP2 music option values.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Ctp2MusicOptions {
    /// One-based selected music track number.
    pub selected_track: usize,
    /// Whether music playback is enabled.
    pub music_enabled: bool,
    /// Whether the selected/playlist music should repeat.
    pub auto_repeat: bool,
    /// Whether playlist ordering should be randomized.
    pub random_order: bool,
}

impl Default for Ctp2MusicOptions {
    fn default() -> Self {
        Self {
            selected_track: 2,
            music_enabled: true,
            auto_repeat: false,
            random_order: false,
        }
    }
}

/// Persisted CTP2 gameplay option values.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Ctp2GameplayOptions {
    /// Whether tutorial/advice prompts are enabled.
    pub tutorial_advice: Ctp2Toggle,
    /// Whether automatic turn cycling is enabled.
    pub auto_cycle_turns: Ctp2Toggle,
    /// Whether automatic unit cycling is enabled.
    pub auto_cycle_units: Ctp2Toggle,
    /// Whether automatic map centering is enabled.
    pub auto_center: Ctp2Toggle,
    /// Whether automatic control-panel tab switching is enabled.
    pub auto_tab_select: Ctp2Toggle,
    /// Whether autosave is enabled.
    pub auto_save: Ctp2Toggle,
    /// Whether zoomed battle view is enabled.
    pub battle_view: Ctp2Toggle,
    /// Whether all battles should be shown in zoomed battle view.
    pub show_all_battles: Ctp2Toggle,
    /// Whether enemy movement animations are shown.
    pub enemy_moves: Ctp2Toggle,
    /// Whether left-handed mouse mode is enabled.
    pub left_handed_mouse: Ctp2Toggle,
    /// Mouse speed on the original 0-15 scale.
    pub mouse_speed: i32,
}

impl Default for Ctp2GameplayOptions {
    fn default() -> Self {
        Self {
            tutorial_advice: Ctp2Toggle::new(false),
            auto_cycle_turns: Ctp2Toggle::new(true),
            auto_cycle_units: Ctp2Toggle::new(true),
            auto_center: Ctp2Toggle::new(true),
            auto_tab_select: Ctp2Toggle::new(true),
            auto_save: Ctp2Toggle::new(true),
            battle_view: Ctp2Toggle::new(true),
            show_all_battles: Ctp2Toggle::new(false),
            enemy_moves: Ctp2Toggle::new(true),
            left_handed_mouse: Ctp2Toggle::new(false),
            mouse_speed: 5,
        }
    }
}

/// Keyboard-mapping category shown by the original CTP2 keyboard screen.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Ctp2KeyboardCategory {
    /// Basic commands category.
    Basic,
    /// Unit controls category.
    Unit,
    /// Screen controls category.
    Screen,
    /// Map controls category.
    Map,
    /// Game options category.
    Game,
}

/// One persisted CTP2 keyboard binding override.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Ctp2KeyBinding {
    /// Stable action identifier.
    pub action: String,
    /// Human-readable key name.
    pub key: String,
}

/// Persisted CTP2 keyboard option values.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Ctp2KeyboardOptions {
    /// Category selected the last time the keyboard mapping screen was shown.
    pub selected_category: Ctp2KeyboardCategory,
    /// User key-binding overrides; empty means default bindings.
    pub bindings: Vec<Ctp2KeyBinding>,
}

impl Default for Ctp2KeyboardOptions {
    fn default() -> Self {
        Self {
            selected_category: Ctp2KeyboardCategory::Basic,
            bindings: Vec::new(),
        }
    }
}

/// Persisted CTP2 shell option values.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct Ctp2Options {
    /// Sound options.
    pub sound: Ctp2SoundOptions,
    /// Graphics options.
    pub graphics: Ctp2GraphicsOptions,
    /// Music options.
    pub music: Ctp2MusicOptions,
    /// Gameplay options.
    pub gameplay: Ctp2GameplayOptions,
    /// Keyboard mapping options.
    pub keyboard: Ctp2KeyboardOptions,
}
