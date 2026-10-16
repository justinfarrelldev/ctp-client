//! Original goods are viewer facts, and their fog variants never tint each other.

use crate::gameplay::model::{FogState, GameState};
use crate::gameplay::runtime::Camera;
use crate::gameplay::runtime::test_support::with_canvas;
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::server::ServerProjection;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::path::Path;

#[test]
fn resource_visibility_and_exact_fog_variants_use_only_projected_good_ids() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let projection: ServerProjection = serde_json::from_str(r#"{"tiles":[{"id":"tile-6-4","x":6,"y":4,"terrain":"TERRAIN_PLAINS","good_id":"TERRAIN_PLAINS_GOOD_TWO"}]}"#).map_err(|error| error.to_string())?;
        let mut state = GameState::from_server_projection(&projection, 5_u8, 42_u64);
        let index = 102_usize;
        let good = state.tiles[index].good_id.take();
        let empty = render_resources(canvas, &mut textures, &state)?;
        if rgb_sum(&empty) != 0_u64 {
            return Err("resource was invented from terrain without a projected ID".to_owned());
        }
        state.tiles[index].good_id = good;
        let visible = render_resources(canvas, &mut textures, &state)?;
        state.tiles[index].fog = FogState::Explored;
        let explored = render_resources(canvas, &mut textures, &state)?;
        if rgb_sum(&explored) == 0_u64 || rgb_sum(&explored) >= rgb_sum(&visible) {
            return Err("remembered elephant must retain actual dimmed art".to_owned());
        }
        state.tiles[index].fog = FogState::Visible;
        let revealed = render_resources(canvas, &mut textures, &state)?;
        if visible != revealed {
            return Err("shared resource retained a stale fog tint after revealing it".to_owned());
        }
        state.tiles[index].fog = FogState::Hidden;
        state.filters.fog_of_war = false;
        if rgb_sum(&render_resources(canvas, &mut textures, &state)?) != 0_u64 {
            return Err("unknown resources leaked despite viewer visibility".to_owned());
        }
        Ok(())
    });
    assert!(result.is_ok(), "resource renderer regression: {result:?}");
}

/// Reads only resource sprites back into memory, without presenting or any file sink.
fn render_resources(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
) -> Result<Vec<u8>, String> {
    canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
    canvas.clear();
    super::actors::draw_goods(canvas, textures, state, &Camera::for_test_identity())
        .map_err(|error| error.to_string())?;
    canvas.read_pixels(None, PixelFormatEnum::RGBA32)
}

/// Summarizes only visible color channels; an opaque black buffer has sum zero.
fn rgb_sum(pixels: &[u8]) -> u64 {
    pixels.chunks_exact(4_usize).fold(0_u64, |sum, pixel| {
        sum.saturating_add(u64::from(pixel[0_usize]))
            .saturating_add(u64::from(pixel[1_usize]))
            .saturating_add(u64::from(pixel[2_usize]))
    })
}
