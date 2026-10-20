//! Explicitly synthetic, fileless management data for opt-in visual inspection.
//!
//! These values are diagnostic inputs, not installed rules or a server response.
//! Live gameplay never calls this fixture and no command client is constructed.

use crate::gameplay::model::GameState;
use crate::gameplay::server::interface::{LegalAction, Research, ServerInterface};
use crate::gameplay::server::{
    ServerCityProjection, ServerDiplomacyProjection, ServerProjection, ServerTileProjection,
    ServerUnitProjection,
};
use serde_json::json;

/// Shared explicit synthetic viewer data, never used by interactive gameplay.
fn projection() -> ServerProjection {
    ServerProjection {
        revision: 7_i64,
        turn: 4_u32,
        tiles: (21_u16..=23_u16)
            .flat_map(|y| {
                (29_u16..=31_u16).map(move |x| ServerTileProjection {
                    id: format!("tile-{x}-{y}"),
                    x,
                    y,
                    terrain: "TERRAIN_PLAINS".to_owned(),
                    ..ServerTileProjection::default()
                })
            })
            .collect(),
        units: vec![ServerUnitProjection {
            id: "fixture-hoplite".to_owned(),
            owner_id: "human".to_owned(),
            type_id: "UNIT_HOPLITE".to_owned(),
            tile_id: "tile-30-22".to_owned(),
            movement: 100_i16,
            max_movement: 100_i16,
            facts: json!({"health":10_i64,"max_health":10_i64,"firepower":1_i64}),
            ..ServerUnitProjection::default()
        }],
        cities: vec![ServerCityProjection {
            id: "fixture-city".to_owned(),
            owner_id: "human".to_owned(),
            tile_id: "tile-30-22".to_owned(),
            population: 3_u32,
            facts: json!({"queue":[{"id":"UNIT_HOPLITE","kind":"unit","cost":20_i64},{"id":"BUILDING_GRANARY","kind":"building","cost":50_i64}],"stored_production":10_i64}),
            ..ServerCityProjection::default()
        }],
        diplomacy: Some(ServerDiplomacyProjection {
            gold: 400_i32,
            public_works: 100_i32,
            ..ServerDiplomacyProjection::default()
        }),
        interface: ServerInterface {
            research: Some(Research {
                target_id: "ADVANCE_BALLISTICS".to_owned(),
                progress: 36_i64,
                advances: vec![
                    "ADVANCE_AGRICULTURE".to_owned(),
                    "ADVANCE_TOOLMAKING".to_owned(),
                ],
                available: vec![
                    "ADVANCE_BALLISTICS".to_owned(),
                    "ADVANCE_IRON_WORKING".to_owned(),
                ],
            }),
            rules: Some(json!({
                "ages":[{"id":"AGE_ANCIENT","index":0_i64}],
                "advances":[
                    {"id":"ADVANCE_AGRICULTURE","age_id":"AGE_ANCIENT","cost":20_i64,"prerequisites":[]},
                    {"id":"ADVANCE_TOOLMAKING","age_id":"AGE_ANCIENT","cost":20_i64,"prerequisites":[]},
                    {"id":"ADVANCE_BALLISTICS","age_id":"AGE_ANCIENT","cost":132_i64,"prerequisites":["ADVANCE_AGRICULTURE","ADVANCE_TOOLMAKING"]},
                    {"id":"ADVANCE_IRON_WORKING","age_id":"AGE_ANCIENT","cost":120_i64,"prerequisites":["ADVANCE_TOOLMAKING"]},
                    {"id":"ADVANCE_ALCHEMY","age_id":"AGE_ANCIENT","cost":200_i64,"prerequisites":["ADVANCE_BALLISTICS"]}
                ],
                "units":[{"id":"UNIT_SETTLER","attack":0_i64,"defense":10_i64,"armor":1_i64,"ranged":0_i64,"firepower":1_i64,"visibility_range":1_i64,"production_cost":100_i64},{"id":"UNIT_HOPLITE","attack":10_i64,"defense":15_i64,"armor":1_i64,"ranged":0_i64,"firepower":1_i64,"visibility_range":1_i64,"production_cost":20_i64}],
                "production":[{"id":"BUILDING_GRANARY","kind":"building","cost":50_i64,"upkeep":1_i64,"effects":[{"kind":"yield_food_percent","value":10_i64}]}]
            })),
            legal_actions: vec![LegalAction {
                command: "queue-production".to_owned(),
                route: "queue-production".to_owned(),
                entity_id: "fixture-city".to_owned(),
                target_id: "UNIT_SETTLER".to_owned(),
                payload: json!({"city_id":"fixture-city","item":{"id":"UNIT_SETTLER","kind":"unit","cost":100_i64}}),
                ..LegalAction::default()
            }],
            ..ServerInterface::default()
        },
    }
}

/// Builds representative populated city/research descriptions with public shapes.
pub(super) fn state() -> GameState {
    from_projection(&projection())
}

/// Separate current-policy diagnostic; ordinary populated scenes stay unchanged.
pub(super) fn policy_state() -> GameState {
    let mut projection = projection();
    if let Some(diplomacy) = projection.diplomacy.as_mut() {
        diplomacy.facts = json!({"government_id":"GOVERNMENT_TYRANNY","science_percent":50_i64,"public_works_percent":10_i64,"workday":1_i64,"wages":-1_i64,"rations":0_i64});
    }
    if let Some(rules) = projection.interface.rules.as_mut() {
        rules["governments"] = json!([{"id":"GOVERNMENT_TYRANNY","max_science_percent":80_i64}]);
    }
    for (route, fields, payload) in [
        (
            "set-science-rate",
            vec!["science_percent"],
            json!({"science_percent":100_i64}),
        ),
        (
            "set-public-works-rate",
            vec!["public_works_percent"],
            json!({"public_works_percent":20_i64}),
        ),
        (
            "set-social-policy",
            vec!["workday", "wages", "rations"],
            json!({"workday":0_i64,"wages":0_i64,"rations":0_i64}),
        ),
    ] {
        projection.interface.legal_actions.push(LegalAction {
            route: route.to_owned(),
            command: route.to_owned(),
            entity_id: "human".to_owned(),
            fields: fields.into_iter().map(str::to_owned).collect(),
            payload,
            ..LegalAction::default()
        });
    }
    from_projection(&projection)
}

/// A separate two-city diagnostic for detached city-selector popup inspection.
pub(super) fn chooser_state() -> GameState {
    let mut projection = projection();
    projection.cities.push(ServerCityProjection {
        id: "fixture-city-2".to_owned(),
        owner_id: "human".to_owned(),
        tile_id: "tile-31-22".to_owned(),
        population: 2_u32,
        facts: json!({"queue":[],"stored_production":0_i64}),
        ..ServerCityProjection::default()
    });
    from_projection(&projection)
}

/// Keeps automatic connection/turn popups out of explicitly offline previews.
fn from_projection(projection: &ServerProjection) -> GameState {
    let mut state = GameState::from_server_projection(projection, 5_u8, 42_u64);
    state.popups.clear();
    state
}
