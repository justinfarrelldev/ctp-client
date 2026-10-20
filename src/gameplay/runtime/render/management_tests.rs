//! Native SDL manager renders use read-only art and in-memory framebuffer checks.

use super::management::draw_management;
use crate::gameplay::assets::PICTURE_ARCHIVE;
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::assets::rim::load_rim_rgba_entry;
use crate::gameplay::model::GameState;
use crate::gameplay::runtime::test_support::with_canvas;
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::ui_model::InGameUiModel;
use crate::gameplay::ui_model::management::{ManagementUi, ManagerKind};
use sdl2::pixels::{Color, PixelFormatEnum};
use std::path::Path;

#[test]
fn every_manager_renders_without_touching_files_and_uses_its_original_corner() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let state = GameState::new(24, 18, 1, 42);
        let mut ui = InGameUiModel {
            management: ManagementUi::new(
                LayoutBook::load(&root).map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?,
            ..InGameUiModel::default()
        };
        for kind in ManagerKind::ALL {
            ui.management.open(kind, &state, None);
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            draw_management(canvas, &mut textures, &state, &ui, None)
                .map_err(|error| format!("{kind:?}: {error}"))?;
            let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
            if pixels.chunks_exact(4).all(|pixel| pixel[..3] == [0_u8; 3]) {
                return Err(format!("{kind:?} is blank"));
            }
        }
        for (kind, art, x) in [
            (ManagerKind::Build, "uptg25a.rim", 0_usize),
            (ManagerKind::Science, "uptg24a.rim", 224),
        ] {
            ui.management.open(kind, &state, None);
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            draw_management(canvas, &mut textures, &state, &ui, None)
                .map_err(|error| error.to_string())?;
            let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
            let corner = load_rim_rgba_entry(&root, PICTURE_ARCHIVE, art, Some((248, 0, 248)))
                .map_err(|error| error.to_string())?;
            let mut compared = 0_usize;
            for row in 0_usize..10 {
                for column in 0_usize..40 {
                    let source = row
                        .saturating_mul(corner.pitch)
                        .saturating_add(column.saturating_mul(4));
                    if corner.data[source.saturating_add(3)] != 255 {
                        continue;
                    }
                    let destination = row
                        .saturating_add(108)
                        .saturating_mul(800)
                        .saturating_add(x.saturating_add(column))
                        .saturating_mul(4);
                    if pixels[destination..destination.saturating_add(3)]
                        != corner.data[source..source.saturating_add(3)]
                    {
                        return Err(format!("{kind:?} source corner mismatch at {column},{row}"));
                    }
                    compared = compared.saturating_add(1);
                }
            }
            if compared == 0 {
                return Err(format!("no source pixels checked for {kind:?}"));
            }
        }
        Ok(())
    });
    assert!(result.is_ok(), "native source manager art: {result:?}");
}
