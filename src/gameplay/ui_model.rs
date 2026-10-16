//! Testable in-game menu bar, control panel, shortcuts, and popup UI model.

extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use sdl2::keyboard::{Keycode, Mod};

use self::menu_manifest::{
    dropdown_width_for_rows, original_menu_buttons, original_menu_items, text_width,
};
use super::audio::GameplaySound;
use super::model::{GameState, MapPoint};

/// Original CTP2 top-menu manifest and geometry helpers.
mod menu_manifest;

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
    /// Open ranking/info screen.
    OpenInfo,
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
    /// Put selected unit into sentinel.
    Sentinel,
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

/// Context-menu row for right-click interactions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextMenuItem {
    /// Visible label.
    pub label: &'static str,
    /// Action emitted when selected.
    pub action: InGameAction,
    /// Whether row can currently be selected.
    pub enabled: bool,
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
    /// Measured original menu labels/shortcuts, shared by rendering and hit tests.
    /// Empty in font-independent tests, which use the documented fallback metric.
    pub menu_text_widths: Vec<(&'static str, u32)>,
    /// Optional tile beneath the pointer, used by the original yellow map cursor.
    pub hovered_tile: Option<MapPoint>,
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
            menu_text_widths: Vec::new(),
            hovered_tile: None,
        }
    }
}

impl InGameUiModel {
    /// Updates menu geometry from the same font that paints its labels.
    pub fn measure_menu_text(&mut self, measure: impl Fn(&str) -> u32) {
        self.menu_text_widths = self
            .menu_items
            .iter()
            .flat_map(|item| [item.label, item.shortcut])
            .map(|text| (text, measure(text)))
            .collect();
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
            (Keycode::A | Keycode::RETURN, false) => InGameAction::EndTurn,
            (Keycode::SPACE, false) => InGameAction::Sentinel,
            (Keycode::B, false) => InGameAction::Settle,
            (Keycode::B, true) => InGameAction::OpenCityBuild,
            (Keycode::Q, false) => InGameAction::Quit,
            (Keycode::X | Keycode::Z, true) => InGameAction::NewGame,
            (Keycode::F1 | Keycode::F2, false) | (Keycode::NUM_1 | Keycode::NUM_2, true) => {
                InGameAction::OpenCivilizationStatus
            }
            (Keycode::F3, false) | (Keycode::NUM_3, true) => InGameAction::OpenCityManager,
            (Keycode::F4 | Keycode::S | Keycode::L, false)
            | (Keycode::NUM_4 | Keycode::T, true) => InGameAction::OutOfScope,
            (Keycode::F5, false) | (Keycode::NUM_5, true) => InGameAction::OpenGreatLibrary,
            (Keycode::F6, false) | (Keycode::NUM_6, true) => InGameAction::OpenScienceStatus,
            (Keycode::F7, false) | (Keycode::NUM_7 | Keycode::D, true) => {
                InGameAction::OpenDiplomacy
            }
            (Keycode::F8, false) | (Keycode::NUM_8, true) => InGameAction::OpenUnitManager,
            (Keycode::NUM_9 | Keycode::K, true) | (Keycode::W, false) => InGameAction::OpenInfo,
            (Keycode::NUM_0, true) | (Keycode::F9 | Keycode::F, false) => {
                InGameAction::OpenGameplayOptions
            }
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

    /// Applies an action to pure game/UI state and returns requested sounds.
    pub fn apply_action(
        &mut self,
        state: &mut GameState,
        action: InGameAction,
    ) -> Vec<GameplaySound> {
        let mut sounds = Vec::new();
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
                if let Some(city) = state.cities.first() {
                    state.city_build_menu = Some(city.id);
                }
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
            InGameAction::OpenInfo => {
                "Info".clone_into(&mut self.status_text);
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
    pub fn context_menu_for(state: &GameState, point: MapPoint) -> Vec<ContextMenuItem> {
        let has_selected_settler = state
            .selected_unit()
            .is_some_and(|unit| matches!(unit.kind, super::model::UnitKind::Settler));
        let has_city = state.city_at(point).is_some();
        vec![
            ContextMenuItem {
                label: "Move",
                action: InGameAction::MoveBy(0, 0),
                enabled: state.selected_unit().is_some(),
            },
            ContextMenuItem {
                label: "Settle",
                action: InGameAction::Settle,
                enabled: has_selected_settler,
            },
            ContextMenuItem {
                label: "Sentinel",
                action: InGameAction::Sentinel,
                enabled: state.selected_unit().is_some(),
            },
            ContextMenuItem {
                label: "Build Queue",
                action: InGameAction::OpenCityBuild,
                enabled: has_city,
            },
            ContextMenuItem {
                label: "Army Manager",
                action: InGameAction::OpenArmyManager,
                enabled: true,
            },
        ]
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
