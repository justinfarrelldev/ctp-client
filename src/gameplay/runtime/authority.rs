//! Helpers for keeping render state aligned with the authoritative server session.

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use super::super::model::{GameState, MapPoint, PlayerId, UnitId};
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

/// Returns the exact server identity, including for units sharing one tile.
pub(super) fn server_unit_id_for_unit(authority: &RuntimeAuthority, unit_id: UnitId) -> String {
    let Some(projection) = authority.session.projection.as_ref() else {
        return String::new();
    };
    let human_owner_id = format!("account-{}", authority.session.account_id);
    projection
        .units
        .iter()
        .find(|unit| unit.owner_id == human_owner_id && UnitId::from_server_id(&unit.id) == unit_id)
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
        let id = UnitId::from_server_id(&server_unit.id);
        let owner = player_id_from_authority_owner(&server_unit.owner_id, &human_owner_id);
        if let Some(unit) = state
            .units
            .iter_mut()
            .find(|unit| unit.id == id && unit.alive)
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
    let Some(projection) = projection else {
        return String::new();
    };
    projection
        .tiles
        .iter()
        .find(|tile| tile.x == point.x && tile.y == point.y)
        .map_or_else(
            || canonical_local_server_tile_id(point),
            |tile| tile.id.clone(),
        )
}

/// Returns the Open CTP local-server canonical tile id for an unprojected frontier tile.
fn canonical_local_server_tile_id(point: MapPoint) -> String {
    format!("tile-{}-{}", point.x, point.y)
}

#[cfg(test)]
mod tests {
    //! Authoritative projection tile-id conversion tests.

    use alloc::vec;

    use super::{MapPoint, server_tile_ids_for_path};
    use crate::gameplay::server::{ServerProjection, ServerTileProjection};

    #[test]
    fn server_path_conversion_preserves_dragged_tile_order() {
        let projection = ServerProjection {
            tiles: vec![
                ServerTileProjection {
                    id: "origin".to_owned(),
                    x: 4,
                    y: 4,
                    terrain: "plains".to_owned(),
                    ..ServerTileProjection::default()
                },
                ServerTileProjection {
                    id: "east".to_owned(),
                    x: 5,
                    y: 4,
                    terrain: "plains".to_owned(),
                    ..ServerTileProjection::default()
                },
                ServerTileProjection {
                    id: "south-east".to_owned(),
                    x: 6,
                    y: 5,
                    terrain: "plains".to_owned(),
                    ..ServerTileProjection::default()
                },
            ],
            ..ServerProjection::default()
        };

        let ids = server_tile_ids_for_path(
            Some(&projection),
            &[MapPoint::new(5, 4), MapPoint::new(6, 5)],
        )
        .expect("all dragged tiles should map to server ids");

        assert_eq!(
            ids,
            vec!["east".to_owned(), "south-east".to_owned()],
            "authoritative move payload should preserve the drag path, not collapse to one target"
        );
    }

    #[test]
    fn server_path_conversion_synthesizes_frontier_tile_ids_missing_from_projection() {
        let projection = ServerProjection {
            tiles: vec![ServerTileProjection {
                id: "east".to_owned(),
                x: 5,
                y: 4,
                terrain: "plains".to_owned(),
                ..ServerTileProjection::default()
            }],
            ..ServerProjection::default()
        };

        let ids = server_tile_ids_for_path(
            Some(&projection),
            &[MapPoint::new(5, 4), MapPoint::new(6, 5)],
        )
        .expect("projected and frontier tiles should map to server ids");

        assert_eq!(
            ids,
            vec!["east".to_owned(), "tile-6-5".to_owned()],
            "frontier exploration should use canonical local server tile ids when the viewer projection omits an unseen tile"
        );
    }
}
