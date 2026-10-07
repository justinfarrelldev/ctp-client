//! Testable CTP2 single-player shell screen models.
//!
//! The values in this module are copied from the original CTP2 LDL files and
//! interface source so the SDL runtime can render and navigate the shell without
//! mutating or extracting copyrighted assets.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use super::{ButtonStyleId, MenuAssetReference, MenuRect};

/// Shared archive containing CTP2 shell graphics.
const SHELL_PICTURES_ARCHIVE: &str = "ctp2_data/default/graphics/pictures/pic565.zfs";
/// Original click sound when a button is pressed.
const BUTTON_PRESS_SOUND: &str = "AUI_SOUNDBASE_SOUND_ENGAGE";
/// Original click sound when a button executes.
const BUTTON_EXECUTE_SOUND: &str = "AUI_SOUNDBASE_SOUND_EXECUTE";

/// Control builder helpers.
mod builders;
/// Single-player hub screen builder.
mod hub;
/// New-game screen builder.
mod new_game;
/// Options screen builders.
mod options;
/// Picker and option-list builders.
mod pickers;
/// Original shell screen dispatcher.
mod screens;

/// Stable identifier for screens in the single-player shell flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ShellScreenId {
    /// Landing page before the single-player button is activated.
    Landing,
    /// `SPWindow` single-player hub.
    SinglePlayerHub,
    /// `SPNewGameWindow` game-creation page.
    NewGame,
    /// `OptionsWindow` root options popup.
    Options,
    /// `GraphicsWindow` options popup.
    Graphics,
    /// `ScreenResScreen` resolution picker.
    Resolution,
    /// `SoundWindow` volume popup.
    Sound,
    /// `MusicScreen` music options popup.
    Music,
    /// `MusicTrackScreen` track picker popup.
    MusicTrack,
    /// Nation selection popup opened from `TribeButton`.
    NationPicker,
    /// Difficulty selection popup opened from `DifficultyButton`.
    DifficultyPicker,
    /// Map-size selection popup opened from `MapSizeButton`.
    MapSizePicker,
    /// World-shape selection popup opened from `WorldShapeButton`.
    WorldShapePicker,
    /// World-type selection popup opened from `MapButton` / `WorldType`.
    WorldTypePicker,
    /// Player-count selection popup opened from `PlayersButton` / `Opponent`.
    PlayersPicker,
    /// Rules selection popup opened from `RulesButton`.
    RulesPicker,
}

/// Configurable game-setup field shown on the new-game screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GameSetupField {
    /// Player nation/civilisation.
    Nation,
    /// Difficulty level.
    Difficulty,
    /// Map size.
    MapSize,
    /// World shape.
    WorldShape,
    /// World type.
    WorldType,
    /// Number of computer opponents.
    Players,
    /// Rule preset.
    Rules,
}

/// Configurable option field from the original pre-game options screens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ShellOptionField {
    /// SFX volume slider.
    SfxVolume,
    /// Voice volume slider.
    VoiceVolume,
    /// Music volume slider.
    MusicVolume,
    /// Unit animation speed slider.
    UnitSpeed,
    /// Resolution picker.
    Resolution,
    /// Music enabled switch.
    MusicEnabled,
    /// Auto-repeat switch.
    AutoRepeat,
    /// Random order switch.
    RandomOrder,
    /// User music track.
    MusicTrack,
}

/// Action emitted by a shell control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ShellAction {
    /// Navigate to another shell screen.
    Navigate(ShellScreenId),
    /// Open a game-setup picker.
    OpenPicker(GameSetupField),
    /// Open an option picker/control.
    OpenOption(ShellOptionField),
    /// Select one option by index for the active picker and close it.
    SelectGameSetup(GameSetupField, usize),
    /// Select one option by index while keeping the active picker open.
    SelectGameSetupInPlace(GameSetupField, usize),
    /// Select one option value by index.
    SelectOption(ShellOptionField, usize),
    /// Toggle one boolean option.
    ToggleOption(ShellOptionField),
    /// Start the single-player game using the selected setup.
    LaunchGame,
    /// Return to the previous shell screen.
    Back,
    /// Quit the shell/client.
    Quit,
    /// Control is present but not yet backed by gameplay/server behavior.
    Disabled,
}

/// Original sound policy attached to an actionable shell control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ShellSoundPolicy {
    /// Press sound identifier from the AUI source.
    press: &'static str,
    /// Execute sound identifier from the AUI source.
    execute: &'static str,
}

