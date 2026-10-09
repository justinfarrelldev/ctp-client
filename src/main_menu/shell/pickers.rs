//! Picker screen builders and option lists for the CTP2 shell.

use alloc::vec::Vec;

use super::builders::{
    asset, field_with_style, image_switch_control, label, left_label, list_item, listbox_chrome,
    popup_close_button,
};
use super::options_state::ShellOptionsState;
use super::picker_data::{
    BARBARIAN_RISKS, DIFFICULTIES, MAP_SIZES, MUSIC_TRACKS, NATION_LEADER_NAMES,
    NATION_PICKER_LABELS, NATION_VALUES, PLAYER_COUNTS, RESOLUTIONS, RULE_PRESETS, WORLD_SHAPES,
};
use super::text_field::ShellTextFieldStyle;
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
/// Width reserved by the original `CTP2_LISTBOX` vertical ranger.
pub(super) const LISTBOX_SCROLLBAR_WIDTH: u32 = 15;
/// Visible rows in the 89px `ScreenResScreen.ResList` viewport.
pub(super) const RESOLUTION_VISIBLE_ROWS: usize = 4;
/// Visible rows in the 100px `MusicTrackScreen.TrackList` viewport.
pub(super) const MUSIC_TRACK_VISIBLE_ROWS: usize = 5;

/// Returns setup options for a field.
pub(super) const fn setup_options(field: GameSetupField) -> &'static [&'static str] {
    match field {
        GameSetupField::Nation => NATION_VALUES,
        GameSetupField::Difficulty => DIFFICULTIES,
        GameSetupField::BarbarianRisk => BARBARIAN_RISKS,
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
        GameSetupField::Difficulty | GameSetupField::BarbarianRisk => {
            ShellScreenId::DifficultyPicker
        }
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
        | ShellOptionField::MouseSpeed
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
        | ShellOptionField::RandomOrder
        | ShellOptionField::TutorialAdvice
        | ShellOptionField::AutoCycleTurns
        | ShellOptionField::AutoCycleUnits
        | ShellOptionField::BattleView
        | ShellOptionField::EnemyMoves
        | ShellOptionField::AutoCenter
        | ShellOptionField::AutoTabSelect
        | ShellOptionField::ShowAllBattles
        | ShellOptionField::AutoSave
        | ShellOptionField::LeftHandedMouse => ShellScreenId::Options,
    }
}

/// Builds resolution picker popup.
pub(super) fn resolution_screen(options: &ShellOptionsState) -> ShellScreen {
    picker_from_values(
        ShellScreenId::Resolution,
        "ScreenResScreen",
        "Screen Size",
        MenuRect::new(282, 183, 235, 234),
        55,
        RESOLUTIONS,
        options.option_list_scroll_offset(ShellOptionField::Resolution),
        RESOLUTION_VISIBLE_ROWS,
        Some(options.resolution()),
        |index| ShellAction::SelectOption(ShellOptionField::Resolution, index),
    )
}

