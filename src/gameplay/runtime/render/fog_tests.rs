//! Foreign actors require explicit currently visible viewer facts, even at bad coordinates.

use super::actors::draw_units;
use crate::gameplay::model::{FogState, GameState, MapPoint, PlayerId};
use crate::gameplay::runtime::test_support::with_canvas;
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::runtime::{Camera, MovementInputState};
use sdl2::pixels::{Color, PixelFormatEnum};

#[test]
fn foreign_units_with_hidden_stale_or_missing_tiles_never_render() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let mut state = GameState::new(24, 18, 1, 42);
        let mut unit = state.selected_unit().ok_or("starter missing")?.clone();
        unit.owner = PlayerId::Empire(1);
        unit.position = MapPoint::new(1, 1);
        state.units = vec![unit];
        state.selected_unit = None;
        state.authoritative = true;
        state.filters.fog_of_war = false;
        let mut textures = GameplayTextures::without_assets_for_test();
        for fog in [FogState::Hidden, FogState::Explored, FogState::Visible] {
            state.tiles[25_usize].fog = fog;
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            draw_units(
                canvas,
                &mut textures,
                &state,
                &Camera::for_test_identity(),
                &MovementInputState::default(),
            )
            .map_err(|error| error.to_string())?;
            let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
            let drawn = pixels.chunks_exact(4).any(|pixel| pixel[..3] != [0_u8; 3]);
            if drawn != (fog == FogState::Visible) {
                return Err(format!("foreign unit leaked at {fog:?}"));
            }
        }
        state.tiles.clear();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        draw_units(
            canvas,
            &mut textures,
            &state,
            &Camera::for_test_identity(),
            &MovementInputState::default(),
        )
        .map_err(|error| error.to_string())?;
        let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        if pixels.chunks_exact(4).any(|pixel| pixel[..3] != [0_u8; 3]) {
            return Err("foreign actor with missing tile leaked".to_owned());
        }
        Ok(())
    });
    assert!(result.is_ok(), "fog-safe actor pixels: {result:?}");
}
