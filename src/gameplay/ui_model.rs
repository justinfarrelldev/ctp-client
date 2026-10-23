//! Testable in-game menu bar, control panel, shortcuts, and popup UI model.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use sdl2::keyboard::{Keycode, Mod};

use self::menu_manifest::{
    dropdown_width_for_rows, original_menu_buttons, original_menu_items, text_width,
};
use super::audio::GameplaySound;
use super::model::{GameState, MapPoint};

/// Source-aligned, viewer-safe map context menus.
pub mod context;
#[cfg(test)]
mod context_tests;
/// Actual own-history metrics and overflow-safe native graph geometry.
pub mod history;
#[cfg(test)]
mod history_tests;
/// Local original-article browsing and shared rich-text link layout.
pub mod library;
#[cfg(test)]
mod library_tests;
/// Source-derived manager windows and authoritative semantic interactions.
pub mod management;
#[cfg(test)]
mod management_tests;
/// Original CTP2 top-menu manifest and geometry helpers.
mod menu_manifest;
#[cfg(test)]
mod message_tests;
/// Bounded private semantic results and local read/unread state.
pub mod messages;
/// Cached observation/quote/forecast presentation without local simulation.
pub mod observed;
#[cfg(test)]
mod observed_tests;
#[cfg(test)]
mod operation_tests;
/// Explicit revision-bound input and target/priority dialogs.
pub mod operations;
/// Stable source-aligned expanded unit/army grid.
pub mod stack;
/// Local source tooltip ownership and native/semantic footer agreement.
mod status;
/// Source footer descriptions use semantic identities and preserve authority status.
#[cfg(test)]
mod status_tests;

/// Source tile-palette category, independent of any authoritative improvement order.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TileCategory {
    /// Ordinary land improvements.
    #[default]
    Land,
    /// Ordinary water improvements.
    Ocean,
    /// Special-purpose improvements.
    Special,
    /// Land terrain transformations.
    TerraformLand,
    /// Water terrain transformations.
    TerraformOcean,
}

impl TileCategory {
    /// Original radio identity and caption.
    #[must_use]
    pub const fn control(self) -> (&'static str, &'static str) {
        match self {
            Self::Land => ("LandImpButton", "Land"),
            Self::Ocean => ("OceanImpButton", "Ocean"),
            Self::Special => ("SpecialImpButton", "Special"),
            Self::TerraformLand => ("TerraformLandButton", "Terraform Land"),
            Self::TerraformOcean => ("TerraformOceanButton", "Terraform Ocean"),
        }
    }
}

/// Original in-game menu bar height in pixels.
pub const MENU_BAR_HEIGHT: u32 = 24;
/// Original `MENU_BUTTON` width from `controlpanel.ldl`.
pub const MENU_BUTTON_WIDTH: u32 = 80;
/// Original `PlainMenuListItem` row height from `ctp_template.ldl`.
pub const MENU_DROPDOWN_ROW_HEIGHT: u32 = 16;
/// Original `CTP2_MENU`/list border inset from `ctp_template.ldl`.
pub const MENU_DROPDOWN_BORDER: u32 = 2;
/// Extra window width added by `ctp2_Menu::Resize`.
pub const MENU_DROPDOWN_EXTRA_WINDOW_WIDTH: u32 = 4;
/// Extra window height added by `ctp2_Menu::Resize`.
pub const MENU_DROPDOWN_EXTRA_WINDOW_HEIGHT: u32 = 4;
/// Initial LDL dropdown width, replaced (not enforced) by `ctp2_Menu::Resize`.
pub const MENU_DROPDOWN_MIN_WIDTH: u32 = 150;
/// Approximate 10-point Times menu glyph width for pure hit-test geometry.
const MENU_APPROX_GLYPH_WIDTH: u32 = 7;
/// Extra shortcut column width added by `ctp2_Menu::CreateItem`.
const MENU_EXTRA_SHORTCUT_WIDTH: u32 = 10;
/// Original left menu item margin.
const MENU_LEFT_ITEM_MARGIN: u32 = 3;
/// Original right menu item margin.
const MENU_RIGHT_ITEM_MARGIN: u32 = 3;
/// Extra left spacing before text in `ctp2_Menu::Reformat`.
const MENU_EXTRA_LEFT_SPACE: u32 = 4;
/// Approximate original control-panel width from `controlpanel.ldl`.
pub const CONTROL_PANEL_WIDTH: u32 = 601;
/// Approximate original control-panel height from `controlpanel.ldl`.
pub const CONTROL_PANEL_HEIGHT: u32 = 184;
/// Original end-turn button X within `ControlPanelWindow.ControlPanel`.
pub const END_TURN_BUTTON_X: i32 = 563;
/// Original end-turn button Y within `ControlPanelWindow.ControlPanel`.
pub const END_TURN_BUTTON_Y: i32 = 60;
/// Original `upcb14*` end-turn button width.
pub const END_TURN_BUTTON_WIDTH: u32 = 38;
/// Original `upcb14*` end-turn button height.
pub const END_TURN_BUTTON_HEIGHT: u32 = 86;
/// Original status-bar X within the control panel.
pub const STATUS_BAR_X: i32 = 126;
/// Original status-bar Y within the control panel.
pub const STATUS_BAR_Y: i32 = 167;

