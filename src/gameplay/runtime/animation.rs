//! Presentation-only actor clocks. No animation changes gameplay state.
//!
//! CTP2 `spriteutils_DeltaToFacing` and `FacedSprite::Draw` define eight
//! directions backed by five stored facings. `Action::GetPosition` interpolates
//! each path leg. Unlike the original frame-count-driven loop, elapsed time
//! drives both travel and the animation table, so FPS cannot cause foot sliding.

extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};

use super::Camera;
use crate::gameplay::assets::sprite::SpriteAnimation;
use crate::gameplay::model::{GameState, MapPoint, Unit, UnitId};

/// Original clockwise facing, north=0, southeast/default=3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Facing(u8);

impl Facing {
    /// Resolves signed screen displacement, not rectangular map coordinates.
    pub(super) const fn from_delta((dx, dy): (i32, i32)) -> Self {
        Self(match (dx.signum(), dy.signum()) {
            (0, -1) => 0,
            (1, -1) => 1,
            (1, 0) => 2,
            (1, 1) => 3,
            (-1, 1) => 5,
            (-1, 0) => 6,
            (-1, -1) => 7,
            _ => 4,
        })
    }

    /// Original full eight-direction index.
    #[cfg(test)]
    pub(super) const fn index(self) -> u8 {
        self.0
    }

    /// Stored facing for mirrored western directions.
    pub(super) const fn stored(self) -> u8 {
        if self.flipped() {
            8_u8.saturating_sub(self.0)
        } else {
            self.0
        }
    }

    /// Whether the source image must be reflected horizontally.
    pub(super) const fn flipped(self) -> bool {
        self.0 >= 5_u8
    }

    /// Mirroring the artwork must also reflect its hotspot about its full width.
    pub(super) fn hotpoint(self, point: (i32, i32), width: u16) -> (i32, i32) {
        if self.flipped() {
            (i32::from(width).saturating_sub(point.0), point.1)
        } else {
            point
        }
    }
}

/// Movement duration follows the original 10-speed profile, with a minimum
/// positive interval and a sane fallback for SPRs with no playback metadata.
pub(super) fn move_duration(playback: u16, speed: i32, enabled: bool) -> u64 {
    if !enabled {
        return 1;
    }
    let base = if playback == 0 {
        500
    } else {
        u64::from(playback)
    };
    base.saturating_mul(u64::try_from(10_i32.saturating_sub(speed.clamp(0, 9))).unwrap_or(1))
        .div_euclid(5)
        .max(1)
}

/// Samples image index, per-frame drawing offset and SDL alpha from one clock.
/// Idle delay holds frame zero like `UnitActor::AddIdle`; movement ignores delay.
pub(super) fn frame_sample(
    animation: &SpriteAnimation,
    elapsed: u64,
    duration: u64,
    looping: bool,
) -> (u16, (i32, i32), u8) {
    let duration = duration.max(1);
    let time = if looping {
        let cycle = duration.saturating_add(u64::from(animation.delay)).max(1);
        let time = elapsed.rem_euclid(cycle);
        if time >= duration { 0 } else { time }
    } else {
        elapsed.min(duration.saturating_sub(1))
    };
    let count = u64::try_from(animation.frames.len()).unwrap_or(0);
    let slot = usize::try_from(time.saturating_mul(count).div_euclid(duration)).unwrap_or(0);
    let frame = animation.frames.get(slot).copied().unwrap_or(0);
    let delta = animation
        .deltas
        .get(slot)
        .copied()
        .unwrap_or((0_i32, 0_i32));
    let opacity = animation
        .transparencies
        .get(slot)
        .copied()
        .unwrap_or(15)
        .min(15)
        .saturating_mul(17);
    (frame, delta, u8::try_from(opacity).unwrap_or(255))
}

/// A server-confirmed path prefix, including its origin.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ConfirmedMotion {
    /// First is origin, last is the accepted endpoint, never an unaccepted tile.
    points: Vec<MapPoint>,
    /// Monotonic presentation time at receipt of the accepted projection.
    started: u64,
    /// Receipt is anchored at the first frame after synchronous HTTP returns.
    pending_start: bool,
}

/// Render-time pose shared by sprite, herald and selection reticle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ActorPose {
    /// Interpolated tile-image origin in screen pixels at the current zoom.
    pub(super) position: (i32, i32),
    /// Last actual travel direction, preserved while idle.
    pub(super) facing: Facing,
    /// Whether the accepted path is still being presented.
    pub(super) moving: bool,
    /// Elapsed time within this path leg or subsequent idle period.
    pub(super) elapsed: u64,
}

/// Bounded by live units; idle clocks and motion clocks are injectable in tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ActorAnimations {
    /// Current monotonic elapsed milliseconds, updated once per frame.
    pub(super) now_ms: u64,
    /// Original profile unit-speed setting, shared by travel and frame timing.
    pub(super) speed: i32,
    /// Whether the player's graphics settings permit animated unit travel.
    pub(super) enabled: bool,
    /// Confirmed motions, retained after arrival only for last-facing/idle epoch.
    motions: BTreeMap<UnitId, ConfirmedMotion>,
    /// Temporary pixels only; canonical unit positions remain unchanged.
    provisional: BTreeSet<UnitId>,
}

