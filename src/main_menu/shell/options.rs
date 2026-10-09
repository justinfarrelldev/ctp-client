//! Options, graphics, sound, and music screen builders.

use alloc::vec::Vec;

use super::MenuRect;
use super::builders::{
    ShellButtonSpec, append_button_specs, asset, button, image_switch_control, label,
    popup_close_button, slider,
};
use super::options_state::ShellOptionsState;
use super::{
    ShellAction, ShellControl, ShellImageSet, ShellOptionField, ShellScreen, ShellScreenId,
};
use crate::main_menu::ButtonVisualStyle;

/// Standard root-options button size from `options.ldl`.
const OPTIONS_BUTTON_SIZE: (u32, u32) = (146, 24);
/// Left-column configuration buttons in `OptionsWindow.ConfigHeader`.
const OPTIONS_CONFIG_BUTTONS: [ShellButtonSpec; 5] = [
    ShellButtonSpec::enabled(
        "GamePlayButton",
        "GAME PLAY",
        "Game Play",
        MenuRect::new(28, 65, OPTIONS_BUTTON_SIZE.0, OPTIONS_BUTTON_SIZE.1),
        ButtonVisualStyle::blue_large(),
        ShellAction::Navigate(ShellScreenId::Gameplay),
        "options.ldl:94",
    ),
    ShellButtonSpec::enabled(
        "GraphicsButton",
        "GRAPHICS",
        "Graphics",
        MenuRect::new(28, 110, OPTIONS_BUTTON_SIZE.0, OPTIONS_BUTTON_SIZE.1),
        ButtonVisualStyle::blue_large(),
        ShellAction::Navigate(ShellScreenId::Graphics),
        "options.ldl:106",
    ),
    ShellButtonSpec::enabled(
        "SoundButton",
        "SOUND",
        "Sound",
        MenuRect::new(28, 155, OPTIONS_BUTTON_SIZE.0, OPTIONS_BUTTON_SIZE.1),
        ButtonVisualStyle::blue_large(),
        ShellAction::Navigate(ShellScreenId::Sound),
        "options.ldl:115",
    ),
    ShellButtonSpec::enabled(
        "MusicButton",
        "MUSIC",
        "Music",
        MenuRect::new(28, 200, OPTIONS_BUTTON_SIZE.0, OPTIONS_BUTTON_SIZE.1),
        ButtonVisualStyle::blue_large(),
        ShellAction::Navigate(ShellScreenId::Music),
        "options.ldl:124",
    ),
    ShellButtonSpec::enabled(
        "KeyboardButton",
        "KEYBOARD",
        "Keyboard",
        MenuRect::new(28, 245, OPTIONS_BUTTON_SIZE.0, OPTIONS_BUTTON_SIZE.1),
        ButtonVisualStyle::blue_large(),
        ShellAction::Navigate(ShellScreenId::Keyboard),
        "options.ldl:133",
    ),
];
/// Right-column game buttons disabled by `optionswindow.cpp::DisableButtons` pre-game.
const OPTIONS_DISABLED_GAME_BUTTONS: [ShellButtonSpec; 5] = [
    ShellButtonSpec::disabled(
        "SaveGameButton",
        "SAVE GAME",
        "Save Game",
        MenuRect::new(206, 65, OPTIONS_BUTTON_SIZE.0, OPTIONS_BUTTON_SIZE.1),
        ButtonVisualStyle::blue_large(),
        "options.ldl:162 + optionswindow.cpp:DisableButtons",
    ),
    ShellButtonSpec::disabled(
        "LoadGameButton",
        "LOAD GAME",
        "Load Game",
        MenuRect::new(206, 110, OPTIONS_BUTTON_SIZE.0, OPTIONS_BUTTON_SIZE.1),
        ButtonVisualStyle::blue_large(),
        "options.ldl:170 + optionswindow.cpp:DisableButtons",
    ),
    ShellButtonSpec::disabled(
        "RestartButton",
        "RESTART",
        "Restart",
        MenuRect::new(206, 155, OPTIONS_BUTTON_SIZE.0, OPTIONS_BUTTON_SIZE.1),
        ButtonVisualStyle::blue_large(),
        "options.ldl:179 + optionswindow.cpp:DisableButtons",
    ),
    ShellButtonSpec::disabled(
        "NewGameButton",
        "NEW GAME",
        "New Game",
        MenuRect::new(206, 200, OPTIONS_BUTTON_SIZE.0, OPTIONS_BUTTON_SIZE.1),
        ButtonVisualStyle::blue_large(),
        "options.ldl:187 + optionswindow.cpp:DisableButtons",
    ),
    ShellButtonSpec::disabled(
        "MapEditorButton",
        "CHEAT MODE",
        "Cheat Mode",
        MenuRect::new(117, 295, OPTIONS_BUTTON_SIZE.0, OPTIONS_BUTTON_SIZE.1),
        ButtonVisualStyle::blue_large(),
        "options.ldl:205 + unavailable in pre-game shell",
    ),
];
/// Original graphics checkbox size from `graphics.ldl` toggle rows.
const GRAPHICS_TOGGLE_SIZE: (u32, u32) = (300, 24);
/// Reusable data for one `GraphicsWindow` checkbox row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GraphicsToggleSpec {
    /// Original LDL object name.
    ldl_name: &'static str,
    /// Accessible name and stable option label stem.
    accessible_name: &'static str,
    /// Original row bounds.
    rect: MenuRect,
    /// Modeled option toggled by this row.
    field: ShellOptionField,
    /// Visible label when the option is enabled.
    on_label: &'static str,
    /// Visible label when the option is disabled.
    off_label: &'static str,
    /// Source reference in `graphics.ldl`.
    source: &'static str,
}

