//! Picker screen builders and option lists for the CTP2 shell.

use alloc::vec::Vec;

use super::builders::{asset, field, image_switch_control, label, list_item, popup_close_button};
use super::{
    GameSetupField, GameSetupState, MenuRect, ShellAction, ShellImageSet, ShellOptionField,
    ShellScreen, ShellScreenId,
};

/// X positions of the four original tribe-radio columns.
const TRIBE_COLUMN_X: [i32; 4] = [25, 176, 327, 478];
/// Number of original tribe rows per column.
const TRIBE_ROWS_PER_COLUMN: usize = 11;
/// Vertical spacing between original popup radio rows.
const POPUP_RADIO_ROW_STEP: i32 = 30;

/// Returns setup options for a field.
pub(super) const fn setup_options(field: GameSetupField) -> &'static [&'static str] {
    match field {
        GameSetupField::Nation => NATION_VALUES,
        GameSetupField::Difficulty => DIFFICULTIES,
        GameSetupField::MapSize => MAP_SIZES,
        GameSetupField::WorldShape => WORLD_SHAPES,
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
        | ShellOptionField::UnitAnimation
        | ShellOptionField::AnimatedTradeRoutes
        | ShellOptionField::WonderMovies
        | ShellOptionField::CityNames
        | ShellOptionField::PoliticalBorders
        | ShellOptionField::TradeRoutes
        | ShellOptionField::CityInfluence
        | ShellOptionField::Grid
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
        "Screen Size",
        MenuRect::new(282, 183, 235, 234),
        55,
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
        MenuRect::new(282, 203, 235, 194),
        45,
        MUSIC_TRACKS,
        |index| ShellAction::SelectOption(ShellOptionField::MusicTrack, index),
    )
}

/// Builds the original CTP2 tribe popup.
pub(super) fn nation_picker_screen(setup: &GameSetupState) -> ShellScreen {
    let mut controls = vec![label(
        "Name",
        "Empire",
        "Tribe Title",
        162,
        20,
        325,
        20,
        "spnewgamepopups.ldl:168",
    )];

    for (index, nation) in NATION_PICKER_LABELS.iter().enumerate() {
        let column = index
            .checked_div(TRIBE_ROWS_PER_COLUMN)
            .map_or(0_usize, |value| value);
        let row = index
            .checked_rem(TRIBE_ROWS_PER_COLUMN)
            .map_or(0_usize, |value| value);
        let x = TRIBE_COLUMN_X
            .get(column)
            .copied()
            .map_or(25_i32, |value| value);
        let y = 45_i32.saturating_add(i32::try_from(row).map_or(0_i32, |row_index| {
            row_index.saturating_mul(POPUP_RADIO_ROW_STEP)
        }));
        controls.push(image_switch_control(
            tribe_ldl_name(column),
            nation,
            nation,
            x,
            y,
            146,
            24,
            NATION_VALUES
                .get(index)
                .is_some_and(|selected| *selected == setup.selected(GameSetupField::Nation)),
            ShellImageSet::popup_checkbox(),
            ShellAction::SelectGameSetupInPlace(GameSetupField::Nation, index),
            "spnewgamepopups.ldl:204",
        ));
    }

    controls.extend([
        label(
            "leadernamestatic",
            "LEADER NAME:",
            "Leader Name",
            20,
            403,
            100,
            20,
            "spnewgamepopups.ldl:228",
        ),
        field(
            "leadernametextfield",
            setup.leader_name(),
            "Leader Name Field",
            120,
            405,
            180,
            30,
            "spnewgamepopups.ldl:241",
        ),
        image_switch_control(
            "maleradio",
            "MALE",
            "Male",
            327,
            399,
            146,
            24,
            true,
            ShellImageSet::popup_checkbox(),
            ShellAction::Disabled,
            "spnewgamepopups.ldl:263",
        ),
        image_switch_control(
            "femaleradio",
            "FEMALE",
            "Female",
            478,
            399,
            146,
            24,
            false,
            ShellImageSet::popup_checkbox(),
            ShellAction::Disabled,
            "spnewgamepopups.ldl:279",
        ),
    ]);
    controls.push(popup_close_button(650, 465));

    ShellScreen {
        id: ShellScreenId::NationPicker,
        ldl_name: "SPNewGameTribeScreen",
        rect: MenuRect::new(75, 67, 650, 465),
        background: None,
        pattern: Some(asset("uptg20e.rim")),
        controls,
    }
}