/// Builds music-track picker popup.
pub(super) fn music_track_screen(options: &ShellOptionsState) -> ShellScreen {
    let rect = MenuRect::new(282, 203, 235, 194);
    let mut controls = vec![
        label(
            "Name",
            "SELECT TRACK",
            "SELECT TRACK",
            20,
            20,
            195,
            20,
            "musicscreen.ldl:93",
        ),
        listbox_chrome(
            "TrackList",
            "Track List",
            21,
            45,
            177,
            100,
            options.option_list_scroll_offset(ShellOptionField::MusicTrack),
            MUSIC_TRACKS.len(),
            MUSIC_TRACK_VISIBLE_ROWS,
            "musicscreen.ldl:104",
        ),
    ];
    append_list_rows(
        &mut controls,
        "MusicTrackListItem",
        MUSIC_TRACKS,
        21,
        45,
        listbox_content_width(177),
        options.option_list_scroll_offset(ShellOptionField::MusicTrack),
        MUSIC_TRACK_VISIBLE_ROWS,
        selected_music_track_label(options),
        |index| ShellAction::SelectOption(ShellOptionField::MusicTrack, index),
        "musicscreen.ldl:173",
    );
    controls.push(popup_close_button(rect.width, rect.height));
    ShellScreen {
        id: ShellScreenId::MusicTrack,
        ldl_name: "MusicTrackScreen",
        rect,
        background: None,
        pattern: Some(asset("uptg20e.rim")),
        controls,
    }
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
        field_with_style(
            "leadernametextfield",
            setup.leader_name(),
            "Leader Name Field",
            120,
            405,
            180,
            30,
            ShellTextFieldStyle::popup_default(),
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
pub(super) fn difficulty_picker_screen(setup: &GameSetupState) -> ShellScreen {
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
        setup,
        "spnewgamepopups.ldl:381",
    );
    append_radio_column(
        &mut controls,
        "Risk",
        BARBARIAN_RISKS,
        210,
        60,
        GameSetupField::BarbarianRisk,
        setup,
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
pub(super) fn map_size_picker_screen(setup: &GameSetupState) -> ShellScreen {
    radio_popup_screen(
        ShellScreenId::MapSizePicker,
        "SPNewGameMapSizeScreen",
        "Map Size",
        MenuRect::new(200, 205, 400, 190),
        MAP_SIZES,
        125,
        45,
        GameSetupField::MapSize,
        setup,
        "spnewgamepopups.ldl:596",
    )
}

/// Builds the original CTP2 map-shape popup.
pub(super) fn world_shape_picker_screen(setup: &GameSetupState) -> ShellScreen {
    let mut controls = popup_base_controls(
        ShellScreenId::WorldShapePicker,
        "SPNewGameMapShapeScreen",
        "World Shape",
        MenuRect::new(150, 225, 500, 150),
    );
    controls.extend([
        left_label(
            "EWLabel",
            "East and West edges of the map connect.",
            "East West Description",
            190,
            51,
            250,
            42,
            "spnewgamepopups.ldl:675",
        ),
        left_label(
            "DWLabel",
            "East/West and North/South edges of the map connect.",
            "Doughnut Description",
            190,
            84,
            250,
            42,
            "spnewgamepopups.ldl:685",
        ),
        left_label(
            "FWLabel",
            "No wrapping; map edges do not connect.",
            "Flat Description",
            190,
            116,
            250,
            42,
            "spnewgamepopups.ldl:695",
        ),
    ]);
    append_radio_rows(
        &mut controls,
        "MapShape",
        WORLD_SHAPES,
        20,
        &[45_i32, 80_i32, 115_i32],
        GameSetupField::WorldShape,
        setup,
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
pub(super) fn players_picker_screen(setup: &GameSetupState) -> ShellScreen {
    radio_popup_screen(
        ShellScreenId::PlayersPicker,
        "SPNewGamePlayersScreen",
        "Players",
        MenuRect::new(200, 175, 400, 250),
        PLAYER_COUNTS,
        125,
        45,
        GameSetupField::Players,
        setup,
        "spnewgamepopups.ldl:821",
    )
}

/// Builds the original CTP2 rules popup.
pub(super) fn rules_picker_screen(setup: &GameSetupState) -> ShellScreen {
    let mut controls = popup_base_controls(
        ShellScreenId::RulesPicker,
        "SPNewGameRulesScreen",
        "Rules",
        MenuRect::new(275, 220, 250, 160),
    );
    controls.push(image_switch_control(
        "RuleOne",
        setup.rule_label(0),
        setup.rule_label(0),
        50,
        50,
        160,
        24,
        setup.rule_enabled(0),
        ShellImageSet::popup_checkbox(),
        ShellAction::SelectGameSetupInPlace(GameSetupField::Rules, 0),
        "spnewgamepopups.ldl:899",
    ));
    controls.push(image_switch_control(
        "RuleTwo",
        setup.rule_label(1),
        setup.rule_label(1),
        50,
        80,
        160,
        24,
        setup.rule_enabled(1),
        ShellImageSet::popup_checkbox(),
        ShellAction::SelectGameSetupInPlace(GameSetupField::Rules, 1),
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
#[expect(
    clippy::too_many_arguments,
    reason = "Original listbox popup builders carry LDL geometry, scroll, and selection state."
)]
fn picker_from_values<ActionBuilder>(
    id: ShellScreenId,
    ldl_name: &'static str,
    title: &'static str,
    rect: MenuRect,
    first_row_y: i32,
    values: &'static [&'static str],
    scroll_offset: usize,
    visible_rows: usize,
    selected_value: Option<&str>,
    action_builder: ActionBuilder,
) -> ShellScreen
where
    ActionBuilder: Fn(usize) -> ShellAction,
{
    let mut controls = vec![
        label("Name", title, title, 20, 20, 195, 20, "graphics.ldl:185"),
        listbox_chrome(
            "ResList",
            "Resolution List",
            21,
            55,
            177,
            89,
            scroll_offset,
            values.len(),
            visible_rows,
            "graphics.ldl:192",
        ),
    ];
    append_list_rows(
        &mut controls,
        "ScreenResListItem",
        values,
        21,
        first_row_y,
        listbox_content_width(177),
        scroll_offset,
        visible_rows,
        selected_value,
        action_builder,
        "graphics.ldl:223",
    );
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

/// Appends visible list rows using original list item templates.
#[expect(
    clippy::too_many_arguments,
    reason = "Original list row construction carries LDL geometry and scroll metadata."
)]
fn append_list_rows<ActionBuilder>(
    controls: &mut Vec<super::ShellControl>,
    ldl_name: &'static str,
    values: &'static [&'static str],
    x: i32,
    first_row_y: i32,
    width: u32,
    scroll_offset: usize,
    visible_rows: usize,
    selected_value: Option<&str>,
    action_builder: ActionBuilder,
    source: &'static str,
) where
    ActionBuilder: Fn(usize) -> ShellAction,
{
    for (index, value) in values
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_rows)
    {
        let visible_index = index.saturating_sub(scroll_offset);
        let y = first_row_y.saturating_add(
            i32::try_from(visible_index).map_or(0_i32, |row| row.saturating_mul(20_i32)),
        );
        controls.push(list_item(
            ldl_name,
            value,
            value,
            x,
            y,
            width,
            20,
            selected_value.is_some_and(|selected| selected == *value),
            action_builder(index),
            source,
        ));
    }
}

/// Returns the content width left after the original always-visible vertical ranger.
const fn listbox_content_width(width: u32) -> u32 {
    width.saturating_sub(LISTBOX_SCROLLBAR_WIDTH)
}

/// Returns the selected music track label.
fn selected_music_track_label(options: &ShellOptionsState) -> Option<&'static str> {
    options
        .music_track()
        .checked_sub(1_usize)
        .and_then(|index| MUSIC_TRACKS.get(index).copied())
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
    setup: &GameSetupState,
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
        setup,
        source,
    );
    controls.push(popup_close_button(rect.width, rect.height));
    screen_with_controls(id, ldl_name, rect, controls)
}

