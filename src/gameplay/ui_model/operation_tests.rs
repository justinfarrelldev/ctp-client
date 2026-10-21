//! Fileless selected city/queue operation forms; authorization stays server-owned.
use super::operations::{InputKind, OperationDialog};
use crate::gameplay::{model::GameState, server::ServerProjection};
use serde_json::json;

#[test]
fn mayor_toggle_preserves_current_focus_and_disables_without_fabricating_a_priority() {
    let viewer:ServerProjection=serde_json::from_value(json!({"revision":6_i64,"turn":2_u32,"rules":{"governor_priorities":["growth"]},"cities":[{"id":"c","owner_id":"human","governor":{"enabled":true,"priority":"growth"}}],"legal_actions":[{"route":"set-governor","entity_id":"c","payload":{"city_id":"c","enabled":false}}]})).expect("governor fixture");
    let mut state = GameState::from_server_projection(&viewer, 1, 1);
    assert_eq!(
        OperationDialog::toggle_governor(&state, "c")
            .expect("authorized disable")
            .payload,
        json!({"city_id":"c","enabled":false}),
        "toggle-off does not guess another focus"
    );
    state.viewer.as_mut().expect("viewer").cities[0]
        .governor
        .as_mut()
        .expect("governor")
        .enabled = false;
    assert_eq!(
        OperationDialog::toggle_governor(&state, "c")
            .expect("authorized re-enable")
            .payload,
        json!({"city_id":"c","enabled":true,"priority":"growth"}),
        "toggle-on preserves the actual pinned focus"
    );
    state
        .viewer
        .as_mut()
        .expect("viewer")
        .interface
        .legal_actions
        .clear();
    assert!(
        OperationDialog::toggle_governor(&state, "c").is_none(),
        "remembered mayor state is not authorization"
    );
}

#[test]
fn canceling_any_retained_army_member_uses_the_authorized_leader_identity() {
    let viewer:ServerProjection=serde_json::from_value(json!({"revision":6_i64,"turn":2_u32,"tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"plains"}],"units":[{"id":"leader","owner_id":"human","tile_id":"a","health":10_i16,"max_health":10_i16,"type_id":"UNIT_WARRIOR"},{"id":"member","owner_id":"human","tile_id":"a","health":10_i16,"max_health":10_i16,"type_id":"UNIT_WARRIOR"}],"movement_orders":[{"unit_id":"leader","unit_ids":["leader","member"],"destination":{"x":10_u16,"y":10_u16}}],"legal_actions":[{"route":"cancel-movement-order","entity_id":"leader","payload":{"unit_id":"leader"}}]})).expect("retained army fixture");
    let mut state = GameState::from_server_projection(&viewer, 1, 1);
    state.selected_unit = Some(crate::gameplay::model::UnitId::from_server_id("member"));
    let rows = super::context::context_at(&state, crate::gameplay::model::MapPoint::new(1, 1));
    assert!(rows.iter().any(|row| row.label == "Cancel Movement" && matches!(row.intent.clone(),super::context::ContextIntent::Submit(action) if action.payload == json!({"unit_id":"leader"}))),"selecting a non-leading member cancels the exact server journey, not an invented order");
}

#[test]
fn rename_governor_and_custom_queue_forms_emit_current_exact_parameters() {
    let viewer:ServerProjection=serde_json::from_value(json!({"revision":6_i64,"turn":2_u32,"rules":{"units":[{"id":"UNIT_WARRIOR"}],"governor_priorities":["growth","science"]},"cities":[{"id":"c","owner_id":"human","name":"Rome","queue":[{"entry_id":"c:1","id":"UNIT_WARRIOR"}]}],"legal_actions":[{"route":"rename-city","entity_id":"c","payload":{"city_id":"c","name":"Rome"}},{"route":"edit-production","entity_id":"c","payload":{"city_id":"c","entries":[]}},{"route":"set-governor","entity_id":"c","payload":{"city_id":"c","enabled":true,"priority":"growth"}}]})).expect("viewer");
    let state = GameState::from_server_projection(&viewer, 1, 1);
    let mut rename = OperationDialog::input(&state, "c", InputKind::Rename).expect("rename form");
    rename.text = "New Rome".to_owned();
    assert_eq!(
        rename.command(&state).expect("valid rename").payload,
        json!({"city_id":"c","name":"New Rome"}),
        "only selected text/city"
    );
    let governor = OperationDialog::governor(&state, "c").expect("governor choices");
    assert!(
        governor.choices.iter().any(|choice| choice.command.payload
            == json!({"city_id":"c","enabled":true,"priority":"science"})),
        "all pinned focuses, not just suggestion"
    );
    let mut queue = OperationDialog::input(&state, "c", InputKind::Queue).expect("queue editor");
    queue.text = "@c:1, UNIT_WARRIOR, UNIT_WARRIOR".to_owned();
    assert_eq!(
        queue.command(&state).expect("custom queue").payload,
        json!({"city_id":"c","entries":[{"entry_id":"c:1"},{"item_id":"UNIT_WARRIOR"},{"item_id":"UNIT_WARRIOR"}]}),
        "retain identities and repeated new unit orders"
    );
    queue.text = "FORGED".to_owned();
    assert!(
        queue.command(&state).is_none(),
        "cannot invent canonical content"
    );
    let mut newer = state;
    newer.viewer.as_mut().expect("snapshot").revision = 7;
    assert!(
        rename.command(&newer).is_none(),
        "stale input never rebases silently"
    );
}
