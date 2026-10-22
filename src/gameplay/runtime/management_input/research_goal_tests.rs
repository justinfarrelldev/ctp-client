//! Original Library goal callback through an isolated exact authoritative gateway.
use super::{
    apply,
    tests::{authority, payload_after_validating_envelope, respond},
};
use crate::gameplay::{
    runtime::authority::state_from_authority_projection, ui_model::management::ManagerKind,
};
use sdl2::{
    event::Event,
    keyboard::{Keycode, Mod},
};
use serde_json::json;
use std::{net::TcpListener, thread};
#[test]
fn source_library_unit_goal_chooses_second_enabling_advance_and_queued_receipt_never_changes_research_locally()
 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("isolated research-goal gateway");
    let port = listener.local_addr().expect("port").port();
    let initial = json!({"revision":17_i64,"turn":3_u32,"contract":{"version":2_u32},"research":{"target_id":"ADVANCE_NOW","goal_id":"ADVANCE_OLD","available":["ADVANCE_NOW"]},"rules":{"units":[{"id":"UNIT_WARRIOR","prerequisites":["ADVANCE_FUTURE_A","ADVANCE_FUTURE_B"]}],"production":[{"id":"UNIT_WARRIOR","kind":"unit","prerequisites":["ADVANCE_FUTURE_A","ADVANCE_FUTURE_B"]}]},"legal_actions":[{"route":"set-research-goal","entity_id":"account-7","target_id":"ADVANCE_FUTURE_A","fields":["advance_id"],"payload":{"advance_id":"ADVANCE_FUTURE_A"}},{"route":"set-research-goal","entity_id":"account-7","target_id":"ADVANCE_FUTURE_B","fields":["advance_id"],"payload":{"advance_id":"ADVANCE_FUTURE_B"}},{"route":"set-research-goal","entity_id":"account-7","target_id":"","fields":["advance_id"],"payload":{"advance_id":""}}]});
    let mut returned = initial.clone();
    returned["revision"] = json!(18_i64);
    returned["planning_status"] = json!("queued");
    let worker = thread::spawn(move || respond(&listener, &returned));
    let mut active = authority(port, initial);
    let mut state = state_from_authority_projection(
        &active,
        active.session.projection.as_ref().expect("snapshot"),
    );
    state.popups.clear();
    let mut ui = super::tests::ui();
    ui.management.open(ManagerKind::Library, &state, None);
    ui.management.library.current = Some("UNIT_WARRIOR".to_owned());
    let intent = ui
        .management
        .activate_id("SetGoalButton", &state)
        .expect("original item-to-enable-advance callback");
    apply(intent, &mut state, &mut ui, Some(&mut active));
    let form = ui.operation.as_mut().expect("explicit future goal chooser");
    assert_eq!(
        form.choices.len(),
        3_usize,
        "two actual enabling advances plus clear, never direct UNIT id or all legal goals"
    );
    form.focused = form
        .choices
        .iter()
        .position(|choice| {
            choice.command.payload["advance_id"].as_str() == Some("ADVANCE_FUTURE_B")
        })
        .expect("explicit second prerequisite");
    let event = Event::KeyDown {
        timestamp: 0_u32,
        window_id: 0_u32,
        keycode: Some(Keycode::RETURN),
        scancode: None,
        keymod: Mod::NOMOD,
        repeat: false,
    };
    assert!(
        super::super::operation_input::consume(&event, &mut state, &mut ui, Some(&mut active)),
        "source callback chooser keyboard confirmation"
    );
    let request = worker
        .join()
        .expect("fake worker")
        .expect("one exact envelope");
    assert!(
        request.line.starts_with("POST /game/44/set-research-goal "),
        "persistent goal route, not immediate set-research"
    );
    assert_eq!(
        payload_after_validating_envelope(&request.body),
        json!({"advance_id":"ADVANCE_FUTURE_B"}),
        "exact selected advertised enabling advance and immutable envelope"
    );
    let research = state
        .viewer
        .as_ref()
        .and_then(|viewer| viewer.interface.research.as_ref())
        .expect("received research");
    assert_eq!(
        (research.goal_id.as_str(), research.target_id.as_str()),
        ("ADVANCE_OLD", "ADVANCE_NOW"),
        "queued intent does not locally pick a recursive research path or claim the new goal is executed"
    );
    assert!(
        ui.operation.is_none(),
        "confirmation closes only its local chooser"
    );
}
