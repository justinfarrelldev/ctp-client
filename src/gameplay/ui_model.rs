//! Testable in-game menu bar, control panel, shortcuts, and popup UI model.

extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use sdl2::keyboard::{Keycode, Mod};

use super::audio::GameplaySound;
use super::model::{GameState, MapPoint};

/// Original in-game menu bar height in pixels.
pub const MENU_BAR_HEIGHT: u32 = 24;
/// Approximate original control-panel width from `controlpanel.ldl`.
pub const CONTROL_PANEL_WIDTH: u32 = 601;
/// Approximate original control-panel height from `controlpanel.ldl`.
pub const CONTROL_PANEL_HEIGHT: u32 = 184;

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
    /// Civilization menu.
    Civilization,
    /// Cities menu.
    Cities,
    /// Units menu.
    Units,
    /// Diplomacy menu.
    Diplomacy,
    /// Info menu.
    Info,
    /// Options menu.
    Options,
}

impl MenuCategory {
    /// Returns original visible title.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Civilization => "Civ",
            Self::Cities => "Cities",
            Self::Units => "Units",
            Self::Diplomacy => "Diplomacy",
            Self::Info => "Info",
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
}

impl Default for InGameUiModel {
    fn default() -> Self {
        Self {
            menu_buttons: original_menu_buttons(),
            menu_items: original_menu_items(),
            active_tab: ControlPanelTab::Domestic,
            open_menu: None,
            great_library: None,
            status_text: String::new(),
        }
    }
}

