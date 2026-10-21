//! Stacked settlers retain identity across viewer projections, without network I/O.

use super::{
    RuntimeAuthority, replace_state_from_projection, server_unit_id_for_unit,
    state_from_authority_projection,
};
use crate::gameplay::model::PlayerId;
use crate::gameplay::server::{
    ServerClient, ServerConfig, ServerGameplaySession, ServerProjection,
};

#[test]
fn every_projected_settler_in_a_stack_belongs_to_the_authenticated_player() {
    let authority = stacked_authority();
    let projection = authority
        .session
        .projection
        .as_ref()
        .expect("fixture projection");
    let state = state_from_authority_projection(&authority, projection);
    assert_eq!(
        state.units.len(),
        2_usize,
        "both original starting settlers rendered"
    );
    assert!(
        state.units.iter().all(|unit| unit.owner == PlayerId::Human),
        "owner mapping must use identity, not repeatedly update the first unit at a shared position"
    );
}

#[test]
fn selecting_second_stacked_settler_targets_and_preserves_that_exact_unit() {
    let authority = stacked_authority();
    let projection = authority
        .session
        .projection
        .as_ref()
        .expect("fixture projection");
    let mut state = state_from_authority_projection(&authority, projection);
    let second = state.units[1_usize].id;
    assert!(state.select_unit(second), "second stacked unit selectable");
    assert_eq!(
        server_unit_id_for_unit(&authority, second),
        "settler-b",
        "selected identity must not resolve to the first unit merely sharing its tile"
    );
    replace_state_from_projection(&authority, &mut state, projection);
    assert_eq!(
        state.selected_unit,
        Some(second),
        "authoritative receipts must not silently switch the selected portrait/reticle to another settler"
    );
}

/// Mirrors the server's two-starting-settlers-on-one-spawn contract; no requests.
fn stacked_authority() -> RuntimeAuthority {
    let projection: ServerProjection = serde_json::from_str(r#"{
        "revision":1,"turn":1,
        "tiles":[{"id":"tile-4-4","x":4,"y":4,"terrain":"TERRAIN_PLAINS"}],
        "units":[
            {"id":"settler-a","type_id":"UNIT_SETTLER","owner_id":"account-42","tile_id":"tile-4-4","movement":100,"max_movement":100},
            {"id":"settler-b","type_id":"UNIT_SETTLER","owner_id":"account-42","tile_id":"tile-4-4","movement":100,"max_movement":100}
        ]
    }"#).expect("accurate viewer fixture");
    RuntimeAuthority {
        io: None,
        client: ServerClient::from_config(&ServerConfig::disabled_for_test())
            .expect("constructing a client only parses the URL"),
        session: ServerGameplaySession {
            game_id: 44_i64,
            session_id: 88_i64,
            account_id: 42_i64,
            projection: Some(projection),
        },
        empire_count: 5_u8,
        seed: 42_u64,
    }
}
