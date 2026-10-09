//! Original CTP2 custom-map popup builder.

use alloc::vec::Vec;

use super::builders::{asset, label, popup_ok_button, slider_with_action};
use super::{
    CustomMapField, GameSetupState, MenuRect, ShellAction, ShellControl, ShellScreen, ShellScreenId,
};

/// Custom-map popup width from `custommap.ldl`.
const CUSTOM_MAP_WIDTH: u32 = 450;
/// Custom-map popup height from `custommap.ldl`.
const CUSTOM_MAP_HEIGHT: u32 = 310;
/// Custom-map slider minimum.
const CUSTOM_MAP_MIN: i32 = 0;
/// Custom-map slider maximum.
const CUSTOM_MAP_MAX: i32 = 10;
/// Custom-map slider width from `custommap.ldl`.
const CUSTOM_MAP_SLIDER_WIDTH: u32 = 250;
/// Custom-map slider height from `custommap.ldl`.
const CUSTOM_MAP_SLIDER_HEIGHT: u32 = 40;

/// Builds the original custom-map popup.
pub(super) fn custom_map_screen(setup: &GameSetupState) -> ShellScreen {
    let mut controls = vec![label(
        "Name",
        "Map",
        "Map",
        title_x(CUSTOM_MAP_WIDTH),
        20,
        title_width(CUSTOM_MAP_WIDTH),
        20,
        "custommap.ldl:Name",
    )];
    append_custom_map_slider(
        &mut controls,
        setup,
        "WetDrySlider",
        "Wet",
        "Dry",
        40,
        CustomMapField::WetDry,
    );
    append_custom_map_slider(
        &mut controls,
        setup,
        "WarmColdSlider",
        "Warm",
        "Cold",
        80,
        CustomMapField::WarmCold,
    );
    append_custom_map_slider(
        &mut controls,
        setup,
        "OceanLandSlider",
        "Ocean",
        "Land",
        120,
        CustomMapField::OceanLand,
    );
    append_custom_map_slider(
        &mut controls,
        setup,
        "IslandContinentSlider",
        "Island",
        "Continent",
        160,
        CustomMapField::IslandContinent,
    );
    append_custom_map_slider(
        &mut controls,
        setup,
        "HomoDiverseSlider",
        "Uniform",
        "Diverse",
        200,
        CustomMapField::HomoDiverse,
    );
    append_custom_map_slider(
        &mut controls,
        setup,
        "GoodCountSlider",
        "Few Goods",
        "Many Goods",
        240,
        CustomMapField::GoodCount,
    );
    controls.push(popup_ok_button(
        "BACK",
        "Back",
        CUSTOM_MAP_WIDTH,
        CUSTOM_MAP_HEIGHT,
        ShellAction::Back,
        "custommapscreen.cpp:AddOk",
    ));
    ShellScreen {
        id: ShellScreenId::CustomMap,
        ldl_name: "CustomMapWindow",
        rect: MenuRect::new(175, 145, CUSTOM_MAP_WIDTH, CUSTOM_MAP_HEIGHT),
        background: None,
        pattern: Some(asset("uptg20e.rim")),
        controls,
    }
}

/// Appends one original custom-map slider and its left/right labels.
fn append_custom_map_slider(
    controls: &mut Vec<ShellControl>,
    setup: &GameSetupState,
    ldl_name: &'static str,
    left_label: &'static str,
    right_label: &'static str,
    y: i32,
    field: CustomMapField,
) {
    controls.push(slider_with_action(
        ldl_name,
        ldl_name,
        ldl_name,
        100,
        y,
        CUSTOM_MAP_SLIDER_WIDTH,
        CUSTOM_MAP_SLIDER_HEIGHT,
        CUSTOM_MAP_MIN,
        CUSTOM_MAP_MAX,
        setup.custom_map_value(field),
        ShellAction::AdjustCustomMap(field),
        "custommap.ldl:slider",
    ));
    controls.push(label(
        left_label,
        left_label,
        left_label,
        10,
        y.saturating_add(10),
        80,
        20,
        "custommap.ldl:label-left",
    ));
    controls.push(label(
        right_label,
        right_label,
        right_label,
        360,
        y.saturating_add(10),
        80,
        20,
        "custommap.ldl:label-right",
    ));
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
