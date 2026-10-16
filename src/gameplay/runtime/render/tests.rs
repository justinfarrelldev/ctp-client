//! Tests for gameplay rendering geometry helpers.

#![expect(
    clippy::default_numeric_fallback,
    reason = "Geometry assertions use original fixed i32 SDL pixel literals"
)]

use sdl2::pixels::Color;

use super::actors::unit_target_rect;
use super::{
    DROPDOWN_DISABLED_TEXT_COLOR, DROPDOWN_ENABLED_TEXT_COLOR, TOP_MENU_TEXT_COLOR, movement,
};
use crate::gameplay::model::MapPoint;
use crate::gameplay::runtime::Camera;

#[test]
fn unit_sprite_rect_preserves_native_size_and_original_hotspot() {
    let rect = unit_target_rect(96, 96, (48, 60), (100, 200), 1);
    assert_eq!(
        rect,
        sdl2::rect::Rect::new(100, 188, 96, 96),
        "96px frames must not be distorted into 64×72 boxes"
    );
    let zoomed = unit_target_rect(96, 96, (48, 60), (100, 200), 2);
    assert_eq!(
        zoomed,
        sdl2::rect::Rect::new(100, 176, 192, 192),
        "hotspot offsets scale with the whole frame"
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
    let brackets = movement::tile_selection_bracket_segments(&camera, origin);
    let legacy_sprite_brackets =
        movement::selection_bracket_segments(sdl2::rect::Rect::new(100, 200, 64, 72));

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
    assert_ne!(
        brackets[0], legacy_sprite_brackets[0],
        "runtime selection feedback should use CTP2 tile-centered brackets, not a sprite-bounding CTP1-style reticle"
    );
}

#[test]
fn menu_text_colors_keep_readable_contrast_against_original_chrome() {
    let fallback_top_chrome = Color::RGB(200, 192, 174);
    let original_dropdown_chrome = Color::RGB(200, 192, 174);

    assert!(
        luma_contrast(TOP_MENU_TEXT_COLOR, fallback_top_chrome) >= 95_u16,
        "original grey/brown labels must stay dark over the actual pale menu art"
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
