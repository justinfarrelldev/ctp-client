//! Complete append options are authority, never the public catalog or a feature alone.
use super::{Harness, buildable_fixture, begin_row_press, key, manager_event};
use crate::gameplay::{model::NetworkStatus, ui_model::management::{ManagerIntent, ManagerKind}};
use sdl2::keyboard::Keycode;
use serde_json::json;
use std::sync::mpsc::TryRecvError;

/// Complete owned-city query with exact snapshot identities, including a nonempty queue.
fn options() -> Harness {
    let mut harness=buildable_fixture();
    let viewer=harness.state.viewer.as_mut().expect("viewer");
    viewer.interface.contract.as_mut().expect("v2").features.insert("production_options".to_owned(),true);
    viewer.interface.rules.as_mut().expect("pin")["content_hash"]=json!("pin-7");
    let city=viewer.cities.iter_mut().find(|city|city.id=="c2").expect("city");
    city.facts["production_options"]=json!({"revision":7_i64,"city_id":"c2","content_hash":"pin-7","operation":"append","status":"complete","items":[{"id":"UNIT_SETTLER","kind":"unit","cost":330_i64}]});
    harness.authority.session.projection=harness.state.viewer.clone();harness
}

#[test]
fn repaired_production_complete_empty_is_empty_despite_catalog_and_old_queue_types() {
    let mut harness=options();
    harness.state.viewer.as_mut().expect("viewer").cities[1_usize].facts["production_options"]["items"]=json!([]);
    let rows=harness.ui.management.widgets(&harness.state);
    assert!(!rows.iter().any(|row|matches!(row.intent.clone(),Some(ManagerIntent::Select(role,_)) if role=="buildable")),"complete-empty must not fall back to catalog, old action or queued content");
    harness.ui.management.apply_local(ManagerIntent::Select("buildable".to_owned(),"UNIT_WARRIOR".to_owned()));
    assert!(harness.ui.management.activate_id("ItemsBox.AddButton",&harness.state).is_none(),"forged/stale catalog selection cannot Add");
}

#[test]
fn repaired_production_nonempty_queue_add_uses_exact_option_and_keeps_received_queue_pending() {
    let mut harness=options();let received=harness.state.viewer.clone();
    begin_row_press(&mut harness,"ItemsBox.UnitsList/UNIT_SETTLER");
    harness.ui.management.focused=Some("ItemsBox.AddButton".to_owned());manager_event(&mut harness,&key(Keycode::RETURN));
    let command=harness.jobs.as_ref().expect("jobs").try_recv().expect("one Add");
    assert_eq!(command.body["entries"],json!([{"entry_id":"c2:head"},{"entry_id":"c2:future"},{"item_id":"UNIT_SETTLER"}]),"one advertised append retains exact ordered identities");
    assert_eq!(command.body["expected_revision"],7_i64,"exact displayed revision");
    assert_eq!(harness.state.viewer,received,"enqueue isn't execution");assert_eq!(harness.state.network,NetworkStatus::Pending,"pending");
    assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"single immutable operation");
}

#[test]
fn repaired_production_unavailable_stale_wrong_pin_city_operation_or_partial_never_unlocks_catalog() {
    for (field,value) in [("status",json!("unavailable")),("status",json!("partial")),("revision",json!(6_i64)),("city_id",json!("c1")),("content_hash",json!("other-pin")),("operation",json!("insert"))] {
        let mut harness=options();harness.state.viewer.as_mut().expect("viewer").cities[1_usize].facts["production_options"][field]=value;
        harness.ui.management.apply_local(ManagerIntent::Select("buildable".to_owned(),"UNIT_SETTLER".to_owned()));
        assert!(harness.ui.management.activate_id("ItemsBox.AddButton",&harness.state).is_none(),"bad {field} cannot Add");
        assert!(!harness.ui.management.widgets(&harness.state).iter().any(|row|matches!(row.intent.clone(),Some(ManagerIntent::Select(role,_)) if role=="buildable")),"bad {field} cannot publish partial candidates");
        begin_row_press(&mut harness,"QueueGroup.QueueList/c2:future");
        assert!(harness.ui.management.activate_id("QueueGroup.RemoveButton",&harness.state).is_some(),"append query unavailability does not prohibit independent exact future removal");
    }
}

#[test]
fn repaired_production_custom_authoring_remains_public_catalog_not_city_complete_empty() {
    let mut harness=options();harness.state.viewer.as_mut().expect("viewer").cities[1_usize].facts["production_options"]["items"]=json!([]);
    let intent=harness.ui.management.activate_id("NormalModeButtons.CustomButton",&harness.state).expect("local Custom");
    super::management_input::apply(intent,&mut harness.state,&mut harness.ui,Some(&mut harness.authority));
    assert_eq!(harness.ui.management.view.as_ref().map(|view|view.kind),Some(ManagerKind::Build),"source local editor");
    assert!(harness.ui.management.widgets(&harness.state).iter().any(|row|row.id=="ItemsBox.UnitsList/UNIT_WARRIOR"),"public authoring does not require city append eligibility");
}