impl ShellSoundPolicy {
    /// Returns the original AUI button sound policy.
    const fn original_button() -> Self {
        Self {
            press: BUTTON_PRESS_SOUND,
            execute: BUTTON_EXECUTE_SOUND,
        }
    }

    /// Returns the press sound identifier.
    #[cfg(test)]
    pub(super) const fn press(self) -> &'static str {
        self.press
    }

    /// Returns the execute sound identifier.
    #[cfg(test)]
    pub(super) const fn execute(self) -> &'static str {
        self.execute
    }
}

/// Original single-image shell art used by radio, checkbox, and toggle controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ShellImageSet {
    /// Normal/unselected image name from the LDL file.
    normal: &'static str,
    /// Selected/on image name from the LDL file.
    selected: &'static str,
    /// Disabled normal image name from the LDL file.
    disabled_normal: Option<&'static str>,
    /// Disabled selected image name from the LDL file.
    disabled_selected: Option<&'static str>,
}

impl ShellImageSet {
    /// Creates a new shell image set from LDL asset names.
    const fn new(
        normal: &'static str,
        selected: &'static str,
        disabled_normal: Option<&'static str>,
        disabled_selected: Option<&'static str>,
    ) -> Self {
        Self {
            normal,
            selected,
            disabled_normal,
            disabled_selected,
        }
    }

    /// Returns the popup checkbox/radio image family from `spnewgamepopups.ldl`.
    pub(super) const fn popup_checkbox() -> Self {
        Self::new(
            "upba2317.tga",
            "upba2318.tga",
            Some("upba2323.tga"),
            Some("upba2324.tga"),
        )
    }

    /// Returns the graphics options checkbox image family from `graphics.ldl`.
    pub(super) const fn graphics_checkbox() -> Self {
        Self::new(
            "upbt11u0.tga",
            "upbt11u1.tga",
            Some("upbt11x0.tga"),
            Some("upbt11x1.tga"),
        )
    }

    /// Returns the selected or normal image for the current control state.
    pub(super) const fn image_for_state(self, selected: bool, enabled: bool) -> &'static str {
        match (selected, enabled) {
            (false, true) => self.normal,
            (true, true) => self.selected,
            (false, false) => match self.disabled_normal {
                Some(image) => image,
                None => self.normal,
            },
            (true, false) => match self.disabled_selected {
                Some(image) => image,
                None => self.selected,
            },
        }
    }
}

/// Visual/control family for one shell element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ShellControlKind {
    /// Original CTP2 button with three-piece shell art.
    Button(ButtonStyleId),
    /// Static text label.
    Label,
    /// Text-entry field.
    TextField,
    /// Slider with a bounded integer range.
    Slider {
        /// Minimum selectable value.
        min: i32,
        /// Maximum selectable value.
        max: i32,
        /// Current value.
        value: i32,
    },
    /// Toggle switch.
    Switch {
        /// Whether the switch is currently selected/on.
        enabled: bool,
        /// Optional original image family for the switch state.
        images: Option<ShellImageSet>,
    },
    /// Pattern/image decoration or popup frame piece.
    Decoration,
    /// List or dropdown item.
    ListItem {
        /// Optional original image family for selectable list items.
        images: Option<ShellImageSet>,
        /// Whether the list item is selected/on.
        selected: bool,
    },
}

/// One modeled element from an original LDL shell screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ShellControl {
    /// Original LDL object name.
    ldl_name: &'static str,
    /// Current label text, already translated to the visible English base-game label.
    label: String,
    /// Accessible name for tests and non-coordinate activation.
    accessible_name: String,
    /// Original logical bounds.
    rect: MenuRect,
    /// Kind of control and visual family.
    kind: ShellControlKind,
    /// Optional action emitted by activation.
    action: Option<ShellAction>,
    /// Original sounds for interactive controls.
    sound: Option<ShellSoundPolicy>,
    /// Short validation source reference.
    source: &'static str,
}

