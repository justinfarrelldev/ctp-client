//! Accessible keyboard details/return/snapshot recovery without any transport.
use super::{apply, consume};
use crate::gameplay::{
    model::GameState, runtime::MovementInputState, server::ServerProjection,
    ui_model::management::ManagerKind,
};
use sdl2::{
    event::Event,
    keyboard::{Keycode, Mod},
};
use serde_json::json;
/// Key events use the same native dispatch as the game, not coordinate clicking.
fn key(keycode: Keycode) -> Event {
    Event::KeyDown {
        timestamp: 0_u32,
        window_id: 0_u32,
        keycode: Some(keycode),
        scancode: None,
        keymod: Mod::NOMOD,
        repeat: false,
    }
}
#[test]
fn second_contact_details_keyboard_tabs_escape_and_removed_contact_recover_without_transport_or_retargeting()
 {
    let viewer:ServerProjection=serde_json::from_value(json!({"revision":7_i64,"turn":3_u32,"contract":{"version":2_u32},"diplomacy":{"relations":[{"target_player_id":"other-1","treaty":"peace","regard":"neutral"},{"target_player_id":"other-2","treaty":"trade","trade":true,"regard":"neutral"}]}})).expect("viewer-safe contact contract");
    let mut state = GameState::from_server_projection(&viewer, 1, 1);
    state.popups.clear();
    let mut ui = super::tests::ui();
    ui.management.open(ManagerKind::Diplomacy, &state, None);
    let mut movement = MovementInputState::default();
    let select = ui
        .management
        .activate_id("DiplomacyTabs.Intelligence.TabPanel.List/other-2", &state)
        .expect("exact accessible second contact");
    apply(select, &mut state, &mut ui, None);
    ui.management.focused = Some("DiplomacyTabs.Intelligence.TabPanel.DetailsButton".to_owned());
    assert!(
        consume(
            &key(Keycode::RETURN),
            &mut state,
            &mut ui,
            &mut movement,
            None
        ),
        "keyboard activation consumes manager event"
    );
    assert_eq!(
        ui.management.view.as_ref().map(|view| view.kind),
        Some(ManagerKind::DiplomacyDetails),
        "original details window, not proposal submission"
    );
    assert_eq!(
        ui.management
            .view
            .as_ref()
            .and_then(|view| view.selections.get("contact"))
            .map(String::as_str),
        Some("other-2"),
        "exact selected namespace"
    );
    ui.management.focused = Some("TabGroup.Tab3.TabButton".to_owned());
    assert!(
        consume(
            &key(Keycode::RETURN),
            &mut state,
            &mut ui,
            &mut movement,
            None
        ),
        "original Science tab remains keyboard-accessible"
    );
    assert!(
        ui.management.widgets(&state).iter().any(|widget| widget.id
            == "TabGroup.Tab3.TabPanel.YourAdvancesList/unavailable"
            && widget.intent.is_none()),
        "own-minus-foreign set is unavailable without foreign knowledge, not every own advance"
    );
    assert!(
        consume(
            &key(Keycode::ESCAPE),
            &mut state,
            &mut ui,
            &mut movement,
            None
        ),
        "ESC returns one window"
    );
    assert_eq!(
        ui.management.view.as_ref().map(|view| view.kind),
        Some(ManagerKind::Diplomacy),
        "parent remains open"
    );
    assert_eq!(
        ui.management
            .view
            .as_ref()
            .and_then(|view| view.selections.get("contact"))
            .map(String::as_str),
        Some("other-2"),
        "parent identity is preserved"
    );
    let open = ui
        .management
        .activate_id("DiplomacyTabs.Intelligence.TabPanel.DetailsButton", &state)
        .expect("same known contact");
    apply(open, &mut state, &mut ui, None);
    let received = state.viewer.as_mut().expect("receipt");
    received.revision = 8_i64;
    received.diplomacy.as_mut().expect("relations").facts["relations"] =
        json!([{"target_player_id":"other-1","treaty":"peace","regard":"neutral"}]);
    ui.management.reconcile(&state);
    assert_eq!(
        ui.management.view.as_ref().map(|view| view.kind),
        Some(ManagerKind::Diplomacy),
        "removed detail target restores parent"
    );
    assert!(
        ui.management
            .view
            .as_ref()
            .is_some_and(|view| !view.selections.contains_key("contact")),
        "do not select other-1 on recovery"
    );
    assert!(ui.management.parents.is_empty(), "no parent stack leak");
}
