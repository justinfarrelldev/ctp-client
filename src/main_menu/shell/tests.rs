//! Tests for the single-player shell model.

use super::options_state::ShellOptionsState;
use super::*;

/// Returns image metadata from a switch-like control for assertions.
fn switch_images(kind: ShellControlKind) -> Option<ShellImageSet> {
    match kind {
        ShellControlKind::Switch { images, .. } => images,
        ShellControlKind::Button(_)
        | ShellControlKind::Label
        | ShellControlKind::TextField
        | ShellControlKind::Slider { .. }
        | ShellControlKind::ListBoxChrome
        | ShellControlKind::ListItem { .. } => None,
    }
}

#[test]
fn single_player_hub_preserves_original_ldl_metadata_and_sound_policy() {
    let screen = screen_for(
        ShellScreenId::SinglePlayerHub,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );

    assert_eq!(
        screen.id(),
        ShellScreenId::SinglePlayerHub,
        "single-player hub should keep its stable screen id"
    );
    assert_eq!(
        screen.ldl_name(),
        "SPWindow",
        "single-player hub should validate against the original SPWindow block"
    );
    let new_game = screen
        .find_accessible("New Game")
        .expect("New Game control should be accessible by name");
    assert_eq!(
        new_game.ldl_name(),
        "NewGameButton",
        "New Game button should keep the original LDL object name"
    );
    assert_eq!(
        new_game.accessible_name(),
        "New Game",
        "accessible name should be stable for tests"
    );
    assert_eq!(
        new_game.source(),
        "spnewgame.ldl:181",
        "source reference should point at the LDL control"
    );
    let sound = new_game
        .sound()
        .expect("actionable buttons should expose original sound policy");
    assert_eq!(
        sound.press(),
        BUTTON_PRESS_SOUND,
        "press sound should match original AUI button policy"
    );
    assert_eq!(
        sound.execute(),
        BUTTON_EXECUTE_SOUND,
        "execute sound should match original AUI button policy"
    );
}

#[test]
fn new_game_flow_allows_nation_difficulty_and_launch_selection() {
    let mut model = SinglePlayerShellModel::default();

    assert_eq!(
        model.current(),
        ShellScreenId::Landing,
        "shell model should start at the landing screen"
    );
    assert_eq!(
        model.apply_action(ShellAction::Navigate(ShellScreenId::SinglePlayerHub)),
        ShellFlowOutcome::Continue,
        "opening single player should keep the shell active"
    );
    assert_eq!(
        model.activate_accessible("New Game"),
        Some(ShellFlowOutcome::Continue),
        "accessible New Game activation should keep the shell active"
    );
    assert_eq!(
        model.apply_action(ShellAction::OpenPicker(GameSetupField::Nation)),
        ShellFlowOutcome::Continue,
        "opening the nation picker should keep the shell active"
    );
    assert_eq!(
        model.current(),
        ShellScreenId::NationPicker,
        "nation picker should be the current screen"
    );
    assert_eq!(
        model.apply_action(ShellAction::SelectGameSetupInPlace(
            GameSetupField::Nation,
            1_usize
        )),
        ShellFlowOutcome::Continue,
        "selecting a nation should update without launching"
    );
    assert_eq!(
        model.current(),
        ShellScreenId::NationPicker,
        "empire selection should stay in the original modal until Back is pressed"
    );
    assert_eq!(
        model.apply_action(ShellAction::Back),
        ShellFlowOutcome::Continue,
        "Back should close the empire picker"
    );
    assert_ne!(
        model.setup().selected(GameSetupField::Nation),
        "ROMANS",
        "selecting the second original nation should update the setup"
    );
    assert_eq!(
        model.setup().leader_name(),
        "George Washington",
        "selecting a new empire should immediately update the leader field like SPNewGameTribeScreen"
    );
    assert_eq!(
        model.apply_action(ShellAction::OpenPicker(GameSetupField::Difficulty)),
        ShellFlowOutcome::Continue,
        "opening difficulty should keep the shell active"
    );
    assert_eq!(
        model.apply_action(ShellAction::SelectGameSetup(
            GameSetupField::Difficulty,
            0_usize
        )),
        ShellFlowOutcome::Continue,
        "selecting difficulty should return to the setup screen"
    );
    assert_eq!(
        model.apply_action(ShellAction::LaunchGame),
        ShellFlowOutcome::LaunchGame,
        "launch action should leave the shell with a launch outcome"
    );
}

