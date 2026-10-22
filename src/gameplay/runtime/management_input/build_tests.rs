//! Original selected-head Build purchase through native input and an isolated gateway.
use super::tests::{authority, key, payload_after_validating_envelope, respond, ui};
use crate::gameplay::{runtime::{MovementInputState, authority::state_from_authority_projection}, ui_model::management::ManagerKind};
use sdl2::keyboard::Keycode;
use serde_json::json;
use std::{net::TcpListener, thread};

/*
 * Source trace: ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::
 * UpdateButtons/RushBuy/RushBuyCallback require the explicitly selected queue
 * index0. editqueue.ldl::BuildEditorWindow.{QueueGroup.QueueList,RushBuyButton,
 * RushBuyCost} supplies the real controls; the native Return dispatcher below
 * uses those semantic IDs, not coordinate targets or a hidden first queue item.
 * This accurate queued receipt keeps gold/production unchanged. The loopback
 * gateway is an isolated test double, never the configured/local game server.
 */
#[test]
fn build_rush_native_selected_head_posts_exact_city_and_queued_receipt_does_not_spend_or_complete() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("isolated head-rush gateway");
    let port = listener.local_addr().expect("ephemeral port").port();
    let initial = json!({"revision":17_i64,"turn":3_u32,"contract":{"version":2_u32},"tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"plains"},{"id":"b","x":2_u16,"y":1_u16,"terrain":"plains"}],"cities":[{"id":"c1","owner_id":"account-7","tile_id":"a","name":"Other","queue":[]},{"id":"c2","owner_id":"account-7","tile_id":"b","name":"Rome","queue":[{"entry_id":"c2:head","id":"UNIT_HOPLITE","kind":"unit","cost":80_i64},{"entry_id":"c2:duplicate","id":"UNIT_HOPLITE","kind":"unit","cost":80_i64}],"stored_production":25_i64,"rush_buy_quote":{"revision":17_i64,"status":"available","cost":30_i64}}],"diplomacy":{"gold":100_i32},"legal_actions":[{"route":"rush-buy","entity_id":"c2","target_id":"UNIT_HOPLITE","fields":["city_id"],"payload":{"city_id":"c2"}}]});
    let mut returned = initial.clone();
    returned["revision"] = json!(18_i64);
    returned["planning_status"] = json!("queued");
    returned["cities"][1]["rush_buy_quote"]["revision"] = json!(18_i64);
    let mut active = authority(port, initial);
    let mut state = state_from_authority_projection(&active, active.session.projection.as_ref().expect("viewer"));
    state.popups.clear();
    let mut model = ui();
    model.management.open(ManagerKind::Build, &state, Some("c2"));
    let mut movement = MovementInputState::default();
    for entry in ["c2:duplicate", "c2:head"] {
        model.management.focused = Some(format!("QueueGroup.QueueList/{entry}"));
        assert!(super::consume(&key(Keycode::RETURN), &mut state, &mut model, &mut movement, Some(&mut active)), "native exact queue-entry selection");
        assert_eq!(model.management.activate_id("RushBuyButton", &state).is_some(), entry == "c2:head", "same content in another entry never acquires head permission");
    }
    let before_queue = state.viewer.as_ref().expect("viewer").cities.iter().find(|city| city.id == "c2").expect("exact city").facts["queue"].clone();
    let worker = thread::spawn(move || respond(&listener, &returned));
    model.management.focused = Some("RushBuyButton".to_owned());
    assert!(super::consume(&key(Keycode::RETURN), &mut state, &mut model, &mut movement, Some(&mut active)), "native original rush callback");
    let request = worker.join().expect("isolated worker").expect("one request");
    assert!(request.line.starts_with("POST /game/44/rush-buy "), "registered source operation");
    assert_eq!(payload_after_validating_envelope(&request.body), json!({"city_id":"c2"}), "exact selected city, no client quote/cost/content/execution fields");
    assert_eq!(state.gold, 100_i32, "queued receipt is not local gold spending");
    let received = state.viewer.as_ref().expect("received viewer").cities.iter().find(|city| city.id == "c2").expect("same city");
    assert_eq!(received.facts["queue"], before_queue, "queued purchase doesn't remove/complete/reorder head");
    assert_eq!(received.facts["stored_production"], 25_i64, "never fill production optimistically");
    assert_eq!(model.management.view.as_ref().and_then(|view| view.selections.get("queue")).map(String::as_str), Some("c2:head"), "receipt preserves exact selected entry");
}
