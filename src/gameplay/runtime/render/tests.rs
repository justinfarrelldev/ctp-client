//! Tests for gameplay rendering geometry helpers.

use sdl2::pixels::Color;

use super::{
    DROPDOWN_DISABLED_TEXT_COLOR, DROPDOWN_ENABLED_TEXT_COLOR, TILE_HEIGHT, TILE_WIDTH,
    TOP_MENU_TEXT_COLOR, movement, unit_target_rect_from_size,
};
use crate::gameplay::model::MapPoint;
use crate::gameplay::runtime::Camera;

#[test]
fn unit_sprite_rect_is_centered_and_bottom_anchored_on_tile() {
    let rect = unit_target_rect_from_size(96, 96, 100, 200);

    assert_eq!(
        rect.x(),
        100_i32 + (TILE_WIDTH - 64_i32).checked_div(2_i32).unwrap_or(0_i32),
        "wide unit sprites should be centered over the 94px tile"
    );
    assert_eq!(
        rect.y()
            .saturating_add(i32::try_from(rect.height()).unwrap_or(0_i32)),
        200_i32 + TILE_HEIGHT,
        "unit sprite feet should anchor to the bottom of the tile grid"
    );
}

#[test]
fn movement_feedback_uses_ctp2_line_bracket_and_turn_box_geometry() {
    let offsets = movement::movement_feedback_stroke_offsets();
    let camera = Camera::for_test_identity();
    let origin = MapPoint::new(2, 2);
    let path = [MapPoint::new(3, 3), MapPoint::new(4, 3)];
    let segments = movement::planned_path_segments(&camera, origin, &path);
    let turn_markers = movement::planned_path_turn_markers(&camera, &path, 1);
    let brackets =
        movement::selection_bracket_segments(unit_target_rect_from_size(64, 72, 100, 200));

    assert!(
        offsets.len() >= 3,
        "path lines should use multiple strokes so they remain visible after viewport scaling"
    );
    assert!(
        offsets.contains(&-1) && offsets.contains(&0) && offsets.contains(&1),
        "line strokes should cover the center line and neighboring pixels"
    );
    assert_eq!(
        segments.len(),
        path.len(),
        "planned movement should render CTP2-style line segments, not one tile marker per reachable tile"
    );
    assert_eq!(
        turn_markers.len(),
        path.len(),
        "one-move ancient units should show CTP2 turn boxes at each completed path step"
    );
    assert_eq!(
        turn_markers[0].label, 1,
        "the first source-style turn box should be labeled with turn one"
    );
    assert!(
        turn_markers[0].rect.width() >= 8,
        "turn boxes should keep the original visible minimum size"
    );
    assert_eq!(
        brackets.len(),
        8,
        "selected units should draw four CTP2-style bracket corners with two line segments each"
    );
    assert!(
        brackets
            .iter()
            .all(|segment| segment.start.x != segment.end.x || segment.start.y != segment.end.y),
        "selection bracket segments should have visible length"
    );
}

#[test]
fn menu_text_colors_keep_readable_contrast_against_original_chrome() {
    let fallback_top_chrome = Color::RGB(126, 113, 81);
    let original_dropdown_chrome = Color::RGB(200, 192, 174);

    assert!(
        luma_contrast(TOP_MENU_TEXT_COLOR, fallback_top_chrome) >= 95_u16,
        "top menu labels should stay bright over dark gold/brown button chrome"
    );
    assert!(
        luma_contrast(DROPDOWN_ENABLED_TEXT_COLOR, original_dropdown_chrome) >= 95_u16,
        "enabled dropdown labels should stay dark over pale menu chrome"
    );
    assert!(
        luma_contrast(DROPDOWN_DISABLED_TEXT_COLOR, original_dropdown_chrome) >= 45_u16,
        "disabled dropdown labels should remain visible without looking enabled"
    );
}

/// Returns approximate WCAG-style luminance contrast distance using integer weights.
fn luma_contrast(foreground: Color, background: Color) -> u16 {
    let foreground_luma = luma(foreground);
    let background_luma = luma(background);
    foreground_luma.abs_diff(background_luma)
}

/// Returns approximate perceived luminance using integer sRGB weights.
fn luma(color: Color) -> u16 {
    let weighted = u32::from(color.r)
        .saturating_mul(299_u32)
        .saturating_add(u32::from(color.g).saturating_mul(587_u32))
        .saturating_add(u32::from(color.b).saturating_mul(114_u32));
    u16::try_from(weighted.checked_div(1_000_u32).unwrap_or(0_u32)).unwrap_or(0_u16)
}
