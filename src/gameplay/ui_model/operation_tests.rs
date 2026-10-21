//! Fileless selected city/queue operation forms; authorization stays server-owned.
use super::operations::{OperationDialog,InputKind};
use crate::gameplay::{model::GameState,server::ServerProjection};
use serde_json::json;

#[test]
fn rename_governor_and_custom_queue_forms_emit_current_exact_parameters() {
    let viewer:ServerProjection=serde_json::from_value(json!({"revision":6_i64,"turn":2_u32,"rules":{"units":[{"id":"UNIT_WARRIOR"}],"governor_priorities":["growth","science"]},"cities":[{"id":"c","owner_id":"human","name":"Rome","queue":[{"entry_id":"c:1","id":"UNIT_WARRIOR"}]}],"legal_actions":[{"route":"rename-city","entity_id":"c","payload":{"city_id":"c","name":"Rome"}},{"route":"edit-production","entity_id":"c","payload":{"city_id":"c","entries":[]}},{"route":"set-governor","entity_id":"c","payload":{"city_id":"c","enabled":true,"priority":"growth"}}]})).expect("viewer");
    let state=GameState::from_server_projection(&viewer,1,1);
    let mut rename=OperationDialog::input(&state,"c",InputKind::Rename).expect("rename form");
    rename.text="New Rome".to_owned();
    assert_eq!(rename.command(&state).expect("valid rename").payload,json!({"city_id":"c","name":"New Rome"}),"only selected text/city");
    let governor=OperationDialog::governor(&state,"c").expect("governor choices");
    assert!(governor.choices.iter().any(|choice| choice.command.payload == json!({"city_id":"c","enabled":true,"priority":"science"})),"all pinned focuses, not just suggestion");
    let mut queue=OperationDialog::input(&state,"c",InputKind::Queue).expect("queue editor");
    queue.text="@c:1, UNIT_WARRIOR, UNIT_WARRIOR".to_owned();
    assert_eq!(queue.command(&state).expect("custom queue").payload,json!({"city_id":"c","entries":[{"entry_id":"c:1"},{"item_id":"UNIT_WARRIOR"},{"item_id":"UNIT_WARRIOR"}]}),"retain identities and repeated new unit orders");
    queue.text="FORGED".to_owned();
    assert!(queue.command(&state).is_none(),"cannot invent canonical content");
    let mut newer=state;
    newer.viewer.as_mut().expect("snapshot").revision=7;
    assert!(rename.command(&newer).is_none(),"stale input never rebases silently");
}
