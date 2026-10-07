//! Tests for the single-player shell model.

use super::*;

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
