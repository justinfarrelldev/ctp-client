//! Native framebuffer pixels must agree with installed LDL artwork placements.
//!
//! Expected colors are read from the installation, not copied into golden files.
//! Every rendered frame and comparison stays in memory.

use crate::gameplay::assets::{PICTURE_ARCHIVE, rim::load_rim_rgba_entry};
use crate::gameplay::runtime::controls::order_rect;
use crate::gameplay::runtime::snapshot::scene::{PreviewScene, preview_state};
use crate::gameplay::runtime::test_support::with_canvas;
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::runtime::{Camera, MovementInputState};
use crate::gameplay::ui_model::InGameUiModel;
use sdl2::pixels::PixelFormatEnum;
use std::path::Path;

#[test]
fn disabling_fog_never_fabricates_unknown_authoritative_terrain() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let mut state = crate::gameplay::model::GameState::from_server_projection(
            &crate::gameplay::server::ServerProjection::default(),
            5_u8,
            42_u64,
        );
        state.filters.fog_of_war = false;
        let mut textures = GameplayTextures::without_assets_for_test();
        canvas.set_draw_color(sdl2::pixels::Color::RGB(255_u8, 0_u8, 255_u8));
        canvas.clear();
        super::draw_map(
            canvas,
            &mut textures,
            &state,
            &Camera::for_test_identity(),
            &MovementInputState::default(),
        )
        .map_err(|error| error.to_string())?;
        let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        let offset = (312_usize * 800_usize + 47_usize) * 4_usize;
        if pixels[offset..offset.saturating_add(3_usize)] != [0_u8; 3_usize] {
            return Err("fog preference revealed fabricated unknown terrain".to_owned());
        }
        Ok(())
    });
    assert!(result.is_ok(), "viewer authority regression: {result:?}");
}

#[test]
fn original_panel_caps_shortcuts_and_all_order_cells_land_at_exact_native_pixels() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let state = preview_state(PreviewScene::Terrain);
        super::draw_frame(
            canvas,
            &mut textures,
            &state,
            &InGameUiModel::default(),
            &Camera::centered_on_selected(&state),
            &MovementInputState::default(),
            None,
        )
        .map_err(|error| error.to_string())?;
        let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        for (name, x, y, source_x, source_y) in [
            ("upcb14u.rim", 762_usize, 476_usize, 18_usize, 40_usize),
            ("upcb05u.rim", 227, 445, 16, 16),
            ("upfg03.rim", 741, 447, 10, 30),
        ] {
            compare_art_pixel(&root, &pixels, name, (x, y), (source_x, source_y))?;
        }
        for index in 0_i32..12_i32 {
            let rect = order_rect(index);
            let name = if matches!(index, 0_i32 | 3_i32 | 4_i32 | 8_i32 | 9_i32 | 11_i32) {
                "upbt02u.rim"
            } else {
                "upbt02x.rim"
            };
            compare_art_pixel(
                &root,
                &pixels,
                name,
                (
                    usize::try_from(rect.x).map_err(|error| error.to_string())?,
                    usize::try_from(rect.y).map_err(|error| error.to_string())?,
                ),
                (1_usize, 15_usize),
            )?;
        }
        Ok(())
    });
    assert!(
        result.is_ok(),
        "native chrome placement regression: {result:?}"
    );
}

#[test]
fn no_selected_unit_draws_only_disabled_order_cells() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let mut state = preview_state(PreviewScene::Terrain);
        state.selected_unit = None;
        super::draw_frame(
            canvas,
            &mut textures,
            &state,
            &InGameUiModel::default(),
            &Camera::centered_on_selected(&state),
            &MovementInputState::default(),
            None,
        )
        .map_err(|error| error.to_string())?;
        let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        for index in 0_i32..12_i32 {
            let rect = order_rect(index);
            compare_art_pixel(
                &root,
                &pixels,
                "upbt02x.rim",
                (
                    usize::try_from(rect.x).map_err(|error| error.to_string())?,
                    usize::try_from(rect.y).map_err(|error| error.to_string())?,
                ),
                (1_usize, 15_usize),
            )?;
        }
        Ok(())
    });
    assert!(result.is_ok(), "disabled order art regression: {result:?}");
}

/// Compares an opaque source pixel against its exact screen-space destination.
fn compare_art_pixel(
    root: &Path,
    pixels: &[u8],
    name: &str,
    position: (usize, usize),
    sample: (usize, usize),
) -> Result<(), String> {
    let source = load_rim_rgba_entry(root, PICTURE_ARCHIVE, name, None)
        .map_err(|error| error.to_string())?;
    let expected_offset = sample
        .1
        .saturating_mul(source.pitch)
        .saturating_add(sample.0.saturating_mul(4_usize));
    let actual_offset = position
        .1
        .saturating_add(sample.1)
        .saturating_mul(800_usize)
        .saturating_add(position.0.saturating_add(sample.0))
        .saturating_mul(4_usize);
    let expected = &source.data[expected_offset..expected_offset.saturating_add(3_usize)];
    let actual = &pixels[actual_offset..actual_offset.saturating_add(3_usize)];
    if actual != expected {
        return Err(format!(
            "{name} at {position:?} sample {sample:?}: {actual:?} != source {expected:?}"
        ));
    }
    Ok(())
}
