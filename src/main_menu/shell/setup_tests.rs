//! Tests for original CTP2 new-game setup and custom-map fidelity.

use super::options_state::ShellOptionsState;
use super::*;

/// Returns slider state for assertions.
const fn slider_value(kind: ShellControlKind) -> Option<i32> {
    match kind {
        ShellControlKind::Slider { value, .. } => Some(value),
        ShellControlKind::Button(_)
        | ShellControlKind::Label { .. }
        | ShellControlKind::TextField { .. }
        | ShellControlKind::Switch { .. }
        | ShellControlKind::ListBoxChrome { .. }
        | ShellControlKind::ListItem { .. } => None,
    }
}

/// Returns selected state from a switch-like control for assertions.
const fn switch_enabled_state(kind: ShellControlKind) -> Option<bool> {
    match kind {
        ShellControlKind::Switch { enabled, .. } => Some(enabled),
        ShellControlKind::Button(_)
        | ShellControlKind::Label { .. }
        | ShellControlKind::TextField { .. }
        | ShellControlKind::Slider { .. }
        | ShellControlKind::ListBoxChrome { .. }
        | ShellControlKind::ListItem { .. } => None,
    }
}

/// Returns label layout metadata for assertions.
const fn label_layout(kind: ShellControlKind) -> Option<ShellLabelLayout> {
    match kind {
        ShellControlKind::Label { layout } => Some(layout),
        ShellControlKind::Button(_)
        | ShellControlKind::TextField { .. }
        | ShellControlKind::Slider { .. }
        | ShellControlKind::Switch { .. }
        | ShellControlKind::ListBoxChrome { .. }
        | ShellControlKind::ListItem { .. } => None,
    }
}

#[test]
fn custom_map_button_opens_original_custom_map_screen() {
    let screen = screen_for(
        ShellScreenId::NewGame,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );
    let map_button = screen
        .find_accessible("World Type")
        .expect("original MapButton should still be visible");

    assert_eq!(
        map_button.label(),
        "MAP",
        "MapButton should keep the original LDL text until CustomMapScreen is implemented"
    );
    assert_eq!(
        map_button.action(),
        Some(ShellAction::Navigate(ShellScreenId::CustomMap)),
        "MapButton should open the original CustomMapWindow"
    );

    let custom_map = screen_for(
        ShellScreenId::CustomMap,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );
    assert_eq!(
        custom_map.ldl_name(),
        "CustomMapWindow",
        "custom-map popup should use the original LDL block"
    );
    assert!(
        custom_map.find_accessible("WetDrySlider").is_some(),
        "custom-map screen should include the original wet/dry slider"
    );
    assert!(
        custom_map.find_accessible("Many Goods").is_some(),
        "custom-map screen should include the original resource-density label"
    );
}

#[test]
fn custom_map_sliders_and_back_button_match_original_defaults() {
    let custom_map = screen_for(
        ShellScreenId::CustomMap,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );

    for (accessible_name, expected_value) in [
        ("WetDrySlider", 5_i32),
        ("WarmColdSlider", 5_i32),
        ("OceanLandSlider", 3_i32),
        ("IslandContinentSlider", 5_i32),
        ("HomoDiverseSlider", 8_i32),
        ("GoodCountSlider", 5_i32),
    ] {
        let slider = custom_map
            .find_accessible(accessible_name)
            .expect("custom-map slider should be modeled");
        assert_eq!(
            slider_value(slider.kind()),
            Some(expected_value),
            "{accessible_name} should start at the base profile default"
        );
    }

    let back = custom_map
        .find_accessible("Back")
        .expect("custom-map popup should expose the reusable OK/back button");
    assert_eq!(
        back.rect(),
        MenuRect::new(363, 269, 70, 24),
        "custom-map Back should use c3_PopupWindow::AddOk geometry, not the obsolete 40px LDL BackButton"
    );
    assert_eq!(
        back.kind(),
        ShellControlKind::Button(ButtonStyleId::Ctp2Small),
        "custom-map Back should reuse the small popup button style"
    );
}

#[test]
fn difficulty_risk_column_only_contains_source_instantiated_radios() {
    let screen = screen_for(
        ShellScreenId::DifficultyPicker,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );

    assert!(
        screen.find_accessible("Ruins Only").is_some(),
        "the first risk radio should use the original string-table text"
    );
    assert!(
        screen.find_accessible("No Danger Here").is_none(),
        "spnewgamediffscreen.cpp only instantiates four risk radio buttons"
    );
}

