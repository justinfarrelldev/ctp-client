//! Actual SDL output for context overlays; all captures stay in memory.

use super::context::draw_context;
use crate::gameplay::assets::PICTURE_ARCHIVE;
use crate::gameplay::assets::rim::load_rim_rgba_entry;
use crate::gameplay::model::{FogState, GameState, MapPoint};
use crate::gameplay::runtime::test_support::with_canvas;
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::ui_model::{InGameUiModel, context::ContextView};
use sdl2::pixels::{Color, PixelFormatEnum};
use std::path::Path;

#[test]
fn terrain_info_uses_exact_original_gold_corner_and_never_draws_hidden_information() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let mut state = GameState::new(24, 18, 1, 42);
        let point = MapPoint::new(4, 4);
        let index = 100_usize;
        state.tiles[index].fog = FogState::Visible;
        let ui = InGameUiModel {
            map_info: Some(point),
            ..InGameUiModel::default()
        };
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        draw_context(canvas, &mut textures, &state, &ui, None)
            .map_err(|error| error.to_string())?;
        let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        let corner =
            load_rim_rgba_entry(&root, PICTURE_ARCHIVE, "uptg03a.rim", Some((248, 0, 248)))
                .map_err(|error| error.to_string())?;
        let mut checked = 0_usize;
        for y in 0_usize..17_usize {
            for x in 0_usize..17_usize {
                let source = y
                    .saturating_mul(corner.pitch)
                    .saturating_add(x.saturating_mul(4));
                if corner.data[source.saturating_add(3)] != 255 {
                    continue;
                }
                let destination = y
                    .saturating_add(213)
                    .saturating_mul(800)
                    .saturating_add(x.saturating_add(127))
                    .saturating_mul(4);
                if pixels[destination..destination.saturating_add(3)]
                    != corner.data[source..source.saturating_add(3)]
                {
                    return Err(format!("gold corner mismatch at {x},{y}"));
                }
                checked = checked.saturating_add(1);
            }
        }
        if checked == 0 {
            return Err("no gold pixels checked".to_owned());
        }
        state.tiles[index].fog = FogState::Hidden;
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        draw_context(canvas, &mut textures, &state, &ui, None)
            .map_err(|error| error.to_string())?;
        let hidden = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        if hidden.chunks_exact(4).any(|pixel| pixel[..3] != [0_u8; 3]) {
            return Err("hidden map info painted viewer-unknown facts".to_owned());
        }
        Ok(())
    });
    assert!(
        result.is_ok(),
        "terrain dialog pixel regression: {result:?}"
    );
}

#[test]
fn context_rows_have_shared_click_rects_and_visible_selected_row_pixels() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let mut state = GameState::new(24, 18, 1, 42);
        state.popups.clear();
        let point = state.selected_unit().ok_or("starter")?.position;
        let menu = ContextView::new(&state, point, 400, 200).ok_or("menu")?;
        let row = menu.row_rect(1).ok_or("move row")?;
        let ui = InGameUiModel {
            pointer: Some((row.x, row.y)),
            context: Some(menu),
            ..InGameUiModel::default()
        };
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        draw_context(canvas, &mut textures, &state, &ui, None)
            .map_err(|error| error.to_string())?;
        let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        let offset = usize::try_from(row.y)
            .map_err(|error| error.to_string())?
            .saturating_mul(800)
            .saturating_add(usize::try_from(row.x).map_err(|error| error.to_string())?)
            .saturating_mul(4);
        if pixels[offset..offset.saturating_add(3)] != [255_u8, 198_u8, 66_u8] {
            return Err("enabled hovered row not highlighted in actual SDL output".to_owned());
        }
        Ok(())
    });
    assert!(result.is_ok(), "context framebuffer regression: {result:?}");
}