/// Builds the original CTP2 difficulty popup.
pub(super) fn difficulty_picker_screen() -> ShellScreen {
    let mut controls = popup_base_controls(
        ShellScreenId::DifficultyPicker,
        "SPNewGameDiffScreen",
        "Difficulty",
        MenuRect::new(200, 165, 400, 270),
    );
    controls.extend([
        label(
            "SkillLevel",
            "Skill Level",
            "Skill Level",
            30,
            35,
            170,
            20,
            "spnewgamepopups.ldl:331",
        ),
        label(
            "BarbarianThreat",
            "Barbarian Threat",
            "Barbarian Threat",
            200,
            35,
            170,
            20,
            "spnewgamepopups.ldl:347",
        ),
    ]);
    append_radio_column(
        &mut controls,
        "Diff",
        DIFFICULTIES,
        40,
        60,
        GameSetupField::Difficulty,
        "spnewgamepopups.ldl:381",
    );
    append_disabled_radio_column(
        &mut controls,
        "Risk",
        BARBARIAN_RISKS,
        210,
        60,
        "spnewgamepopups.ldl:428",
    );
    controls.push(popup_close_button(400, 270));
    screen_with_controls(
        ShellScreenId::DifficultyPicker,
        "SPNewGameDiffScreen",
        MenuRect::new(200, 165, 400, 270),
        controls,
    )
}

/// Builds the original CTP2 map-size popup.
pub(super) fn map_size_picker_screen() -> ShellScreen {
    radio_popup_screen(
        ShellScreenId::MapSizePicker,
        "SPNewGameMapSizeScreen",
        "Map Size",
        MenuRect::new(200, 205, 400, 190),
        MAP_SIZES,
        125,
        45,
        GameSetupField::MapSize,
        "spnewgamepopups.ldl:596",
    )
}

/// Builds the original CTP2 map-shape popup.
pub(super) fn world_shape_picker_screen() -> ShellScreen {
    let mut controls = popup_base_controls(
        ShellScreenId::WorldShapePicker,
        "SPNewGameMapShapeScreen",
        "World Shape",
        MenuRect::new(150, 225, 500, 150),
    );
    controls.extend([
        label(
            "EWLabel",
            "East and West edges of the map connect.",
            "East West Description",
            190,
            51,
            250,
            42,
            "spnewgamepopups.ldl:675",
        ),
        label(
            "DWLabel",
            "East/West and North/South edges of the map connect.",
            "Doughnut Description",
            190,
            84,
            250,
            42,
            "spnewgamepopups.ldl:685",
        ),
        label(
            "FWLabel",
            "No wrapping",
            "Flat Description",
            150,
            116,
            250,
            42,
            "spnewgamepopups.ldl:695",
        ),
    ]);
    append_radio_column(
        &mut controls,
        "MapShape",
        WORLD_SHAPES,
        20,
        45,
        GameSetupField::WorldShape,
        "spnewgamepopups.ldl:708",
    );
    controls.push(popup_close_button(500, 150));
    screen_with_controls(
        ShellScreenId::WorldShapePicker,
        "SPNewGameMapShapeScreen",
        MenuRect::new(150, 225, 500, 150),
        controls,
    )
}

/// Builds the original CTP2 player-count popup.
pub(super) fn players_picker_screen() -> ShellScreen {
    radio_popup_screen(
        ShellScreenId::PlayersPicker,
        "SPNewGamePlayersScreen",
        "Players",
        MenuRect::new(200, 175, 400, 250),
        PLAYER_COUNTS,
        125,
        45,
        GameSetupField::Players,
        "spnewgamepopups.ldl:821",
    )
}

/// Builds the original CTP2 rules popup.
pub(super) fn rules_picker_screen() -> ShellScreen {
    let mut controls = popup_base_controls(
        ShellScreenId::RulesPicker,
        "SPNewGameRulesScreen",
        "Rules",
        MenuRect::new(275, 220, 250, 160),
    );
    controls.push(image_switch_control(
        "RuleOne",
        RULE_PRESETS[0],
        RULE_PRESETS[0],
        50,
        50,
        160,
        24,
        false,
        ShellImageSet::popup_checkbox(),
        ShellAction::SelectGameSetup(GameSetupField::Rules, 0),
        "spnewgamepopups.ldl:899",
    ));
    controls.push(image_switch_control(
        "RuleTwo",
        RULE_PRESETS[1],
        RULE_PRESETS[1],
        50,
        80,
        160,
        24,
        false,
        ShellImageSet::popup_checkbox(),
        ShellAction::SelectGameSetup(GameSetupField::Rules, 1),
        "spnewgamepopups.ldl:908",
    ));
    controls.push(popup_close_button(250, 160));
    screen_with_controls(
        ShellScreenId::RulesPicker,
        "SPNewGameRulesScreen",
        MenuRect::new(275, 220, 250, 160),
        controls,
    )
}

