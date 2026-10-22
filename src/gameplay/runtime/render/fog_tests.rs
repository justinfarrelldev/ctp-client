//! Foreign actors require explicit currently visible viewer facts, even at bad coordinates.

use super::actors::draw_units;
use crate::gameplay::model::{FogState, GameState, MapPoint, PlayerId, UnitId};
use crate::gameplay::runtime::test_support::with_canvas;
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::runtime::{Camera, MovementInputState};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use std::path::Path;

#[test]
fn hidden_foreign_colocation_never_changes_own_actor_or_herald_pixels_in_either_input_order() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let mut state = GameState::new(24_u16, 18_u16, 1_u8, 42_u64);
        let mut own = state.selected_unit().ok_or("owned starter")?.clone();
        own.position = MapPoint::new(1_u16, 1_u16);
        let mut foreign = state
            .selected_unit()
            .ok_or("foreign fixture template")?
            .clone();
        foreign.position = own.position;
        foreign.id = UnitId(99_999_u32);
        foreign.owner = PlayerId::Empire(1_u8);
        state.authoritative = true;
        state.filters.fog_of_war = false;
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let camera = Camera::for_test_identity();
        let input = MovementInputState::default();
        for selected in [None, Some(own.id)] {
            state.selected_unit = selected;
            for fog in [FogState::Hidden, FogState::Explored] {
                state.tiles[25_usize].fog = fog;
                state.units = vec![own.clone()];
                canvas.set_draw_color(Color::BLACK);
                canvas.clear();
                draw_units(canvas, &mut textures, &state, &camera, &input)
                    .map_err(|error| error.to_string())?;
                let expected = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
                for units in [
                    vec![foreign.clone(), own.clone()],
                    vec![own.clone(), foreign.clone()],
                ] {
                    state.units = units;
                    canvas.set_draw_color(Color::BLACK);
                    canvas.clear();
                    draw_units(canvas, &mut textures, &state, &camera, &input)
                        .map_err(|error| error.to_string())?;
                    let actual = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
                    if actual != expected {
                        return Err(format!(
                            "hidden/stale foreign member changed own glyph/stack count at {fog:?}, selection {selected:?}"
                        ));
                    }
                }
            }
        }
        state.tiles.clear();
        state.selected_unit = None;
        state.units = vec![own.clone()];
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        draw_units(canvas, &mut textures, &state, &camera, &input)
            .map_err(|error| error.to_string())?;
        let missing_reference = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        state.units = vec![foreign, own];
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        draw_units(canvas, &mut textures, &state, &camera, &input)
            .map_err(|error| error.to_string())?;
        if canvas.read_pixels(None, PixelFormatEnum::RGBA32)? != missing_reference {
            return Err("missing foreign tile changed own representative/herald".to_owned());
        }
        Ok(())
    });
    assert!(
        result.is_ok(),
        "whole native frame must not reveal hidden co-located actors or hide permitted own actor: {result:?}"
    );
}
#[test]
fn traveling_selected_actor_does_not_hide_the_stationary_actor_already_at_its_canonical_destination()
 {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let mut state = GameState::new(24_u16, 18_u16, 1_u8, 42_u64);
        let mut before = state.selected_unit().ok_or("starter")?.clone();
        before.position = MapPoint::new(1_u16, 1_u16);
        let destination = MapPoint::new(3_u16, 1_u16);
        let mut traveling = before.clone();
        traveling.position = destination;
        let mut stationary = traveling.clone();
        stationary.id = UnitId(99_999_u32);
        state.units = vec![traveling, stationary];
        state.selected_unit = Some(before.id);
        let mut input = MovementInputState::default();
        input.actors.enabled = true;
        input.actors.confirm(
            core::slice::from_ref(&before),
            &state,
            &[MapPoint::new(2_u16, 1_u16), destination],
        );
        let mut textures = GameplayTextures::without_assets_for_test();
        let camera = Camera::for_test_identity();
        let position = camera.tile_to_screen(destination);
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        draw_units(canvas, &mut textures, &state, &camera, &input)
            .map_err(|error| error.to_string())?;
        let target = canvas.read_pixels(
            Some(Rect::new(
                position.0.saturating_add(42_i32),
                position.1.saturating_add(23_i32),
                3_u32,
                3_u32,
            )),
            PixelFormatEnum::RGBA32,
        )?;
        if target
            .chunks_exact(4_usize)
            .all(|pixel| pixel[..3_usize] == [0_u8; 3_usize])
        {
            return Err("canonical-destination grouping erased the stationary actor while selected unit is still traveling from its origin".to_owned());
        }
        Ok(())
    });
    assert!(
        result.is_ok(),
        "separate presented actors remain drawable: {result:?}"
    );
}

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