#[test]
fn nation_picker_matches_original_tribe_popup_layout_and_art() {
    let screen = screen_for(
        ShellScreenId::NationPicker,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );

    assert_eq!(
        screen.ldl_name(),
        "SPNewGameTribeScreen",
        "nation picker should use the original tribe popup block"
    );
    assert_eq!(
        screen.rect(),
        MenuRect::new(75, 67, 650, 465),
        "nation picker should be the original large centered 650x465 popup"
    );
    let first = screen
        .find_accessible("Greek")
        .expect("first tribe radio should exist");
    assert_eq!(
        first.rect(),
        MenuRect::new(25, 45, 146, 24),
        "first tribe radio should start at the original first column position"
    );
    assert_eq!(
        switch_images(first.kind()),
        Some(ShellImageSet::popup_checkbox()),
        "tribe radios should use original upba2317/upba2318 checkbox art"
    );
    let twelfth = screen
        .find_accessible("Spaniard")
        .expect("twelfth tribe radio should be in the second column");
    assert_eq!(
        twelfth.rect(),
        MenuRect::new(176, 45, 146, 24),
        "tribe list should wrap to the second original radio column after eleven rows"
    );
    let leader_field = screen
        .find_accessible("Leader Name Field")
        .expect("original tribe popup should include the editable leader name field");
    assert_eq!(
        leader_field.kind(),
        ShellControlKind::TextField,
        "leader-name field should be modeled as an editable text field"
    );
    assert_eq!(
        leader_field.action(),
        Some(ShellAction::FocusLeaderName),
        "clicking the leader-name field should focus it for text input"
    );
    assert!(
        screen.find_accessible("Leader Name").is_some(),
        "original tribe popup should include the leader name label"
    );
    assert!(
        screen.find_accessible("Male").is_some(),
        "original tribe popup should include the male gender radio"
    );
    assert!(
        screen.find_accessible("Female").is_some(),
        "original tribe popup should include the female gender radio"
    );
}

#[test]
fn selecting_empire_updates_default_leader_and_leader_field_is_editable() {
    let mut model = SinglePlayerShellModel::default();

    assert_eq!(
        model.apply_action(ShellAction::SelectGameSetupInPlace(
            GameSetupField::Nation,
            8_usize
        )),
        ShellFlowOutcome::Continue,
        "selecting an empire should remain in the picker"
    );
    assert_eq!(
        model.setup().selected(GameSetupField::Nation),
        "JAPANESE",
        "the ninth base-game civilisation in current order should be Japanese"
    );
    assert_eq!(
        model.setup().leader_name(),
        "Minamoto",
        "default leader should come from civilisation.txt/civ_str.txt"
    );

    assert_eq!(
        model.apply_action(ShellAction::FocusLeaderName),
        ShellFlowOutcome::Continue,
        "leader field focus should keep the current shell open"
    );
    model.input_text("Maximilian Alexander Caesar");
    assert_eq!(
        model.setup().leader_name(),
        "Maximilian Alexander Ca",
        "leader name editing should enforce FIELD maxfieldlen 23"
    );
    model.backspace_text();
    assert_eq!(
        model.setup().leader_name(),
        "Maximilian Alexander C",
        "Backspace should edit the focused leader-name field"
    );
}

