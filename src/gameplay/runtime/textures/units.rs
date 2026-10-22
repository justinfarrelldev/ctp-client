//! Lazy directional SPR uploads; all action tables stay in memory, art is read-only.

extern crate alloc;

use alloc::collections::{BTreeMap, btree_map::Entry};
use std::path::Path;

use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;

use super::{UnitTexture, texture_from_frame};
use crate::gameplay::assets::load_unit_sprite;
use crate::gameplay::assets::sprite::{SpriteActionAsset, SpriteGroupAsset};
use crate::gameplay::runtime::animation::{ActorPose, frame_sample, move_duration};
use crate::gameplay::{GameplayError, asset_error};

/// A sampled source frame and its sequence-specific drawing metadata.
pub(in crate::gameplay::runtime) struct UnitDraw<'frame, 'texture> {
    /// Cached image/hotspot/herald anchor, with sampled alpha already applied.
    pub(in crate::gameplay::runtime) frame: &'frame UnitTexture<'texture>,
    /// Reflect western facings using SDL, not another copy of the pixels.
    pub(in crate::gameplay::runtime) flipped: bool,
    /// Original per-frame delta, reflected for western directions.
    pub(in crate::gameplay::runtime) delta: (i32, i32),
}

/// Shared by sprite ID so barbarians do not duplicate warrior artwork.
#[derive(Default)]
pub(super) struct UnitTextures<'texture> {
    /// Decoded actions and animation tables for supported unit art.
    assets: BTreeMap<u16, SpriteGroupAsset>,
    /// Lazy uploads keyed by sprite, action, stored facing and image frame.
    frames: BTreeMap<(u16, usize, u8, u16), UnitTexture<'texture>>,
}

impl<'texture> UnitTextures<'texture> {
    /// Loads supported retail actors once; an asset failure is fatal in live mode.
    pub(super) fn load(root: &Path, strict: bool) -> Result<Self, GameplayError> {
        let mut result = Self::default();
        for id in [2_u16, 3_u16] {
            match load_unit_sprite(root, id) {
                Ok(asset) => {
                    result.assets.insert(id, asset);
                }
                Err(error) if strict => return Err(error),
                Err(_) => {}
            }
        }
        Ok(result)
    }

    /// Lazily loads an installed retail actor exactly once, irrespective of owner.
    pub(super) fn ensure(&mut self, root: &Path, id: u16) -> Result<(), GameplayError> {
        if let Entry::Vacant(entry) = self.assets.entry(id) {
            entry.insert(load_unit_sprite(root, id)?);
        }
        Ok(())
    }

    /// One leg and one animation cycle use the identical speed-scaled duration.
    pub(super) fn duration(&self, sprite_id: u16, speed: i32, enabled: bool) -> u64 {
        let playback = self
            .assets
            .get(&sprite_id)
            .and_then(|group| group.actions.iter().find(|action| action.action_index == 0))
            .map_or(0, |action| action.animation.playback_time);
        move_duration(playback, speed, enabled)
    }

    /// Selects the original action/frame table and reflects its image and anchor.
    pub(super) fn sample(
        &mut self,
        creator: &'texture TextureCreator<WindowContext>,
        sprite_id: u16,
        pose: ActorPose,
        duration: u64,
        enabled: bool,
    ) -> Result<Option<UnitDraw<'_, 'texture>>, GameplayError> {
        let Some(group) = self.assets.get(&sprite_id) else {
            return Ok(None);
        };
        let Some(action) = action_for_pose(group, pose.moving) else {
            return Ok(None);
        };
        let elapsed = if enabled { pose.elapsed } else { 0 };
        let cycle = if pose.moving {
            duration
        } else {
            move_duration(action.animation.playback_time, 5, true)
        };
        let (frame_number, delta, alpha) = frame_sample(
            &action.animation,
            elapsed,
            cycle,
            !pose.moving && action.animation.animation_type == 1,
        );
        let frame = action
            .frame_for_facing(pose.facing.stored(), frame_number)
            .ok_or_else(|| {
                asset_error(format!(
                    "sprite {} action {} lacks frame {frame_number}",
                    sprite_id, action.action_index
                ))
            })?;
        let key = (
            sprite_id,
            action.action_index,
            frame.facing,
            frame.frame_index,
        );
        if let Entry::Vacant(entry) = self.frames.entry(key) {
            entry.insert(UnitTexture {
                sprite: texture_from_frame(creator, frame)?,
                mask: alloc::sync::Arc::new(
                    crate::gameplay::runtime::actor_hits::HitMask::from_rgba(
                        u32::from(frame.width),
                        u32::from(frame.height),
                        &frame.rgba,
                    )?,
                ),
                shield: group
                    .shield_points
                    .get(action.action_index)
                    .map_or((70_i32, 10_i32), |points| points[0]),
            });
        }
        let Some(texture) = self.frames.get_mut(&key) else {
            return Ok(None);
        };
        texture.sprite.texture.set_alpha_mod(alpha);
        Ok(Some(UnitDraw {
            frame: texture,
            flipped: pose.facing.flipped(),
            delta: (
                if pose.facing.flipped() {
                    delta.0.saturating_neg()
                } else {
                    delta.0
                },
                delta.1,
            ),
        }))
    }
}

/// CTP2 idles fall back to the MOVE action when the group has no IDLE sprite.
fn action_for_pose(group: &SpriteGroupAsset, moving: bool) -> Option<&SpriteActionAsset> {
    let requested = if moving { 0_usize } else { 2_usize };
    group
        .actions
        .iter()
        .find(|action| action.action_index == requested)
        .or_else(|| group.actions.iter().find(|action| action.action_index == 0))
}