/// High-level action emitted by the in-game UI model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InGameAction {
    /// No action should be taken.
    None,
    /// Start a new single-player setup/game flow.
    NewGame,
    /// Quit the client.
    Quit,
    /// End the current turn.
    EndTurn,
    /// Open the Great Library.
    OpenGreatLibrary,
    /// Open the city build queue/build menu.
    OpenCityBuild,
    /// Open the city manager.
    OpenCityManager,
    /// Open the national city summary, not the domestic policy dialog.
    OpenNationalManager,
    /// Open the authoritative trade market/summary.
    OpenTradeManager,
    /// Open the unit manager.
    OpenUnitManager,
    /// Open the army manager.
    OpenArmyManager,
    /// Open civilization/government status.
    OpenCivilizationStatus,
    /// Open science status.
    OpenScienceStatus,
    /// Open diplomacy placeholder.
    OpenDiplomacy,
    /// Open the Ranking tab of the Statistics window.
    OpenInfo,
    /// Open the Score tab of the Statistics window.
    OpenScore,
    /// Open the Wonders tab of the Statistics window.
    OpenWonders,
    /// Open gameplay options.
    OpenGameplayOptions,
    /// Open graphics options.
    OpenGraphicsOptions,
    /// Open sound options.
    OpenSoundOptions,
    /// Open advanced options placeholder.
    OpenAdvancedOptions,
    /// Zoom the map in.
    ZoomIn,
    /// Zoom the map out.
    ZoomOut,
    /// Toggle city names.
    ToggleCityNames,
    /// Toggle trade routes.
    ToggleTradeRoutes,
    /// Toggle radar/minimap.
    ToggleRadar,
    /// Toggle fog-of-war overlay.
    ToggleFogOfWar,
    /// Settle selected settler.
    Settle,
    /// Put selected unit into sentinel; distinct from physical fortification.
    Sentinel,
    /// Fortify the selected unit through its exact legal server action.
    Fortify,
    /// Disband the selected unit through its exact legal server action.
    Disband,
    /// Move selected unit by a signed delta.
    MoveBy(i16, i16),
    /// Dismiss the top popup.
    DismissPopup,
    /// Save/Load item is intentionally out of scope.
    OutOfScope,
}

/// Menu category at the top of the in-game screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuCategory {
    /// Empire menu named `CivActivator` in `controlpanel.ldl`.
    Civilization,
    /// Cities menu.
    Cities,
    /// Units menu.
    Units,
    /// Diplomacy menu.
    Diplomacy,
    /// Stats/info menu.
    Info,
    /// Options menu.
    Options,
}

impl MenuCategory {
    /// Returns original visible title from `ldl_str.txt`.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Civilization => "Empire",
            Self::Cities => "Cities",
            Self::Units => "Units",
            Self::Diplomacy => "Diplomacy",
            Self::Info => "Stats",
            Self::Options => "Options",
        }
    }
}

/// One row in a top drop-down menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    /// Owning category.
    pub category: MenuCategory,
    /// Visible label.
    pub label: &'static str,
    /// Shortcut text copied from `keymap.txt` style.
    pub shortcut: &'static str,
    /// Action emitted when selected.
    pub action: InGameAction,
    /// Whether the row is active.
    pub enabled: bool,
    /// Original source reference.
    pub source: &'static str,
}

