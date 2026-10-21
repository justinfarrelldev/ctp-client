//! Stable-entry editor selections remain distinct from library content topics.
use super::{ManagerKind, ManagerView, queue};
use crate::gameplay::{model::GameState, server::ServerProjection};
use alloc::collections::BTreeMap;
use serde_json::json;

#[test]
fn atomic_queue_edits_keep_duplicate_units_distinct_and_never_redefine_costs() {
    let projection:ServerProjection=serde_json::from_value(json!({"revision":9_i64,"turn":2_u32,"tiles":[{"id":"a","x":0_u16,"y":0_u16,"terrain":"plains"}],"cities":[{"id":"c","tile_id":"a","owner_id":"human","stored_production":40_i64,"queue":[{"entry_id":"c:1","id":"UNIT_WARRIOR"},{"entry_id":"c:2","id":"UNIT_WARRIOR"}]}],"rules":{"units":[{"id":"UNIT_WARRIOR"}]},"legal_actions":[{"route":"edit-production","entity_id":"c","payload":{"city_id":"c","entries":[{"entry_id":"c:1"},{"entry_id":"c:2"}]}}]})).expect("viewer");
    let state=GameState::from_server_projection(&projection,1,1);
    let view=ManagerView{kind:ManagerKind::Build,city_id:Some("c".to_owned()),selections:BTreeMap::from([("queue".to_owned(),"c:2".to_owned()),("buildable".to_owned(),"UNIT_WARRIOR".to_owned())]),scroll:BTreeMap::new()};
    for (path,expected) in [
        ("QueueGroup.RemoveButton",json!([{"entry_id":"c:1"}])),
        ("QueueGroup.UpButton",json!([{"entry_id":"c:2"},{"entry_id":"c:1"}])),
        ("ItemsBox.InsertButton",json!([{"entry_id":"c:1"},{"item_id":"UNIT_WARRIOR"},{"entry_id":"c:2"}])),
        ("ItemsBox.AddButton",json!([{"entry_id":"c:1"},{"entry_id":"c:2"},{"item_id":"UNIT_WARRIOR"}])),
        ("QueueGroup.ClearButton",json!([]))] {
        let command=queue::request(&state,&view,path).expect("enabled atomic operation");
        assert_eq!(command.route,"edit-production","one server transaction");
        assert_eq!(command.payload,json!({"city_id":"c","entries":expected}),"exact stable selection for {path}");
        assert!(command.valid_for(&state),"current authoritative shape");
    }
    assert!(queue::request(&state,&view,"QueueGroup.DownButton").is_none(),"tail cannot move down");
    assert_eq!(state.viewer.as_ref().expect("source").cities[0].facts["stored_production"],40_i64,"constructing requests never changes production");
}
