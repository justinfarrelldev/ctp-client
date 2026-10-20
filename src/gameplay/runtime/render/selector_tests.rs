//! Detached city controls preserve installed five-piece artwork and native corners.

use super::management::draw_management;
use crate::gameplay::assets::{PICTURE_ARCHIVE, layout::LayoutBook, rim::load_rim_rgba_entry};
use crate::gameplay::model::GameState;
use crate::gameplay::runtime::{test_support::with_canvas, textures::GameplayTextures};
use crate::gameplay::server::ServerProjection;
use crate::gameplay::ui_model::{
    InGameUiModel,
    management::{ManagementUi, ManagerKind},
};
use sdl2::pixels::{Color, PixelFormatEnum};
use serde_json::json;
use std::path::Path;

#[test]
fn generic_city_arrow_uses_declared_always_or_disabled_layer_not_hover_as_normal() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let mut ui = InGameUiModel {
            management: ManagementUi::new(
                LayoutBook::load(&root).map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?,
            ..InGameUiModel::default()
        };
        for (count, art) in [(1_usize, "upsp01ax.rim"), (2_usize, "upsp01au.rim")] {
            let mut projection:ServerProjection=serde_json::from_value(json!({"revision":9_i64,"turn":3_u32,"tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"TERRAIN_PLAINS"},{"id":"b","x":2_u16,"y":1_u16,"terrain":"TERRAIN_PLAINS"}],"cities":[{"id":"first","owner_id":"human","tile_id":"a","population":1_u32}]})).map_err(|error|error.to_string())?;
            if count == 2_usize {
                projection.cities.push(
                    serde_json::from_value(
                        json!({"id":"second","owner_id":"human","tile_id":"b","population":1_u32}),
                    )
                    .map_err(|error| error.to_string())?,
                );
            }
            let state = GameState::from_server_projection(&projection, 1_u8, 1_u64);
            ui.management
                .open(ManagerKind::Build, &state, Some("first"));
            canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
            canvas.clear();
            draw_management(canvas, &mut textures, &state, &ui, None)
                .map_err(|error| error.to_string())?;
            let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
            let image =
                load_rim_rgba_entry(&root, PICTURE_ARCHIVE, art, Some((248_u8, 0_u8, 248_u8)))
                    .map_err(|error| error.to_string())?;
            for y in 0_u32..image.height.min(29_u32) {
                for x in 0_u32..image.width.min(24_u32) {
                    let source = usize::try_from(y)
                        .unwrap_or(0_usize)
                        .saturating_mul(image.pitch)
                        .saturating_add(
                            usize::try_from(x)
                                .unwrap_or(0_usize)
                                .saturating_mul(4_usize),
                        );
                    if image.data[source.saturating_add(3_usize)] == 255_u8 {
                        let target = usize::try_from(145_u32.saturating_add(y))
                            .unwrap_or(0_usize)
                            .saturating_mul(800_usize)
                            .saturating_add(
                                usize::try_from(49_u32.saturating_add(x)).unwrap_or(0_usize),
                            )
                            .saturating_mul(4_usize);
                        if pixels[target..target.saturating_add(3_usize)]
                            != image.data[source..source.saturating_add(3_usize)]
                        {
                            return Err(format!(
                                "source arrow layer differs for {count} cities at {x},{y}"
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    });
    assert!(
        result.is_ok(),
        "source normal/disabled city arrows: {result:?}"
    );
}

#[test]
fn city_dropdown_right_endcap_matches_original_five_piece_source_art() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let projection:ServerProjection=serde_json::from_value(json!({"revision":9_i64,"turn":3_u32,"tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"TERRAIN_PLAINS"}],"cities":[{"id":"first","owner_id":"human","tile_id":"a","population":1_u32}]})).map_err(|error|error.to_string())?;
        let state = GameState::from_server_projection(&projection, 1_u8, 1_u64);
        let mut ui = InGameUiModel {
            management: ManagementUi::new(
                LayoutBook::load(&root).map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?,
            ..InGameUiModel::default()
        };
        ui.management
            .open(ManagerKind::Build, &state, Some("first"));
        canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
        canvas.clear();
        draw_management(canvas, &mut textures, &state, &ui, None)
            .map_err(|error| error.to_string())?;
        let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        let endcap = load_rim_rgba_entry(
            &root,
            PICTURE_ARCHIVE,
            "uppd02du.rim",
            Some((248_u8, 0_u8, 248_u8)),
        )
        .map_err(|error| error.to_string())?;
        // Source Build root (0,108), chooser (49,37), dropdown offset (24,0), width180.
        let left = 253_u32.saturating_sub(endcap.width);
        for y in 0_u32..endcap.height.min(20_u32) {
            for x in 0_u32..endcap.width {
                let source = usize::try_from(y)
                    .unwrap_or(0_usize)
                    .saturating_mul(endcap.pitch)
                    .saturating_add(
                        usize::try_from(x)
                            .unwrap_or(0_usize)
                            .saturating_mul(4_usize),
                    );
                if endcap.data[source.saturating_add(3_usize)] == 255_u8 {
                    let target = usize::try_from(145_u32.saturating_add(y))
                        .unwrap_or(0_usize)
                        .saturating_mul(800_usize)
                        .saturating_add(usize::try_from(left.saturating_add(x)).unwrap_or(0_usize))
                        .saturating_mul(4_usize);
                    if pixels[target..target.saturating_add(3_usize)]
                        != endcap.data[source..source.saturating_add(3_usize)]
                    {
                        return Err(format!("five-part dropdown endcap mismatch {x},{y}"));
                    }
                }
            }
        }
        Ok(())
    });
    assert!(
        result.is_ok(),
        "native source city dropdown art: {result:?}"
    );
}