/// Builds a generic listbox-style picker popup from values.
fn picker_from_values<ActionBuilder>(
    id: ShellScreenId,
    ldl_name: &'static str,
    title: &'static str,
    rect: MenuRect,
    first_row_y: i32,
    values: &'static [&'static str],
    action_builder: ActionBuilder,
) -> ShellScreen
where
    ActionBuilder: Fn(usize) -> ShellAction,
{
    let mut controls = vec![label("Name", title, title, 20, 20, 195, 20, "picker")];
    for (index, value) in values.iter().enumerate() {
        let y = first_row_y
            .saturating_add(i32::try_from(index).map_or(0_i32, |row| row.saturating_mul(20_i32)));
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
    controls.push(popup_close_button(rect.width, rect.height));
    ShellScreen {
        id,
        ldl_name,
        rect,
        background: None,
        pattern: Some(asset("uptg20e.rim")),
        controls,
    }
}

/// Creates common title controls for popup screens.
fn popup_base_controls(
    _id: ShellScreenId,
    _ldl_name: &'static str,
    title: &'static str,
    rect: MenuRect,
) -> Vec<super::ShellControl> {
    vec![label(
        "Name",
        title,
        title,
        title_x(rect.width),
        20,
        title_width(rect.width),
        20,
        "spnewgamepopups.ldl:title",
    )]
}

/// Computes original-style centered title x for a popup.
fn title_x(width: u32) -> i32 {
    let title_width = title_width(width);
    let remaining = width.saturating_sub(title_width);
    i32::try_from(remaining.checked_div(2_u32).map_or(0_u32, |value| value))
        .map_or(0_i32, |value| value)
}

/// Computes original-style title width for a popup.
fn title_width(width: u32) -> u32 {
    width.checked_div(2_u32).map_or(0_u32, |value| value)
}

/// Builds a standard radio popup screen.
#[expect(
    clippy::too_many_arguments,
    reason = "LDL popup builders keep exact geometry visible at call sites."
)]
fn radio_popup_screen(
    id: ShellScreenId,
    ldl_name: &'static str,
    title: &'static str,
    rect: MenuRect,
    values: &'static [&'static str],
    option_x: i32,
    option_y: i32,
    field: GameSetupField,
    source: &'static str,
) -> ShellScreen {
    let mut controls = popup_base_controls(id, ldl_name, title, rect);
    append_radio_column(
        &mut controls,
        ldl_name,
        values,
        option_x,
        option_y,
        field,
        source,
    );
    controls.push(popup_close_button(rect.width, rect.height));
    screen_with_controls(id, ldl_name, rect, controls)
}

/// Appends an original popup radio column to a control list.
fn append_radio_column(
    controls: &mut Vec<super::ShellControl>,
    ldl_prefix: &'static str,
    values: &'static [&'static str],
    x: i32,
    y: i32,
    field: GameSetupField,
    source: &'static str,
) {
    for (index, value) in values.iter().enumerate() {
        let row_y = y.saturating_add(
            i32::try_from(index).map_or(0_i32, |row| row.saturating_mul(POPUP_RADIO_ROW_STEP)),
        );
        controls.push(image_switch_control(
            ldl_prefix,
            value,
            value,
            x,
            row_y,
            146,
            24,
            false,
            ShellImageSet::popup_checkbox(),
            ShellAction::SelectGameSetup(field, index),
            source,
        ));
    }
}

/// Appends disabled original popup radio controls for non-functional companion columns.
fn append_disabled_radio_column(
    controls: &mut Vec<super::ShellControl>,
    ldl_prefix: &'static str,
    values: &'static [&'static str],
    x: i32,
    y: i32,
    source: &'static str,
) {
    for (index, value) in values.iter().enumerate() {
        let row_y = y.saturating_add(
            i32::try_from(index).map_or(0_i32, |row| row.saturating_mul(POPUP_RADIO_ROW_STEP)),
        );
        controls.push(image_switch_control(
            ldl_prefix,
            value,
            value,
            x,
            row_y,
            146,
            24,
            false,
            ShellImageSet::popup_checkbox(),
            ShellAction::Disabled,
            source,
        ));
    }
}

/// Creates a shell screen with the supplied popup controls.
const fn screen_with_controls(
    id: ShellScreenId,
    ldl_name: &'static str,
    rect: MenuRect,
    controls: Vec<super::ShellControl>,
) -> ShellScreen {
    ShellScreen {
        id,
        ldl_name,
        rect,
        background: None,
        pattern: Some(asset("uptg20e.rim")),
        controls,
    }
}

/// Returns the original LDL name for a tribe column template.
const fn tribe_ldl_name(column: usize) -> &'static str {
    match column {
        0 => "TribeOne",
        1 => "TribeTwo",
        2 => "TribeThree",
        3 => "TribeFour",
        _ => "Tribe",
    }
}

/// Returns the default male leader name for a nation index.
pub(super) fn leader_name_for_nation_index(index: usize) -> Option<&'static str> {
    NATION_LEADER_NAMES.get(index).copied()
}

