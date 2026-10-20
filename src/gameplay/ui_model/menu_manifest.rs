//! Original CTP2 in-game top-menu manifest and geometry helpers.

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use super::{
    InGameAction, MENU_APPROX_GLYPH_WIDTH, MENU_BUTTON_WIDTH, MENU_DROPDOWN_EXTRA_WINDOW_WIDTH,
    MENU_EXTRA_LEFT_SPACE, MENU_EXTRA_SHORTCUT_WIDTH, MENU_LEFT_ITEM_MARGIN,
    MENU_RIGHT_ITEM_MARGIN, MenuButton, MenuCategory, MenuItem,
};

/// Estimates text width the same way for rendering and hit testing.
pub(super) fn text_width(text: &str) -> u32 {
    u32::try_from(text.chars().count())
        .map_or(0_u32, |count| count.saturating_mul(MENU_APPROX_GLYPH_WIDTH))
}

/// Computes a CTP2-style dropdown width from item text and shortcuts.
pub(super) fn dropdown_width_for_rows(rows: &[&MenuItem], measure: impl Fn(&str) -> u32) -> u32 {
    let max_text_width = rows
        .iter()
        .map(|item| measure(item.label))
        .max()
        .unwrap_or(0_u32)
        .max(20_u32);
    let max_shortcut_width = rows
        .iter()
        .map(|item| measure(item.shortcut).saturating_add(MENU_EXTRA_SHORTCUT_WIDTH))
        .max()
        .unwrap_or(0_u32);
    max_text_width
        .saturating_add(max_shortcut_width)
        .saturating_add(MENU_LEFT_ITEM_MARGIN)
        .saturating_add(MENU_RIGHT_ITEM_MARGIN)
        .saturating_add(MENU_EXTRA_LEFT_SPACE)
        .saturating_add(MENU_DROPDOWN_EXTRA_WINDOW_WIDTH)
        .saturating_add(MENU_LEFT_ITEM_MARGIN)
}

/// Builds top menu buttons in original order.
pub(super) fn original_menu_buttons() -> Vec<MenuButton> {
    [
        MenuCategory::Civilization,
        MenuCategory::Cities,
        MenuCategory::Units,
        MenuCategory::Diplomacy,
        MenuCategory::Info,
        MenuCategory::Options,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, category)| MenuButton {
        category,
        label: category.title(),
        x: i32::try_from(index).map_or(0_i32, |value| {
            value.saturating_mul(i32::try_from(MENU_BUTTON_WIDTH).map_or(0_i32, |width| width))
        }),
        width: MENU_BUTTON_WIDTH,
    })
    .collect()
}

/// Builds original top drop-down rows from `controlpanelwindow.cpp`.
#[expect(
    clippy::too_many_lines,
    reason = "Direct menu manifest keeps source rows auditable."
)]
pub(super) fn original_menu_items() -> Vec<MenuItem> {
    vec![
        menu_item(
            MenuCategory::Civilization,
            "Empire Manager",
            "F1",
            InGameAction::OpenCivilizationStatus,
            true,
            "controlpanelwindow.cpp:1447",
        ),
        menu_item(
            MenuCategory::Civilization,
            "Trade - Open Market",
            "F4",
            InGameAction::OpenTradeManager,
            true,
            "controlpanelwindow.cpp:1449",
        ),
        menu_item(
            MenuCategory::Civilization,
            "Trade - Summary",
            "Ctrl + T",
            InGameAction::OpenTradeManager,
            true,
            "controlpanelwindow.cpp:1451",
        ),
        menu_item(
            MenuCategory::Civilization,
            "Science Manager",
            "F6",
            InGameAction::OpenScienceStatus,
            true,
            "controlpanelwindow.cpp:1453",
        ),
        menu_item(
            MenuCategory::Civilization,
            "Gaia Controller",
            "Ctrl + G",
            InGameAction::OpenInfo,
            true,
            "controlpanelwindow.cpp:1455",
        ),
        menu_item(
            MenuCategory::Cities,
            "Build Manager",
            "Ctrl + B",
            InGameAction::OpenCityBuild,
            true,
            "controlpanelwindow.cpp:1480",
        ),
        menu_item(
            MenuCategory::Cities,
            "City Manager",
            "F3",
            InGameAction::OpenCityManager,
            true,
            "controlpanelwindow.cpp:1482",
        ),
        menu_item(
            MenuCategory::Cities,
            "Natl Manager",
            "F2",
            InGameAction::OpenNationalManager,
            true,
            "controlpanelwindow.cpp:1484",
        ),
        menu_item(
            MenuCategory::Units,
            "Unit Manager",
            "F8",
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
            "F7",
            InGameAction::OpenDiplomacy,
            true,
            "controlpanelwindow.cpp:1536",
        ),
        menu_item(
            MenuCategory::Diplomacy,
            "New Proposal",
            "Ctrl + D",
            InGameAction::OpenDiplomacy,
            true,
            "controlpanelwindow.cpp:1538",
        ),
        menu_item(
            MenuCategory::Info,
            "Great Library",
            "F5",
            InGameAction::OpenGreatLibrary,
            true,
            "controlpanelwindow.cpp:1642",
        ),
        menu_item(
            MenuCategory::Info,
            "Ranking",
            "Ctrl + K",
            InGameAction::OpenInfo,
            true,
            "controlpanelwindow.cpp:1644",
        ),
        menu_item(
            MenuCategory::Info,
            "Score",
            "F9",
            InGameAction::OpenScore,
            true,
            "controlpanelwindow.cpp:1646",
        ),
        menu_item(
            MenuCategory::Info,
            "Wonders",
            "W",
            InGameAction::OpenWonders,
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
            "F",
            InGameAction::OpenGameplayOptions,
            true,
            "controlpanelwindow.cpp:1681",
        ),
        menu_item(
            MenuCategory::Options,
            "Graphics",
            "ctrl + f",
            InGameAction::OpenGraphicsOptions,
            true,
            "controlpanelwindow.cpp:1683",
        ),
        menu_item(
            MenuCategory::Options,
            "Sound",
            "ctrl + v",
            InGameAction::OpenSoundOptions,
            true,
            "controlpanelwindow.cpp:1685",
        ),
        menu_item(
            MenuCategory::Options,
            "Music",
            "ctrl + m",
            InGameAction::OpenSoundOptions,
            true,
            "retail Options screenshot: Music",
        ),
        menu_item(
            MenuCategory::Options,
            "Chat screen",
            "",
            InGameAction::OutOfScope,
            false,
            "retail Options screenshot: Chat screen (unavailable in local mode)",
        ),
        menu_item(
            MenuCategory::Options,
            "Advanced",
            "ctrl + p",
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
            "ctrl + z",
            InGameAction::NewGame,
            true,
            "controlpanelwindow.cpp:1698",
        ),
        menu_item(
            MenuCategory::Options,
            "New Game",
            "ctrl + x",
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
