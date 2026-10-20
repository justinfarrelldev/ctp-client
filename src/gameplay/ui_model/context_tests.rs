//! Source-order context behavior with exact server identity and no hidden dispatch.

use super::context::{ContextIntent, context_at};
use crate::gameplay::model::{FogState, GameState, MapPoint};
use crate::gameplay::server::ServerProjection;

#[test]
fn owned_city_takes_context_priority_over_fortified_garrison() {
    let projection: ServerProjection = serde_json::from_str(r#"{"revision":1,"turn":1,"tiles":[{"id":"tile-4-4","x":4,"y":4,"terrain":"TERRAIN_PLAINS"}],"cities":[{"id":"rome","owner_id":"human","tile_id":"tile-4-4"}],"units":[{"id":"guard","owner_id":"human","type_id":"UNIT_WARRIOR","tile_id":"tile-4-4","fortified":true}]}"#).expect("city fixture");
    let state = GameState::from_server_projection(&projection, 1, 42);
    let rows = context_at(&state, MapPoint::new(4, 4));
    assert_eq!(rows[0_usize].label, "City Manager", "city gets priority");
    assert_eq!(
        rows[1_usize].label, "Build Manager",
        "source city row order"
    );
    assert!(
        !rows.iter().any(|row| row.label == "Move"),
        "do not merge unrelated army orders into city context"
    );
    assert!(
        !rows
            .iter()
            .find(|row| row.label == "Rename City")
            .expect("rename")
            .enabled(),
        "no fake rename without server route"
    );
}

#[test]
fn unit_context_uses_that_units_exact_legal_payload_not_selected_other_unit() {
    let projection: ServerProjection = serde_json::from_str(r#"{"revision":9,"turn":1,"tiles":[{"id":"a","x":4,"y":4,"terrain":"TERRAIN_PLAINS"},{"id":"b","x":5,"y":4,"terrain":"TERRAIN_PLAINS"}],"units":[{"id":"one","owner_id":"human","type_id":"UNIT_WARRIOR","tile_id":"a"},{"id":"two","owner_id":"human","type_id":"UNIT_WARRIOR","tile_id":"b"}],"legal_actions":[{"route":"fortify","entity_id":"two","payload":{"unit_id":"two"}}]}"#).expect("unit fixture");
    let state = GameState::from_server_projection(&projection, 1, 42);
    let rows = context_at(&state, MapPoint::new(5, 4));
    let fortify = rows
        .iter()
        .find(|row| row.label == "Fortify")
        .expect("fortify row");
    let action = match fortify.intent.clone() {
        ContextIntent::Submit(action) => Some(action),
        _ => None,
    }
    .expect("expected server command");
    assert_eq!(
        action.payload["unit_id"], "two",
        "target identity preserved"
    );
    assert_eq!(
        state.viewer.as_ref().expect("snapshot").revision,
        9_i64,
        "opening menu never changes revision"
    );
    assert!(
        !rows
            .iter()
            .find(|row| row.label == "Sentinel")
            .expect("sentinel")
            .enabled(),
        "Sentinel is not Fortify"
    );
}

#[test]
fn hidden_tile_has_no_context_information_or_foreign_unit_disclosure() {
    let mut state = GameState::new(24, 18, 1, 42);
    let point = state.selected_unit().expect("starter").position;
    let index = usize::from(point.y)
        .saturating_mul(usize::from(state.width))
        .saturating_add(usize::from(point.x));
    state.tiles[index].fog = FogState::Hidden;
    assert!(
        context_at(&state, point).is_empty(),
        "unknown tiles do not disclose context"
    );
}
