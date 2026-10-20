//! Empty Library rangers use source arrow art, not Unicode or host-font substitutes.

use super::management::draw_management;
use crate::gameplay::assets::{PICTURE_ARCHIVE, layout::LayoutBook, rim::load_rim_rgba_entry};
use crate::gameplay::model::GameState;
use crate::gameplay::runtime::{test_support::with_canvas, textures::GameplayTextures};
use crate::gameplay::ui_model::{
    InGameUiModel,
    management::{ManagementUi, ManagerKind},
};
use sdl2::pixels::{Color, PixelFormatEnum};
use std::path::Path;

#[test]
fn empty_index_scrollbar_up_arrow_matches_independent_installed_rim_pixels() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let state = GameState::new(8_u16, 8_u16, 1_u8, 1_u64);
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
        let arrow = load_rim_rgba_entry(
            &root,
            PICTURE_ARCHIVE,
            "upba5615.rim",
            Some((248_u8, 0_u8, 248_u8)),
        )
        .map_err(|error| error.to_string())?;
        let mut checked = 0_usize;
        for row in 0_usize..9_usize {
            for column in 0_usize..9_usize {
                let source = row
                    .saturating_mul(arrow.pitch)
                    .saturating_add(column.saturating_mul(4_usize));
                if arrow.data[source.saturating_add(3_usize)] == 255_u8 {
                    let target = row
                        .saturating_add(137_usize)
                        .saturating_mul(800_usize)
                        .saturating_add(column.saturating_add(255_usize))
                        .saturating_mul(4_usize);
                    if pixels[target..target.saturating_add(3_usize)]
                        != arrow.data[source..source.saturating_add(3_usize)]
                    {
                        return Err(format!("original ranger arrow mismatch {column},{row}"));
                    }
                    checked = checked.saturating_add(1_usize);
                }
            }
        }
        if checked == 0_usize {
            return Err("no opaque source arrow pixels checked".to_owned());
        }
        Ok(())
    });
    assert!(result.is_ok(), "native original ranger artwork: {result:?}");
}
