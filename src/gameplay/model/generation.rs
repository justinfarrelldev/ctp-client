//! Deterministic world generation and movement helper functions.

#![expect(
    clippy::arithmetic_side_effects,
    reason = "Deterministic world generation uses bounded integer tile noise and modular placement math"
)]
extern crate alloc;

use alloc::vec::Vec;
use core::cmp::Ordering;

use super::{
    DEFAULT_MOVE_POINTS, EmpireContact, FogState, GameplayPopup, MapPoint, MapTile, PlayerId,
    PopupKind, RuinOutcome, TerrainKind, TileFeature, UNIT_VISION_RADIUS, Unit, UnitId, UnitKind,
    UnitOrder,
};

/// Generates deterministic terrain and ruin features for a new world.
pub(in crate::gameplay::model) fn generated_tiles(
    width: u16,
    height: u16,
    seed: u64,
) -> Vec<MapTile> {
    let mut tiles = Vec::with_capacity(usize::from(width).saturating_mul(usize::from(height)));
    for y in 0..height {
        for x in 0..width {
            let point = MapPoint::new(x, y);
            let value = terrain_noise(seed, point);
            let terrain = terrain_from_noise(value);
            let feature =
                (terrain.passable_by_land() && value % 37 == 0).then_some(TileFeature::Ruin);
            tiles.push(MapTile {
                terrain,
                art_base_type: None,
                river: false,
                good_id: None,
                feature,
                fog: FogState::Hidden,
            });
        }
    }
    tiles
}

/// Builds non-barbarian contact rows.
pub(in crate::gameplay::model) fn generated_contacts(empire_count: u8) -> Vec<EmpireContact> {
    let bounded = empire_count.max(1);
    let mut contacts = vec![EmpireContact {
        owner: PlayerId::Human,
        encountered: true,
    }];
    for index in 1..bounded {
        contacts.push(EmpireContact {
            owner: PlayerId::Empire(index),
            encountered: false,
        });
    }
    contacts
}

/// Returns the beginning-of-game popup.
pub(in crate::gameplay::model) fn start_popup() -> GameplayPopup {
    GameplayPopup {
        title: "Call to Power II".to_owned(),
        body: "At the dawn of history your people look to you for leadership. Explore, settle, and survive.".to_owned(),
        kind: PopupKind::GameStart,
    }
}

/// Creates a new unit.
pub(in crate::gameplay::model) const fn new_unit(
    id: UnitId,
    kind: UnitKind,
    owner: PlayerId,
    position: MapPoint,
) -> Unit {
    Unit {
        id,
        kind,
        owner,
        position,
        movement_points: DEFAULT_MOVE_POINTS,
        max_movement_points: DEFAULT_MOVE_POINTS,
        vision_radius: UNIT_VISION_RADIUS,
        alive: true,
        order: UnitOrder::None,
    }
}

/// Returns terrain noise for a point.
pub(in crate::gameplay::model) fn terrain_noise(seed: u64, point: MapPoint) -> u64 {
    let x = u64::from(point.x).wrapping_mul(0x9E37_79B1_u64);
    let y = u64::from(point.y).wrapping_mul(0x85EB_CA77_u64);
    lcg_next(seed ^ x ^ y) % 100
}

/// Converts terrain noise into a terrain family.
pub(in crate::gameplay::model) const fn terrain_from_noise(value: u64) -> TerrainKind {
    match value {
        0..=13 => TerrainKind::Water,
        14..=24 => TerrainKind::Forest,
        25..=34 => TerrainKind::Hills,
        35..=40 => TerrainKind::Mountain,
        41..=49 => TerrainKind::Desert,
        50..=56 => TerrainKind::Swamp,
        57..=76 => TerrainKind::Plains,
        _ => TerrainKind::Grassland,
    }
}

/// Deterministic linear congruential step.
pub(in crate::gameplay::model) const fn lcg_next(value: u64) -> u64 {
    value
        .wrapping_mul(6_364_136_223_846_793_005_u64)
        .wrapping_add(1_442_695_040_888_963_407_u64)
}

/// Generates an edge-biased starting point.
pub(in crate::gameplay::model) fn seeded_edge_point(
    width: u16,
    height: u16,
    seed: u64,
    index: u8,
) -> MapPoint {
    let noise = lcg_next(seed ^ u64::from(index));
    let width_limit = width.saturating_sub(2).max(1);
    let height_limit = height.saturating_sub(2).max(1);
    let x_inner = u16::try_from(noise % u64::from(width_limit)).unwrap_or(0);
    let y_inner = u16::try_from(noise.div_euclid(17) % u64::from(height_limit)).unwrap_or(0);
    match index % 4 {
        0 => MapPoint::new(x_inner.saturating_add(1), 1),
        1 => MapPoint::new(width.saturating_sub(2), y_inner.saturating_add(1)),
        2 => MapPoint::new(x_inner.saturating_add(1), height.saturating_sub(2)),
        _ => MapPoint::new(1, y_inner.saturating_add(1)),
    }
}

/// Returns a straight eight-direction path.
pub(in crate::gameplay::model) fn straight_line_path(
    start: MapPoint,
    target: MapPoint,
    width: u16,
    height: u16,
) -> Vec<MapPoint> {
    let mut path = Vec::new();
    let mut current = start;
    while current != target {
        let dx = signed_step(current.x, target.x);
        let dy = signed_step(current.y, target.y);
        let Some(next) = current.offset(dx, dy, width, height) else {
            break;
        };
        if next == current {
            break;
        }
        path.push(next);
        current = next;
    }
    path
}

/// Returns a one-tile signed step along an axis.
pub(in crate::gameplay::model) fn signed_step(current: u16, target: u16) -> i16 {
    match current.cmp(&target) {
        Ordering::Less => 1,
        Ordering::Equal => 0,
        Ordering::Greater => -1,
    }
}

/// Returns one step from start toward target.
pub(in crate::gameplay::model) fn step_toward(
    start: MapPoint,
    target: MapPoint,
    width: u16,
    height: u16,
) -> Option<MapPoint> {
    start.offset(
        signed_step(start.x, target.x),
        signed_step(start.y, target.y),
        width,
        height,
    )
}

/// Returns whether two owners are enemies for local gameplay.
pub(in crate::gameplay::model) fn owners_are_enemies(left: &PlayerId, right: &PlayerId) -> bool {
    left != right
        && (left == &PlayerId::Barbarians
            || right == &PlayerId::Barbarians
            || left == &PlayerId::Human
            || right == &PlayerId::Human)
}

/// Computes a deterministic ruin outcome.
pub(in crate::gameplay::model) fn ruin_outcome(
    seed: u64,
    turn: u32,
    position: MapPoint,
) -> RuinOutcome {
    match terrain_noise(seed ^ u64::from(turn), position) % 4 {
        0 => RuinOutcome::Gold,
        1 => RuinOutcome::Knowledge,
        2 => RuinOutcome::Barbarians,
        _ => RuinOutcome::City,
    }
}

/// Converts a list length into a one-based id component.
pub(in crate::gameplay::model) fn next_id_from_len(len: usize) -> u32 {
    u32::try_from(len)
        .map_or(u32::MAX.saturating_sub(1), |value| value)
        .saturating_add(1)
}

/// Direction delta for deterministic AI wandering.
pub(in crate::gameplay::model) const fn direction_delta(direction: u8) -> (i16, i16) {
    match direction {
        0 => (0, -1),
        1 => (1, -1),
        2 => (1, 0),
        3 => (1, 1),
        4 => (0, 1),
        5 => (-1, 1),
        6 => (-1, 0),
        _ => (-1, -1),
    }
}
