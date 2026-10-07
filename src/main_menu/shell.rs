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
/// Functional shell option state.
mod options_state;
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
    /// Player-count selection popup opened from `PlayersButton` / `Opponent`.
    PlayersPicker,
    /// Rules selection popup opened from `RulesButton`.
    RulesPicker,
    /// `CustomMapWindow` opened from `SPNewGameWindow.MapButton`.
    CustomMap,
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
    /// Unit animation toggle.
    UnitAnimation,
    /// Animated trade-route toggle.
    AnimatedTradeRoutes,
    /// Wonder movie toggle.
    WonderMovies,
    /// City-name toggle.
    CityNames,
    /// Political-border toggle.
    PoliticalBorders,
    /// Trade-route visibility toggle.
    TradeRoutes,
    /// City-influence visibility toggle.
    CityInfluence,
    /// Grid visibility toggle.
    Grid,
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
    /// Focus the editable leader-name field.
    FocusLeaderName,
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
    /// Listbox chrome preserving original CTP2 list frame metadata.
    ListBoxChrome,
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
    /// Whether the control is enabled for visual state and activation.
    enabled: bool,
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

    /// Returns whether the control is enabled.
    pub(super) const fn enabled(&self) -> bool {
        self.enabled
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
        self.controls.iter().rev().find(|control| {
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
    /// Selected player count.
    players: String,
    /// Selected rules preset.
    rules: String,
    /// Whether the leader-name field is focused for text input.
    leader_name_focused: bool,
    /// Whether the next leader-name text input replaces the current contents.
    leader_name_replace_on_input: bool,
}

impl Default for GameSetupState {
    fn default() -> Self {
        Self {
            nation: "ROMANS".to_owned(),
            leader_name: "Julius Caesar".to_owned(),
            difficulty: "Medium".to_owned(),
            map_size: "Regular".to_owned(),
            world_shape: "Earth World".to_owned(),
            players: "4".to_owned(),
            rules: "Bloodlust - Off".to_owned(),
            leader_name_focused: false,
            leader_name_replace_on_input: false,
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
            GameSetupField::Nation => {
                (*value).clone_into(&mut self.nation);
                if let Some(leader_name) = pickers::leader_name_for_nation_index(index) {
                    leader_name.clone_into(&mut self.leader_name);
                }
            }
            GameSetupField::Difficulty => (*value).clone_into(&mut self.difficulty),
            GameSetupField::MapSize => (*value).clone_into(&mut self.map_size),
            GameSetupField::WorldShape => (*value).clone_into(&mut self.world_shape),
            GameSetupField::Players => (*value).clone_into(&mut self.players),
            GameSetupField::Rules => (*value).clone_into(&mut self.rules),
        }
        true
    }

    /// Returns selected leader name.
    pub(super) fn leader_name(&self) -> &str {
        self.leader_name.as_str()
    }

    /// Focuses or unfocuses the leader-name field.
    pub(super) const fn set_leader_name_focused(&mut self, focused: bool) {
        self.leader_name_focused = focused;
        self.leader_name_replace_on_input = focused;
    }

    /// Returns whether leader-name text input is focused.
    pub(super) const fn leader_name_focused(&self) -> bool {
        self.leader_name_focused
    }

    /// Appends editable leader-name text, enforcing original `FIELD.maxfieldlen`.
    pub(super) fn input_leader_name(&mut self, text: &str) {
        const MAX_LEADER_NAME_LEN: usize = 23;
        if self.leader_name_replace_on_input {
            self.leader_name.clear();
            self.leader_name_replace_on_input = false;
        }
        let current_length = self.leader_name.chars().count();
        let remaining = MAX_LEADER_NAME_LEN.saturating_sub(current_length);
        self.leader_name.extend(text.chars().take(remaining));
    }

    /// Removes one character from the editable leader-name field.
    pub(super) fn backspace_leader_name(&mut self) {
        if self.leader_name_replace_on_input {
            self.leader_name.clear();
            self.leader_name_replace_on_input = false;
            return;
        }
        let removed = self.leader_name.pop();
        debug_assert!(
            removed.is_some() || self.leader_name.is_empty(),
            "backspace should either remove a character or leave an empty leader name"
        );
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
    options: options_state::ShellOptionsState,
}

impl Default for SinglePlayerShellModel {
    fn default() -> Self {
        Self {
            current: ShellScreenId::Landing,
            history: Vec::new(),
            setup: GameSetupState::default(),
            options: options_state::ShellOptionsState::default(),
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
    pub(super) const fn options(&self) -> &options_state::ShellOptionsState {
        &self.options
    }

    /// Returns mutable option state.
    pub(super) const fn options_mut(&mut self) -> &mut options_state::ShellOptionsState {
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
            ShellAction::FocusLeaderName => self.setup.set_leader_name_focused(true),
            ShellAction::Back => {
                self.setup.set_leader_name_focused(false);
                self.back();
            }
            ShellAction::LaunchGame => return ShellFlowOutcome::LaunchGame,
            ShellAction::Quit => return ShellFlowOutcome::Quit,
            ShellAction::Disabled => {}
        }
        ShellFlowOutcome::Continue
    }

    /// Applies text input to focused fields at runtime.
    pub(super) fn handle_text_input(&mut self, text: &str) {
        if self.setup.leader_name_focused() {
            self.setup.input_leader_name(text);
        }
    }

    /// Applies backspace to focused fields at runtime.
    pub(super) fn handle_backspace(&mut self) {
        if self.setup.leader_name_focused() {
            self.setup.backspace_leader_name();
        }
    }

    /// Applies text input to focused fields in tests.
    #[cfg(test)]
    pub(super) fn input_text(&mut self, text: &str) {
        self.handle_text_input(text);
    }

    /// Applies backspace to focused fields in tests.
    #[cfg(test)]
    pub(super) fn backspace_text(&mut self) {
        self.handle_backspace();
    }

    /// Navigates to a screen while retaining back history.
    fn navigate(&mut self, screen: ShellScreenId) {
        self.setup.set_leader_name_focused(false);
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
    options: &options_state::ShellOptionsState,
) -> ShellScreen {
    screens::screen_for(id, setup, options)
}

#[cfg(test)]
mod tests;
