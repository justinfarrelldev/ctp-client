//! Tests for the in-game UI model.

use sdl2::keyboard::{Keycode, Mod};

use super::{InGameAction, InGameUiModel, MenuCategory};
use crate::gameplay::model::GameState;

#[test]
fn top_menu_matches_original_ctp2_titles_rows_and_save_load_are_disabled() {
    let ui = InGameUiModel::default();
    let titles: Vec<_> = ui.menu_buttons.iter().map(|button| button.label).collect();
    let options = ui.items_for(MenuCategory::Options);
    let empire = ui.items_for(MenuCategory::Civilization);
    let cities = ui.items_for(MenuCategory::Cities);

    assert_eq!(
        titles,
        vec!["Empire", "Cities", "Units", "Diplomacy", "Stats", "Options"],
        "top menu button titles must match MainMenu/controlpanel.ldl"
    );
    assert_eq!(
        empire.iter().map(|item| item.label).collect::<Vec<_>>(),
        vec![
            "Empire Manager",
            "Trade - Open Market",
            "Trade - Summary",
            "Science Manager",
            "Gaia Controller",
        ],
        "Empire dropdown rows must match controlpanelwindow.cpp/string DB order"
    );
    assert_eq!(
        cities.iter().map(|item| item.label).collect::<Vec<_>>(),
        vec!["Build Manager", "City Manager", "Natl Manager"],
        "Cities dropdown rows must use the original CTP2 string labels"
    );
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
fn top_menu_geometry_uses_original_button_and_dropdown_metrics() {
    let ui = InGameUiModel::default();
    let options_button = ui
        .menu_buttons
        .iter()
        .find(|button| button.category == MenuCategory::Options)
        .expect("Options button should exist");
    let rect = ui
        .dropdown_rect(MenuCategory::Options)
        .expect("Options dropdown should have rows");
    let save_row = ui
        .dropdown_row_rect(MenuCategory::Options, 7)
        .expect("Save Game row should be present");

    assert_eq!(
        options_button.x, 400_i32,
        "six 80px LDL menu buttons place Options at x=400"
    );
    assert_eq!(
        options_button.width, 80,
        "MENU_BUTTON in controlpanel.ldl is 80px wide"
    );
    assert_eq!(
        rect.x, options_button.x,
        "dropdown opens under its activator"
    );
    assert_eq!(
        rect.y, 24_i32,
        "dropdown opens immediately below the 24px menu bar"
    );
    assert!(
        rect.width < 260,
        "dropdown width should be computed from CTP2 menu text, not the old hard-coded 260px panel"
    );
    assert_eq!(
        save_row.height, 16,
        "PlainMenuListItem rows are 16px high in ctp_template.ldl"
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
        InGameUiModel::action_for_key(Keycode::F5, Mod::NOMOD),
        InGameAction::OpenGreatLibrary,
        "F5 should open Great Library as displayed by the original key formatter"
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
