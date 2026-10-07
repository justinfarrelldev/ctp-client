//! Tests for the single-player shell model.

use super::*;

/// Returns image metadata from a switch-like control for assertions.
fn switch_images(kind: ShellControlKind) -> Option<ShellImageSet> {
    match kind {
        ShellControlKind::Switch { images, .. } => images,
        ShellControlKind::Button(_)
        | ShellControlKind::Label
        | ShellControlKind::TextField
        | ShellControlKind::Slider { .. }
        | ShellControlKind::Decoration
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
        model.apply_action(ShellAction::SelectGameSetup(
            GameSetupField::Nation,
            1_usize
        )),
        ShellFlowOutcome::Continue,
        "selecting a nation should close the picker without launching"
    );
    assert_ne!(
        model.setup().selected(GameSetupField::Nation),
        "ROMANS",
        "selecting the second original nation should update the setup"
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
        .find_accessible("GREEKS")
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
        .find_accessible("INDIANS")
        .expect("twelfth tribe radio should be in the second column");
    assert_eq!(
        twelfth.rect(),
        MenuRect::new(176, 45, 146, 24),
        "tribe list should wrap to the second original radio column after eleven rows"
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
fn setup_picker_popups_match_original_ldl_dimensions_and_checkbox_art() {
    let cases = [
        (
            ShellScreenId::DifficultyPicker,
            MenuRect::new(200, 165, 400, 270),
            "CHIEFTAIN",
            MenuRect::new(40, 60, 146, 24),
        ),
        (
            ShellScreenId::WorldTypePicker,
            MenuRect::new(200, 215, 400, 170),
            "RANDOM MAP",
            MenuRect::new(125, 45, 146, 24),
        ),
        (
            ShellScreenId::MapSizePicker,
            MenuRect::new(200, 205, 400, 190),
            "SMALL 24x48",
            MenuRect::new(125, 45, 146, 24),
        ),
        (
            ShellScreenId::WorldShapePicker,
            MenuRect::new(150, 225, 500, 150),
            "EARTH WORLD",
            MenuRect::new(20, 45, 146, 24),
        ),
        (
            ShellScreenId::PlayersPicker,
            MenuRect::new(200, 175, 400, 250),
            "3 PLAYERS",
            MenuRect::new(125, 45, 146, 24),
        ),
        (
            ShellScreenId::RulesPicker,
            MenuRect::new(275, 220, 250, 160),
            "BLOODLUST OFF",
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
fn options_screen_state_supports_volume_resolution_and_music_toggles() {
    let mut options = ShellOptionsState::default();

    assert!(
        options.set_numeric(ShellOptionField::SfxVolume, -5_i32),
        "SFX slider should accept numeric updates"
    );
    assert_eq!(
        options.numeric(ShellOptionField::SfxVolume),
        Some(0_i32),
        "SFX volume should clamp to the original minimum"
    );
    assert!(
        options.set_numeric(ShellOptionField::UnitSpeed, 99_i32),
        "unit speed slider should accept numeric updates"
    );
    assert_eq!(
        options.numeric(ShellOptionField::UnitSpeed),
        Some(9_i32),
        "unit speed should clamp to the original maximum"
    );
    assert!(
        options.select(ShellOptionField::Resolution, 2_usize),
        "resolution picker should select a listed resolution"
    );
    assert_eq!(
        options.resolution, "1024 x 768",
        "resolution label should follow the original picker list"
    );
    assert!(
        options.toggle(ShellOptionField::MusicEnabled),
        "music switch should toggle"
    );
    assert!(
        !options.music_enabled(),
        "music should be off after one toggle from the default state"
    );
    assert!(
        options.toggle(ShellOptionField::RandomOrder),
        "random-order switch should toggle"
    );
    assert!(
        options.random_order(),
        "random order should be enabled after one toggle"
    );
    assert!(
        options.select(ShellOptionField::MusicTrack, 2_usize),
        "music track picker should accept original list indexes"
    );
    assert_eq!(
        options.music_track(),
        3_usize,
        "music track is stored as the original one-based track number"
    );
}