impl InGameUiModel {
    /// Returns menu rows for one category in original order.
    #[must_use]
    pub fn items_for(&self, category: MenuCategory) -> Vec<&MenuItem> {
        self.menu_items
            .iter()
            .filter(|item| item.category == category)
            .collect()
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
            (Keycode::NUM_1 | Keycode::NUM_2, true) => InGameAction::OpenCivilizationStatus,
            (Keycode::NUM_3, true) => InGameAction::OpenCityManager,
            (Keycode::NUM_4 | Keycode::T, true) | (Keycode::S | Keycode::L, false) => {
                InGameAction::OutOfScope
            }
            (Keycode::NUM_5, true) => InGameAction::OpenGreatLibrary,
            (Keycode::NUM_6, true) => InGameAction::OpenScienceStatus,
            (Keycode::NUM_7 | Keycode::D, true) => InGameAction::OpenDiplomacy,
            (Keycode::NUM_8, true) => InGameAction::OpenUnitManager,
            (Keycode::NUM_9 | Keycode::K, true) | (Keycode::W, false) => InGameAction::OpenInfo,
            (Keycode::NUM_0, true) | (Keycode::F9, false) => InGameAction::OpenGameplayOptions,
            (Keycode::F, true) => InGameAction::OpenGraphicsOptions,
            (Keycode::P, true) => InGameAction::OpenAdvancedOptions,
            (Keycode::V, true) => InGameAction::OpenSoundOptions,
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

/// Builds top menu buttons in original order.
fn original_menu_buttons() -> Vec<MenuButton> {
    vec![
        MenuButton {
            category: MenuCategory::Civilization,
            label: "Civ",
            x: 0,
            width: 80,
        },
        MenuButton {
            category: MenuCategory::Cities,
            label: "Cities",
            x: 80,
            width: 80,
        },
        MenuButton {
            category: MenuCategory::Units,
            label: "Units",
            x: 160,
            width: 80,
        },
        MenuButton {
            category: MenuCategory::Diplomacy,
            label: "Diplomacy",
            x: 240,
            width: 96,
        },
        MenuButton {
            category: MenuCategory::Info,
            label: "Info",
            x: 336,
            width: 80,
        },
        MenuButton {
            category: MenuCategory::Options,
            label: "Options",
            x: 416,
            width: 88,
        },
    ]
}

/// Builds original top drop-down rows from `controlpanelwindow.cpp`.
#[expect(
    clippy::too_many_lines,
    reason = "Direct menu manifest keeps source rows auditable."
)]
fn original_menu_items() -> Vec<MenuItem> {
    vec![
        menu_item(
            MenuCategory::Civilization,
            "Empire Manager",
            "^1",
            InGameAction::OpenCivilizationStatus,
            true,
            "controlpanelwindow.cpp:1447",
        ),
        menu_item(
            MenuCategory::Civilization,
            "Trade Open Market",
            "^4",
            InGameAction::OutOfScope,
            true,
            "controlpanelwindow.cpp:1449",
        ),
        menu_item(
            MenuCategory::Civilization,
            "Trade Summary",
            "^t",
            InGameAction::OutOfScope,
            true,
            "controlpanelwindow.cpp:1451",
        ),
        menu_item(
            MenuCategory::Civilization,
            "Science Manager",
            "^6",
            InGameAction::OpenScienceStatus,
            true,
            "controlpanelwindow.cpp:1453",
        ),
        menu_item(
            MenuCategory::Civilization,
            "Gaia",
            "^g",
            InGameAction::OpenInfo,
            true,
            "controlpanelwindow.cpp:1455",
        ),
        menu_item(
            MenuCategory::Cities,
            "Build Queue",
            "^b",
            InGameAction::OpenCityBuild,
            true,
            "controlpanelwindow.cpp:1480",
        ),
        menu_item(
            MenuCategory::Cities,
            "City Manager",
            "^3",
            InGameAction::OpenCityManager,
            true,
            "controlpanelwindow.cpp:1482",
        ),
        menu_item(
            MenuCategory::Cities,
            "National Manager",
            "^2",
            InGameAction::OpenCivilizationStatus,
            true,
            "controlpanelwindow.cpp:1484",
        ),
        menu_item(
            MenuCategory::Units,
            "Unit Manager",
            "^8",
            InGameAction::OpenUnitManager,
            true,
            "controlpanelwindow.cpp:1509",
        ),
        menu_item(
            MenuCategory::Units,
            "Army Management",
            ".",
            InGameAction::OpenArmyManager,
            true,
            "controlpanelwindow.cpp:1511",
        ),
        menu_item(
            MenuCategory::Diplomacy,
            "Diplomacy Manager",
            "^7",
            InGameAction::OpenDiplomacy,
            true,
            "controlpanelwindow.cpp:1536",
        ),
        menu_item(
            MenuCategory::Diplomacy,
            "New Proposal",
            "^d",
            InGameAction::OpenDiplomacy,
            true,
            "controlpanelwindow.cpp:1538",
        ),
        menu_item(
            MenuCategory::Info,
            "Great Library",
            "^5",
            InGameAction::OpenGreatLibrary,
            true,
            "controlpanelwindow.cpp:1642",
        ),
        menu_item(
            MenuCategory::Info,
            "Ranking",
            "^k",
            InGameAction::OpenInfo,
            true,
            "controlpanelwindow.cpp:1644",
        ),
        menu_item(
            MenuCategory::Info,
            "Score",
            "^9",
            InGameAction::OpenInfo,
            true,
            "controlpanelwindow.cpp:1646",
        ),
        menu_item(
            MenuCategory::Info,
            "Wonders",
            "W",
            InGameAction::OpenInfo,
            true,
            "controlpanelwindow.cpp:1648",
        ),
        menu_item(
            MenuCategory::Options,
            "Zoom Out",
            "-",
            InGameAction::ZoomOut,
            true,
            "controlpanelwindow.cpp:1674",
        ),
        menu_item(
            MenuCategory::Options,
            "Zoom In",
            "+",
            InGameAction::ZoomIn,
            true,
            "controlpanelwindow.cpp:1677",
        ),
        menu_item(
            MenuCategory::Options,
            "Gameplay",
            "F9",
            InGameAction::OpenGameplayOptions,
            true,
            "controlpanelwindow.cpp:1681",
        ),
        menu_item(
            MenuCategory::Options,
            "Graphics",
            "^f",
            InGameAction::OpenGraphicsOptions,
            true,
            "controlpanelwindow.cpp:1683",
        ),
        menu_item(
            MenuCategory::Options,
            "Sound",
            "^v",
            InGameAction::OpenSoundOptions,
            true,
            "controlpanelwindow.cpp:1685",
        ),
        menu_item(
            MenuCategory::Options,
            "Advanced",
            "^p",
            InGameAction::OpenAdvancedOptions,
            true,
            "controlpanelwindow.cpp:1687",
        ),
        menu_item(
            MenuCategory::Options,
            "Cheat Mode",
            "!",
            InGameAction::OutOfScope,
            true,
            "controlpanelwindow.cpp:1689",
        ),
        menu_item(
            MenuCategory::Options,
            "Save Game",
            "S",
            InGameAction::OutOfScope,
            false,
            "out-of-scope per request",
        ),
        menu_item(
            MenuCategory::Options,
            "Load Game",
            "L",
            InGameAction::OutOfScope,
            false,
            "out-of-scope per request",
        ),
        menu_item(
            MenuCategory::Options,
            "Restart",
            "^z",
            InGameAction::NewGame,
            true,
            "controlpanelwindow.cpp:1698",
        ),
        menu_item(
            MenuCategory::Options,
            "New Game",
            "^x",
            InGameAction::NewGame,
            true,
            "controlpanelwindow.cpp:1701",
        ),
        menu_item(
            MenuCategory::Options,
            "Quit",
            "Q",
            InGameAction::Quit,
            true,
            "controlpanelwindow.cpp:1703",
        ),
    ]
}