impl ShellControl {
    /// Returns the original LDL name.
    #[cfg(test)]
    pub(super) const fn ldl_name(&self) -> &'static str {
        self.ldl_name
    }

    /// Returns the visible label.
    pub(super) fn label(&self) -> &str {
        self.label.as_str()
    }

    /// Returns the accessible name.
    #[cfg(test)]
    pub(super) fn accessible_name(&self) -> &str {
        self.accessible_name.as_str()
    }

    /// Returns the original bounds.
    pub(super) const fn rect(&self) -> MenuRect {
        self.rect
    }

    /// Returns the control kind.
    pub(super) const fn kind(&self) -> ShellControlKind {
        self.kind
    }

    /// Returns the activation action.
    pub(super) const fn action(&self) -> Option<ShellAction> {
        self.action
    }

    /// Returns original sound policy.
    #[cfg(test)]
    pub(super) const fn sound(&self) -> Option<ShellSoundPolicy> {
        self.sound
    }

    /// Returns the source reference used for validation.
    #[cfg(test)]
    pub(super) const fn source(&self) -> &'static str {
        self.source
    }

    /// Returns whether this control has actionable button semantics.
    pub(super) const fn is_actionable(&self) -> bool {
        self.action.is_some()
    }
}

/// One original shell screen with controls and optional background.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ShellScreen {
    /// Stable screen identifier.
    id: ShellScreenId,
    /// Original LDL block name.
    ldl_name: &'static str,
    /// Logical screen bounds.
    rect: MenuRect,
    /// Optional full-screen background asset.
    background: Option<MenuAssetReference>,
    /// Optional tiled popup pattern asset.
    pattern: Option<MenuAssetReference>,
    /// Modeled controls.
    controls: Vec<ShellControl>,
}

impl ShellScreen {
    /// Returns the screen identifier.
    #[cfg(test)]
    pub(super) const fn id(&self) -> ShellScreenId {
        self.id
    }

    /// Returns the original LDL block name.
    #[cfg(test)]
    pub(super) const fn ldl_name(&self) -> &'static str {
        self.ldl_name
    }

    /// Returns the logical bounds.
    pub(super) const fn rect(&self) -> MenuRect {
        self.rect
    }

    /// Returns the optional original background image.
    pub(super) const fn background(&self) -> Option<MenuAssetReference> {
        self.background
    }

    /// Returns the optional original tiled pattern.
    pub(super) const fn pattern(&self) -> Option<MenuAssetReference> {
        self.pattern
    }

    /// Returns all controls.
    pub(super) fn controls(&self) -> &[ShellControl] {
        self.controls.as_slice()
    }

    /// Finds a control by accessible name.
    #[cfg(test)]
    pub(super) fn find_accessible(&self, accessible_name: &str) -> Option<&ShellControl> {
        self.controls.iter().find(|control| {
            control
                .accessible_name
                .eq_ignore_ascii_case(accessible_name)
        })
    }

    /// Finds a control by point.
    pub(super) fn find_at_point(&self, point_x: i32, point_y: i32) -> Option<&ShellControl> {
        self.controls.iter().find(|control| {
            let global_rect = MenuRect::new(
                self.rect.x.saturating_add(control.rect.x),
                self.rect.y.saturating_add(control.rect.y),
                control.rect.width,
                control.rect.height,
            );
            global_rect.contains_point(point_x, point_y)
        })
    }
}

/// Current game setup selections before pressing Launch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GameSetupState {
    /// Selected nation label.
    nation: String,
    /// Selected leader name.
    leader_name: String,
    /// Selected difficulty label.
    difficulty: String,
    /// Selected map size.
    map_size: String,
    /// Selected world shape.
    world_shape: String,
    /// Selected world type.
    world_type: String,
    /// Selected player count.
    players: String,
    /// Selected rules preset.
    rules: String,
}

impl Default for GameSetupState {
    fn default() -> Self {
        Self {
            nation: "ROMANS".to_owned(),
            leader_name: "Caesar".to_owned(),
            difficulty: "PRINCE".to_owned(),
            map_size: "MEDIUM 48x96".to_owned(),
            world_shape: "EARTH WORLD".to_owned(),
            world_type: "RANDOM MAP".to_owned(),
            players: "4 PLAYERS".to_owned(),
            rules: "BLOODLUST OFF".to_owned(),
        }
    }
}

impl GameSetupState {
    /// Returns the selected value for a setup field.
    pub(super) fn selected(&self, field: GameSetupField) -> &str {
        match field {
            GameSetupField::Nation => self.nation.as_str(),
            GameSetupField::Difficulty => self.difficulty.as_str(),
            GameSetupField::MapSize => self.map_size.as_str(),
            GameSetupField::WorldShape => self.world_shape.as_str(),
            GameSetupField::WorldType => self.world_type.as_str(),
            GameSetupField::Players => self.players.as_str(),
            GameSetupField::Rules => self.rules.as_str(),
        }
    }

