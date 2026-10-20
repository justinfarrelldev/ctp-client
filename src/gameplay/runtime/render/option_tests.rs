//! In-memory SDL option-art regressions at independently specified source pixels.

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
use std::path::Path;

#[test]
fn native_option_thumbs_and_tick_pairs_match_source_and_do_not_erase_popup_corners() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let state = GameState::from_server_projection(&ServerProjection::default(), 1_u8, 1_u64);
        let mut ui = InGameUiModel {
            management: ManagementUi::new(
                LayoutBook::load(&root).map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?,
            ..InGameUiModel::default()
        };
        for (kind, positions) in [
            (
                ManagerKind::Sound,
                vec![
                    (396_usize, 230_usize),
                    (396_usize, 290_usize),
                    (396_usize, 350_usize),
                ],
            ),
            (
                ManagerKind::Graphics,
                vec![(412_usize, 150_usize), (541_usize, 190_usize)],
            ),
        ] {
            ui.management.open(kind, &state, None);
            canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
            canvas.clear();
            draw_management(canvas, &mut textures, &state, &ui, None)
                .map_err(|error| error.to_string())?;
            let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
            let thumb = load_rim_rgba_entry(
                &root,
                PICTURE_ARCHIVE,
                "upba2513.rim",
                Some((248_u8, 0_u8, 248_u8)),
            )
            .map_err(|error| error.to_string())?;
            for (left, top) in positions {
                for y in 0_usize..40_usize {
                    for x in 0_usize..9_usize {
                        let source = y
                            .saturating_mul(thumb.pitch)
                            .saturating_add(x.saturating_mul(4_usize));
                        let dest = top
                            .saturating_add(y)
                            .saturating_mul(800_usize)
                            .saturating_add(left.saturating_add(x))
                            .saturating_mul(4_usize);
                        if thumb.data[source.saturating_add(3_usize)] == 255_u8
                            && pixels[dest..dest.saturating_add(3_usize)]
                                != thumb.data[source..source.saturating_add(3_usize)]
                        {
                            return Err(format!("{kind:?}: source thumb at {left},{top}+{x},{y}"));
                        }
                    }
                }
            }
            if kind == ManagerKind::Sound {
                let corner = load_rim_rgba_entry(&root, PICTURE_ARCHIVE, "uptg03a.rim", None)
                    .map_err(|error| error.to_string())?;
                let dest = 172_usize
                    .saturating_mul(800_usize)
                    .saturating_add(227_usize)
                    .saturating_mul(4_usize);
                let source = 2_usize.saturating_mul(corner.pitch).saturating_add(8_usize);
                if pixels[dest..dest.saturating_add(3_usize)]
                    != corner.data[source..source.saturating_add(3_usize)]
                {
                    return Err("obsolete Background erased Sound's gold corner".to_owned());
                }
                // Source tick at x = 4, y = 6; its raised/lowered halves must
                // differ from untouched pattern above and below the groove.
                let pixel = |x: usize, y: usize| {
                    let start = y
                        .saturating_mul(800_usize)
                        .saturating_add(x)
                        .saturating_mul(4_usize);
                    &pixels[start..start.saturating_add(3_usize)]
                };
                if pixel(324_usize, 236_usize) == pixel(325_usize, 238_usize)
                    || pixel(324_usize, 258_usize) == pixel(325_usize, 260_usize)
                {
                    return Err("C3Slider is missing its paired beveled ticks".to_owned());
                }
            }
        }
        Ok(())
    });
    assert!(result.is_ok(), "source option artwork: {result:?}");
}
