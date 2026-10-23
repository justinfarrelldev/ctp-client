//! Additive viewer contract v2. Missing facts remain unavailable, not zeroes.

use alloc::collections::BTreeMap;
use serde::Deserialize;
use super::projection::deserialize_nullable_vec;

/// Wire protocol capability declarations; never entity/phase authorization.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Contract {
    /// Projection protocol version.
    pub version: u32,
    /// Explicit server-supported features.
    pub features: BTreeMap<String, bool>,
}

/// Public playable extents; this is not a hidden terrain list.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct World {
    /// Lowest public x coordinate.
    pub min_x: i64,
    /// Lowest public y coordinate.
    pub min_y: i64,
    /// Number of columns.
    pub width: i64,
    /// Number of rows.
    pub height: i64,
    /// Whether horizontal wrapping is supported.
    pub wrap_x: bool,
    /// Whether vertical wrapping is supported.
    pub wrap_y: bool,
    /// Pinned generation lineage; absence does not imply the current generator.
    pub generator_version: Option<String>,
    /// Original resource registry lineage, separate from installed artwork.
    pub resource_catalog: Option<String>,
}
impl World {
    /// Representable zero-origin storage extents; preceding cells stay hidden.
    #[must_use]
    pub fn dimensions(&self) -> Option<(u16, u16)> {
        if self.min_x < 0 || self.min_y < 0 || self.width <= 0 || self.height <= 0 || self.wrap_x || self.wrap_y {
            return None;
        }
        Some((u16::try_from(self.min_x.checked_add(self.width)?).ok()?, u16::try_from(self.min_y.checked_add(self.height)?).ok()?))
    }
}

/// Known effective tile outputs before city/national policy.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct TileYield {
    /// Known food, including explicit zero.
    pub food: i64,
    /// Known production.
    pub production: i64,
    /// Known commerce.
    pub gold: i64,
}

/// Pinned authoritative calendar, localized only by presentation.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct Calendar {
    /// Server calendar version.
    pub version: String,
    /// One-based game turn.
    pub turn: u32,
    /// Signed authoritative integer year.
    pub year: i64,
}
impl Calendar {
    /// Original compact BC/AD panel notation, without inventing an epoch.
    #[must_use]
    pub fn label(&self) -> String {
        if self.year < 0 { format!("{}BC", self.year.unsigned_abs()) } else { format!("{}AD", self.year) }
    }
}

/// Owner-only price preview; never a purchase request field.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct Quote {
    /// Revision at which the quote was computed.
    pub revision: i64,
    /// Available/insufficient/completed/non-rushable/etc.
    pub status: String,
    /// Known price; absent is not a free purchase.
    pub cost: Option<i64>,
}

/// Constant-current-economy display estimate, not a promised future result.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct Forecast {
    /// Revision of the estimate.
    pub revision: i64,
    /// Estimated/stalled/completed/no-target/continuous/unavailable.
    pub status: String,
    /// Effective current rate, if known.
    pub rate: Option<i64>,
    /// Finite turns, if known.
    pub turns: Option<u32>,
    /// Explicit economic assumption.
    pub assumption: String,
}
impl Forecast {
    /// Only valid finite estimates at the displayed revision become numbers.
    #[must_use]
    pub fn label(&self, revision: i64) -> String {
        if self.revision == revision && matches!(self.status.as_str(), "estimated" | "completed") {
            self.turns.map_or_else(|| "—".to_owned(), |turns| turns.to_string())
        } else { "—".to_owned() }
    }
}

/// Observed semantic city-art facts, not inferred culture or filenames.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct CityArt {
    /// Observed current-owner age ID.
    pub age_id: String,
    /// Effective capital status at observation.
    pub capital: bool,
}

/// Owned city production governor; not empire AI.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct Governor {
    /// Actual enabled state.
    pub enabled: bool,
    /// Canonical pinned production priority.
    pub priority: String,
}

/// Owner-private retained journey restored from the server, never locally replayed.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct MovementOrder {
    /// Exact initiating unit.
    pub unit_id: String,
    /// Authorized owner.
    pub player_id: String,
    /// Army identity, when present.
    pub army_id: String,
    /// Exact member identities.
    #[serde(deserialize_with = "deserialize_nullable_vec")]
    pub unit_ids: Vec<String>,
    /// Expected current tile.
    pub tile_id: String,
    /// Public destination x, not hidden terrain.
    pub target_x: u16,
    /// Public destination y.
    pub target_y: u16,
    /// Original idempotent command identity.
    pub command_id: String,
}

/// Immutable private semantic result; never executes a reward locally.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Notification {
    /// Stable deduplication identity.
    pub id: String,
    /// Recipient identity.
    pub player_id: String,
    /// Semantic result kind.
    pub kind: String,
    /// Generation turn.
    pub turn: u32,
    /// Historical city identity, not current foreign state.
    pub city_id: String,
    /// Permitted historical tile identity.
    pub tile_id: String,
    /// Content involved in production/reward.
    pub content_id: String,
    /// Produced/exploring unit.
    pub unit_id: String,
    /// Originating order command.
    pub command_id: String,
    /// Authoritative ruin reward category.
    pub reward_kind: String,
    /// Authoritative reward amount.
    pub amount: i64,
}

/// Actual completed-turn own-nation history, not foreign rankings.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct Statistics {
    /// Completed round.
    pub turn: u32,
    /// Engine score-victory value.
    pub score: i64,
    /// Owned cities at that time.
    pub city_count: u32,
    /// Owned units at that time.
    pub unit_count: u32,
    /// Actual population.
    pub population: u64,
}
