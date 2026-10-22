//! Shared source body bounds and exact sampled alpha/pose picking.
use crate::gameplay::{
    GameplayError,
    model::{GameState, PlayerId},
    runtime::{
        Camera, MovementInputState, actor_hits::ActorHit, animation::ActorPose,
        textures::GameplayTextures,
    },
    ui_model::MenuRect,
};
use alloc::sync::Arc;
use sdl2::rect::Rect;
/// Same reflected hotspot, original frame delta and integer zoom as painting.
pub(super) fn sprite_bounds(
    width: u32,
    height: u32,
    hotpoint: (i32, i32),
    delta: (i32, i32),
    pose: ActorPose,
    camera: &Camera,
) -> Rect {
    let zoom = i32::try_from(camera.tile_width())
        .map_or(1_i32, |tile_width| tile_width.div_euclid(94_i32));
    let hotpoint = pose.facing.hotpoint(
        hotpoint,
        u16::try_from(width).map_or(u16::MAX, |width| width),
    );
    let position = (
        pose.position.0.saturating_add(delta.0.saturating_mul(zoom)),
        pose.position.1.saturating_add(delta.1.saturating_mul(zoom)),
    );
    super::unit_target_rect(width, height, hotpoint, position, zoom)
}
/// Unknown art is an explicit body box, not a guessed retail Warrior silhouette.
pub(super) fn fallback_bounds(pose: ActorPose) -> Rect {
    Rect::new(
        pose.position.0.saturating_add(40_i32),
        pose.position.1.saturating_add(20_i32),
        12_u32,
        18_u32,
    )
}
/// Runs at presentation using the same clock/groups; no pointer-event asset IO.
pub(super) fn collect(
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
    camera: &Camera,
    input: &MovementInputState,
) -> Result<Vec<ActorHit>, GameplayError> {
    let mut targets = Vec::new();
    for actor in super::stacks::prepare(textures, state, camera, input)?
        .into_iter()
        .filter(|actor| actor.unit.owner == PlayerId::Human)
    {
        let sampled = if let Some(sprite) = actor.sprite {
            textures.unit_texture(sprite, actor.pose, actor.duration, input.actors.enabled)?
        } else {
            None
        };
        let (rect, mask, flipped, visible) = sampled.map_or_else(
            || (fallback_bounds(actor.pose), None, false, true),
            |draw| {
                let size = draw.frame.sprite.texture.query();
                (
                    sprite_bounds(
                        size.width,
                        size.height,
                        draw.frame.sprite.hotpoint,
                        draw.delta,
                        actor.pose,
                        camera,
                    ),
                    Some(Arc::clone(&draw.frame.mask)),
                    draw.flipped,
                    draw.frame.sprite.texture.alpha_mod() != 0_u8,
                )
            },
        );
        targets.push(ActorHit {
            id: actor.unit.id,
            canonical: actor.unit.position,
            bounds: MenuRect {
                x: rect.x(),
                y: rect.y(),
                width: rect.width(),
                height: rect.height(),
            },
            mask,
            visible,
            flipped,
            moving: actor.pose.moving,
        });
    }
    Ok(targets)
}