impl GraphicsToggleSpec {
    /// Creates a graphics toggle spec from original LDL metadata.
    const fn new(
        ldl_name: &'static str,
        accessible_name: &'static str,
        rect: MenuRect,
        field: ShellOptionField,
        on_label: &'static str,
        off_label: &'static str,
        source: &'static str,
    ) -> Self {
        Self {
            ldl_name,
            accessible_name,
            rect,
            field,
            on_label,
            off_label,
            source,
        }
    }
}

/// Original graphics checkbox rows, in LDL draw order.
const GRAPHICS_TOGGLES: [GraphicsToggleSpec; 8] = [
    GraphicsToggleSpec::new(
        "WalkButton",
        "Unit Animation",
        MenuRect::new(30, 90, GRAPHICS_TOGGLE_SIZE.0, GRAPHICS_TOGGLE_SIZE.1),
        ShellOptionField::UnitAnimation,
        "Unit Animations On",
        "Unit Animations Off",
        "graphics.ldl:57",
    ),
    GraphicsToggleSpec::new(
        "TradeButton",
        "Animated Trade Routes",
        MenuRect::new(30, 120, GRAPHICS_TOGGLE_SIZE.0, GRAPHICS_TOGGLE_SIZE.1),
        ShellOptionField::AnimatedTradeRoutes,
        "Animated Trade Routes On",
        "Animated Trade Routes Off",
        "graphics.ldl:66",
    ),
    GraphicsToggleSpec::new(
        "WonderButton",
        "Wonder Movies",
        MenuRect::new(30, 150, GRAPHICS_TOGGLE_SIZE.0, GRAPHICS_TOGGLE_SIZE.1),
        ShellOptionField::WonderMovies,
        "Wonder Movies On",
        "Wonder Movies Off",
        "graphics.ldl:75",
    ),
    GraphicsToggleSpec::new(
        "CityNamesButton",
        "City Names",
        MenuRect::new(30, 180, GRAPHICS_TOGGLE_SIZE.0, GRAPHICS_TOGGLE_SIZE.1),
        ShellOptionField::CityNames,
        "City Names On",
        "City Names Off",
        "graphics.ldl:84",
    ),
    GraphicsToggleSpec::new(
        "PoliticalBordersButton",
        "Political Borders",
        MenuRect::new(350, 90, GRAPHICS_TOGGLE_SIZE.0, GRAPHICS_TOGGLE_SIZE.1),
        ShellOptionField::PoliticalBorders,
        "Show Political Borders",
        "Hide Political Borders",
        "graphics.ldl:96",
    ),
    GraphicsToggleSpec::new(
        "TradeRoutesButton",
        "Trade Routes",
        MenuRect::new(350, 120, GRAPHICS_TOGGLE_SIZE.0, GRAPHICS_TOGGLE_SIZE.1),
        ShellOptionField::TradeRoutes,
        "Show Trade Routes",
        "Hide Trade Routes",
        "graphics.ldl:105",
    ),
    GraphicsToggleSpec::new(
        "CityInflenceButton",
        "City Influence",
        MenuRect::new(350, 150, GRAPHICS_TOGGLE_SIZE.0, GRAPHICS_TOGGLE_SIZE.1),
        ShellOptionField::CityInfluence,
        "Show City Influence",
        "Hide City Influence",
        "graphics.ldl:114",
    ),
    GraphicsToggleSpec::new(
        "GridButton",
        "Grid",
        MenuRect::new(350, 180, GRAPHICS_TOGGLE_SIZE.0, GRAPHICS_TOGGLE_SIZE.1),
        ShellOptionField::Grid,
        "Map Grid Lines On",
        "Map Grid Lines Off",
        "graphics.ldl:123",
    ),
];
/// Minimum value shared by the original 0-10 sound sliders.
const SOUND_SLIDER_MIN: i32 = 0;
/// Maximum value shared by the original 0-10 sound sliders.
const SOUND_SLIDER_MAX: i32 = 10;
/// Reusable data for one volume slider group in `SoundWindow`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SoundSliderGroupSpec {
    /// LDL object name for the group title label.
    title_ldl_name: &'static str,
    /// Visible group title label.
    title_label: &'static str,
    /// Accessible name for the group title label.
    title_accessible_name: &'static str,
    /// Original group title bounds.
    title_rect: MenuRect,
    /// Source reference for the group title.
    title_source: &'static str,
    /// LDL object name for the minimum endpoint label.
    min_ldl_name: &'static str,
    /// Accessible name for the minimum endpoint label.
    min_accessible_name: &'static str,
    /// Original minimum endpoint bounds.
    min_rect: MenuRect,
    /// LDL object name for the maximum endpoint label.
    max_ldl_name: &'static str,
    /// Accessible name for the maximum endpoint label.
    max_accessible_name: &'static str,
    /// Original maximum endpoint bounds.
    max_rect: MenuRect,
    /// LDL object name for the slider control.
    slider_ldl_name: &'static str,
    /// Visible/accessibility label for the slider control.
    slider_name: &'static str,
    /// Original slider bounds.
    slider_rect: MenuRect,
    /// Modeled option field controlled by the slider.
    field: ShellOptionField,
    /// Source reference for the slider and its endpoint labels.
    slider_source: &'static str,
}

