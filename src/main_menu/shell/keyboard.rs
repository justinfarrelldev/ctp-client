//! Keyboard mapping screen builder.

use crate::config::ctp2_options::Ctp2KeyboardCategory;
use crate::main_menu::ButtonVisualStyle;

use super::MenuRect;
use super::builders::{button, image_switch_control, label, listbox_chrome, popup_close_button};
use super::options_state::ShellOptionsState;
use super::{ShellAction, ShellImageSet, ShellScreen, ShellScreenId};

/// One visible key-list row.
struct KeyListRow {
    /// Original string-table style action label.
    action: &'static str,
    /// Default key label used until remapping is wired to SDL key capture.
    key: &'static str,
}

/// Builds the keyboard mapping popup.
pub(super) fn keyboard_screen(options: &ShellOptionsState) -> ShellScreen {
    let mut controls = vec![
        label(
            "Name",
            "KEYBOARD MAPPING",
            "Keyboard Mapping Title",
            33,
            23,
            250,
            20,
            "km_screen.ldl:31",
        ),
        listbox_chrome(
            "KeyList",
            "Key List",
            25,
            66,
            281,
            234,
            0_usize,
            11_usize,
            11_usize,
            "km_screen.ldl:46",
        ),
        label(
            "KeyListHeaderAction",
            "ACTION",
            "Key List Action Header",
            25,
            46,
            196,
            20,
            "km_screen.ldl:58",
        ),
        label(
            "KeyListHeaderHotKey",
            "HOT KEY",
            "Key List Hot Key Header",
            221,
            46,
            85,
            20,
            "km_screen.ldl:69",
        ),
        button(
            "ResetButton",
            "RESET DEFAULTS",
            "Reset Defaults",
            45,
            308,
            250,
            21,
            ButtonVisualStyle::blue_large(),
            ShellAction::ResetKeyboardDefaults,
            "km_screen.ldl:96",
        ),
    ];

    append_category_switches(&mut controls, options.keyboard_category());
    append_key_rows(&mut controls, key_rows(options.keyboard_category()));
    controls.push(popup_close_button(580, 370));

    ShellScreen {
        id: ShellScreenId::Keyboard,
        ldl_name: "KmScreen",
        rect: MenuRect::new(110, 115, 580, 370),
        background: None,
        pattern: Some(super::builders::asset("uptg20e.rim")),
        controls,
    }
}

/// Appends category radio buttons from `KmScreen.Group`.
fn append_category_switches(
    controls: &mut Vec<super::ShellControl>,
    selected: Ctp2KeyboardCategory,
) {
    for (category, label_text, y) in [
        (Ctp2KeyboardCategory::Basic, "Basic Commands", 0_i32),
        (Ctp2KeyboardCategory::Unit, "Unit Controls", 31_i32),
        (Ctp2KeyboardCategory::Screen, "Screen Controls", 64_i32),
        (Ctp2KeyboardCategory::Map, "Map Controls", 96_i32),
        (Ctp2KeyboardCategory::Game, "Game Options", 129_i32),
    ] {
        controls.push(image_switch_control(
            keyboard_category_ldl_name(category),
            label_text,
            label_text,
            335,
            77_i32.saturating_add(y),
            176,
            24,
            selected == category,
            ShellImageSet::popup_checkbox(),
            ShellAction::SelectKeyboardCategory(category),
            "km_screen.ldl:124",
        ));
    }
}

/// Appends visible key rows for the selected category.
fn append_key_rows(controls: &mut Vec<super::ShellControl>, rows: &[KeyListRow]) {
    for (index, row) in rows.iter().take(11).enumerate() {
        let y = 86_i32.saturating_add(
            i32::try_from(index).map_or(0_i32, |row_index| row_index.saturating_mul(20_i32)),
        );
        controls.push(label(
            "KeyListItemName",
            row.action,
            row.action,
            25,
            y,
            196,
            20,
            "km_screen.ldl:217",
        ));
        controls.push(label(
            "KeyListItemKeycode",
            row.key,
            row.key,
            221,
            y,
            85,
            20,
            "km_screen.ldl:225",
        ));
    }
}

/// Returns the original LDL child name for a category switch.
const fn keyboard_category_ldl_name(category: Ctp2KeyboardCategory) -> &'static str {
    match category {
        Ctp2KeyboardCategory::Basic => "Basic",
        Ctp2KeyboardCategory::Unit => "Unit",
        Ctp2KeyboardCategory::Screen => "Screen",
        Ctp2KeyboardCategory::Map => "Map",
        Ctp2KeyboardCategory::Game => "Game",
    }
}

/// Returns key-list rows matching the original category ranges.
const fn key_rows(category: Ctp2KeyboardCategory) -> &'static [KeyListRow] {
    match category {
        Ctp2KeyboardCategory::Basic => BASIC_ROWS,
        Ctp2KeyboardCategory::Unit => UNIT_ROWS,
        Ctp2KeyboardCategory::Screen => SCREEN_ROWS,
        Ctp2KeyboardCategory::Map => MAP_ROWS,
        Ctp2KeyboardCategory::Game => GAME_ROWS,
    }
}

/// Original basic command rows.
const BASIC_ROWS: &[KeyListRow] = &[
    KeyListRow {
        action: "Move North",
        key: "8",
    },
    KeyListRow {
        action: "Move Northwest",
        key: "7",
    },
    KeyListRow {
        action: "Move Northeast",
        key: "9",
    },
    KeyListRow {
        action: "Move West",
        key: "4",
    },
    KeyListRow {
        action: "Move East",
        key: "6",
    },
    KeyListRow {
        action: "Move Southwest",
        key: "1",
    },
    KeyListRow {
        action: "Move South",
        key: "2",
    },
    KeyListRow {
        action: "Move Southeast",
        key: "3",
    },
    KeyListRow {
        action: "Unit/City Toggle",
        key: "Tab",
    },
    KeyListRow {
        action: "Next Unit/City",
        key: "Space",
    },
    KeyListRow {
        action: "Select Unit",
        key: "Enter",
    },
    KeyListRow {
        action: "Next Round",
        key: "End",
    },
];