    /// Applies a setup selection by field and index.
    pub(super) fn select(&mut self, field: GameSetupField, index: usize) -> bool {
        let Some(value) = pickers::setup_options(field).get(index) else {
            return false;
        };
        match field {
            GameSetupField::Nation => (*value).clone_into(&mut self.nation),
            GameSetupField::Difficulty => (*value).clone_into(&mut self.difficulty),
            GameSetupField::MapSize => (*value).clone_into(&mut self.map_size),
            GameSetupField::WorldShape => (*value).clone_into(&mut self.world_shape),
            GameSetupField::WorldType => (*value).clone_into(&mut self.world_type),
            GameSetupField::Players => (*value).clone_into(&mut self.players),
            GameSetupField::Rules => (*value).clone_into(&mut self.rules),
        }
        true
    }

    /// Returns selected leader name.
    pub(super) fn leader_name(&self) -> &str {
        self.leader_name.as_str()
    }
}

/// Functional pre-game option state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ShellOptionsState {
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
    /// Music on/off.
    music_enabled: bool,
    /// Auto-repeat music.
    auto_repeat: bool,
    /// Randomize music order.
    random_order: bool,
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
            music_enabled: true,
            auto_repeat: true,
            random_order: false,
            music_track: 1,
        }
    }
}

impl ShellOptionsState {
    /// Returns slider value for a functional option.
    pub(super) const fn numeric(&self, field: ShellOptionField) -> Option<i32> {
        match field {
            ShellOptionField::SfxVolume => Some(self.sfx_volume),
            ShellOptionField::VoiceVolume => Some(self.voice_volume),
            ShellOptionField::MusicVolume => Some(self.music_volume),
            ShellOptionField::UnitSpeed => Some(self.unit_speed),
            ShellOptionField::Resolution
            | ShellOptionField::MusicEnabled
            | ShellOptionField::AutoRepeat
            | ShellOptionField::RandomOrder
            | ShellOptionField::MusicTrack => None,
        }
    }

    /// Sets a numeric option, clamping to original bounds.
    pub(super) fn set_numeric(&mut self, field: ShellOptionField, value: i32) -> bool {
        match field {
            ShellOptionField::SfxVolume => self.sfx_volume = value.clamp(0_i32, 10_i32),
            ShellOptionField::VoiceVolume => self.voice_volume = value.clamp(0_i32, 10_i32),
            ShellOptionField::MusicVolume => self.music_volume = value.clamp(0_i32, 10_i32),
            ShellOptionField::UnitSpeed => self.unit_speed = value.clamp(0_i32, 9_i32),
            ShellOptionField::Resolution
            | ShellOptionField::MusicEnabled
            | ShellOptionField::AutoRepeat
            | ShellOptionField::RandomOrder
            | ShellOptionField::MusicTrack => return false,
        }
        true
    }

    /// Toggles a boolean option.
    pub(super) const fn toggle(&mut self, field: ShellOptionField) -> bool {
        match field {
            ShellOptionField::MusicEnabled => self.music_enabled = !self.music_enabled,
            ShellOptionField::AutoRepeat => self.auto_repeat = !self.auto_repeat,
            ShellOptionField::RandomOrder => self.random_order = !self.random_order,
            ShellOptionField::SfxVolume
            | ShellOptionField::VoiceVolume
            | ShellOptionField::MusicVolume
            | ShellOptionField::UnitSpeed
            | ShellOptionField::Resolution
            | ShellOptionField::MusicTrack => return false,
        }
        true
    }

