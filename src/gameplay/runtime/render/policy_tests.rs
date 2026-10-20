//! Memory-only source policy artwork oracle, independent of descriptor positions.

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
fn original_policy_thumb_and_black_groove_are_drawn_at_literal_source_coordinates() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let projection:ServerProjection=serde_json::from_value(json!({"revision":9_i64,"turn":3_u32,"diplomacy":{"rations":0_i64,"workday":1_i64,"wages":-1_i64,"science_percent":50_i64,"public_works_percent":10_i64},"legal_actions":[{"route":"set-social-policy","fields":["rations","workday","wages"],"payload":{"rations":0_i64,"workday":0_i64,"wages":0_i64}}]})).map_err(|error|error.to_string())?;
        let state = GameState::from_server_projection(&projection, 1_u8, 1_u64);
        let mut ui = InGameUiModel {
            management: ManagementUi::new(
                LayoutBook::load(&root).map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?,
            ..InGameUiModel::default()
        };
        ui.management.open(ManagerKind::Empire, &state, None);
        canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
        canvas.clear();
        draw_management(canvas, &mut textures, &state, &ui, None)
            .map_err(|error| error.to_string())?;
        let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        let thumb = load_rim_rgba_entry(
            &root,
            PICTURE_ARCHIVE,
            "upba2512.rim",
            Some((248_u8, 0_u8, 248_u8)),
        )
        .map_err(|error| error.to_string())?;
        // Full-screen reference root(0,28), tab(35,31), panel(0,24),
        // food(5,50), modify(2,19), slider(20,33),
        // thumb round((152-9)*2/4)=72 => (134,185).
        for y in 0_u32..20_u32 {
            for x in 0_u32..9_u32 {
                let source = usize::try_from(y)
                    .unwrap_or(0_usize)
                    .saturating_mul(thumb.pitch)
                    .saturating_add(
                        usize::try_from(x)
                            .unwrap_or(0_usize)
                            .saturating_mul(4_usize),
                    );
                if thumb.data[source.saturating_add(3_usize)] == 255_u8 {
                    let target = usize::try_from(185_u32.saturating_add(y))
                        .unwrap_or(0_usize)
                        .saturating_mul(800_usize)
                        .saturating_add(
                            usize::try_from(134_u32.saturating_add(x)).unwrap_or(0_usize),
                        )
                        .saturating_mul(4_usize);
                    if pixels[target..target.saturating_add(3_usize)]
                        != thumb.data[source..source.saturating_add(3_usize)]
                    {
                        return Err(format!("native original policy thumb mismatch at {x},{y}"));
                    }
                }
            }
        }
        let center = 193_usize
            .saturating_mul(800_usize)
            .saturating_add(72_usize)
            .saturating_mul(4_usize);
        if pixels[center..center.saturating_add(3_usize)] != [0_u8, 0_u8, 0_u8] {
            return Err("C3Slider black source channel was not drawn".to_owned());
        }
        let pattern = load_rim_rgba_entry(&root, PICTURE_ARCHIVE, "uptg20e.rim", None)
            .map_err(|error| error.to_string())?;
        for (local_x, light) in [(0_u32, false), (195_u32, true)] {
            let source = usize::try_from(200_u32.checked_rem(pattern.height).unwrap_or(0_u32))
                .unwrap_or(0_usize)
                .saturating_mul(pattern.pitch)
                .saturating_add(
                    usize::try_from(local_x.checked_rem(pattern.width).unwrap_or(0_u32))
                        .unwrap_or(0_usize)
                        .saturating_mul(4_usize),
                );
            let target = 333_usize
                .saturating_mul(800_usize)
                .saturating_add(
                    40_usize.saturating_add(usize::try_from(local_x).unwrap_or(0_usize)),
                )
                .saturating_mul(4_usize);
            for (channel, shift, maximum) in [
                (0_usize, 3_u8, 31_u8),
                (1_usize, 2_u8, 63_u8),
                (2_usize, 3_u8, 31_u8),
            ] {
                let packed = (pattern.data[source.saturating_add(channel)] >> shift)
                    .saturating_add(if light { maximum } else { 0_u8 })
                    .div_euclid(2_u8);
                let expected = if shift == 2_u8 {
                    (packed << 2_u8) | (packed >> 4_u8)
                } else {
                    (packed << 3_u8) | (packed >> 2_u8)
                };
                if pixels[target.saturating_add(channel)] != expected {
                    return Err(format!(
                        "missing original column bevel at x{local_x} channel{channel}"
                    ));
                }
            }
        }
        Ok(())
    });
    assert!(result.is_ok(), "native source policy controls: {result:?}");
}
