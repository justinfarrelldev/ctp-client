//! Source-order context behavior with exact server identity and no hidden dispatch.

use super::context::{ContextIntent, ContextView, context_at};
use crate::gameplay::model::{FogState, GameState, MapPoint};
use crate::gameplay::server::ServerProjection;

#[test]
fn original_build_list_appends_owned_garrison_before_city_actions_and_map_info_last() {
    let projection: ServerProjection = serde_json::from_str(r#"{"revision":1,"turn":1,"tiles":[{"id":"tile-4-4","x":4,"y":4,"terrain":"TERRAIN_PLAINS"}],"cities":[{"id":"rome","owner_id":"human","tile_id":"tile-4-4"}],"units":[{"id":"guard","owner_id":"human","type_id":"UNIT_WARRIOR","tile_id":"tile-4-4","fortified":true}]}"#).expect("city fixture");
    let state = GameState::from_server_projection(&projection, 1, 42);
    let rows = context_at(&state, MapPoint::new(4, 4));
    assert_eq!(
        rows[0_usize].label, "Army Manager",
        "complete original BuildList calls BuildUnitList before BuildCityList"
    );
    let city = rows
        .iter()
        .position(|row| row.label == "City Manager")
        .expect("owned city actions");
    assert!(city > 0_usize, "garrison first");
    assert_eq!(
        rows[city.saturating_add(1_usize)].label,
        "Build Manager",
        "source city action order"
    );
    assert_eq!(
        rows.last().expect("map info").label,
        "Map Info",
        "source appends map help after both contexts"
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
fn unselected_sheltered_city_garrison_does_not_invent_an_active_army_context_while_selected_member_remains_explicit()
 {
    let projection:ServerProjection=serde_json::from_str(r#"{"revision":1,"turn":1,"tiles":[{"id":"cell","x":4,"y":4,"terrain":"TERRAIN_PLAINS"}],"cities":[{"id":"rome","owner_id":"human","tile_id":"cell"}],"units":[{"id":"guard","owner_id":"human","type_id":"UNIT_WARRIOR","tile_id":"cell","fortified":true}]}"#).expect("known sheltered own unit");
    let mut state = GameState::from_server_projection(&projection, 1_u8, 42_u64);
    state.selected_unit = None;
    let rows = context_at(&state, MapPoint::new(4_u16, 4_u16));
    assert_eq!(
        rows.first().expect("city").label,
        "City Manager",
        "original BuildUnitList fallback does not invent a current army from sheltered city garrison"
    );
    assert!(
        !rows
            .iter()
            .any(|row| matches!(row.intent, ContextIntent::Army(_) | ContextIntent::Move(_))),
        "plain city click cannot activate hidden garrison representative"
    );
    let id = crate::gameplay::model::UnitId::from_server_id("guard");
    assert!(
        state.select_unit(id),
        "explicit selection still known/owned"
    );
    let selected = context_at(&state, MapPoint::new(4_u16, 4_u16));
    assert_eq!(
        selected.first().expect("selected army").intent,
        ContextIntent::Army(id),
        "current selected own army remains explicit, not replaced by city priority"
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
fn explicit_actor_popup_keeps_exact_id_over_selected_same_cell_neighbor_and_rejects_foreign_dead_missing_or_unknown_geography()
 {
    let projection:ServerProjection=serde_json::from_str(r#"{"revision":9,"turn":2,"tiles":[{"id":"cell","x":4,"y":4,"terrain":"TERRAIN_PLAINS"}],"cities":[{"id":"rome","owner_id":"human","tile_id":"cell"}],"units":[{"id":"guard","owner_id":"human","type_id":"UNIT_WARRIOR","tile_id":"cell"},{"id":"other","owner_id":"human","type_id":"UNIT_WARRIOR","tile_id":"cell"}],"legal_actions":[{"route":"fortify","entity_id":"guard","payload":{"unit_id":"guard"}}]}"#).expect("current owner-safe context");
    let mut state = GameState::from_server_projection(&projection, 1_u8, 42_u64);
    let id = crate::gameplay::model::UnitId::from_server_id("guard");
    assert!(
        state.select_unit(crate::gameplay::model::UnitId::from_server_id("other")),
        "different selected member"
    );
    let before = state.clone();
    let popup = ContextView::for_unit(&state, id, 400_i32, 200_i32).expect("explicit own popup");
    assert_eq!(
        popup.activate("Move"),
        Some(ContextIntent::Move(id)),
        "not selected neighbor"
    );
    assert_eq!(
        popup.activate("Army Manager"),
        Some(ContextIntent::Army(id)),
        "exact actor management"
    );
    assert!(
        matches!(popup.activate("Fortify"),Some(ContextIntent::Submit(action)) if action.entity_id=="guard" && action.payload["unit_id"]=="guard"),
        "exact authorized payload"
    );
    let city = popup
        .rows
        .iter()
        .position(|row| row.label == "City Manager")
        .expect("known owned city");
    assert!(city > 0_usize, "unit first, city second");
    assert_eq!(
        popup.rows.last().expect("map info").label,
        "Map Info",
        "map last once"
    );
    assert_eq!(
        state, before,
        "opening popup never selects/submits/mutates authority"
    );
    state
        .units
        .iter_mut()
        .find(|unit| unit.id == id)
        .expect("guard")
        .owner = crate::gameplay::model::PlayerId::Empire(1_u8);
    assert!(
        ContextView::for_unit(&state, id, 400_i32, 200_i32).is_none(),
        "foreign cannot acquire own menu"
    );
    state
        .units
        .iter_mut()
        .find(|unit| unit.id == id)
        .expect("guard")
        .owner = crate::gameplay::model::PlayerId::Human;
    state
        .units
        .iter_mut()
        .find(|unit| unit.id == id)
        .expect("guard")
        .alive = false;
    assert!(
        ContextView::for_unit(&state, id, 400_i32, 200_i32).is_none(),
        "dead unavailable"
    );
    state
        .units
        .iter_mut()
        .find(|unit| unit.id == id)
        .expect("guard")
        .alive = true;
    state.tiles.clear();
    let unknown = ContextView::for_unit(&state, id, 400_i32, 200_i32)
        .expect("explicit owned ID remains known");
    assert!(
        unknown.rows.iter().all(|row| !matches!(
            row.intent,
            ContextIntent::MapInfo(_) | ContextIntent::City(_) | ContextIntent::Build(_)
        )),
        "unit permission cannot invent missing cell/city geography"
    );
    assert!(
        ContextView::for_unit(
            &state,
            crate::gameplay::model::UnitId(99_999_u32),
            400_i32,
            200_i32
        )
        .is_none(),
        "missing never retargets neighbor"
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
