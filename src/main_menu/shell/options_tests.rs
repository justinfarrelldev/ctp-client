//! Tests for modeled shell option screens and persistent option state.

use super::options_state::ShellOptionsState;
use super::*;

/// Returns selected state from a switch-like control for assertions.
fn switch_enabled(kind: ShellControlKind) -> Option<bool> {
    match kind {
        ShellControlKind::Switch { enabled, .. } => Some(enabled),
        ShellControlKind::Button(_)
        | ShellControlKind::Label
        | ShellControlKind::TextField
        | ShellControlKind::Slider { .. }
        | ShellControlKind::ListBoxChrome { .. }
        | ShellControlKind::ListItem { .. } => None,
    }
}

/// Returns image metadata from a switch-like control for assertions.
fn switch_images(kind: ShellControlKind) -> Option<ShellImageSet> {
    match kind {
        ShellControlKind::Switch { images, .. } => images,
        ShellControlKind::Button(_)
        | ShellControlKind::Label
        | ShellControlKind::TextField
        | ShellControlKind::Slider { .. }
        | ShellControlKind::ListBoxChrome { .. }
        | ShellControlKind::ListItem { .. } => None,
    }
}

/// Returns whether a list item is selected.
fn list_item_selected(kind: ShellControlKind) -> Option<bool> {
    match kind {
        ShellControlKind::ListItem { selected, .. } => Some(selected),
        ShellControlKind::Button(_)
        | ShellControlKind::Label
        | ShellControlKind::TextField
        | ShellControlKind::Slider { .. }
        | ShellControlKind::Switch { .. }
        | ShellControlKind::ListBoxChrome { .. } => None,
    }
}

#[test]
fn gameplay_options_toggle_and_persist_modeled_state() {
    let mut model = SinglePlayerShellModel::default();
    assert_eq!(
        model.apply_action(ShellAction::Navigate(ShellScreenId::Gameplay)),
        ShellFlowOutcome::Continue,
        "Game Play should navigate to gameplay options"
    );
    assert_eq!(
        model.apply_action(ShellAction::ToggleOption(ShellOptionField::AutoCycleTurns)),
        ShellFlowOutcome::Continue,
        "Auto Cycle Turns should toggle"
    );
    assert!(
        !model
            .options()
            .bool_option(ShellOptionField::AutoCycleTurns),
        "Auto Cycle Turns should be off after one toggle from the default"
    );
    assert!(
        !model
            .options()
            .to_persisted()
            .gameplay
            .auto_cycle_turns
            .enabled(),
        "Auto Cycle Turns should be represented in persisted options"
    );

    let gameplay = screen_for(ShellScreenId::Gameplay, model.setup(), model.options());
    let auto_cycle_turns = gameplay
        .find_accessible("Auto Cycle Turns")
        .expect("Auto Cycle Turns control should exist");
    assert_eq!(
        auto_cycle_turns.label(),
        "Auto Cycle Turns Off",
        "gameplay labels should reflect toggled state"
    );
    assert_eq!(
        switch_images(auto_cycle_turns.kind()),
        Some(ShellImageSet::graphics_checkbox()),
        "gameplay switches should use the original upbt11 checkbox art"
    );
    assert_eq!(
        gameplay
            .find_accessible("Mouse Speed")
            .expect("Mouse Speed slider should exist")
            .kind(),
        ShellControlKind::Slider {
            min: 0_i32,
            max: 15_i32,
            value: 5_i32,
        },
        "Mouse Speed should use the original 0-15 slider range"
    );
}

#[test]
fn keyboard_options_categories_and_reset_defaults_work() {
    let mut model = SinglePlayerShellModel::default();
    assert_eq!(
        model.apply_action(ShellAction::Navigate(ShellScreenId::Keyboard)),
        ShellFlowOutcome::Continue,
        "Keyboard should navigate to the keyboard mapping screen"
    );
    assert_eq!(
        model.apply_action(ShellAction::SelectKeyboardCategory(
            crate::config::ctp2_options::Ctp2KeyboardCategory::Game,
        )),
        ShellFlowOutcome::Continue,
        "selecting a keyboard category should keep the screen open"
    );
    assert_eq!(
        model.options().keyboard_category(),
        crate::config::ctp2_options::Ctp2KeyboardCategory::Game,
        "selected keyboard category should update model state"
    );
    assert_eq!(
        model.apply_action(ShellAction::ResetKeyboardDefaults),
        ShellFlowOutcome::Continue,
        "Reset Defaults should clear custom bindings without closing the screen"
    );
    assert!(
        model.options().keyboard_bindings().is_empty(),
        "Reset Defaults should leave no custom key-binding overrides"
    );

    let keyboard = screen_for(ShellScreenId::Keyboard, model.setup(), model.options());
    assert!(
        keyboard.find_accessible("Open Options Screen").is_some(),
        "Game Options category should show original key-list rows"
    );
    assert_eq!(
        switch_enabled(
            keyboard
                .find_accessible("Game Options")
                .expect("Game Options category should exist")
                .kind(),
        ),
        Some(true),
        "selected keyboard category should render selected"
    );
}

