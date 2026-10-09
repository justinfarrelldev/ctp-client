//! Tests for modeled shell option screens and persistent option state.

use super::options_state::ShellOptionsState;
use super::*;

/// Returns selected state from a switch-like control for assertions.
fn switch_enabled(kind: ShellControlKind) -> Option<bool> {
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

/// Returns image metadata from a switch-like control for assertions.
fn switch_images(kind: ShellControlKind) -> Option<ShellImageSet> {
    match kind {
        ShellControlKind::Switch { images, .. } => images,
        ShellControlKind::Button(_)
        | ShellControlKind::Label { .. }
        | ShellControlKind::TextField { .. }
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
        | ShellControlKind::Label { .. }
        | ShellControlKind::TextField { .. }
        | ShellControlKind::Slider { .. }
        | ShellControlKind::Switch { .. }
        | ShellControlKind::ListBoxChrome { .. } => None,
    }
}

/// Expected metadata for one original options-root button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedOptionsButton {
    /// Accessible name used to find the modeled control.
    accessible_name: &'static str,
    /// Original LDL object name.
    ldl_name: &'static str,
    /// Visible button label.
    label_text: &'static str,
    /// Original LDL bounds.
    rect: MenuRect,
    /// Whether the modeled button should be enabled.
    enabled: bool,
    /// Expected activation action.
    action: Option<ShellAction>,
    /// Original source reference.
    source: &'static str,
}

