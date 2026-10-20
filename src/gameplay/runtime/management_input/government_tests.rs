//! Government chooser and command handoff share source identities, not city state.

use super::{
    apply, consume,
    tests::{authority, key, payload_after_validating_envelope, respond, snapshot, ui},
};
use crate::gameplay::model::GameState;
use crate::gameplay::runtime::{MovementInputState, authority::state_from_authority_projection};
use crate::gameplay::ui_model::{
    InGameUiModel,
    management::{ManagerIntent, ManagerKind},
};
use sdl2::keyboard::Keycode;
use serde_json::{Value, json};
use std::net::TcpListener;
use std::thread;

/// Exact public rules and legal target; no installed rule/economy substitution.
fn projection() -> Value {
    let mut value = snapshot();
    value["diplomacy"]["government_id"] = json!("GOVERNMENT_TYRANNY");
    value["rules"] = json!({"governments":[{"id":"GOVERNMENT_TYRANNY","too_many_cities_threshold":12_i64},{"id":"GOVERNMENT_REPUBLIC","too_many_cities_threshold":20_i64}]});
    value["legal_actions"] = json!([{"route":"set-government","entity_id":"account-7","target_id":"GOVERNMENT_REPUBLIC","fields":["government_id"],"payload":{"government_id":"GOVERNMENT_REPUBLIC"}}]);
    value
}

/// Opens the original comparison tab and detached picker through semantic IDs.
fn open(ui: &mut InGameUiModel, state: &mut GameState) {
    ui.management
        .open(ManagerKind::Empire, state, Some("city-7"));
    apply(
        ManagerIntent::Select("tab:TabGroup".to_owned(), "Tab2".to_owned()),
        state,
        ui,
        None,
    );
    let intent = ui
        .management
        .activate_id("TabGroup.Tab2.TabPanel.CompareGovernment.autobutton", state)
        .expect("source dropdown");
    apply(intent, state, ui, None);
}

#[test]
fn government_popup_keyboard_accepts_government_without_changing_city_or_world() {
    let authority = authority(9_u16, projection());
    let mut state = state_from_authority_projection(
        &authority,
        authority.session.projection.as_ref().expect("projection"),
    );
    state.popups.clear();
    let before = state.clone();
    let mut ui = ui();
    open(&mut ui, &mut state);
    let mut movement = MovementInputState::default();
    assert!(
        consume(
            &key(Keycode::HOME),
            &mut state,
            &mut ui,
            &mut movement,
            None
        ),
        "explicit highlight"
    );
    assert!(
        consume(
            &key(Keycode::RETURN),
            &mut state,
            &mut ui,
            &mut movement,
            None
        ),
        "explicit comparison selection"
    );
    let view = ui.management.view.as_ref().expect("Empire");
    assert_eq!(
        view.selections.get("government").map(String::as_str),
        Some("GOVERNMENT_REPUBLIC"),
        "government identity must not pass through owned-city validation"
    );
    assert_eq!(
        view.city_id.as_deref(),
        Some("city-7"),
        "independent city survives"
    );
    assert!(
        !view.selections.contains_key("dropdown"),
        "detached popup dismissed"
    );
    assert_eq!(state, before, "comparison remains local");
}

#[test]
fn stale_government_rows_are_rejected_at_actual_semantic_dispatch() {
    let authority = authority(9_u16, projection());
    let mut state = state_from_authority_projection(
        &authority,
        authority.session.projection.as_ref().expect("projection"),
    );
    state.popups.clear();
    let mut ui = ui();
    open(&mut ui, &mut state);
    let intent = ui
        .management
        .activate_id(
            "TabGroup.Tab2.TabPanel.CompareGovernment.window.listbox/GOVERNMENT_REPUBLIC",
            &state,
        )
        .expect("current row");
    state
        .viewer
        .as_mut()
        .expect("viewer")
        .interface
        .legal_actions
        .clear();
    apply(intent, &mut state, &mut ui, None);
    assert!(
        !ui.management
            .view
            .as_ref()
            .expect("Empire")
            .selections
            .contains_key("government"),
        "a removed capability cannot establish a comparison target"
    );
    assert!(
        ui.status_text.contains("no longer eligible"),
        "explicit stale-government feedback"
    );
}

#[test]
fn government_enactment_sends_exact_current_target_and_displays_only_its_receipt() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("isolated listener");
    let mut authority = authority(listener.local_addr().expect("address").port(), projection());
    let mut state = state_from_authority_projection(
        &authority,
        authority.session.projection.as_ref().expect("projection"),
    );
    state.popups.clear();
    let selected = state.selected_unit;
    let mut ui = ui();
    open(&mut ui, &mut state);
    let choose = ui
        .management
        .activate_id(
            "TabGroup.Tab2.TabPanel.CompareGovernment.window.listbox/GOVERNMENT_REPUBLIC",
            &state,
        )
        .expect("government row");
    apply(choose, &mut state, &mut ui, None);
    let enact = ui
        .management
        .activate_id("TabGroup.Tab2.TabPanel.EnactGovernment", &state)
        .expect("exact enactment");
    let mut after = projection();
    after["revision"] = json!(18_i64);
    after["diplomacy"]["government_id"] = json!("GOVERNMENT_REPUBLIC");
    after["legal_actions"] = json!([]);
    let worker = thread::spawn(move || respond(&listener, &after));
    apply(enact, &mut state, &mut ui, Some(&mut authority));
    let request = worker.join().expect("stub thread").expect("command");
    assert_eq!(
        request.line, "POST /game/44/set-government HTTP/1.1\r\n",
        "registered government route"
    );
    assert_eq!(
        payload_after_validating_envelope(&request.body),
        json!({"government_id":"GOVERNMENT_REPUBLIC"}),
        "exact session/revision/identity payload"
    );
    assert_eq!(authority.session.revision(), 18_i64, "accepted receipt");
    assert_eq!(state.selected_unit, selected, "unit selection survives");
    assert_eq!(
        ui.management
            .widgets(&state)
            .iter()
            .find(|widget| widget.id == "TabGroup.Tab2.TabPanel.CurrentGovernment")
            .map(|widget| widget.label.as_str()),
        Some("Republic"),
        "receipt-owned government name"
    );
    assert!(
        ui.management
            .activate_id("TabGroup.Tab2.TabPanel.EnactGovernment", &state)
            .is_none(),
        "enacted current government cannot be submitted again"
    );
    assert!(state.popups.is_empty(), "no hidden failure");
}
