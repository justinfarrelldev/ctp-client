//! Real SDL city building pixels, original anchoring and adversarial fog fixtures.

use super::cities::draw_cities;
use crate::gameplay::assets::{SPRITE_DIRECTORY, pixel::shade_explored, sprite::SpriteGroupAsset};
use crate::gameplay::model::GameState;
use crate::gameplay::runtime::{Camera, test_support::with_canvas, textures::GameplayTextures};
use crate::gameplay::server::ServerProjection;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use serde_json::json;
use std::path::Path;

#[test]
fn native_city_is_the_original_gc_building_sprite_not_an_owner_colored_rectangle() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let bytes = std::fs::read(root.join(SPRITE_DIRECTORY).join("GC028.SPR"))
            .map_err(|error| error.to_string())?;
        let group = SpriteGroupAsset::parse(&bytes).map_err(|error| error.to_string())?;
        let original = group
            .actions
            .iter()
            .find(|action| action.action_index == 2_usize)
            .and_then(|action| action.frames.first())
            .ok_or_else(|| "source city idle frame".to_owned())?;
        for remembered in [false, true] {
            let projection:ServerProjection=serde_json::from_value(json!({"revision":1_i64,"turn":1_u32,"tiles":[{"id":"a","x":2_u16,"y":2_u16,"terrain":"TERRAIN_PLAINS","stale":remembered}],"cities":[{"id":"city-a","owner_id":"rival","tile_id":"a","population":1_u32,"stale":remembered}]})).map_err(|error|error.to_string())?;
            let state = GameState::from_server_projection(&projection, 1_u8, 1_u64);
            let mut frame = original.clone();
            if remembered {
                shade_explored(&mut frame.rgba, true);
            }
            let mut picture = creator
                .create_texture_streaming(
                    PixelFormatEnum::RGBA32,
                    u32::from(frame.width),
                    u32::from(frame.height),
                )
                .map_err(|error| error.to_string())?;
            picture
                .update(
                    None,
                    &frame.rgba,
                    usize::from(frame.width).saturating_mul(4_usize),
                )
                .map_err(|error| error.to_string())?;
            picture.set_blend_mode(sdl2::render::BlendMode::Blend);
            let target = Rect::new(
                48_i32.saturating_sub(frame.hotpoint.0),
                168_i32.saturating_sub(frame.hotpoint.1),
                u32::from(frame.width),
                u32::from(frame.height),
            );
            canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
            canvas.clear();
            canvas.copy(&picture, None, target)?;
            let expected = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
            canvas.clear();
            draw_cities(canvas, &mut textures, &state, &Camera::for_test_identity())
                .map_err(|error| error.to_string())?;
            let actual = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
            if actual != expected {
                return Err(format!(
                    "city must match native GC028 pixels, remembered={remembered}"
                ));
            }
        }
        Ok(())
    });
    assert!(result.is_ok(), "original city artwork: {result:?}");
}

#[test]
fn authoritative_city_captions_use_viewer_names_not_outdated_render_cache_strings() {
    let result=with_canvas((800_u32,600_u32),|canvas| {
        let projection:ServerProjection=serde_json::from_value(json!({"revision":1_i64,"turn":1_u32,"tiles":[{"id":"a","x":2_u16,"y":2_u16,"terrain":"TERRAIN_PLAINS"}],"cities":[{"id":"city-a","owner_id":"human","tile_id":"a","population":1_u32,"name":"Rome"}]})).map_err(|error|error.to_string())?;
        let mut state=GameState::from_server_projection(&projection,1_u8,1_u64);
        let ui=crate::gameplay::ui_model::InGameUiModel::default();
        canvas.set_draw_color(Color::RGB(0_u8,0_u8,0_u8));canvas.clear();
        super::cities::draw_labels(canvas,&state,&ui,&Camera::for_test_identity(),None).map_err(|error|error.to_string())?;
        let expected=canvas.read_pixels(None,PixelFormatEnum::RGBA32)?;
        state.cities.first_mut().ok_or_else(||"city".to_owned())?.name="WRONG CACHED MACHINE ID".to_owned();
        canvas.set_draw_color(Color::RGB(0_u8,0_u8,0_u8));
        canvas.clear();
        super::cities::draw_labels(canvas,&state,&ui,&Camera::for_test_identity(),None).map_err(|error|error.to_string())?;
        if canvas.read_pixels(None,PixelFormatEnum::RGBA32)?!=expected {return Err("map caption used a stale render-cache name instead of the viewer's authoritative name".to_owned());}
        Ok(())
    });
    assert!(result.is_ok(),"authoritative city caption: {result:?}");
}

#[test]
fn hidden_or_inconsistently_fresh_foreign_cities_do_not_leak_even_with_local_fog_disabled() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        for (tiles, stale) in [
            (json!([]), false),
            (
                json!([{"id":"a","x":2_u16,"y":2_u16,"terrain":"TERRAIN_PLAINS","stale":true}]),
                false,
            ),
        ] {
            let projection:ServerProjection=serde_json::from_value(json!({"revision":1_i64,"turn":1_u32,"tiles":tiles,"cities":[{"id":"city-a","owner_id":"rival","tile_id":"a","population":99_u32,"stale":stale}]})).map_err(|error|error.to_string())?;
            let mut state = GameState::from_server_projection(&projection, 1_u8, 1_u64);
            state.filters.fog_of_war = false;
            canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
            canvas.clear();
            let expected = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
            draw_cities(canvas, &mut textures, &state, &Camera::for_test_identity())
                .map_err(|error| error.to_string())?;
            if canvas.read_pixels(None, PixelFormatEnum::RGBA32)? != expected {
                return Err("foreign city disclosed through stale/missing terrain".to_owned());
            }
        }
        Ok(())
    });
    assert!(result.is_ok(), "city fog boundary: {result:?}");
}