/// Original volume slider groups, preserving the historical LDL object names.
const SOUND_SLIDER_GROUPS: [SoundSliderGroupSpec; 3] = [
    SoundSliderGroupSpec {
        title_ldl_name: "SFXName",
        title_label: "Sound Volume",
        title_accessible_name: "SFX Volume Label",
        title_rect: MenuRect::new(95, 40, 160, 20),
        title_source: "sound.ldl:275",
        min_ldl_name: "SFXMin",
        min_accessible_name: "SFX Volume Minimum",
        min_rect: MenuRect::new(75, 70, 15, 20),
        max_ldl_name: "SFXMax",
        max_accessible_name: "SFX Volume Maximum",
        max_rect: MenuRect::new(260, 70, 20, 20),
        slider_ldl_name: "SFXSlider",
        slider_name: "SFX Volume",
        slider_rect: MenuRect::new(95, 60, 160, 40),
        field: ShellOptionField::SfxVolume,
        slider_source: "sound.ldl:91",
    },
    SoundSliderGroupSpec {
        title_ldl_name: "MusicName",
        title_label: "Voice Volume",
        title_accessible_name: "Voice Volume Label",
        title_rect: MenuRect::new(95, 100, 160, 20),
        title_source: "sound.ldl:322",
        min_ldl_name: "VoiceMin",
        min_accessible_name: "Voice Volume Minimum",
        min_rect: MenuRect::new(75, 130, 15, 20),
        max_ldl_name: "VoiceMax",
        max_accessible_name: "Voice Volume Maximum",
        max_rect: MenuRect::new(260, 130, 20, 20),
        slider_ldl_name: "VoiceSlider",
        slider_name: "Voice Volume",
        slider_rect: MenuRect::new(95, 120, 160, 40),
        field: ShellOptionField::VoiceVolume,
        slider_source: "sound.ldl:207",
    },
    SoundSliderGroupSpec {
        title_ldl_name: "VoiceName",
        title_label: "Music Volume",
        title_accessible_name: "Music Volume Label",
        title_rect: MenuRect::new(95, 160, 160, 20),
        title_source: "sound.ldl:370",
        min_ldl_name: "MusicMin",
        min_accessible_name: "Music Volume Minimum",
        min_rect: MenuRect::new(75, 190, 15, 20),
        max_ldl_name: "MusicMax",
        max_accessible_name: "Music Volume Maximum",
        max_rect: MenuRect::new(260, 190, 20, 20),
        slider_ldl_name: "MusicSlider",
        slider_name: "Music Volume",
        slider_rect: MenuRect::new(95, 180, 160, 40),
        field: ShellOptionField::MusicVolume,
        slider_source: "sound.ldl:149",
    },
];
/// Original checkbox size in `MusicScreen`.
const MUSIC_TOGGLE_SIZE: (u32, u32) = (146, 24);
/// Reusable data for one `MusicScreen` toggle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MusicToggleSpec {
    /// Original LDL object name.
    ldl_name: &'static str,
    /// Stable accessible label stem.
    accessible_name: &'static str,
    /// Original row bounds.
    rect: MenuRect,
    /// Modeled option toggled by this row.
    field: ShellOptionField,
    /// Visible label when selected.
    on_label: &'static str,
    /// Visible label when unselected.
    off_label: &'static str,
    /// Source reference in `musicscreen.ldl`.
    source: &'static str,
}