/// Builds a menu item row.
const fn menu_item(
    category: MenuCategory,
    label: &'static str,
    shortcut: &'static str,
    action: InGameAction,
    enabled: bool,
    source: &'static str,
) -> MenuItem {
    MenuItem {
        category,
        label,
        shortcut,
        action,
        enabled,
        source,
    }
}

/// Returns first non-empty Great Library paragraph.
fn first_non_empty_library_paragraph(text: &str) -> Option<&str> {
    text.split("\n\n")
        .map(str::trim)
        .find(|paragraph| !paragraph.is_empty())
}

#[cfg(test)]
mod tests {
    use sdl2::keyboard::{Keycode, Mod};

    use super::{InGameAction, InGameUiModel, MenuCategory};
    use crate::gameplay::model::GameState;

    #[test]
    fn top_menu_matches_original_rows_and_save_load_are_disabled() {
        let ui = InGameUiModel::default();
        let options = ui.items_for(MenuCategory::Options);

        assert!(
            options
                .iter()
                .any(|item| item.label == "New Game" && item.action == InGameAction::NewGame),
            "Options menu should include original New Game row"
        );
        assert!(
            options
                .iter()
                .any(|item| item.label == "Quit" && item.action == InGameAction::Quit),
            "Options menu should include original Quit row"
        );
        assert!(
            options
                .iter()
                .any(|item| item.label == "Save Game" && !item.enabled),
            "Save Game should remain visible but disabled as out of scope"
        );
        assert!(
            options
                .iter()
                .any(|item| item.label == "Load Game" && !item.enabled),
            "Load Game should remain visible but disabled as out of scope"
        );
    }

    #[test]
    fn keymap_shortcuts_fire_original_in_scope_actions() {
        assert_eq!(
            InGameUiModel::action_for_key(Keycode::Q, Mod::NOMOD),
            InGameAction::Quit,
            "Q should quit per original keymap"
        );
        assert_eq!(
            InGameUiModel::action_for_key(Keycode::X, Mod::LCTRLMOD),
            InGameAction::NewGame,
            "Ctrl-X should open New Game per original keymap"
        );
        assert_eq!(
            InGameUiModel::action_for_key(Keycode::NUM_5, Mod::LCTRLMOD),
            InGameAction::OpenGreatLibrary,
            "Ctrl-5 should open Great Library"
        );
        assert_eq!(
            InGameUiModel::action_for_key(Keycode::B, Mod::NOMOD),
            InGameAction::Settle,
            "B should trigger settle"
        );
    }

    #[test]
    fn context_menu_exposes_settle_and_city_build_when_available() {
        let mut state = GameState::new(24, 18, 1, 22);
        let point = state
            .selected_unit()
            .expect("selected unit should exist")
            .position;
        let before = InGameUiModel::context_menu_for(&state, point);

        assert!(
            before
                .iter()
                .any(|item| item.label == "Settle" && item.enabled),
            "settlers should expose enabled Settle in the context menu"
        );
        state.settle_selected("Alpha");
        let city_point = state.cities[0].position;
        let after = InGameUiModel::context_menu_for(&state, city_point);

        assert!(
            after
                .iter()
                .any(|item| item.label == "Build Queue" && item.enabled),
            "city context menu should expose enabled Build Queue"
        );
    }
}