#[test]
fn sound_sliders_include_original_min_max_labels() {
    let sound = screen_for(
        ShellScreenId::Sound,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );

    for accessible_name in [
        "SFX Volume Minimum",
        "SFX Volume Maximum",
        "Voice Volume Minimum",
        "Voice Volume Maximum",
        "Music Volume Minimum",
        "Music Volume Maximum",
    ] {
        assert!(
            sound.find_accessible(accessible_name).is_some(),
            "{accessible_name} should preserve the original slider endpoint label"
        );
    }
}

#[test]
fn rules_picker_toggles_in_place_and_reflects_selected_state() {
    let mut model = SinglePlayerShellModel::default();
    assert_eq!(
        model.apply_action(ShellAction::OpenPicker(GameSetupField::Rules)),
        ShellFlowOutcome::Continue,
        "opening rules picker should keep the shell active"
    );
    assert_eq!(
        model.apply_action(ShellAction::SelectGameSetupInPlace(
            GameSetupField::Rules,
            1_usize,
        )),
        ShellFlowOutcome::Continue,
        "rules switches should update in place like the original popup"
    );
    assert_eq!(
        model.current(),
        ShellScreenId::RulesPicker,
        "rules popup should remain open after switching a rule"
    );

    let rules = screen_for(ShellScreenId::RulesPicker, model.setup(), model.options());
    let pollution = rules
        .find_accessible("Pollution - Off")
        .expect("pollution switch should exist");
    assert_eq!(
        switch_enabled(pollution.kind()),
        Some(true),
        "selected rule switch should render selected state"
    );
}

#[test]
fn listbox_pickers_use_original_chrome_and_warning_labels() {
    let resolution = screen_for(
        ShellScreenId::Resolution,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );
    assert_eq!(
        resolution
            .find_accessible("Resolution List")
            .expect("resolution picker should model ResList chrome")
            .kind(),
        ShellControlKind::ListBoxChrome {
            scroll_offset: 0_usize,
            item_count: 19_usize,
            visible_rows: 4_usize,
        },
        "ScreenResScreen should use original CTP2_LISTBOX chrome metadata and visible row count"
    );
    assert!(
        resolution.find_accessible("Resolution Warning").is_none(),
        "immediate runtime resolution changes should not show the obsolete restart warning"
    );
    assert!(
        resolution.find_accessible("640 x 480").is_none(),
        "base-game display mode filter rejects 640x480"
    );
    let visible_resolution_rows = resolution
        .controls()
        .iter()
        .filter(|control| matches!(control.kind(), ShellControlKind::ListItem { .. }))
        .count();
    assert_eq!(
        visible_resolution_rows, 4_usize,
        "the 89px ScreenResScreen listbox should show four 20px rows plus native chrome instead of spilling all modes over the popup"
    );
    assert_eq!(
        list_item_selected(
            resolution
                .find_accessible("800 x 600")
                .expect("800 x 600 should be initially visible")
                .kind(),
        ),
        Some(true),
        "the current resolution should be force-selected like the original c3_ListBox"
    );
    assert!(
        resolution.find_accessible("1920 x 1200").is_none(),
        "modes outside the initial visible listbox window should be reached by scrolling rather than spilling outside the popup"
    );

    let music_tracks = screen_for(
        ShellScreenId::MusicTrack,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );
    assert_eq!(
        music_tracks
            .find_accessible("Track List")
            .expect("music picker should model TrackList chrome")
            .kind(),
        ShellControlKind::ListBoxChrome {
            scroll_offset: 0_usize,
            item_count: 10_usize,
            visible_rows: 5_usize,
        },
        "MusicTrackScreen should use original TrackList listbox chrome metadata"
    );
}

#[test]
fn resolution_picker_scrolls_to_all_legal_modes_without_row_spill() {
    let mut model = SinglePlayerShellModel::default();

    assert_eq!(
        model.apply_action(ShellAction::OpenOption(ShellOptionField::Resolution)),
        ShellFlowOutcome::Continue,
        "opening the resolution picker should keep the shell active"
    );
    assert_eq!(
        model.apply_action(ShellAction::ScrollOptionList(
            ShellOptionField::Resolution,
            12_i32,
        )),
        ShellFlowOutcome::Continue,
        "scrolling should keep the resolution picker open"
    );
    let scrolled = screen_for(ShellScreenId::Resolution, model.setup(), model.options());
    assert!(
        scrolled.find_accessible("1920 x 1200").is_some(),
        "scrolling the original listbox should reveal later legal display modes"
    );
    assert_eq!(
        model.activate_accessible("1920 x 1200"),
        Some(ShellFlowOutcome::Continue),
        "visible scrolled modes should remain selectable by accessible interaction"
    );
    assert_eq!(
        model.options().resolution(),
        "1920 x 1200",
        "selecting a scrolled row should update the modeled resolution"
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
        options.select(ShellOptionField::Resolution, 1_usize),
        "resolution picker should select a listed resolution"
    );
    assert_eq!(
        options.resolution(),
        "1024 x 768",
        "resolution label should follow the original base-game legal picker list"
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