/// Appends an original popup radio column to a control list.
#[expect(
    clippy::too_many_arguments,
    reason = "Radio columns preserve original LDL geometry plus selected setup state."
)]
fn append_radio_column(
    controls: &mut Vec<super::ShellControl>,
    ldl_prefix: &'static str,
    values: &'static [&'static str],
    x: i32,
    y: i32,
    field: GameSetupField,
    setup: &GameSetupState,
    source: &'static str,
) {
    for (index, value) in values.iter().enumerate() {
        let row_y = y.saturating_add(
            i32::try_from(index).map_or(0_i32, |row| row.saturating_mul(POPUP_RADIO_ROW_STEP)),
        );
        append_radio_control(
            controls, ldl_prefix, value, x, row_y, field, index, setup, source,
        );
    }
}

/// Appends popup radio controls at explicit LDL row positions.
#[expect(
    clippy::too_many_arguments,
    reason = "Explicit row radio helpers preserve original LDL geometry."
)]
fn append_radio_rows(
    controls: &mut Vec<super::ShellControl>,
    ldl_prefix: &'static str,
    values: &'static [&'static str],
    x: i32,
    row_y_values: &[i32],
    field: GameSetupField,
    setup: &GameSetupState,
    source: &'static str,
) {
    for (index, value) in values.iter().enumerate() {
        let row_y = row_y_values
            .get(index)
            .copied()
            .map_or(0_i32, |row_y| row_y);
        append_radio_control(
            controls, ldl_prefix, value, x, row_y, field, index, setup, source,
        );
    }
}

/// Appends one selectable original popup radio control.
#[expect(
    clippy::too_many_arguments,
    reason = "Original radio controls require geometry, selected state, and action metadata."
)]
fn append_radio_control(
    controls: &mut Vec<super::ShellControl>,
    ldl_prefix: &'static str,
    value: &'static str,
    x: i32,
    y: i32,
    field: GameSetupField,
    index: usize,
    setup: &GameSetupState,
    source: &'static str,
) {
    controls.push(image_switch_control(
        ldl_prefix,
        value,
        value,
        x,
        y,
        146,
        24,
        setup.selected(field) == value,
        ShellImageSet::popup_checkbox(),
        ShellAction::SelectGameSetup(field, index),
        source,
    ));
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