/// Original music toggle rows, in LDL order.
const MUSIC_TOGGLES: [MusicToggleSpec; 3] = [
    MusicToggleSpec {
        ldl_name: "AutoRepeatSwitch",
        accessible_name: "Auto Repeat",
        rect: MenuRect::new(57, 100, MUSIC_TOGGLE_SIZE.0, MUSIC_TOGGLE_SIZE.1),
        field: ShellOptionField::AutoRepeat,
        on_label: "Auto Repeat On",
        off_label: "Auto Repeat Off",
        source: "musicscreen.ldl:67",
    },
    MusicToggleSpec {
        ldl_name: "RandomOrderSwitch",
        accessible_name: "Random Order",
        rect: MenuRect::new(57, 130, MUSIC_TOGGLE_SIZE.0, MUSIC_TOGGLE_SIZE.1),
        field: ShellOptionField::RandomOrder,
        on_label: "Random Order On",
        off_label: "Random Order Off",
        source: "musicscreen.ldl:70",
    },
    MusicToggleSpec {
        ldl_name: "MusicOnSwitch",
        accessible_name: "Music On",
        rect: MenuRect::new(57, 160, MUSIC_TOGGLE_SIZE.0, MUSIC_TOGGLE_SIZE.1),
        field: ShellOptionField::MusicEnabled,
        on_label: "Music On",
        off_label: "Music Off",
        source: "musicscreen.ldl:73",
    },
];

