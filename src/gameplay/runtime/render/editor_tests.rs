//! Search selection/caret/composition pixels stay in an in-memory SDL framebuffer.

use super::management::draw_management;
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::model::GameState;
use crate::gameplay::runtime::test_support::with_canvas;
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::ui_model::InGameUiModel;
use crate::gameplay::ui_model::library::editor::Motion;
use crate::gameplay::ui_model::management::{ManagementUi, ManagerKind};
use sdl2::pixels::{Color, PixelFormatEnum};
use std::path::Path;

#[test]
fn native_selection_and_composition_match_editor_positions_without_committing_ime() {
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
        ui.management
            .editor
            .insert(&mut ui.management.library.query, "abcdef");
        ui.management
            .editor
            .move_caret(&ui.management.library.query, Motion::Home, false);
        ui.management
            .editor
            .move_caret(&ui.management.library.query, Motion::Right, true);
        ui.management
            .editor
            .move_caret(&ui.management.library.query, Motion::Right, true);
        canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
        canvas.clear();
        draw_management(canvas, &mut textures, &state, &ui, None)
            .map_err(|error| error.to_string())?;
        let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        if rgb(&pixels, 130_usize, 98_usize) != [0_u8, 0_u8, 128_u8] {
            return Err("actual two-character selection has no native highlight".to_owned());
        }
        ui.management
            .editor
            .move_caret(&ui.management.library.query, Motion::End, false);
        ui.management.editor.compose("xy", 1_i32, 0_i32);
        draw_management(canvas, &mut textures, &state, &ui, None)
            .map_err(|error| error.to_string())?;
        let composed = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        if rgb(&composed, 165_usize, 90_usize) != [0_u8; 3_usize] {
            return Err("uncommitted IME composition is not underlined at the caret".to_owned());
        }
        if ui.management.library.query != "abcdef" || canvas.clip_rect().is_some() {
            return Err("composition changed query data or leaked the control clip".to_owned());
        }
        Ok(())
    });
    assert!(result.is_ok(), "native text-editor pixels: {result:?}");
}

/// Fixed 800x600 fixture coordinates are independent of the painting helpers.
fn rgb(pixels: &[u8], x: usize, y: usize) -> [u8; 3] {
    let index = y
        .saturating_mul(800_usize)
        .saturating_add(x)
        .saturating_mul(4_usize);
    [
        pixels[index],
        pixels[index.saturating_add(1_usize)],
        pixels[index.saturating_add(2_usize)],
    ]
}