#[test]
fn setup_picker_popups_match_original_ldl_dimensions_and_checkbox_art() {
    let cases = [
        (
            ShellScreenId::DifficultyPicker,
            MenuRect::new(200, 165, 400, 270),
            "Beginner",
            MenuRect::new(40, 60, 146, 24),
        ),
        (
            ShellScreenId::MapSizePicker,
            MenuRect::new(200, 205, 400, 190),
            "Small",
            MenuRect::new(125, 45, 146, 24),
        ),
        (
            ShellScreenId::WorldShapePicker,
            MenuRect::new(150, 225, 500, 150),
            "Earth World",
            MenuRect::new(20, 45, 146, 24),
        ),
        (
            ShellScreenId::PlayersPicker,
            MenuRect::new(200, 175, 400, 250),
            "3",
            MenuRect::new(125, 45, 146, 24),
        ),
        (
            ShellScreenId::RulesPicker,
            MenuRect::new(275, 220, 250, 160),
            "Bloodlust - Off",
            MenuRect::new(50, 50, 160, 24),
        ),
    ];

    for (screen_id, expected_rect, accessible_name, expected_control_rect) in cases {
        let screen = screen_for(
            screen_id,
            &GameSetupState::default(),
            &ShellOptionsState::default(),
        );
        assert_eq!(
            screen.rect(),
            expected_rect,
            "{screen_id:?} should preserve original popup geometry"
        );
        let control = screen
            .find_accessible(accessible_name)
            .expect("original popup option should be addressable");
        assert_eq!(
            control.rect(),
            expected_control_rect,
            "{screen_id:?} first selectable option should preserve original geometry"
        );
        assert_eq!(
            switch_images(control.kind()),
            Some(ShellImageSet::popup_checkbox()),
            "{screen_id:?} selectable options should use original popup checkbox/toggle art"
        );
    }
}

#[test]
fn popup_close_buttons_use_original_c3_popup_close_style_and_top_layer() {
    for (screen_id, expected_rect) in [
        (ShellScreenId::Options, MenuRect::new(293, 324, 70, 24)),
        (ShellScreenId::Graphics, MenuRect::new(593, 249, 70, 24)),
        (ShellScreenId::Sound, MenuRect::new(263, 219, 70, 24)),
        (ShellScreenId::Music, MenuRect::new(173, 219, 70, 24)),
        (ShellScreenId::NationPicker, MenuRect::new(563, 424, 70, 24)),
        (
            ShellScreenId::DifficultyPicker,
            MenuRect::new(313, 229, 70, 24),
        ),
    ] {
        let screen = screen_for(
            screen_id,
            &GameSetupState::default(),
            &ShellOptionsState::default(),
        );
        let close = screen
            .find_accessible("Close")
            .expect("popup should expose the AddClose-generated close button");
        assert_eq!(
            close.rect(),
            expected_rect,
            "{screen_id:?} close button should match c3_PopupWindow::AddClose placement"
        );
        assert_eq!(
            close.kind(),
            ShellControlKind::Button(ButtonStyleId::Ctp2Small),
            "{screen_id:?} close button should use c3_PopupClose/CTP2_BUTTON_SMALL art"
        );
        let global = MenuRect::new(
            screen.rect().x.saturating_add(expected_rect.x),
            screen.rect().y.saturating_add(expected_rect.y),
            expected_rect.width,
            expected_rect.height,
        );
        let hit = screen
            .find_at_point(
                global.x.saturating_add(5_i32),
                global.y.saturating_add(5_i32),
            )
            .expect("close button should be hit-testable on the top layer");
        assert_eq!(
            hit.action(),
            Some(ShellAction::Back),
            "{screen_id:?} close button should return to the previous screen"
        );
    }
}

