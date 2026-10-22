//! Pixel-accurate source actor picking and current-frame identity cache.
use crate::gameplay::{
    GameplayError, asset_error,
    model::{GameState, MapPoint, PlayerId, UnitId},
    ui_model::MenuRect,
};
use alloc::sync::Arc;
/// One immutable bit per nontransparent source pixel, shared with uploaded frames.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct HitMask {
    /// Original sprite width, before display zoom.
    width: u32,
    /// Original sprite height, before display zoom.
    height: u32,
    /// Row-major opaque bits; no copied RGBA buffers per actor/frame.
    bits: Vec<u8>,
}
impl HitMask {
    /// Validates before allocating; malformed/overflowing source rasters fail closed.
    pub(super) fn from_rgba(width: u32, height: u32, rgba: &[u8]) -> Result<Self, GameplayError> {
        let pixels = usize::try_from(
            u64::from(width)
                .checked_mul(u64::from(height))
                .ok_or_else(|| asset_error("sprite hit-mask dimensions overflow".to_owned()))?,
        )
        .map_err(|error| asset_error(error.to_string()))?;
        let bytes = pixels
            .checked_mul(4_usize)
            .ok_or_else(|| asset_error("sprite hit-mask byte size overflow".to_owned()))?;
        if width == 0_u32 || height == 0_u32 || bytes != rgba.len() {
            return Err(asset_error("invalid sprite hit-mask raster".to_owned()));
        }
        let length = pixels.div_ceil(8_usize);
        let mut bits = vec![0_u8; length];
        for (index, pixel) in rgba.chunks_exact(4_usize).enumerate() {
            if pixel[3_usize] != 0_u8 {
                bits[index.div_euclid(8_usize)] |= 1_u8 << (index % 8_usize);
            }
        }
        Ok(Self {
            width,
            height,
            bits,
        })
    }
    /// SDL nearest-neighbor source lookup, including reflected western facings.
    pub(super) fn hit(&self, bounds: MenuRect, x: i32, y: i32, flipped: bool) -> bool {
        let Some((dx, dy)) = local(bounds, x, y) else {
            return false;
        };
        let Some(source_x) = dx
            .checked_mul(u64::from(self.width))
            .and_then(|value| value.checked_div(u64::from(bounds.width)))
            .and_then(|value| u32::try_from(value).ok())
        else {
            return false;
        };
        let Some(source_y) = dy
            .checked_mul(u64::from(self.height))
            .and_then(|value| value.checked_div(u64::from(bounds.height)))
        else {
            return false;
        };
        let source_x = if flipped {
            self.width.saturating_sub(1_u32).saturating_sub(source_x)
        } else {
            source_x
        };
        let Some(index) = source_y
            .checked_mul(u64::from(self.width))
            .and_then(|row| row.checked_add(u64::from(source_x)))
            .and_then(|index| usize::try_from(index).ok())
        else {
            return false;
        };
        self.bits
            .get(index.div_euclid(8_usize))
            .is_some_and(|byte| byte & (1_u8 << (index % 8_usize)) != 0_u8)
    }
}
/// Exact drawn own actor, not inferred canonical-cell contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ActorHit {
    /// Model identity tied to this current source pose.
    pub id: UnitId,
    /// Canonical origin for authority validation/orders, never the displayed position.
    pub canonical: MapPoint,
    /// Same source frame/hotspot/delta rectangle used by SDL painting.
    pub bounds: MenuRect,
    /// Immutable source alpha bits; None is the actually drawn unknown-art box.
    pub mask: Option<Arc<HitMask>>,
    /// Overall sampled transparency must not create an invisible clickable sprite.
    pub visible: bool,
    /// Same reflection used by SDL `copy_ex`.
    pub flipped: bool,
    /// Prevent canonical destination from acting like an invisible resting glyph.
    pub moving: bool,
}
impl ActorHit {
    /// Rechecks ownership/liveness/identity/position without retargeting a neighbor.
    fn valid(&self, state: &GameState) -> bool {
        state.units.iter().any(|unit| {
            unit.id == self.id
                && unit.owner == PlayerId::Human
                && unit.alive
                && unit.position == self.canonical
        })
    }
    /// Alpha-aware hit; all-transparent source frames do not become opaque boxes.
    fn contains(&self, x: i32, y: i32) -> bool {
        self.visible
            && self.mask.as_ref().map_or_else(
                || local(self.bounds, x, y).is_some(),
                |mask| mask.hit(self.bounds, x, y, self.flipped),
            )
    }
}
/// Last actually presented scene. Never acknowledges execution or mutates a model.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ActorHits {
    /// Distinguishes initial asset-free tests from an empty rendered frame.
    initialized: bool,
    /// Revision of the frame on screen; disagreement blocks map dispatch.
    revision: Option<i64>,
    /// Painter order; last matching opaque glyph wins.
    targets: Vec<ActorHit>,
}
impl ActorHits {
    /// Replaces the full frame atomically after the same source grouping was drawn.
    pub(super) fn replace(&mut self, state: &GameState, targets: Vec<ActorHit>) {
        self.initialized = true;
        self.revision = state.viewer.as_ref().map(|viewer| viewer.revision);
        self.targets = targets;
    }
    /// A changed receipt cannot silently rebase pointer actions onto unseen facts.
    pub(super) fn stale(&self, state: &GameState) -> bool {
        self.initialized && self.revision != state.viewer.as_ref().map(|viewer| viewer.revision)
    }
    /// Exact permitted opaque top actor in the last presented scene.
    pub(super) fn at(&self, state: &GameState, x: i32, y: i32) -> Option<UnitId> {
        if self.stale(state) {
            return None;
        }
        self.targets
            .iter()
            .rev()
            .find(|hit| hit.contains(x, y) && hit.valid(state))
            .map(|hit| hit.id)
    }
    /// Current moving actor cannot be selected through an empty destination cell.
    pub(super) fn moving(&self, state: &GameState, id: UnitId) -> bool {
        !self.stale(state)
            && self
                .targets
                .iter()
                .any(|hit| hit.id == id && hit.moving && hit.valid(state))
    }
}
/// Overflow-safe local destination coordinates, independent of saturating SDL edges.
fn local(bounds: MenuRect, x: i32, y: i32) -> Option<(u64, u64)> {
    let dx = u64::try_from(i64::from(x).checked_sub(i64::from(bounds.x))?).ok()?;
    let dy = u64::try_from(i64::from(y).checked_sub(i64::from(bounds.y))?).ok()?;
    (dx < u64::from(bounds.width) && dy < u64::from(bounds.height)).then_some((dx, dy))
}
/// Fileless source pixel/identity/cache tests.
#[cfg(test)]
mod tests;