/// Builds the root options popup.
pub(super) fn options_screen() -> ShellScreen {
    let mut controls = vec![
        label(
            "Name",
            "OPTIONS",
            "Options Title",
            0,
            20,
            380,
            23,
            "options.ldl:42",
        ),
        label(
            "ConfigHeader",
            "CONFIG",
            "Config Header",
            57,
            40,
            89,
            20,
            "options.ldl:75",
        ),
    ];
    append_button_specs(&mut controls, &OPTIONS_CONFIG_BUTTONS);
    controls.push(label(
        "GameHeader",
        "GAME",
        "Game Header",
        240,
        40,
        89,
        20,
        "options.ldl:143",
    ));
    append_button_specs(&mut controls, &OPTIONS_DISABLED_GAME_BUTTONS);
    controls.push(popup_close_button(380, 365));

    ShellScreen {
        id: ShellScreenId::Options,
        ldl_name: "OptionsWindow",
        rect: MenuRect::new(210, 118, 380, 365),
        background: None,
        pattern: Some(asset("uptg20e.rim")),
        controls,
    }
}

/// Builds graphics options popup.
pub(super) fn graphics_screen(options: &ShellOptionsState) -> ShellScreen {
    let mut controls = vec![
        label(
            "Name",
            "GRAPHICS",
            "Graphics Title",
            170,
            20,
            340,
            20,
            "graphics.ldl:26",
        ),
        slider(
            "UnitSpeedSlider",
            "Unit Speed",
            "Unit Speed",
            190,
            50,
            300,
            40,
            0,
            9,
            options
                .numeric(ShellOptionField::UnitSpeed)
                .unwrap_or(5_i32),
            ShellOptionField::UnitSpeed,
            "graphics.ldl:143",
        ),
        label(
            "UnitSpeedName",
            "Unit Speed:",
            "Unit Speed Label",
            30,
            60,
            140,
            20,
            "graphics.ldl:180",
        ),
    ];
    append_graphics_toggles(&mut controls, options);
    controls.extend([
        button(
            "ResolutionButton",
            "Screen Size",
            "Change Resolution",
            265,
            225,
            150,
            21,
            ButtonVisualStyle::blue_large(),
            ShellAction::OpenOption(ShellOptionField::Resolution),
            "graphics.ldl:134",
        ),
        popup_close_button(680, 290),
    ]);

    ShellScreen {
        id: ShellScreenId::Graphics,
        ldl_name: "GraphicsWindow",
        rect: MenuRect::new(60, 155, 680, 290),
        background: None,
        pattern: Some(asset("uptg20e.rim")),
        controls,
    }
}

/// Appends all original `GraphicsWindow` checkbox rows.
fn append_graphics_toggles(controls: &mut Vec<ShellControl>, options: &ShellOptionsState) {
    controls.extend(
        GRAPHICS_TOGGLES
            .iter()
            .map(|spec| graphics_toggle_control(spec, options)),
    );
}

/// Builds one graphics checkbox row from reusable LDL metadata.
fn graphics_toggle_control(spec: &GraphicsToggleSpec, options: &ShellOptionsState) -> ShellControl {
    let enabled = options.bool_option(spec.field);
    image_switch_control(
        spec.ldl_name,
        graphics_toggle_label(enabled, spec.on_label, spec.off_label),
        spec.accessible_name,
        spec.rect.x,
        spec.rect.y,
        spec.rect.width,
        spec.rect.height,
        enabled,
        ShellImageSet::graphics_checkbox(),
        ShellAction::ToggleOption(spec.field),
        spec.source,
    )
}

/// Returns the visible stringtable-like label for one graphics toggle.
const fn graphics_toggle_label(
    enabled: bool,
    on_label: &'static str,
    off_label: &'static str,
) -> &'static str {
    if enabled { on_label } else { off_label }
}

/// Builds sound options popup.
pub(super) fn sound_screen(options: &ShellOptionsState) -> ShellScreen {
    let mut controls = vec![label(
        "Name",
        "SOUND",
        "Sound Title",
        88,
        20,
        175,
        20,
        "sound.ldl:41",
    )];
    append_sound_slider_groups(&mut controls, options);
    controls.push(popup_close_button(350, 260));

    ShellScreen {
        id: ShellScreenId::Sound,
        ldl_name: "SoundWindow",
        rect: MenuRect::new(225, 170, 350, 260),
        background: None,
        pattern: Some(asset("uptg20e.rim")),
        controls,
    }
}