#[test]
fn new_game_labels_use_original_ldl_text_and_geometry() {
    let screen = screen_for(
        ShellScreenId::NewGame,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );

    for (accessible_name, expected_label, expected_rect) in [
        (
            "Difficulty Label",
            "DIFFICULTY",
            MenuRect::new(165, 192, 180, 18),
        ),
        (
            "Players Label",
            "# EMPIRES",
            MenuRect::new(165, 258, 180, 18),
        ),
        ("Rules Label", "RULES", MenuRect::new(165, 322, 180, 18)),
        (
            "World Shape Label",
            "WORLD SHAPE",
            MenuRect::new(455, 192, 180, 18),
        ),
        (
            "Map Size Label",
            "MAP SIZE",
            MenuRect::new(455, 258, 180, 18),
        ),
        (
            "World Type Label",
            "CUSTOMIZE MAP",
            MenuRect::new(455, 322, 180, 18),
        ),
        (
            "Editor Label",
            "SCENARIO EDITOR",
            MenuRect::new(455, 388, 180, 18),
        ),
    ] {
        let control = screen
            .find_accessible(accessible_name)
            .expect("new-game label should exist");
        assert_eq!(
            control.label(),
            expected_label,
            "{accessible_name} should use the original visible string"
        );
        assert_eq!(
            control.rect(),
            expected_rect,
            "{accessible_name} should keep the original LDL geometry"
        );
    }

    let editor = screen
        .find_accessible("Editor")
        .expect("editor launcher should exist");
    assert_eq!(
        editor.label(),
        "Launch Editor",
        "EditorButton should resolve str_ldl_Editor from cht_str.txt"
    );
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
fn option_screen_labels_use_original_visible_strings() {
    let options_screen = screen_for(
        ShellScreenId::Options,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );
    assert_eq!(
        options_screen
            .find_accessible("Sound")
            .expect("sound button should exist")
            .label(),
        "SOUND",
        "OptionsWindow.SoundButton uses str_ldl_SFX, whose English text is SOUND"
    );
    assert!(
        options_screen.find_accessible("Quit To Shell").is_none(),
        "base-game options popup should not expose a quit button"
    );

    let graphics = screen_for(
        ShellScreenId::Graphics,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );
    assert_eq!(
        graphics
            .find_accessible("Graphics Title")
            .expect("graphics title should exist")
            .label(),
        "GRAPHICS"
    );
    assert_eq!(
        graphics
            .find_accessible("Change Resolution")
            .expect("resolution button should exist")
            .label(),
        "Screen Size"
    );

    let sound = screen_for(
        ShellScreenId::Sound,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );
    assert_eq!(
        sound
            .find_accessible("Sound Title")
            .expect("sound title should exist")
            .label(),
        "SOUND"
    );
    assert_eq!(
        sound
            .find_accessible("SFX Volume Label")
            .expect("SFX label should exist")
            .label(),
        "Sound Volume"
    );

    let music = screen_for(
        ShellScreenId::Music,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );
    assert_eq!(
        music
            .find_accessible("Select Track")
            .expect("select-track button should exist")
            .label(),
        "Select Track"
    );
    assert_eq!(
        music
            .find_accessible("Auto Repeat")
            .expect("auto-repeat switch should exist")
            .label(),
        "Auto Repeat Off"
    );
}

#[test]
fn graphics_and_music_switches_use_original_art_families() {
    let graphics = screen_for(
        ShellScreenId::Graphics,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );
    let walk = graphics
        .find_accessible("Unit Animation")
        .expect("graphics unit animation switch should exist");
    assert_eq!(
        switch_images(walk.kind()),
        Some(ShellImageSet::graphics_checkbox()),
        "graphics switches should use original upbt11 toggle art"
    );

    let music = screen_for(
        ShellScreenId::Music,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );
    let auto_repeat = music
        .find_accessible("Auto Repeat")
        .expect("music auto-repeat switch should exist");
    assert_eq!(
        switch_images(auto_repeat.kind()),
        Some(ShellImageSet::popup_checkbox()),
        "music switches should use the original popup checkbox art"
    );
}

#[test]
fn graphics_switches_are_toggleable_and_update_labels() {
    let mut model = SinglePlayerShellModel::default();
    assert_eq!(
        model.apply_action(ShellAction::ToggleOption(ShellOptionField::UnitAnimation)),
        ShellFlowOutcome::Continue,
        "unit animation switch should toggle"
    );
    assert!(
        !model.options().bool_option(ShellOptionField::UnitAnimation),
        "unit animation should be off after one toggle"
    );

    let graphics = screen_for(ShellScreenId::Graphics, model.setup(), model.options());
    let walk = graphics
        .find_accessible("Unit Animation")
        .expect("graphics unit animation switch should exist");
    assert_eq!(
        walk.label(),
        "Unit Animations Off",
        "graphics switch label should follow the original on/off stringtable behavior"
    );
    assert_eq!(
        switch_images(walk.kind()),
        Some(ShellImageSet::graphics_checkbox()),
        "toggled graphics switches should retain original art"
    );
}

#[test]
fn new_game_screen_omits_uninstantiated_profile_world_dropdown_panels() {
    let screen = screen_for(
        ShellScreenId::NewGame,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );

    for bad_control in [
        "Clan",
        "Gender",
        "World Map Size",
        "World Type List",
        "World Shape List",
        "World Custom",
    ] {
        assert!(
            screen.find_accessible(bad_control).is_none(),
            "{bad_control} is an LDL leftover/dropdown panel control not instantiated by SPNewGameWindow"
        );
    }
    assert!(
        screen.find_accessible("Scenario Static Text").is_none(),
        "SPNewGameWindow::Update hides the scenario caption when no scenario is active"
    );
    assert!(
        screen.find_accessible("Scenario Name").is_none(),
        "SPNewGameWindow::Update hides the scenario name when no scenario is active"
    );
    assert!(
        screen.find_accessible("Scenario Picker").is_some(),
        "Scenario picker belongs on the New Game screen in the original source"
    );
    assert!(
        screen.find_accessible("Editor").is_some(),
        "scenario editor launcher belongs on the New Game screen in the original source"
    );
}

#[test]
fn single_player_hub_omits_scenario_and_map_editor() {
    let screen = screen_for(
        ShellScreenId::SinglePlayerHub,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );

    assert!(
        screen.find_accessible("Scenario").is_none(),
        "Scenario should be removed from the second shell screen"
    );
    assert!(
        screen.find_accessible("Map Editor").is_none(),
        "Map Editor should be removed from the second shell screen"
    );
}

#[test]
fn options_screen_hides_quit_and_disables_game_management_controls() {
    let screen = screen_for(
        ShellScreenId::Options,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );

    assert!(
        screen.find_accessible("Quit To Windows").is_none(),
        "OptionsWindow source never instantiates the LDL-only QuitButton"
    );
    assert!(
        screen.find_accessible("Quit To Shell").is_none(),
        "base-game options popup removes the quit-to-shell control from the active child list"
    );

    for accessible_name in [
        "Save Game",
        "Load Game",
        "Restart",
        "New Game",
        "Cheat Mode",
    ] {
        let control = screen
            .find_accessible(accessible_name)
            .expect("unavailable pre-game control should remain modeled");
        assert_eq!(
            control.action(),
            Some(ShellAction::Disabled),
            "{accessible_name} should remain disabled in the base-game options popup"
        );
        assert!(
            !control.enabled(),
            "{accessible_name} should render with disabled visual state"
        );
    }

    for accessible_name in ["Game Play", "Keyboard"] {
        let control = screen
            .find_accessible(accessible_name)
            .expect("non-right-side config/control should remain modeled");
        assert!(
            control.enabled(),
            "{accessible_name} should not be visually disabled by unimplemented action state"
        );
    }
    assert_eq!(
        screen
            .find_accessible("Game Play")
            .expect("Game Play should be accessible")
            .action(),
        Some(ShellAction::Navigate(ShellScreenId::Gameplay)),
        "Game Play should open the gameplay options screen"
    );
    assert_eq!(
        screen
            .find_accessible("Keyboard")
            .expect("Keyboard should be accessible")
            .action(),
        Some(ShellAction::Navigate(ShellScreenId::Keyboard)),
        "Keyboard should open the keyboard mapping screen"
    );
}