impl Default for ActorAnimations {
    fn default() -> Self {
        Self {
            now_ms: 0,
            speed: 5,
            enabled: true,
            motions: BTreeMap::new(),
            provisional: BTreeSet::new(),
        }
    }
}

impl ActorAnimations {
    /// Starts a one-step provisional presentation without touching canonical state.
    pub(super) fn preview(&mut self, unit: &Unit, target: MapPoint) {
        if unit.position.tile_distance(target) != 1 { return; }
        self.provisional.insert(unit.id);
        self.motions.insert(unit.id, ConfirmedMotion { points: vec![unit.position, target], started: self.now_ms, pending_start: true });
    }

    /// Queued/rejected attempts and resyncs return provisional pixels to authority.
    pub(super) fn rollback(&mut self) {
        for id in &self.provisional { self.motions.remove(id); }
        self.provisional.clear();
    }

    /// Animates only actual owner movement observed in a complete new snapshot.
    pub(super) fn reconcile(&mut self, before: &GameState, after: &GameState) {
        self.rollback();
        for unit in before.units.iter().filter(|unit| unit.owner == crate::gameplay::model::PlayerId::Human) {
            if let Some(next) = after.units.iter().find(|next| next.id == unit.id && next.owner == unit.owner) {
                if unit.position != next.position {
                    self.confirm(core::slice::from_ref(unit), after, &super::journeys::line(unit.position, next.position));
                }
            }
        }
    }

    /// Anchors new paths after event/HTTP processing, so network latency cannot
    /// consume their animation before the accepted result can be displayed.
    pub(super) fn begin_frame(&mut self, now: u64) {
        self.now_ms = now;
        for motion in self
            .motions
            .values_mut()
            .filter(|motion| motion.pending_start)
        {
            motion.started = now;
            motion.pending_start = false;
        }
    }

    /// Captures only an accepted prefix of the exact submitted path. A failed
    /// request, teleport, removal or unrelated projection cannot invent travel.
    pub(super) fn confirm(&mut self, before: &[Unit], after: &GameState, path: &[MapPoint]) {
        self.motions
            .retain(|id, _| after.units.iter().any(|unit| unit.alive && unit.id == *id));
        for old in before.iter().filter(|unit| unit.alive) {
            let Some(next) = after
                .units
                .iter()
                .find(|unit| unit.alive && unit.id == old.id && unit.owner == old.owner)
            else {
                continue;
            };
            if old.position == next.position {
                continue;
            }
            let Some(endpoint) = path.iter().position(|point| *point == next.position) else {
                continue;
            };
            let mut points = vec![old.position];
            for point in path.iter().take(endpoint.saturating_add(1)) {
                if points
                    .last()
                    .is_some_and(|last| last.tile_distance(*point) != 1_u16)
                {
                    break;
                }
                points.push(*point);
            }
            if points.last().copied() == Some(next.position) {
                self.motions.insert(
                    old.id,
                    ConfirmedMotion {
                        points,
                        started: self.now_ms,
                        pending_start: true,
                    },
                );
            }
        }
    }

    /// Computes position and animation phase together; never mutates a unit.
    pub(super) fn pose(&self, unit: &Unit, camera: &Camera, duration: u64) -> ActorPose {
        let stationary = ActorPose {
            position: camera.tile_to_screen(unit.position),
            facing: Facing(3),
            moving: false,
            elapsed: self.now_ms,
        };
        let Some(motion) = self
            .motions
            .get(&unit.id)
            .filter(|motion| motion.points.last() == Some(&unit.position) || self.provisional.contains(&unit.id))
        else {
            return stationary;
        };
        let duration = duration.max(1);
        let elapsed = self.now_ms.saturating_sub(motion.started);
        let legs = motion.points.len().saturating_sub(1);
        let leg = usize::try_from(elapsed.div_euclid(duration))
            .unwrap_or(usize::MAX)
            .min(legs.saturating_sub(1));
        let (Some(from), Some(to)) = (
            motion.points.get(leg),
            motion.points.get(leg.saturating_add(1)),
        ) else {
            return stationary;
        };
        let start = camera.tile_to_screen(*from);
        let end = camera.tile_to_screen(*to);
        let facing =
            Facing::from_delta((end.0.saturating_sub(start.0), end.1.saturating_sub(start.1)));
        let total = u64::try_from(legs)
            .unwrap_or(u64::MAX)
            .saturating_mul(duration);
        if elapsed >= total || !self.enabled {
            return ActorPose {
                facing,
                elapsed: elapsed.saturating_sub(total),
                ..stationary
            };
        }
        let phase = elapsed.rem_euclid(duration);
        ActorPose {
            position: (
                interpolate(start.0, end.0, phase, duration),
                interpolate(start.1, end.1, phase, duration),
            ),
            facing,
            moving: true,
            elapsed: phase,
        }
    }
}

/// Signed, overflow-safe linear interpolation. The original float-to-unsigned
/// cast made negative displacement non-portable. Division truncates toward zero.
fn interpolate(start: i32, end: i32, phase: u64, duration: u64) -> i32 {
    let delta = i128::from(end).saturating_sub(i128::from(start));
    let offset = delta
        .saturating_mul(i128::from(phase))
        .checked_div(i128::from(duration.max(1)))
        .unwrap_or(0);
    i32::try_from(i128::from(start).saturating_add(offset)).unwrap_or(end)
}

#[cfg(test)]
mod tests;