/// Original nation values from `civilisation.txt`, excluding barbarians.
pub(super) const NATION_VALUES: &[&str] = &[
    "GREEKS",
    "AMERICANS",
    "SCOTS",
    "IRISH",
    "ENGLISH",
    "NATIVE AMERICANS",
    "GERMANS",
    "FRENCH",
    "JAPANESE",
    "CHINESE",
    "RUSSIANS",
    "SPANIARDS",
    "PORTUGUESE",
    "DUTCH",
    "VIKINGS",
    "ROMANS",
    "EGYPTIANS",
    "PERSIANS",
    "TURKS",
    "PHOENICIANS",
    "WELSH",
    "CUBANS",
    "ZULUS",
    "HEBREWS",
    "ASSYRIANS",
    "INDIANS",
    "THAI",
    "POLYNESIANS",
    "INDONESIANS",
    "POLISH",
    "INCANS",
    "MAYANS",
    "CANADIANS",
    "NIGERIANS",
    "KOREANS",
    "BRAZILIANS",
    "AUSTRALIANS",
    "MEXICANS",
    "ETHIOPIANS",
    "MONGOLS",
    "NICARAGUANS",
];

/// Original singular tribe labels shown by `SPNewGameTribeScreen`.
const NATION_PICKER_LABELS: &[&str] = &[
    "Greek",
    "American",
    "Scottish",
    "Irish",
    "English",
    "Native American",
    "German",
    "French",
    "Japanese",
    "Chinese",
    "Russian",
    "Spaniard",
    "Portuguese",
    "Dutch",
    "Viking",
    "Roman",
    "Egyptian",
    "Persian",
    "Turk",
    "Phoenician",
    "Welsh",
    "Cuban",
    "Zulu",
    "Hebrew",
    "Assyrian",
    "Indian",
    "Thai",
    "Polynesian",
    "Indonesian",
    "Polish",
    "Incan",
    "Mayan",
    "Canadian",
    "Nigerian",
    "Korean",
    "Brazilian",
    "Australian",
    "Mexican",
    "Ethiopian",
    "Mongol",
    "Nicaraguan",
];

/// Default male leader names from `civilisation.txt` + `civ_str.txt`.
const NATION_LEADER_NAMES: &[&str] = &[
    "Alexander the Great",
    "George Washington",
    "William Wallace",
    "Brian Boru",
    "Winston Churchill",
    "Sitting Bull",
    "Frederick",
    "Napoleon Bonaparte",
    "Minamoto",
    "Sun Tzu",
    "Ivan The Terrible",
    "Ferdinand",
    "Vasco da Gama",
    "Willem I",
    "Sweyn Forkbeard",
    "Julius Caesar",
    "Ramses",
    "Cyrus the Great",
    "Osman I",
    "Hiram",
    "Owain Glyndwr",
    "Jose Marti",
    "Shaka Zulu",
    "David",
    "Tiglath-pileser III",
    "Mohandas Gandhi",
    "Yot Fa",
    "Kamehameha",
    "Airlangga",
    "Lech Walesa",
    "Viracocha",
    "Hunahpu",
    "Pierre Trudeau",
    "Azikiwe",
    "Yi Songgye",
    "Manuel Deodoro da Fonseca",
    "Gough Whitlam",
    "Santa Anna",
    "Haile Selassie I",
    "Genghis Khan",
    "Moctezuma II",
];
/// Original difficulty picker values.
const DIFFICULTIES: &[&str] = &[
    "Beginner",
    "Easy",
    "Medium",
    "Hard",
    "Very Hard",
    "Impossible",
];
/// Original barbarian risk companion column values instantiated by source.
const BARBARIAN_RISKS: &[&str] = &["Ruins Only", "Bandits", "Raiders", "Marauders"];
/// Original map-size values.
const MAP_SIZES: &[&str] = &["Small", "Regular", "Huge", "Gigantic"];
/// Original world-shape values.
const WORLD_SHAPES: &[&str] = &["Earth World", "Doughnut World", "Flat World"];
/// Supported opponent/player presets before launch.
const PLAYER_COUNTS: &[&str] = &["3", "4", "5", "6", "7", "8"];
/// Rule toggles surfaced by the original shell rules popup.
const RULE_PRESETS: &[&str] = &["Bloodlust - Off", "Pollution - Off"];
/// Resolution choices used by the original resolution picker flow.
pub(super) const RESOLUTIONS: &[&str] = &["640 x 480", "800 x 600", "1024 x 768"];
/// Music tracks listed by `MusicTrackScreen.TrackNames`.
pub(super) const MUSIC_TRACKS: &[&str] = &[
    "A Fortiori",
    "Pacari Tampu",
    "Amaethon",
    "A'tugewinu",
    "Summum Bonum",
    "Casus Belli",
    "Ankh Djet",
    "Shaolin San",
    "Quiet Outback",
    "Kali Dance",
];
