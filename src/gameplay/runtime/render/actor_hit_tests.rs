//! Native SPR pixel/pose/flip/zoom proof; no capture files or live services.
use super::{actor_hit_targets, actors::draw_units};
use crate::gameplay::runtime::test_support::with_canvas;
use crate::gameplay::{
    model::{GameState, MapPoint},
    runtime::{Camera, MovementInputState, actor_hits::ActorHits, textures::GameplayTextures},
};
use sdl2::pixels::{Color, PixelFormatEnum};
use std::path::Path;
#[test]
fn native_hit_descriptors_follow_nontransparent_installed_sprite_pixels_at_actual_moving_pose_zoom_and_western_flip()
 {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        for reverse in [false, true] {
            for enlarged in [false, true] {
                let mut state = GameState::new(24_u16, 18_u16, 1_u8, 42_u64);
                let mut before = state.selected_unit().ok_or("starter")?.clone();
                let (origin, destination) = if reverse {
                    (MapPoint::new(3_u16, 1_u16), MapPoint::new(1_u16, 1_u16))
                } else {
                    (MapPoint::new(1_u16, 1_u16), MapPoint::new(3_u16, 1_u16))
                };
                before.position = origin;
                let mut received = before.clone();
                received.position = destination;
                state.units = vec![received];
                state.selected_unit = None;
                let mut input = MovementInputState::default();
                input.actors.enabled = true;
                input.actors.confirm(
                    core::slice::from_ref(&before),
                    &state,
                    &[MapPoint::new(2_u16, 1_u16), destination],
                );
                input.actors.begin_frame(15_u64);
                let mut camera = Camera::for_test_identity();
                if enlarged {
                    camera.zoom_in();
                }
                canvas.set_draw_color(Color::RGB(255_u8, 0_u8, 255_u8));
                canvas.clear();
                draw_units(canvas, &mut textures, &state, &camera, &input)
                    .map_err(|error| error.to_string())?;
                let targets = actor_hit_targets(&mut textures, &state, &camera, &input)
                    .map_err(|error| error.to_string())?;
                let descriptor = targets.first().ok_or("own moving descriptor")?;
                if descriptor.id != before.id
                    || !descriptor.moving
                    || descriptor.flipped != reverse
                    || descriptor.mask.is_none()
                {
                    return Err("native source descriptor lost exact identity, moving pose, real mask or west flip".to_owned());
                }
                let bounds = descriptor.bounds;
                let mut cache = ActorHits::default();
                cache.replace(&state, targets);
                let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
                let mut opaque = 0_usize;
                let mut transparent = 0_usize;
                for y in bounds.y
                    ..bounds.y.saturating_add(
                        i32::try_from(bounds.height).map_err(|error| error.to_string())?,
                    )
                {
                    for x in bounds.x
                        ..bounds.x.saturating_add(
                            i32::try_from(bounds.width).map_err(|error| error.to_string())?,
                        )
                    {
                        if !(0_i32..800_i32).contains(&x) || !(0_i32..600_i32).contains(&y) {
                            continue;
                        }
                        if cache.at(&state, x, y) == Some(before.id) {
                            opaque = opaque.saturating_add(1_usize);
                            let index = (usize::try_from(y)
                                .map_err(|error| error.to_string())?
                                .saturating_mul(800_usize)
                                .saturating_add(
                                    usize::try_from(x).map_err(|error| error.to_string())?,
                                ))
                            .saturating_mul(4_usize);
                            if pixels.get(index..index.saturating_add(3_usize))
                                == Some([255_u8, 0_u8, 255_u8].as_slice())
                            {
                                return Err(format!(
                                    "hit at untouched native background {x},{y}, flip {reverse}, enlarged {enlarged}"
                                ));
                            }
                        } else {
                            transparent = transparent.saturating_add(1_usize);
                        }
                    }
                }
                if opaque == 0_usize || transparent == 0_usize {
                    return Err("source mask must contain both painted body and non-clickable transparent pixels".to_owned());
                }
            }
        }
        Ok(())
    });
    assert!(
        result.is_ok(),
        "native source frame/pose pixels and pointer identity must coincide: {result:?}"
    );
}
