//! Independent original list-frame corners and discovery colors; memory-only SDL.

use super::management::draw_management;
use crate::gameplay::assets::PICTURE_ARCHIVE;
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::assets::rim::load_rim_rgba_entry;
use crate::gameplay::model::GameState;
use crate::gameplay::runtime::test_support::with_canvas;
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::server::ServerProjection;
use crate::gameplay::ui_model::InGameUiModel;
use crate::gameplay::ui_model::management::{ManagementUi, ManagerKind};
use sdl2::pixels::{Color, PixelFormatEnum};
use serde_json::json;
use std::path::Path;

#[test]
fn source_list_frame_outsets_and_own_discovery_color_match_independent_native_coordinates() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let projection:ServerProjection=serde_json::from_value(json!({"revision":7_i64,"turn":4_u32,"research":{"advances":["ADVANCE_AGRICULTURE"],"target_id":"ADVANCE_BALLISTICS","available":["ADVANCE_BALLISTICS"]}})).map_err(|error|error.to_string())?;
        let state = GameState::from_server_projection(&projection, 1_u8, 1_u64);
        let mut ui = InGameUiModel {
            management: ManagementUi::new(
                LayoutBook::load(&root).map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?,
            ..InGameUiModel::default()
        };
        ui.management.open(ManagerKind::Library, &state, None);
        canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
        canvas.clear();
        draw_management(canvas, &mut textures, &state, &ui, None)
            .map_err(|error| error.to_string())?;
        let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        let corner = load_rim_rgba_entry(
            &root,
            PICTURE_ARCHIVE,
            "uptg06a.rim",
            Some((248_u8, 0_u8, 248_u8)),
        )
        .map_err(|error| error.to_string())?;
        for y in 0_u32..corner.height {
            for x in 0_u32..corner.width {
                let expected = usize::try_from(y)
                    .unwrap_or(0_usize)
                    .saturating_mul(corner.pitch)
                    .saturating_add(
                        usize::try_from(x)
                            .unwrap_or(0_usize)
                            .saturating_mul(4_usize),
                    );
                if corner.data[expected.saturating_add(3_usize)] == 255_u8 {
                    let actual = offset(98_u32.saturating_add(x), 129_u32.saturating_add(y));
                    if pixels[actual..actual.saturating_add(3_usize)]
                        != corner.data[expected..expected.saturating_add(3_usize)]
                    {
                        return Err(format!("list border corner differs at ({x},{y})"));
                    }
                }
            }
        }
        ui.management.open(ManagerKind::Science, &state, None);
        canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
        canvas.clear();
        draw_management(canvas, &mut textures, &state, &ui, None)
            .map_err(|error| error.to_string())?;
        let science = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        let own = offset(500_u32, 302_u32);
        if science[own..own.saturating_add(3_usize)] != [0_u8, 0_u8, 255_u8] {
            return Err(
                "source own-player knowledge block missing at independent column coordinates"
                    .to_owned(),
            );
        }
        let header = offset(500_u32, 286_u32);
        if science[header..header.saturating_add(3_usize)] != [0_u8, 0_u8, 255_u8] {
            return Err("source own-player discovery header color missing".to_owned());
        }
        let foreign = offset(528_u32, 302_u32);
        if science[foreign..foreign.saturating_add(3_usize)] == [0_u8, 0_u8, 255_u8] {
            return Err("foreign knowledge inferred from missing intelligence".to_owned());
        }
        Ok(())
    });
    assert!(result.is_ok(), "native list/knowledge layout: {result:?}");
}

/// Fixed native frame stride, independent of the descriptors under test.
fn offset(x: u32, y: u32) -> usize {
    usize::try_from(y)
        .unwrap_or(0_usize)
        .saturating_mul(800_usize)
        .saturating_add(usize::try_from(x).unwrap_or(0_usize))
        .saturating_mul(4_usize)
}