/// One top menu button.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuButton {
    /// Category represented by this button.
    pub category: MenuCategory,
    /// Visible label.
    pub label: &'static str,
    /// X coordinate in original menu coordinates.
    pub x: i32,
    /// Width in original menu coordinates.
    pub width: u32,
}

/// Integer rectangle used by pure UI hit-testing and SDL rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuRect {
    /// Left coordinate.
    pub x: i32,
    /// Top coordinate.
    pub y: i32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl MenuRect {
    /// Returns whether this rectangle contains the point.
    #[must_use]
    pub fn contains(self, x: i32, y: i32) -> bool {
        let right = self
            .x
            .saturating_add(i32::try_from(self.width).map_or(0_i32, |width| width));
        let bottom = self
            .y
            .saturating_add(i32::try_from(self.height).map_or(0_i32, |height| height));
        x >= self.x && x < right && y >= self.y && y < bottom
    }
}

/// Control-panel tab identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlPanelTab {
    /// Domestic/government tab.
    Domestic,
    /// City tab.
    City,
    /// Unit tab.
    Unit,
    /// Messages tab.
    Messages,
    /// Tile-improvement/filter tab.
    TileImprovements,
}

/// Great Library text opened from shortcut/menu/shortcut pad.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GreatLibraryView {
    /// Window title.
    pub title: String,
    /// Current topic title.
    pub topic: String,
    /// Body text excerpt.
    pub body: String,
}

/*
 * Source: ctp2_source/ctp2_code/ui/aui_common/aui_listbox.cpp::
 * MouseLDoubleClickInside and ui/interface/EditQueue.cpp::{ListCallback,
 * QueueListCallback} assume the first click selected the intended row.
 * Client safety: retain only presentation identity/city and received revision,
 * so an asynchronous snapshot cannot reinterpret the second press as another
 * row/city or silently rebase a command. No layout/style/scaling is changed.
 */
/// Original focused Build-list inputs, independent of OS key representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildListKey {
    /// Select the previous row, wrapping first to last.
    Up,
    /// Select the next row, wrapping last to first.
    Down,
    /// Source Space delegates a left list to its Add control.
    Activate,
}

/// First source Build row press; never a command or local queue execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagerRowPress {
    /// Exact revision visible when the first native press selected this row.
    pub revision: i64,
    /// Source manager identity.
    pub kind: management::ManagerKind,
    /// Explicit effective city; identical content in another city isn't a match.
    pub city_id: Option<String>,
    /// Independent source selection role, such as queue versus buildable.
    pub key: String,
    /// Canonical current entry/content ID, never the row's ordinal or label.
    pub id: String,
}

/// Full in-game UI model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InGameUiModel {
    /// Top menu buttons.
    pub menu_buttons: Vec<MenuButton>,
    /// Drop-down rows.
    pub menu_items: Vec<MenuItem>,
    /// Active control-panel tab.
    pub active_tab: ControlPanelTab,
    /// Currently open top-menu category.
    pub open_menu: Option<MenuCategory>,
    /// Whether the Great Library is open.
    pub great_library: Option<GreatLibraryView>,
    /// Status bar text.
    pub status_text: String,
    /// Persistent safe command rejection; a routine refresh is not acknowledgement.
    pub command_rejection: Option<crate::gameplay::server::rejection::Rejection>,
    /// Keyboard focus, rather than pointer hover, currently owns source descriptions.
    pub keyboard_status: bool,
    /// Measured original menu labels/shortcuts, shared by rendering and hit tests.
    /// Empty in font-independent tests, which use the documented fallback metric.
    pub menu_text_widths: Vec<(&'static str, u32)>,
    /// Optional tile beneath the pointer, used by the original yellow map cursor.
    pub hovered_tile: Option<MapPoint>,
    /// Logical-frame pointer location; absent outside/unfocused SDL windows.
    pub pointer: Option<(i32, i32)>,
    /// Open map context, owned by UI rather than immediately executing an order.
    pub context: Option<context::ContextView>,
    /// Visible/remembered tile explicitly opened for map information.
    pub map_info: Option<MapPoint>,
    /// Exact city selected for management, independent of unit selection.
    pub selected_city: Option<super::model::CityId>,
    /// Original single versus multiple unit-selection display mode.
    pub stack_expanded: bool,
    /// Presentation-only source tile palette, not a selected gameplay order.
    pub tile_category: TileCategory,
    /// Source-layout management windows, independent of the lower panel tabs.
    pub management: management::ManagementUi,
    /// Source row press canceled by revision/scope mismatch or modal/focus changes.
    pub manager_press: Option<ManagerRowPress>,
    /// Explicit original-style input/target form above any manager.
    pub operation: Option<operations::OperationDialog>,
    /// Viewer-authorized message window, never a local reward executor.
    pub messages: messages::MessageLog,
}

