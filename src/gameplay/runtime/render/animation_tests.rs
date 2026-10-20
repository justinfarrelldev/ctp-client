//! Actual SDL framebuffer comparisons against independently selected retail SPR pixels.
//! Every capture remains in memory; no golden files or copied game assets.

use super::actors::{draw_units, unit_target_rect};
use crate::gameplay::assets::load_unit_sprite;
use crate::gameplay::assets::sprite::SpriteActionAsset;
use crate::gameplay::model::{FogState, GameState, UnitKind};
use crate::gameplay::runtime::test_support::with_canvas;
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::runtime::{Camera, MovementInputState};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas};
use sdl2::surface::Surface;
use sdl2::video::Window;
use std::path::Path;

/// Map deltas corresponding to north, northeast, east, southeast and their opposites.
const DIRECTIONS: [(i16, i16); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
];

#[test]
fn retail_moving_actors_match_source_frames_and_reflected_hotspots_in_all_directions() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let mut state = GameState::new(24, 18, 1, 42);
        state.units.truncate(1);
        let camera = Camera::centered_on_selected(&state);
        state.selected_unit = None;
        for tile in &mut state.tiles {
            tile.fog = FogState::Visible;
        }
        let origin = state.units[0_usize].position;
        for (kind, id, type_id) in [
            (UnitKind::Settler, 2_u16, "UNIT_SETTLER"),
            (UnitKind::Warrior, 3, "UNIT_WARRIOR"),
            (UnitKind::Warrior, 8, "UNIT_HOPLITE"),
            (UnitKind::Warrior, 13, "UNIT_ARCHER"),
        ] {
            state.units[0_usize].kind = kind;
            state.units[0_usize].id = crate::gameplay::model::UnitId::from_server_id("actor");
            state.viewer=Some(serde_json::from_value(serde_json::json!({"revision":1_i64,"turn":1_u32,"tiles":[],"units":[{"id":"actor","type_id":type_id}]})).map_err(|error|error.to_string())?);
            textures
                .unit_art(&state, &state.units[0_usize])
                .map_err(|error| error.to_string())?;
            let group = load_unit_sprite(&root, id).map_err(|error| error.to_string())?;
            let action = group
                .actions
                .iter()
                .find(|action| action.action_index == 0)
                .ok_or("move action missing")?;
            let duration = textures.unit_duration(id, 5, true);
            for (direction, (dx, dy)) in DIRECTIONS.iter().copied().enumerate() {
                state.units[0_usize].position = origin;
                let before = state.units.clone();
                let target = origin
                    .offset(dx, dy, state.width, state.height)
                    .ok_or("target missing")?;
                state.units[0_usize].position = target;
                let mut input = MovementInputState::default();
                input.actors.confirm(&before, &state, &[target]);
                for quarter in 0_u64..4_u64 {
                    input.actors.now_ms = duration.saturating_mul(quarter).div_euclid(4);
                    canvas.set_draw_color(Color::RGB(20, 30, 40));
                    canvas.clear();
                    draw_units(canvas, &mut textures, &state, &camera, &input)
                        .map_err(|error| error.to_string())?;
                    let actual = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
                    let pose = input.actors.pose(&state.units[0_usize], &camera, duration);
                    let expected =
                        reference_frame(canvas, action, pose.position, direction, quarter)?;
                    let shield = group.shield_points[0_usize][0_usize];
                    let overlay = Rect::new(
                        pose.position.0.saturating_add(shield.0).saturating_sub(5),
                        pose.position.1.saturating_add(shield.1).saturating_sub(30),
                        35,
                        65,
                    );
                    compare_actor_pixels(&actual, &expected, overlay).map_err(|error| {
                        format!("{type_id} direction {direction} quarter {quarter}: {error}")
                    })?;
                }
            }
        }
        Ok(())
    });
    assert!(
        result.is_ok(),
        "directional framebuffer regression: {result:?}"
    );
}

/// Independent reference compositor: selects source table entries without using
/// the production facing/sequence/cache code, then reads the real SDL pixels.
fn reference_frame(
    canvas: &mut Canvas<Window>,
    action: &SpriteActionAsset,
    position: (i32, i32),
    direction: usize,
    quarter: u64,
) -> Result<Vec<u8>, String> {
    let stored = u8::try_from(if direction > 4 {
        8_usize.saturating_sub(direction)
    } else {
        direction
    })
    .map_err(|error| error.to_string())?;
    let slot = action
        .animation
        .frames
        .len()
        .saturating_mul(usize::try_from(quarter).map_err(|error| error.to_string())?)
        .div_euclid(4);
    let index = action.animation.frames.get(slot).copied().unwrap_or(0);
    let frame = action
        .frames
        .iter()
        .find(|frame| frame.facing == stored && frame.frame_index == index)
        .ok_or("directional source frame missing")?;
    let mut rgba = frame.rgba.clone();
    let surface = Surface::from_data(
        &mut rgba,
        u32::from(frame.width),
        u32::from(frame.height),
        u32::from(frame.width).saturating_mul(4),
        PixelFormatEnum::RGBA32,
    )?;
    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_from_surface(surface)
        .map_err(|error| error.to_string())?;
    texture.set_blend_mode(BlendMode::Blend);
    let alpha = action
        .animation
        .transparencies
        .get(slot)
        .copied()
        .unwrap_or(15)
        .min(15)
        .saturating_mul(17);
    texture.set_alpha_mod(u8::try_from(alpha).map_err(|error| error.to_string())?);
    let flipped = direction > 4;
    let hotpoint = if flipped {
        (
            i32::from(frame.width).saturating_sub(frame.hotpoint.0),
            frame.hotpoint.1,
        )
    } else {
        frame.hotpoint
    };
    let delta = action
        .animation
        .deltas
        .get(slot)
        .copied()
        .unwrap_or((0_i32, 0_i32));
    let adjusted = (
        position.0.saturating_add(if flipped {
            delta.0.saturating_neg()
        } else {
            delta.0
        }),
        position.1.saturating_add(delta.1),
    );
    canvas.set_draw_color(Color::RGB(20, 30, 40));
    canvas.clear();
    canvas.copy_ex(
        &texture,
        None,
        unit_target_rect(
            u32::from(frame.width),
            u32::from(frame.height),
            hotpoint,
            adjusted,
            1,
        ),
        0.0_f64,
        None,
        flipped,
        false,
    )?;
    canvas.read_pixels(None, PixelFormatEnum::RGBA32)
}

/// Heralds/flags are tested elsewhere; compare every pixel outside their small
/// explicit overlay region, including transparent sprite margins and shadows.
fn compare_actor_pixels(actual: &[u8], expected: &[u8], overlay: Rect) -> Result<(), String> {
    for (pixel, (observed, source)) in actual
        .chunks_exact(4)
        .zip(expected.chunks_exact(4))
        .enumerate()
    {
        let x = i32::try_from(pixel.rem_euclid(800)).map_err(|error| error.to_string())?;
        let y = i32::try_from(pixel.div_euclid(800)).map_err(|error| error.to_string())?;
        if !overlay.contains_point((x, y)) && observed != source {
            return Err(format!("pixel ({x},{y}) mismatch"));
        }
    }
    Ok(())
}
