//! Picker screen builders and option lists for the CTP2 shell.

use super::MenuRect;
use super::builders::{asset, decoration, label, list_item};
use super::{GameSetupField, ShellAction, ShellOptionField, ShellScreen, ShellScreenId};

/// Returns setup options for a field.
pub(super) const fn setup_options(field: GameSetupField) -> &'static [&'static str] {
    match field {
        GameSetupField::Nation => NATIONS,
        GameSetupField::Difficulty => DIFFICULTIES,
        GameSetupField::MapSize => MAP_SIZES,
        GameSetupField::WorldShape => WORLD_SHAPES,
        GameSetupField::WorldType => WORLD_TYPES,
        GameSetupField::Players => PLAYER_COUNTS,
        GameSetupField::Rules => RULE_PRESETS,
    }
}

/// Returns the picker screen for a setup field.
pub(super) const fn picker_screen_for(field: GameSetupField) -> ShellScreenId {
    match field {
        GameSetupField::Nation => ShellScreenId::NationPicker,
        GameSetupField::Difficulty => ShellScreenId::DifficultyPicker,
        GameSetupField::MapSize => ShellScreenId::MapSizePicker,
        GameSetupField::WorldShape => ShellScreenId::WorldShapePicker,
        GameSetupField::WorldType => ShellScreenId::WorldTypePicker,
        GameSetupField::Players => ShellScreenId::PlayersPicker,
        GameSetupField::Rules => ShellScreenId::RulesPicker,
    }
}

/// Returns the destination screen for an option field.
pub(super) const fn option_screen_for(field: ShellOptionField) -> ShellScreenId {
    match field {
        ShellOptionField::Resolution => ShellScreenId::Resolution,
        ShellOptionField::MusicTrack => ShellScreenId::MusicTrack,
        ShellOptionField::SfxVolume
        | ShellOptionField::VoiceVolume
        | ShellOptionField::MusicVolume
        | ShellOptionField::UnitSpeed
        | ShellOptionField::MusicEnabled
        | ShellOptionField::AutoRepeat
        | ShellOptionField::RandomOrder => ShellScreenId::Options,
    }
}

/// Builds resolution picker popup.
pub(super) fn resolution_screen() -> ShellScreen {
    picker_from_values(
        ShellScreenId::Resolution,
        "ScreenResScreen",
        "RESOLUTION",
        RESOLUTIONS,
        |index| ShellAction::SelectOption(ShellOptionField::Resolution, index),
    )
}

/// Builds music-track picker popup.
pub(super) fn music_track_screen() -> ShellScreen {
    picker_from_values(
        ShellScreenId::MusicTrack,
        "MusicTrackScreen",
        "SELECT TRACK",
        MUSIC_TRACKS,
        |index| ShellAction::SelectOption(ShellOptionField::MusicTrack, index),
    )
}

/// Builds a game-setup picker popup.
pub(super) fn picker_screen(
    id: ShellScreenId,
    ldl_name: &'static str,
    title: &'static str,
    field: GameSetupField,
) -> ShellScreen {
    picker_from_values(id, ldl_name, title, setup_options(field), |index| {
        ShellAction::SelectGameSetup(field, index)
    })
}

/// Builds a generic picker popup from values.
fn picker_from_values<ActionBuilder>(
    id: ShellScreenId,
    ldl_name: &'static str,
    title: &'static str,
    values: &'static [&'static str],
    action_builder: ActionBuilder,
) -> ShellScreen
where
    ActionBuilder: Fn(usize) -> ShellAction,
{
    let mut controls = vec![label("Name", title, title, 20, 20, 195, 20, "picker")];
    for (index, value) in values.iter().enumerate() {
        let y = 55_i32
            .saturating_add(i32::try_from(index).map_or(0_i32, |row| row.saturating_mul(22_i32)));
        controls.push(list_item(
            "ListItem",
            value,
            value,
            21,
            y,
            177,
            20,
            action_builder(index),
            "picker-list",
        ));
    }
    controls.push(decoration(
        "BackButton",
        "Back",
        80,
        200,
        75,
        20,
        Some(ShellAction::Back),
        "picker-back",
    ));
    ShellScreen {
        id,
        ldl_name,
        rect: MenuRect::new(282, 183, 235, 234),
        background: None,
        pattern: Some(asset("uptg20e.rim")),
        controls,
    }
}

/// Original nation list from `civilisation.txt`, excluding barbarians for player choice.
pub(super) const NATIONS: &[&str] = &[
    "GREEKS",
    "AMERICANS",
    "SCOTTISH",
    "IRISH",
    "ENGLISH",
    "GERMANS",
    "FRENCH",
    "RUSSIANS",
    "MEXICANS",
    "SPANISH",
    "INCAS",
    "INDIANS",
    "JAPANESE",
    "CHINESE",
    "KOREANS",
    "MONGOLS",
    "VIETNAMESE",
    "ARABS",
    "PERSIANS",
    "ISRAELIS",
    "EGYPTIANS",
    "NUBIANS",
    "ZULUS",
    "ETHIOPIANS",
    "ROMANS",
    "VIKINGS",
    "AZTECS",
    "MAYANS",
    "CARTHAGINIANS",
    "BYZANTINES",
    "TURKS",
    "DUTCH",
    "PORTUGUESE",
    "BRAZILIANS",
    "AUSTRALIANS",
    "CANADIANS",
    "THAI",
    "INDONESIANS",
    "POLYNESIANS",
    "BABYLONIANS",
    "SUMERIANS",
];
/// Original difficulty picker values.
const DIFFICULTIES: &[&str] = &["EASIEST", "EASY", "AVERAGE", "HARD", "HARDEST"];
/// Original map-size values.
const MAP_SIZES: &[&str] = &[
    "SMALL 32x48",
    "MEDIUM 48x64",
    "LARGE 96x64",
    "GIGANTIC 128x96",
];
/// Original world-shape values.
const WORLD_SHAPES: &[&str] = &["EAST WEST WRAP", "NO WRAP", "NORTH SOUTH AND EAST WEST"];
/// Original world-type values.
const WORLD_TYPES: &[&str] = &["RANDOM", "EARTH WORLD", "DOUGHNUT WORLD", "FLAT WORLD"];
/// Supported opponent/player presets before launch.
const PLAYER_COUNTS: &[&str] = &[
    "2 PLAYERS",
    "3 PLAYERS",
    "4 PLAYERS",
    "5 PLAYERS",
    "6 PLAYERS",
    "7 PLAYERS",
    "8 PLAYERS",
];
/// Rule presets surfaced by the original shell rules area.
const RULE_PRESETS: &[&str] = &["DEFAULT", "GENOCIDE", "TRADE", "SIMPLE COMBAT", "POLLUTION"];
/// Resolution choices used by the original resolution picker flow.
pub(super) const RESOLUTIONS: &[&str] = &["640 x 480", "800 x 600", "1024 x 768"];
/// Music tracks listed by `MusicTrackScreen.TrackNames`.
pub(super) const MUSIC_TRACKS: &[&str] = &[
    "MUSIC TRACK 1",
    "MUSIC TRACK 2",
    "MUSIC TRACK 3",
    "MUSIC TRACK 4",
    "MUSIC TRACK 5",
    "MUSIC TRACK 6",
    "MUSIC TRACK 7",
    "MUSIC TRACK 8",
    "MUSIC TRACK 9",
    "MUSIC TRACK 10",
];