/// Asserts that one options button preserves its LDL-derived metadata.
fn assert_options_button_metadata(options: &ShellScreen, expected: ExpectedOptionsButton) {
    let control = options
        .find_accessible(expected.accessible_name)
        .expect("options button should keep its accessible name");
    assert_eq!(
        control.ldl_name(),
        expected.ldl_name,
        "{} should keep its original LDL object name",
        expected.accessible_name
    );
    assert_eq!(
        control.label(),
        expected.label_text,
        "{} should keep its original visible label",
        expected.accessible_name
    );
    assert_eq!(
        control.rect(),
        expected.rect,
        "{} should keep its original LDL rectangle",
        expected.accessible_name
    );
    assert_eq!(
        control.kind(),
        ShellControlKind::Button(ButtonStyleId::BlueLarge),
        "{} should keep the original blue-large button art",
        expected.accessible_name
    );
    assert_eq!(
        control.enabled(),
        expected.enabled,
        "{} should keep its original enabled/disabled state",
        expected.accessible_name
    );
    assert_eq!(
        control.action(),
        expected.action,
        "{} should keep its modeled action",
        expected.accessible_name
    );
    assert_eq!(
        control.source(),
        expected.source,
        "{} should retain its CTP2 source reference",
        expected.accessible_name
    );
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
fn options_root_navigation_buttons_preserve_original_ldl_metadata() {
    let options = screen_for(
        ShellScreenId::Options,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );

    for expected in [
        ExpectedOptionsButton {
            accessible_name: "Game Play",
            ldl_name: "GamePlayButton",
            label_text: "GAME PLAY",
            rect: MenuRect::new(28, 65, 146, 24),
            enabled: true,
            action: Some(ShellAction::Navigate(ShellScreenId::Gameplay)),
            source: "options.ldl:94",
        },
        ExpectedOptionsButton {
            accessible_name: "Graphics",
            ldl_name: "GraphicsButton",
            label_text: "GRAPHICS",
            rect: MenuRect::new(28, 110, 146, 24),
            enabled: true,
            action: Some(ShellAction::Navigate(ShellScreenId::Graphics)),
            source: "options.ldl:106",
        },
        ExpectedOptionsButton {
            accessible_name: "Sound",
            ldl_name: "SoundButton",
            label_text: "SOUND",
            rect: MenuRect::new(28, 155, 146, 24),
            enabled: true,
            action: Some(ShellAction::Navigate(ShellScreenId::Sound)),
            source: "options.ldl:115",
        },
        ExpectedOptionsButton {
            accessible_name: "Music",
            ldl_name: "MusicButton",
            label_text: "MUSIC",
            rect: MenuRect::new(28, 200, 146, 24),
            enabled: true,
            action: Some(ShellAction::Navigate(ShellScreenId::Music)),
            source: "options.ldl:124",
        },
        ExpectedOptionsButton {
            accessible_name: "Keyboard",
            ldl_name: "KeyboardButton",
            label_text: "KEYBOARD",
            rect: MenuRect::new(28, 245, 146, 24),
            enabled: true,
            action: Some(ShellAction::Navigate(ShellScreenId::Keyboard)),
            source: "options.ldl:133",
        },
    ] {
        assert_options_button_metadata(&options, expected);
    }
}

#[test]
fn options_root_disabled_buttons_preserve_original_ldl_metadata() {
    let options = screen_for(
        ShellScreenId::Options,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );

    for expected in [
        ExpectedOptionsButton {
            accessible_name: "Save Game",
            ldl_name: "SaveGameButton",
            label_text: "SAVE GAME",
            rect: MenuRect::new(206, 65, 146, 24),
            enabled: false,
            action: Some(ShellAction::Disabled),
            source: "options.ldl:162 + optionswindow.cpp:DisableButtons",
        },
        ExpectedOptionsButton {
            accessible_name: "Load Game",
            ldl_name: "LoadGameButton",
            label_text: "LOAD GAME",
            rect: MenuRect::new(206, 110, 146, 24),
            enabled: false,
            action: Some(ShellAction::Disabled),
            source: "options.ldl:170 + optionswindow.cpp:DisableButtons",
        },
        ExpectedOptionsButton {
            accessible_name: "Restart",
            ldl_name: "RestartButton",
            label_text: "RESTART",
            rect: MenuRect::new(206, 155, 146, 24),
            enabled: false,
            action: Some(ShellAction::Disabled),
            source: "options.ldl:179 + optionswindow.cpp:DisableButtons",
        },
        ExpectedOptionsButton {
            accessible_name: "New Game",
            ldl_name: "NewGameButton",
            label_text: "NEW GAME",
            rect: MenuRect::new(206, 200, 146, 24),
            enabled: false,
            action: Some(ShellAction::Disabled),
            source: "options.ldl:187 + optionswindow.cpp:DisableButtons",
        },
        ExpectedOptionsButton {
            accessible_name: "Cheat Mode",
            ldl_name: "MapEditorButton",
            label_text: "CHEAT MODE",
            rect: MenuRect::new(117, 295, 146, 24),
            enabled: false,
            action: Some(ShellAction::Disabled),
            source: "options.ldl:205 + unavailable in pre-game shell",
        },
    ] {
        assert_options_button_metadata(&options, expected);
    }
}

#[test]
fn graphics_toggles_preserve_original_positions_and_sources() {
    let graphics = screen_for(
        ShellScreenId::Graphics,
        &GameSetupState::default(),
        &ShellOptionsState::default(),
    );

    for (accessible_name, ldl_name, rect, field, source) in [
        (
            "Unit Animation",
            "WalkButton",
            MenuRect::new(30, 90, 300, 24),
            ShellOptionField::UnitAnimation,
            "graphics.ldl:57",
        ),
        (
            "Animated Trade Routes",
            "TradeButton",
            MenuRect::new(30, 120, 300, 24),
            ShellOptionField::AnimatedTradeRoutes,
            "graphics.ldl:66",
        ),
        (
            "Wonder Movies",
            "WonderButton",
            MenuRect::new(30, 150, 300, 24),
            ShellOptionField::WonderMovies,
            "graphics.ldl:75",
        ),
        (
            "City Names",
            "CityNamesButton",
            MenuRect::new(30, 180, 300, 24),
            ShellOptionField::CityNames,
            "graphics.ldl:84",
        ),
        (
            "Political Borders",
            "PoliticalBordersButton",
            MenuRect::new(350, 90, 300, 24),
            ShellOptionField::PoliticalBorders,
            "graphics.ldl:96",
        ),
        (
            "Trade Routes",
            "TradeRoutesButton",
            MenuRect::new(350, 120, 300, 24),
            ShellOptionField::TradeRoutes,
            "graphics.ldl:105",
        ),
        (
            "City Influence",
            "CityInflenceButton",
            MenuRect::new(350, 150, 300, 24),
            ShellOptionField::CityInfluence,
            "graphics.ldl:114",
        ),
        (
            "Grid",
            "GridButton",
            MenuRect::new(350, 180, 300, 24),
            ShellOptionField::Grid,
            "graphics.ldl:123",
        ),
    ] {
        let control = graphics
            .find_accessible(accessible_name)
            .expect("graphics toggle should keep its accessible name");
        assert_eq!(
            control.ldl_name(),
            ldl_name,
            "{accessible_name} should keep its original LDL object name"
        );
        assert_eq!(
            control.rect(),
            rect,
            "{accessible_name} should keep its original LDL rectangle"
        );
        assert_eq!(
            switch_images(control.kind()),
            Some(ShellImageSet::graphics_checkbox()),
            "{accessible_name} should use the original graphics checkbox art"
        );
        assert_eq!(
            control.action(),
            Some(ShellAction::ToggleOption(field)),
            "{accessible_name} should toggle its original option field"
        );
        assert_eq!(
            control.source(),
            source,
            "{accessible_name} should retain its graphics.ldl source reference"
        );
    }
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
        .expect("pollution switch should exist after toggling off");
    assert_eq!(
        switch_enabled(pollution.kind()),
        Some(false),
        "pollution should render off after toggling from the base-game default on state"
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
