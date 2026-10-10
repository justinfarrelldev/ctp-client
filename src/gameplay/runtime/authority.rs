//! Helpers for keeping render state aligned with the authoritative server session.

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use super::super::model::{GameState, MapPoint, PlayerId};
use super::super::server::{ServerClient, ServerGameplaySession, ServerProjection};

/// Live authoritative server state for an interactive runtime session.
pub(super) struct RuntimeAuthority {
    /// Blocking command client.
    pub(super) client: ServerClient,
    /// Current authoritative session metadata.
    pub(super) session: ServerGameplaySession,
    /// Configured empire count for rebuilding projection state.
    pub(super) empire_count: u8,
    /// Configured deterministic fallback seed.
    pub(super) seed: u64,
}

/// Builds local render state from an authoritative projection and session account.
pub(super) fn state_from_authority_projection(
    authority: &RuntimeAuthority,
    projection: &ServerProjection,
) -> GameState {
    let mut state =
        GameState::from_server_projection(projection, authority.empire_count, authority.seed);
    apply_authority_owner_mapping(authority, &mut state, projection);
    state
}

/// Returns the server unit id for a local selected unit position.
pub(super) fn server_unit_id_for_position(
    authority: &RuntimeAuthority,
    position: MapPoint,
) -> String {
    let Some(projection) = authority.session.projection.as_ref() else {
        return String::new();
    };
    let human_owner_id = format!("account-{}", authority.session.account_id);
    projection
        .units
        .iter()
        .find(|unit| {
            unit.owner_id == human_owner_id
                && projection
                    .tiles
                    .iter()
                    .find(|tile| tile.id == unit.tile_id)
                    .is_some_and(|tile| tile.x == position.x && tile.y == position.y)
        })
        .map_or_else(String::new, |unit| unit.id.clone())
}

/// Returns authoritative tile ids for every point in a drawn path.
pub(super) fn server_tile_ids_for_path(
    projection: Option<&ServerProjection>,
    path: &[MapPoint],
) -> Option<Vec<String>> {
    if path.is_empty() {
        return None;
    }
    let projection = projection?;
    let mut ids = Vec::with_capacity(path.len());
    for point in path {
        let id = server_tile_id(Some(projection), *point);
        if id.is_empty() {
            return None;
        }
        ids.push(id);
    }
    Some(ids)
}

/// Applies the authenticated account id to ownership in the render-only model.
fn apply_authority_owner_mapping(
    authority: &RuntimeAuthority,
    state: &mut GameState,
    projection: &ServerProjection,
) {
    let human_owner_id = format!("account-{}", authority.session.account_id);
    for server_unit in &projection.units {
        let Some(tile) = projection
            .tiles
            .iter()
            .find(|tile| tile.id == server_unit.tile_id)
        else {
            continue;
        };
        let point = MapPoint::new(tile.x, tile.y);
        let owner = player_id_from_authority_owner(&server_unit.owner_id, &human_owner_id);
        if let Some(unit) = state
            .units
            .iter_mut()
            .find(|unit| unit.position == point && unit.alive)
        {
            unit.owner = owner;
        }
    }
    for server_city in &projection.cities {
        let Some(tile) = projection
            .tiles
            .iter()
            .find(|tile| tile.id == server_city.tile_id)
        else {
            continue;
        };
        let point = MapPoint::new(tile.x, tile.y);
        let owner = player_id_from_authority_owner(&server_city.owner_id, &human_owner_id);
        if let Some(city) = state.cities.iter_mut().find(|city| city.position == point) {
            city.owner = owner;
        }
    }
    state.selected_unit = state
        .units
        .iter()
        .find(|unit| unit.owner == PlayerId::Human && unit.alive)
        .map(|unit| unit.id);
}

/// Converts a server owner id into the local owner id for the authenticated account.
fn player_id_from_authority_owner(owner_id: &str, human_owner_id: &str) -> PlayerId {
    let lowered = owner_id.to_ascii_lowercase();
    if owner_id == human_owner_id {
        PlayerId::Human
    } else if lowered.contains("barbarian") {
        PlayerId::Barbarians
    } else {
        PlayerId::Empire(1)
    }
}

/// Returns the server tile id for a local map point.
fn server_tile_id(projection: Option<&ServerProjection>, point: MapPoint) -> String {
    projection
        .and_then(|projection| {
            projection
                .tiles
                .iter()
                .find(|tile| tile.x == point.x && tile.y == point.y)
        })
        .map_or_else(String::new, |tile| tile.id.clone())
}
