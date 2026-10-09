//! Unit tests for shell runtime effect queuing.

use std::path::PathBuf;

use super::*;

/// Clicks a shell point and ignores non-terminal navigation results.
fn click(state: &mut ShellRuntimeState, _model: &MainMenuModel, x: i32, y: i32) {
    assert!(
        state.click_shell(x, y).is_none(),
        "shell setup clicks should not exit the menu loop"
    );
}

/// Creates a runtime shell test model.
fn test_model() -> MainMenuModel {
    MainMenuModel::ctp2_landing(PathBuf::from("/tmp/ctp2-test"))
}

#[test]
fn resolution_picker_click_queues_window_resize_effect() {
    let model = test_model();
    let mut state = ShellRuntimeState::new();
    state.open_single_player();

    click(&mut state, &model, 156_i32, 276_i32);
    click(&mut state, &model, 238_i32, 228_i32);
    click(&mut state, &model, 325_i32, 380_i32);
    click(&mut state, &model, 303_i32, 263_i32);

    let effects = state.take_effects();
    assert_eq!(
        effects.resolution,
        Some((1024_u32, 768_u32)),
        "selecting the 1024x768 row should request an SDL window resize"
    );
}

#[test]
fn music_controls_queue_track_and_toggle_effects() {
    let model = test_model();
    let mut state = ShellRuntimeState::new();
    state.open_single_player();

    click(&mut state, &model, 156_i32, 276_i32);
    click(&mut state, &model, 238_i32, 318_i32);
    click(&mut state, &model, 316_i32, 217_i32);
    click(&mut state, &model, 303_i32, 303_i32);
    click(&mut state, &model, 327_i32, 330_i32);

    let effects = state.take_effects();
    assert_eq!(
        effects.music_track,
        Some(3_usize),
        "selecting the third music row should queue Track03 playback"
    );
    assert_eq!(
        effects.music_enabled,
        Some(false),
        "clicking Music On should queue a pause when default music is enabled"
    );
}

#[test]
fn graphics_unit_speed_slider_drags_after_pointer_capture() {
    let model = test_model();
    let mut state = ShellRuntimeState::new();
    state.open_single_player();

    click(&mut state, &model, 156_i32, 276_i32);
    click(&mut state, &model, 238_i32, 228_i32);
    click(&mut state, &model, 250_i32, 205_i32);
    state.drag_shell(550_i32, 205_i32);
    state.release_pointer();

    assert_eq!(
        state.numeric_option_for_test(ShellOptionField::UnitSpeed),
        Some(9_i32),
        "captured slider drags should update unit speed after the initial click"
    );
}

#[test]
fn sound_slider_click_queues_volume_and_persistence_effects() {
    let model = test_model();
    let mut state = ShellRuntimeState::new();
    state.open_single_player();

    click(&mut state, &model, 156_i32, 276_i32);
    click(&mut state, &model, 238_i32, 273_i32);
    click(&mut state, &model, 385_i32, 238_i32);

    let effects = state.take_effects();
    assert_eq!(
        effects.sfx_volume,
        Some(4_i32),
        "clicking inside the SFX slider should queue the computed 0-10 volume"
    );
    assert_eq!(
        effects
            .persisted_options
            .expect("volume changes should queue persisted options")
            .sound
            .sound_volume,
        4_i32,
        "SFX volume should be included in persisted CTP2 options"
    );
}