/// Original unit-control rows.
const UNIT_ROWS: &[KeyListRow] = &[
    KeyListRow {
        action: "Open Army Management",
        key: "A",
    },
    KeyListRow {
        action: "Sleep",
        key: "S",
    },
    KeyListRow {
        action: "Entrench",
        key: "E",
    },
    KeyListRow {
        action: "Settle",
        key: "B",
    },
    KeyListRow {
        action: "Pillage",
        key: "P",
    },
    KeyListRow {
        action: "Bombard",
        key: "V",
    },
    KeyListRow {
        action: "Reform",
        key: "R",
    },
    KeyListRow {
        action: "Expel",
        key: "X",
    },
    KeyListRow {
        action: "Unload Transport",
        key: "U",
    },
    KeyListRow {
        action: "Investigate City",
        key: "I",
    },
    KeyListRow {
        action: "Plant Nuke",
        key: "N",
    },
    KeyListRow {
        action: "Bioinfect",
        key: "K",
    },
    KeyListRow {
        action: "Nano Terror",
        key: "T",
    },
    KeyListRow {
        action: "Create Park",
        key: "C",
    },
    KeyListRow {
        action: "Launch",
        key: "L",
    },
];

/// Original screen-control rows.
const SCREEN_ROWS: &[KeyListRow] = &[
    KeyListRow {
        action: "Open Civ Status",
        key: "F1",
    },
    KeyListRow {
        action: "Open City Status",
        key: "F2",
    },
    KeyListRow {
        action: "Open City Management",
        key: "F3",
    },
    KeyListRow {
        action: "Open Build Queue",
        key: "F4",
    },
    KeyListRow {
        action: "Clear Queue",
        key: "-",
    },
    KeyListRow {
        action: "Open Science Status",
        key: "F5",
    },
    KeyListRow {
        action: "Open Unit Status",
        key: "F6",
    },
    KeyListRow {
        action: "Open Trade Status",
        key: "F7",
    },
    KeyListRow {
        action: "Open Trade Summary",
        key: "F8",
    },
    KeyListRow {
        action: "Open Diplomacy",
        key: "F9",
    },
    KeyListRow {
        action: "Open Dip Wizard",
        key: "F10",
    },
    KeyListRow {
        action: "Message",
        key: "M",
    },
    KeyListRow {
        action: "Gaia",
        key: "G",
    },
    KeyListRow {
        action: "Open Great Library",
        key: "F11",
    },
    KeyListRow {
        action: "Open Info Screen",
        key: "F12",
    },
    KeyListRow {
        action: "Open Wonders",
        key: "W",
    },
    KeyListRow {
        action: "Open Ranking",
        key: "R",
    },
    KeyListRow {
        action: "Civ Tab",
        key: "1",
    },
    KeyListRow {
        action: "City Tab",
        key: "2",
    },
    KeyListRow {
        action: "Unit Tab",
        key: "3",
    },
    KeyListRow {
        action: "Message Tab",
        key: "4",
    },
    KeyListRow {
        action: "Tile Improvement Tab",
        key: "5",
    },
    KeyListRow {
        action: "Open Chat",
        key: "C",
    },
    KeyListRow {
        action: "Open Network Screen",
        key: "N",
    },
    KeyListRow {
        action: "Open Scenario Editor",
        key: "Z",
    },
];

/// Original map-control rows.
const MAP_ROWS: &[KeyListRow] = &[
    KeyListRow {
        action: "Toggle City Names",
        key: "Ctrl + N",
    },
    KeyListRow {
        action: "Toggle Trade Routes",
        key: "Ctrl + T",
    },
    KeyListRow {
        action: "Center Map on Unit/City",
        key: "C",
    },
    KeyListRow {
        action: "Close Message",
        key: "Esc",
    },
    KeyListRow {
        action: "Toggle Radar",
        key: "R",
    },
    KeyListRow {
        action: "Toggle Control Panel",
        key: "P",
    },
    KeyListRow {
        action: "Toggle All",
        key: "A",
    },
    KeyListRow {
        action: "Zoom In",
        key: "+",
    },
    KeyListRow {
        action: "Zoom Out",
        key: "-",
    },
];

/// Original game-options rows.
const GAME_ROWS: &[KeyListRow] = &[
    KeyListRow {
        action: "Remap Keyboard",
        key: "K",
    },
    KeyListRow {
        action: "Open Options Screen",
        key: "O",
    },
    KeyListRow {
        action: "Sound Options",
        key: "S",
    },
    KeyListRow {
        action: "Graphics Options",
        key: "G",
    },
    KeyListRow {
        action: "Gameplay Options",
        key: "P",
    },
    KeyListRow {
        action: "Advanced Options",
        key: "A",
    },
    KeyListRow {
        action: "Restart Game",
        key: "R",
    },
    KeyListRow {
        action: "New Game",
        key: "N",
    },
    KeyListRow {
        action: "Save Game",
        key: "Ctrl + S",
    },
    KeyListRow {
        action: "Load Game",
        key: "Ctrl + L",
    },
    KeyListRow {
        action: "Save World",
        key: "W",
    },
    KeyListRow {
        action: "Load World",
        key: "L",
    },
    KeyListRow {
        action: "Quit",
        key: "Q",
    },
];
