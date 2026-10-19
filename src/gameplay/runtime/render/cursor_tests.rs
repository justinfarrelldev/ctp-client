//! Software-cursor pixel readback, including transparent black and exact hotspots.

use super::draw_cursor;
use crate::gameplay::assets::pixel::{expand5, expand6};
use crate::gameplay::model::GameState;
use crate::gameplay::runtime::MovementInputState;
use crate::gameplay::runtime::test_support::with_canvas;
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::ui_model::InGameUiModel;
use sdl2::image::LoadSurface;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::surface::Surface;
use std::path::Path;

/// Source art specification: pointer coordinate, retail UC number and hotspot.
type CursorCase = ((i32, i32), u16, (i32, i32));

#[test]
fn original_hand_and_four_edge_cursors_match_retail_pixels_not_os_glyphs() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let mut state = GameState::new(24, 18, 1, 42);
        state.popups.clear();
        let cases: [CursorCase; 5] = [
            ((400, 200), 1, (0, 0)),
            ((400, 0), 10, (23, 0)),
            ((400, 599), 11, (23, 45)),
            ((0, 200), 12, (0, 23)),
            ((799, 200), 13, (45, 22)),
        ];
        for (pointer, number, hotspot) in cases {
            let ui = InGameUiModel {
                pointer: Some(pointer),
                ..InGameUiModel::default()
            };
            canvas.set_draw_color(Color::RGB(11, 22, 33));
            canvas.clear();
            draw_cursor(
                canvas,
                &mut textures,
                &state,
                &ui,
                &MovementInputState::default(),
            )
            .map_err(|error| error.to_string())?;
            let actual = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
            let surface = Surface::from_file(root.join(format!(
                "ctp2_data/default/graphics/cursors/UC{number:03}.tga"
            )))?
            .convert_format(PixelFormatEnum::RGBA32)?;
            let pixels = surface.without_lock().ok_or("cursor pixels unavailable")?;
            let pitch = usize::try_from(surface.pitch()).map_err(|error| error.to_string())?;
            let mut checked = 0_usize;
            for y in 0..surface.height() {
                for x in 0..surface.width() {
                    let screen_x = pointer
                        .0
                        .saturating_sub(hotspot.0)
                        .saturating_add(i32::try_from(x).map_err(|error| error.to_string())?);
                    let screen_y = pointer
                        .1
                        .saturating_sub(hotspot.1)
                        .saturating_add(i32::try_from(y).map_err(|error| error.to_string())?);
                    if !(0_i32..800_i32).contains(&screen_x)
                        || !(0_i32..600_i32).contains(&screen_y)
                    {
                        continue;
                    }
                    let index = usize::try_from(y)
                        .map_err(|error| error.to_string())?
                        .saturating_mul(pitch)
                        .saturating_add(
                            usize::try_from(x)
                                .map_err(|error| error.to_string())?
                                .saturating_mul(4),
                        );
                    let color = [
                        expand5(pixels[index] >> 3),
                        expand6(pixels[index.saturating_add(1)] >> 2),
                        expand5(pixels[index.saturating_add(2)] >> 3),
                    ];
                    let expected = if color == [0_u8; 3] {
                        [11_u8, 22_u8, 33_u8]
                    } else {
                        color
                    };
                    let destination = usize::try_from(screen_y)
                        .map_err(|error| error.to_string())?
                        .saturating_mul(800)
                        .saturating_add(
                            usize::try_from(screen_x).map_err(|error| error.to_string())?,
                        )
                        .saturating_mul(4);
                    if actual[destination..destination.saturating_add(3)] != expected {
                        return Err(format!(
                            "cursor UC{number:03} hotspot/pixel mismatch at ({x},{y})"
                        ));
                    }
                    checked = checked.saturating_add(1);
                }
            }
            if checked < 100 {
                return Err("cursor test did not inspect enough visible pixels".to_owned());
            }
        }
        Ok(())
    });
    assert!(result.is_ok(), "retail software cursor: {result:?}");
}
