//! Exact city identity/caption/production projection, with no invented city facts.

use super::GameState;
use crate::gameplay::server::ServerProjection;
use serde_json::json;

#[test]
fn projected_city_names_and_production_are_not_replaced_by_machine_ids_and_warriors() {
    for (record, name, production) in [
        (
            json!({"id":"city-a","owner_id":"human","tile_id":"a","name":"Rome","queue":[{"id":"UNIT_HOPLITE","kind":"unit","cost":80_i64}]}),
            "Rome",
            "UNIT_HOPLITE",
        ),
        (
            json!({"id":"city-a","owner_id":"human","tile_id":"a"}),
            "—",
            "",
        ),
    ] {
        let projection:ServerProjection=serde_json::from_value(json!({"revision":1_i64,"turn":1_u32,"tiles":[{"id":"a","x":2_u16,"y":2_u16,"terrain":"TERRAIN_PLAINS"}],"cities":[record]})).expect("viewer city fixture");
        let state = GameState::from_server_projection(&projection, 1_u8, 1_u64);
        let city = state.cities.first().expect("actual projected city");
        assert_eq!(
            city.name, name,
            "name must be projected, never the canonical machine identity"
        );
        assert_eq!(
            city.build_item, production,
            "empty queues do not magically build Warriors"
        );
    }
}