/// Appends all original sound volume slider groups.
fn append_sound_slider_groups(controls: &mut Vec<ShellControl>, options: &ShellOptionsState) {
    for spec in SOUND_SLIDER_GROUPS {
        append_sound_slider_group(controls, spec, options);
    }
}

/// Appends one title, endpoint labels, and slider using original LDL names.
fn append_sound_slider_group(
    controls: &mut Vec<ShellControl>,
    spec: SoundSliderGroupSpec,
    options: &ShellOptionsState,
) {
    controls.extend([
        label(
            spec.title_ldl_name,
            spec.title_label,
            spec.title_accessible_name,
            spec.title_rect.x,
            spec.title_rect.y,
            spec.title_rect.width,
            spec.title_rect.height,
            spec.title_source,
        ),
        label(
            spec.min_ldl_name,
            "0",
            spec.min_accessible_name,
            spec.min_rect.x,
            spec.min_rect.y,
            spec.min_rect.width,
            spec.min_rect.height,
            spec.slider_source,
        ),
        label(
            spec.max_ldl_name,
            "10",
            spec.max_accessible_name,
            spec.max_rect.x,
            spec.max_rect.y,
            spec.max_rect.width,
            spec.max_rect.height,
            spec.slider_source,
        ),
        slider(
            spec.slider_ldl_name,
            spec.slider_name,
            spec.slider_name,
            spec.slider_rect.x,
            spec.slider_rect.y,
            spec.slider_rect.width,
            spec.slider_rect.height,
            SOUND_SLIDER_MIN,
            SOUND_SLIDER_MAX,
            options.numeric(spec.field).unwrap_or(5_i32),
            spec.field,
            spec.slider_source,
        ),
    ]);
}

/// Builds music options popup.
pub(super) fn music_screen(options: &ShellOptionsState) -> ShellScreen {
    let mut controls = vec![
        label(
            "Name",
            "MUSIC",
            "Music Title",
            65,
            22,
            130,
            20,
            "musicscreen.ldl:39",
        ),
        button(
            "SelectTrackButton",
            "Select Track",
            "Select Track",
            46,
            47,
            146,
            24,
            ButtonVisualStyle::blue_large(),
            ShellAction::OpenOption(ShellOptionField::MusicTrack),
            "musicscreen.ldl:58",
        ),
    ];
    append_music_toggles(&mut controls, options);
    controls.push(popup_close_button(260, 260));

    ShellScreen {
        id: ShellScreenId::Music,
        ldl_name: "MusicScreen",
        rect: MenuRect::new(270, 170, 260, 260),
        background: None,
        pattern: Some(asset("uptg20e.rim")),
        controls,
    }
}

/// Appends all original music option toggles.
fn append_music_toggles(controls: &mut Vec<ShellControl>, options: &ShellOptionsState) {
    controls.extend(
        MUSIC_TOGGLES
            .iter()
            .map(|spec| music_toggle_control(spec, options)),
    );
}

/// Builds one music checkbox row from reusable LDL metadata.
fn music_toggle_control(spec: &MusicToggleSpec, options: &ShellOptionsState) -> ShellControl {
    let enabled = options.bool_option(spec.field);
    image_switch_control(
        spec.ldl_name,
        music_toggle_label(enabled, spec.on_label, spec.off_label),
        spec.accessible_name,
        spec.rect.x,
        spec.rect.y,
        spec.rect.width,
        spec.rect.height,
        enabled,
        ShellImageSet::popup_checkbox(),
        ShellAction::ToggleOption(spec.field),
        spec.source,
    )
}

/// Returns the visible label for one music toggle.
const fn music_toggle_label(
    enabled: bool,
    on_label: &'static str,
    off_label: &'static str,
) -> &'static str {
    if enabled { on_label } else { off_label }
}