#[test]
fn setup_picker_defaults_match_original_base_profile() {
    for (screen_id, accessible_name) in [
        (ShellScreenId::DifficultyPicker, "Medium"),
        (ShellScreenId::DifficultyPicker, "Bandits"),
        (ShellScreenId::MapSizePicker, "Regular"),
        (ShellScreenId::WorldShapePicker, "Earth World"),
        (ShellScreenId::PlayersPicker, "5"),
        (ShellScreenId::RulesPicker, "Pollution - On"),
    ] {
        let screen = screen_for(
            screen_id,
            &GameSetupState::default(),
            &ShellOptionsState::default(),
        );
        let control = screen
            .find_accessible(accessible_name)
            .expect("defaulted picker control should exist");
        assert_eq!(
            switch_enabled_state(control.kind()),
            Some(true),
            "{accessible_name} should be selected by default on {screen_id:?}"
        );
    }
}

#[test]
fn new_game_header_text_is_raised_clear_of_native_button_art() {
    let screen = screen_for(
        ShellScreenId::NewGame,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );

    for (label_name, button_name) in [
        ("Civilization Label", "Nation"),
        ("Difficulty Label", "Difficulty"),
        ("Players Label", "Players"),
        ("Rules Label", "Rules"),
        ("World Shape Label", "World Shape"),
        ("Map Size Label", "Map Size"),
        ("World Type Label", "World Type"),
        ("Editor Label", "Editor"),
    ] {
        let label_control = screen
            .find_accessible(label_name)
            .expect("new-game setup header should exist");
        let button_control = screen
            .find_accessible(button_name)
            .expect("new-game setup button should exist");
        let layout = label_layout(label_control.kind()).expect("setup header should be a label");
        let visual_bottom = label_control
            .rect()
            .y
            .saturating_add(layout.offset_y())
            .saturating_add(
                i32::try_from(label_control.rect().height).expect("label height should fit i32"),
            );

        assert_eq!(
            layout,
            ShellLabelLayout::centered_with_offset_y(-10_i32),
            "{label_name} should have the shared visual raise used for setup headers"
        );
        assert!(
            visual_bottom <= button_control.rect().y,
            "{label_name} should visually clear the button art for {button_name}"
        );
    }
}

#[test]
fn world_shape_side_descriptions_use_left_top_text_layout() {
    let screen = screen_for(
        ShellScreenId::WorldShapePicker,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );

    for accessible_name in [
        "East West Description",
        "Doughnut Description",
        "Flat Description",
    ] {
        let control = screen
            .find_accessible(accessible_name)
            .expect("world-shape side description should exist");
        assert_eq!(
            label_layout(control.kind()),
            Some(ShellLabelLayout::left_top()),
            "{accessible_name} should render from its LDL left/top anchor instead of centered"
        );
    }
}

#[test]
fn world_shape_rows_align_with_original_descriptions() {
    let screen = screen_for(
        ShellScreenId::WorldShapePicker,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );

    for (accessible_name, expected_y) in [
        ("Earth World", 45_i32),
        ("Doughnut World", 80_i32),
        ("Flat World", 115_i32),
        ("East West Description", 51_i32),
        ("Doughnut Description", 84_i32),
        ("Flat Description", 116_i32),
    ] {
        let control = screen
            .find_accessible(accessible_name)
            .expect("world-shape row or description should exist");
        assert_eq!(
            control.rect().y,
            expected_y,
            "{accessible_name} should keep the original aligned LDL y position"
        );
    }
}

#[test]
fn flat_world_description_aligns_with_other_world_shape_explanations() {
    let screen = screen_for(
        ShellScreenId::WorldShapePicker,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );

    for accessible_name in [
        "East West Description",
        "Doughnut Description",
        "Flat Description",
    ] {
        let control = screen
            .find_accessible(accessible_name)
            .expect("world-shape description should exist");
        assert_eq!(
            control.rect().x,
            190_i32,
            "{accessible_name} should use the same left edge for all side explanations"
        );
    }

    let flat = screen
        .find_accessible("Flat Description")
        .expect("flat-world explanation should exist");
    assert_eq!(
        flat.label(),
        "No wrapping; map edges do not connect.",
        "flat world should explain that no map edge wraps"
    );
}