    /// Selects a list/switch option.
    pub(super) fn select(&mut self, field: ShellOptionField, index: usize) -> bool {
        match field {
            ShellOptionField::Resolution => {
                let Some(value) = pickers::RESOLUTIONS.get(index) else {
                    return false;
                };
                (*value).clone_into(&mut self.resolution);
            }
            ShellOptionField::MusicEnabled => self.music_enabled = index != 0,
            ShellOptionField::AutoRepeat => self.auto_repeat = index != 0,
            ShellOptionField::RandomOrder => self.random_order = index != 0,
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

    /// Returns selected resolution label.
    pub(super) fn resolution(&self) -> &str {
        self.resolution.as_str()
    }

    /// Returns whether music is enabled.
    pub(super) const fn music_enabled(&self) -> bool {
        self.music_enabled
    }

    /// Returns whether auto-repeat is enabled.
    pub(super) const fn auto_repeat(&self) -> bool {
        self.auto_repeat
    }

    /// Returns whether random order is enabled.
    pub(super) const fn random_order(&self) -> bool {
        self.random_order
    }

    /// Returns selected music track number.
    pub(super) const fn music_track(&self) -> usize {
        self.music_track
    }
}

/// Full single-player shell state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SinglePlayerShellModel {
    /// Current visible screen.
    current: ShellScreenId,
    /// Previous screen stack for popup back behavior.
    history: Vec<ShellScreenId>,
    /// Game setup selections.
    setup: GameSetupState,
    /// Functional option selections.
    options: ShellOptionsState,
}

impl Default for SinglePlayerShellModel {
    fn default() -> Self {
        Self {
            current: ShellScreenId::Landing,
            history: Vec::new(),
            setup: GameSetupState::default(),
            options: ShellOptionsState::default(),
        }
    }
}

impl SinglePlayerShellModel {
    /// Returns current screen id.
    pub(super) const fn current(&self) -> ShellScreenId {
        self.current
    }

    /// Returns setup state.
    pub(super) const fn setup(&self) -> &GameSetupState {
        &self.setup
    }

    /// Returns options state.
    pub(super) const fn options(&self) -> &ShellOptionsState {
        &self.options
    }

    /// Returns mutable option state.
    pub(super) const fn options_mut(&mut self) -> &mut ShellOptionsState {
        &mut self.options
    }

    /// Activates a control by accessible name for non-coordinate tests.
    #[cfg(test)]
    pub(super) fn activate_accessible(
        &mut self,
        accessible_name: &str,
    ) -> Option<ShellFlowOutcome> {
        let screen = screen_for(self.current, &self.setup, &self.options);
        let action = screen.find_accessible(accessible_name)?.action()?;
        Some(self.apply_action(action))
    }

    /// Applies a shell action and returns whether it launched or quit.
    pub(super) fn apply_action(&mut self, action: ShellAction) -> ShellFlowOutcome {
        match action {
            ShellAction::Navigate(screen) => self.navigate(screen),
            ShellAction::OpenPicker(field) => self.navigate(pickers::picker_screen_for(field)),
            ShellAction::OpenOption(field) => self.navigate(pickers::option_screen_for(field)),
            ShellAction::SelectGameSetup(field, index) => {
                let selected = self.setup.select(field, index);
                if selected {
                    self.back();
                }
            }
            ShellAction::SelectGameSetupInPlace(field, index) => {
                let selected = self.setup.select(field, index);
                debug_assert!(
                    selected,
                    "in-place setup selection should target a valid option"
                );
            }
            ShellAction::SelectOption(field, index) => {
                let selected = self.options.select(field, index);
                if selected {
                    self.back();
                }
            }
            ShellAction::ToggleOption(field) => {
                let toggled = self.options.toggle(field);
                if !toggled {
                    self.navigate(pickers::option_screen_for(field));
                }
            }
            ShellAction::Back => self.back(),
            ShellAction::LaunchGame => return ShellFlowOutcome::LaunchGame,
            ShellAction::Quit => return ShellFlowOutcome::Quit,
            ShellAction::Disabled => {}
        }
        ShellFlowOutcome::Continue
    }

    /// Navigates to a screen while retaining back history.
    fn navigate(&mut self, screen: ShellScreenId) {
        if self.current != screen {
            self.history.push(self.current);
            self.current = screen;
        }
    }

    /// Returns to the previous screen.
    fn back(&mut self) {
        self.current = self.history.pop().unwrap_or(ShellScreenId::Landing);
    }
}

/// Result of applying a shell action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ShellFlowOutcome {
    /// Continue showing the shell.
    Continue,
    /// Launch gameplay.
    LaunchGame,
    /// Quit the shell/client.
    Quit,
}

/// Builds a screen using current setup/option state.
pub(super) fn screen_for(
    id: ShellScreenId,
    setup: &GameSetupState,
    options: &ShellOptionsState,
) -> ShellScreen {
    screens::screen_for(id, setup, options)
}

#[cfg(test)]
mod tests;