impl Default for InGameUiModel {
    fn default() -> Self {
        Self {
            menu_buttons: original_menu_buttons(),
            menu_items: original_menu_items(),
            active_tab: ControlPanelTab::Unit,
            open_menu: None,
            great_library: None,
            status_text: String::new(),
            command_rejection: None,
            keyboard_status: false,
            menu_text_widths: Vec::new(),
            hovered_tile: None,
            pointer: None,
            context: None,
            map_info: None,
            selected_city: None,
            stack_expanded: false,
            tile_category: TileCategory::Land,
            management: management::ManagementUi::default(),
            manager_press: None,
            operation: None,
            messages: messages::MessageLog::default(),
        }
    }
}

impl InGameUiModel {
    /// Exact source description/disabled reason, shared with accessible automation.
    #[must_use]
    pub fn source_status_for_id(&self, id: &str, state: &GameState) -> Option<String> {
        status::source_id(self, id, state)
    }
    /// Ephemeral pointer/focus descriptions preserve authoritative result/recovery text.
    #[must_use]
    pub fn footer_status(&self, state: &GameState) -> alloc::borrow::Cow<'_, str> {
        status::footer(self, state)
    }
    /// Updates menu geometry from the same font that paints its labels.
    pub fn measure_menu_text(&mut self, measure: impl Fn(&str) -> u32) {
        self.menu_text_widths = self
            .menu_items
            .iter()
            .flat_map(|item| [item.label, item.shortcut])
            .map(|text| (text, measure(text)))
            .collect();
        self.management.measure_text(&measure, 16_u32);
    }

    /// Returns installed-font width, or the pure-model fallback when no font exists.
    fn measured_text_width(&self, text: &str) -> u32 {
        self.menu_text_widths
            .iter()
            .find(|entry| entry.0 == text)
            .map_or_else(|| text_width(text), |entry| entry.1)
    }

    /// Returns menu rows for one category in original order.
    #[must_use]
    pub fn items_for(&self, category: MenuCategory) -> Vec<&MenuItem> {
        self.menu_items
            .iter()
            .filter(|item| item.category == category)
            .collect()
    }

    /// Returns the menu button under a point in screen coordinates.
    #[must_use]
    pub fn menu_button_at(&self, x: i32, y: i32) -> Option<MenuCategory> {
        let menu_height = i32::try_from(MENU_BAR_HEIGHT).map_or(0_i32, |height| height);
        if y < 0_i32 || y >= menu_height {
            return None;
        }
        self.menu_buttons
            .iter()
            .find(|button| {
                let right = button
                    .x
                    .saturating_add(i32::try_from(button.width).map_or(0_i32, |width| width));
                x >= button.x && x < right
            })
            .map(|button| button.category)
    }

    /// Returns the CTP2 dropdown window rectangle for a category.
    #[must_use]
    pub fn dropdown_rect(&self, category: MenuCategory) -> Option<MenuRect> {
        let button = self
            .menu_buttons
            .iter()
            .find(|button| button.category == category)?;
        let rows = self.items_for(category);
        if rows.is_empty() {
            return None;
        }
        let row_count = u32::try_from(rows.len()).ok()?;
        Some(MenuRect {
            x: button.x,
            y: i32::try_from(MENU_BAR_HEIGHT).map_or(0_i32, |height| height),
            width: dropdown_width_for_rows(&rows, |text| self.measured_text_width(text)),
            height: row_count
                .saturating_mul(MENU_DROPDOWN_ROW_HEIGHT)
                .saturating_add(MENU_DROPDOWN_EXTRA_WINDOW_HEIGHT),
        })
    }

    /// Returns one dropdown row rectangle including the list inset.
    #[must_use]
    pub fn dropdown_row_rect(&self, category: MenuCategory, index: usize) -> Option<MenuRect> {
        let rect = self.dropdown_rect(category)?;
        let rows = self.items_for(category);
        if index >= rows.len() {
            return None;
        }
        let border = i32::try_from(MENU_DROPDOWN_BORDER).map_or(0_i32, |value| value);
        let index_offset = i32::try_from(index)
            .ok()?
            .saturating_mul(i32::try_from(MENU_DROPDOWN_ROW_HEIGHT).map_or(0_i32, |value| value));
        Some(MenuRect {
            x: rect.x.saturating_add(border),
            y: rect.y.saturating_add(border).saturating_add(index_offset),
            width: rect
                .width
                .saturating_sub(MENU_DROPDOWN_BORDER.saturating_mul(2)),
            height: MENU_DROPDOWN_ROW_HEIGHT,
        })
    }

    /// Returns the dropdown row at a point without filtering disabled rows.
    #[must_use]
    pub fn dropdown_item_at(&self, category: MenuCategory, x: i32, y: i32) -> Option<&MenuItem> {
        if !self.dropdown_rect(category)?.contains(x, y) {
            return None;
        }
        let rows = self.items_for(category);
        rows.iter()
            .enumerate()
            .find(|entry| {
                self.dropdown_row_rect(category, entry.0)
                    .is_some_and(|row| row.contains(x, y))
            })
            .map(|entry| *entry.1)
    }

    /// Returns the x coordinate of the shortcut column for rendering.
    #[must_use]
    pub fn dropdown_shortcut_x(&self, category: MenuCategory) -> Option<i32> {
        let rect = self.dropdown_rect(category)?;
        let rows = self.items_for(category);
        let max_shortcut_width = rows
            .iter()
            .map(|item| {
                self.measured_text_width(item.shortcut)
                    .saturating_add(MENU_EXTRA_SHORTCUT_WIDTH)
            })
            .max()
            .unwrap_or(0_u32);
        let right_padding = MENU_RIGHT_ITEM_MARGIN.saturating_add(MENU_DROPDOWN_BORDER);
        let offset = rect
            .width
            .saturating_sub(max_shortcut_width)
            .saturating_sub(right_padding);
        Some(
            rect.x
                .saturating_add(i32::try_from(offset).map_or(0_i32, |value| value)),
        )
    }

    /// Activates a row by exact label.
    #[must_use]
    pub fn activate_menu_label(&self, label: &str) -> Option<InGameAction> {
        self.menu_items
            .iter()
            .find(|item| item.enabled && item.label.eq_ignore_ascii_case(label))
            .map(|item| item.action)
    }

    /// Maps an SDL keydown to an in-game action using the original keymap.
    #[must_use]
    pub fn action_for_key(keycode: Keycode, keymod: Mod) -> InGameAction {
        let ctrl = keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD);
        match (keycode, ctrl) {
            (Keycode::A | Keycode::RETURN | Keycode::KP_ENTER, false) => InGameAction::EndTurn,
            (Keycode::SPACE, false) => InGameAction::Sentinel,
            (Keycode::B, false) => InGameAction::Settle,
            (Keycode::B, true) => InGameAction::OpenCityBuild,
            (Keycode::Q, false) => InGameAction::Quit,
            (Keycode::X | Keycode::Z, true) => InGameAction::NewGame,
            (Keycode::F1, false) | (Keycode::NUM_1, true) => InGameAction::OpenCivilizationStatus,
            (Keycode::F2, false) | (Keycode::NUM_2, true) => InGameAction::OpenNationalManager,
            (Keycode::F3, false) | (Keycode::NUM_3, true) => InGameAction::OpenCityManager,
            (Keycode::F4, false) | (Keycode::NUM_4 | Keycode::T, true) => {
                InGameAction::OpenTradeManager
            }
            (Keycode::S | Keycode::L, false) => InGameAction::OutOfScope,
            (Keycode::F5, false) | (Keycode::NUM_5, true) => InGameAction::OpenGreatLibrary,
            (Keycode::F6, false) | (Keycode::NUM_6, true) => InGameAction::OpenScienceStatus,
            (Keycode::F7, false) | (Keycode::NUM_7 | Keycode::D, true) => {
                InGameAction::OpenDiplomacy
            }
            (Keycode::F8, false) | (Keycode::NUM_8, true) => InGameAction::OpenUnitManager,
            (Keycode::NUM_9 | Keycode::K, true) => InGameAction::OpenInfo,
            (Keycode::W, false) => InGameAction::OpenWonders,
            (Keycode::F9, false) => InGameAction::OpenScore,
            (Keycode::NUM_0, true) | (Keycode::F, false) => InGameAction::OpenGameplayOptions,
            (Keycode::F, true) => InGameAction::OpenGraphicsOptions,
            (Keycode::P, true) => InGameAction::OpenAdvancedOptions,
            (Keycode::V | Keycode::M, true) => InGameAction::OpenSoundOptions,
            (Keycode::PERIOD, false) => InGameAction::OpenArmyManager,
            (Keycode::C, false) => InGameAction::ToggleCityNames,
            (Keycode::QUOTE, false) => InGameAction::ToggleTradeRoutes,
            (Keycode::R, false) => InGameAction::ToggleRadar,
            (Keycode::SLASH, false) => InGameAction::ToggleFogOfWar,
            (Keycode::PLUS | Keycode::KP_PLUS, false) => InGameAction::ZoomIn,
            (Keycode::MINUS | Keycode::KP_MINUS, false) => InGameAction::ZoomOut,
            (Keycode::NUM_8 | Keycode::KP_8, false) => InGameAction::MoveBy(0, -1),
            (Keycode::NUM_7 | Keycode::KP_7, false) => InGameAction::MoveBy(-1, -1),
            (Keycode::NUM_9 | Keycode::KP_9, false) => InGameAction::MoveBy(1, -1),
            (Keycode::NUM_4 | Keycode::KP_4, false) => InGameAction::MoveBy(-1, 0),
            (Keycode::NUM_6 | Keycode::KP_6, false) => InGameAction::MoveBy(1, 0),
            (Keycode::NUM_1 | Keycode::KP_1, false) => InGameAction::MoveBy(-1, 1),
            (Keycode::NUM_2 | Keycode::KP_2, false) => InGameAction::MoveBy(0, 1),
            (Keycode::NUM_3 | Keycode::KP_3, false) => InGameAction::MoveBy(1, 1),
            (Keycode::ESCAPE, false) => InGameAction::DismissPopup,
            _ => InGameAction::None,
        }
    }

    /// Opens an exact source manager, independently of gameplay/sound dispatch.
    fn open_manager_action(&mut self,state:&mut GameState,action:InGameAction) {
        if let Some(kind) = management::kind_for_action(action) {
            /* Released ctp2_source/ctp2_code/ui/aui_ctp2/keypress.cpp::KEY_FUNCTION_OPEN_SCIENCE_STATUS calls
             * ctp2_source/ctp2_code/ui/interface/screenutils.cpp::close_AllScreens/open_ScienceStatus, including
             * hiding EditQueue, before science.ldl::ScienceManagementDialog.
             * Clear only obsolete client presentation requests/Library overlay:
             * a deferred founding Build must not override this explicit choice.
             * No canonical city/research fact or local Custom draft is changed.
             */
            if action==InGameAction::OpenScienceStatus {
                state.city_build_menu=None;self.great_library=None;
            }
            let city = self.selected_city.and_then(|id| {
                state
                    .viewer
                    .as_ref()?
                    .cities
                    .iter()
                    .find(|city| super::model::CityId::from_server_id(&city.id) == id)
                    .map(|city| city.id.clone())
            });
            self.management.open(kind, state, city.as_deref());
            let tab = match action {
                InGameAction::OpenInfo => Some("Tab3"),
                InGameAction::OpenScore => Some("Tab1"),
                InGameAction::OpenWonders => Some("Tab2"),
                _ => None,
            };
            if let Some(tab) = tab {
                self.management
                    .apply_local(management::ManagerIntent::Select(
                        "tab:TabGroup".to_owned(),
                        tab.to_owned(),
                    ));
            }
        }
    }

    /// Applies an action to pure game/UI state and returns requested sounds.
    pub fn apply_action(
        &mut self,
        state: &mut GameState,
        action: InGameAction,
    ) -> Vec<GameplaySound> {
        let mut sounds = Vec::new();
        self.open_manager_action(state,action);
        match action {
            InGameAction::EndTurn => {
                state.end_turn();
                sounds.push(GameplaySound::EndTurn);
            }
            InGameAction::OpenGreatLibrary => {
                self.open_great_library(None);
                sounds.push(GameplaySound::GreatLibrary);
            }
            InGameAction::OpenCityBuild => {
                let selected = self
                    .management
                    .view
                    .as_ref()
                    .and_then(|view| view.city_id.as_deref())
                    .map(super::model::CityId::from_server_id);
                state.city_build_menu = selected;
                self.selected_city = selected;
                sounds.push(GameplaySound::UiClick);
            }
            InGameAction::Settle => {
                state.settle_selected("New City");
                sounds.push(GameplaySound::Settle);
            }
            InGameAction::Sentinel => {
                state.sentinel_selected();
                sounds.push(GameplaySound::UnitAcknowledge);
            }
            InGameAction::MoveBy(dx, dy) => {
                state.move_selected_by(dx, dy);
                sounds.push(GameplaySound::UnitMove);
            }
            InGameAction::ToggleCityNames => state.toggle_city_names(),
            InGameAction::ToggleTradeRoutes => state.toggle_trade_routes(),
            InGameAction::ToggleRadar => state.toggle_radar(),
            InGameAction::ToggleFogOfWar => state.toggle_fog_of_war(),
            InGameAction::DismissPopup => {
                state.dismiss_popup();
            }
            InGameAction::OpenTradeManager | InGameAction::OpenNationalManager => {
                sounds.push(GameplaySound::UiClick);
            }
            InGameAction::OpenCivilizationStatus => {
                self.active_tab = ControlPanelTab::Domestic;
                "Civilization/Government".clone_into(&mut self.status_text);
            }
            InGameAction::OpenCityManager => {
                self.active_tab = ControlPanelTab::City;
            }
            InGameAction::OpenUnitManager | InGameAction::OpenArmyManager => {
                self.active_tab = ControlPanelTab::Unit;
            }
            InGameAction::OpenInfo | InGameAction::OpenScore | InGameAction::OpenWonders => {
                /* This explicit Info status adds pinned server provenance, not
                 * original-game rules selection: controlpanelwindow.cpp::
                 * StatsMenuCallback/BuildStatsMenu and controlpanel.ldl::
                 * MainMenu.StatsActivator remain unchanged. Never infer a
                 * retail pin from compatibility version or technical unit IDs.
                 */
                self.status_text=state.viewer.as_ref().map_or_else(||"Info".to_owned(),crate::gameplay::server::observations::provenance);
            }
            InGameAction::Fortify | InGameAction::Disband => {
                "This order requires an authoritative server action"
                    .clone_into(&mut self.status_text);
            }
            InGameAction::OpenGameplayOptions
            | InGameAction::OpenGraphicsOptions
            | InGameAction::OpenSoundOptions
            | InGameAction::OpenAdvancedOptions
            | InGameAction::OpenScienceStatus
            | InGameAction::OpenDiplomacy
            | InGameAction::ZoomIn
            | InGameAction::ZoomOut
            | InGameAction::OutOfScope
            | InGameAction::NewGame
            | InGameAction::Quit
            | InGameAction::None => {}
        }
        sounds
    }

    /// Opens the Great Library with optional source text.
    pub fn open_great_library(&mut self, source_text: Option<&str>) {
        let body = source_text
            .and_then(first_non_empty_library_paragraph)
            .unwrap_or("The Great Library contains every concept, advance, unit, wonder, and rule discovered so far.")
            .to_owned();
        self.great_library = Some(GreatLibraryView {
            title: "Great Library".to_owned(),
            topic: "Contents".to_owned(),
            body,
        });
    }

    /// Builds a context menu for a point using current game state.
    #[must_use]
    pub fn context_menu_for(state: &GameState, point: MapPoint) -> Vec<context::ContextRow> {
        context::context_at(state, point)
    }
}

/// Returns first non-empty Great Library paragraph.
fn first_non_empty_library_paragraph(text: &str) -> Option<&str> {
    text.split("\n\n")
        .map(str::trim)
        .find(|paragraph| !paragraph.is_empty())
}

#[cfg(test)]
mod tests;
