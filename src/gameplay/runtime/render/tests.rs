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

    assert_eq!(
        offsets,
        &[0_i32],
        "TileHighlight uses a single thin path line, not a five-pixel cross stroke"
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
fn path_lines_and_turn_boxes_use_ground_centers_not_the_bottom_of_the_tile_canvas() {
    let mut camera = Camera::for_test_identity();
    for zoom in 1_i32..=3_i32 {
        let segments = movement::planned_path_segments(
            &camera,
            MapPoint::new(2_u16, 2_u16),
            &[MapPoint::new(3_u16, 2_u16)],
        );
        assert_eq!(
            (segments[0].start.x, segments[0].start.y),
            (
                47_i32.saturating_mul(zoom),
                24_i32.saturating_add(144_i32.saturating_mul(zoom))
            ),
            "source path center is image origin+(47,48), not +(47,72)"
        );
        assert_eq!(
            (segments[0].end.x, segments[0].end.y),
            (
                94_i32.saturating_mul(zoom),
                24_i32.saturating_add(168_i32.saturating_mul(zoom))
            ),
            "path ends on the clicked diamond"
        );
        let markers =
            movement::planned_path_turn_markers(&camera, &[MapPoint::new(3_u16, 2_u16)], 1_usize);
        assert_eq!(
            markers[0].rect.width(),
            16_u32.saturating_mul(u32::try_from(zoom).expect("positive zoom")),
            "original turn box scales with map zoom"
        );
        camera.zoom_in();
    }
}

#[test]
fn native_movement_line_is_one_pixel_wide_and_crosses_the_actual_ground_centers() {
    let result = crate::gameplay::runtime::test_support::with_canvas((800_u32,600_u32), |canvas| {
        let mut state = crate::gameplay::model::GameState::new(24_u16,18_u16,1_u8,42_u64);
        let id = state.selected_unit.ok_or_else(|| "selected unit".to_owned())?;
        state.units.retain(|unit|unit.id==id);
        for unit in &mut state.units { unit.position=MapPoint::new(2_u16,2_u16);unit.movement_points=300_i16;unit.max_movement_points=300_i16; }
        for tile in &mut state.tiles { tile.terrain=crate::gameplay::model::TerrainKind::Plains;tile.fog=crate::gameplay::model::FogState::Visible; }
        let mut input = crate::gameplay::runtime::MovementInputState::default();
        crate::gameplay::runtime::begin_or_activate_unit_drag(&mut state,&mut input,MapPoint::new(2_u16,2_u16));
        input.preview_path_to(&state,MapPoint::new(3_u16,3_u16));
        canvas.set_draw_color(Color::RGB(0_u8,0_u8,0_u8));canvas.clear();
        movement::draw_planned_path(canvas,&input,&state,&Camera::for_test_identity(),None).map_err(|error|error.to_string())?;
        let pixels = canvas.read_pixels(None,sdl2::pixels::PixelFormatEnum::RGBA32)?;
        for (x,y,expected) in [(47_usize,175_usize,[0_u8,255_u8,0_u8]),(48_usize,175_usize,[0_u8,0_u8,0_u8]),(47_usize,235_usize,[0_u8,0_u8,0_u8])] {
            let offset=y.saturating_mul(800_usize).saturating_add(x).saturating_mul(4_usize);
            if pixels[offset..offset.saturating_add(3_usize)]!=expected {return Err(format!("path pixel {x},{y} is not source-aligned {expected:?}"));}
        }
        Ok(())
    });
    assert!(result.is_ok(),"native path anchoring/stroke: {result:?}");
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
