//! Isolated gateway receipts for new parameterized and retained operations.
use super::super::authority::state_from_authority_projection;
use super::tests::{authority, payload_after_validating_envelope, respond};
use crate::gameplay::ui_model::{
    InGameUiModel,
    management::{ManagerCommand, ManagerIntent},
};
use serde_json::json;
use std::{net::TcpListener, thread};

#[test]
fn trade_selection_posts_exact_source_to_shared_destination_and_never_invents_queued_route() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("isolated transport");
    let port = listener.local_addr().expect("ephemeral port").port();
    let initial = json!({"revision":17_i64,"turn":3_u32,"contract":{"version":2_u32},"diplomacy":{"trade_transport_points":10_i64,"used_trade_transport_points":0_i64,"relations":[{"target_player_id":"other","treaty":"peace","trade":true}]},"tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"plains"},{"id":"b","x":2_u16,"y":1_u16,"terrain":"plains"},{"id":"d","x":3_u16,"y":1_u16,"terrain":"plains"}],"cities":[{"id":"c1","name":"Rome","owner_id":"account-7","tile_id":"a"},{"id":"c2","name":"Athens","owner_id":"account-7","tile_id":"b"},{"id":"dest","name":"Known target","owner_id":"other","tile_id":"d"}],"trade_routes":[],"legal_actions":[{"route":"create-trade-route","entity_id":"c1","target_id":"dest","payload":{"source_city_id":"c1","destination_city_id":"dest"}},{"route":"create-trade-route","entity_id":"c2","target_id":"dest","payload":{"source_city_id":"c2","destination_city_id":"dest"}}]});
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
    let mut ui = InGameUiModel {
        management: crate::gameplay::ui_model::management::ManagementUi::new(
            crate::gameplay::assets::layout::LayoutBook::load(
                &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"),
            )
            .expect("read-only source"),
        )
        .expect("source cache"),
        ..InGameUiModel::default()
    };
    ui.management.open(
        crate::gameplay::ui_model::management::ManagerKind::Trade,
        &state,
        None,
    );
    let filter = ui
        .management
        .activate_id("TradeTabs.Market.TabPanel.Filters.AllButton", &state)
        .expect("all known authorized candidates");
    super::apply(filter, &mut state, &mut ui, Some(&mut active));
    let select = ui
        .management
        .widgets(&state)
        .into_iter()
        .filter_map(|widget| widget.intent)
        .find_map(|intent| match intent {
            ManagerIntent::Select(key, id)
                if key == "trade-market"
                    && serde_json::from_str::<serde_json::Value>(&id)
                        .is_ok_and(|parts| parts[0] == "c2") =>
            {
                Some(ManagerIntent::Select(key, id))
            }
            _ => None,
        })
        .expect("accessible distinct second-source row");
    super::apply(select, &mut state, &mut ui, Some(&mut active));
    let create = ui
        .management
        .activate_id("TradeTabs.Market.TabPanel.CreateRouteButton", &state)
        .expect("exact selected create");
    super::apply(create, &mut state, &mut ui, Some(&mut active));
    let request = worker
        .join()
        .expect("HTTP fixture worker")
        .expect("request");
    assert!(
        request
            .line
            .starts_with("POST /game/44/create-trade-route "),
        "registered route only"
    );
    assert_eq!(
        payload_after_validating_envelope(&request.body),
        json!({"source_city_id":"c2","destination_city_id":"dest"}),
        "no first source, client good/price/cost or second operation identity"
    );
    assert!(
        state
            .viewer
            .as_ref()
            .expect("queued receipt")
            .interface
            .trade_routes
            .is_empty(),
        "queuing is not route execution or income"
    );
}

#[test]
fn source_readiness_slider_posts_exact_enum_and_keeps_queued_receipt_owned_state() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("isolated gateway");
    let port = listener.local_addr().expect("ephemeral port").port();
    let initial = json!({"revision":17_i64,"turn":3_u32,"diplomacy":{"gold":100_i32,"readiness":"war"},"legal_actions":[{"route":"set-readiness","entity_id":"account-7","target_id":"peace","fields":["readiness"],"payload":{"readiness":"peace"}}]});
    let mut returned = initial.clone();
    returned["revision"] = json!(18_i64);
    returned["planning_status"] = json!("queued");
    let worker = thread::spawn(move || respond(&listener, &returned));
    let mut active = authority(port, initial);
    let mut state = state_from_authority_projection(
        &active,
        active.session.projection.as_ref().expect("viewer"),
    );
    state.popups.clear();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
    let mut ui = InGameUiModel {
        management: crate::gameplay::ui_model::management::ManagementUi::new(
            crate::gameplay::assets::layout::LayoutBook::load(&root).expect("read-only source"),
        )
        .expect("source managers"),
        ..InGameUiModel::default()
    };
    ui.management.open(
        crate::gameplay::ui_model::management::ManagerKind::Unit,
        &state,
        None,
    );
    let intent = ui
        .management
        .activate_id("Slider/value/0", &state)
        .expect("semantic peace index");
    super::apply(intent, &mut state, &mut ui, Some(&mut active));
    let request = worker.join().expect("fake worker").expect("exact request");
    assert!(
        request.line.starts_with("POST /game/44/set-readiness "),
        "real gateway route"
    );
    assert_eq!(
        payload_after_validating_envelope(&request.body),
        json!({"readiness":"peace"}),
        "one captured enum, current session/revision and no fabricated rates"
    );
    assert_eq!(
        state
            .viewer
            .as_ref()
            .expect("receipt")
            .diplomacy
            .as_ref()
            .expect("received nation")
            .facts["readiness"],
        json!("war"),
        "hybrid queued acknowledgement is not completed policy execution"
    );
    assert_eq!(
        ui.management
            .widgets(&state)
            .iter()
            .find(|widget| widget.id == "State")
            .map(|widget| widget.label.as_str()),
        Some("War"),
        "received state, not requested peace, drives the original label"
    );
}

#[test]
fn new_operations_send_exact_envelopes_and_only_display_received_authority() {
    let cases = [
        ("rename-city", json!({"city_id":"c","name":"  New Rome  "})),
        (
            "edit-production",
            json!({"city_id":"c","entries":[{"entry_id":"c:2"},{"item_id":"UNIT_WARRIOR"},{"entry_id":"c:1"}]}),
        ),
        (
            "set-governor",
            json!({"city_id":"c","enabled":true,"priority":"science"}),
        ),
        (
            "set-research-goal",
            json!({"advance_id":"ADVANCE_BALLISTICS"}),
        ),
        ("sleep", json!({"unit_id":"u"})),
        ("wake", json!({"unit_id":"u"})),
        (
            "set-movement-order",
            json!({"unit_id":"u","target_x":10_u16,"target_y":8_u16}),
        ),
        ("cancel-movement-order", json!({"unit_id":"u"})),
    ];
    for (route, payload) in cases {
        let listener = TcpListener::bind("127.0.0.1:0").expect("isolated mock endpoint");
        let port = listener.local_addr().expect("ephemeral port").port();
        let initial = json!({"revision":17_i64,"turn":3_u32,"world":{"width":20_u16,"height":12_u16,"min_x":0_u16,"min_y":0_u16},"tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"plains"}],"units":[{"id":"u","tile_id":"a","owner_id":"account-7"}],"cities":[{"id":"c","tile_id":"a","owner_id":"account-7","name":"Rome","queue":[{"entry_id":"c:1","id":"UNIT_WARRIOR"},{"entry_id":"c:2","id":"UNIT_WARRIOR"}]}],"rules":{"units":[{"id":"UNIT_WARRIOR"}],"governor_priorities":["science"]},"action_intents":[{"route":"set-movement-order","entity_id":"u","payload":{"unit_id":"u"}}],"legal_actions":[{"route":route,"entity_id":if ["sleep","wake","cancel-movement-order"].contains(&route) {"u"} else if route == "set-research-goal" {"account-7"} else {"c"},"payload":payload}]});
        let mut returned = initial.clone();
        returned["revision"] = json!(18_i64);
        if route == "rename-city" {
            returned["cities"][0_usize]["name"] = json!("New Rome");
        }
        if route == "edit-production" {
            returned["cities"][0_usize]["queue"] = json!([{"entry_id":"c:2","id":"UNIT_WARRIOR"},{"entry_id":"c:3","id":"UNIT_WARRIOR"},{"entry_id":"c:1","id":"UNIT_WARRIOR"}]);
        }
        if route == "set-governor" {
            returned["cities"][0_usize]["governor"] = json!({"enabled":true,"priority":"science"});
        }
        if route == "set-research-goal" {
            returned["research"] = json!({"goal_id":"ADVANCE_BALLISTICS"});
        }
        if route == "sleep" {
            returned["units"][0_usize]["asleep"] = json!(true);
        }
        let worker = thread::spawn(move || respond(&listener, &returned));
        let mut active = authority(port, initial);
        let mut state = state_from_authority_projection(
            &active,
            active.session.projection.as_ref().expect("snapshot"),
        );
        state.popups.clear();
        let mut ui = InGameUiModel::default();
        super::apply(
            ManagerIntent::Submit(ManagerCommand {
                revision: 17,
                route: route.to_owned(),
                payload: payload.clone(),
            }),
            &mut state,
            &mut ui,
            Some(&mut active),
        );
        let request = worker
            .join()
            .expect("mock thread")
            .expect("exact operation reached gateway");
        assert!(
            request.line.starts_with(&format!("POST /game/44/{route} ")),
            "exact route: {}",
            request.line
        );
        assert_eq!(
            payload_after_validating_envelope(&request.body),
            payload,
            "no display price, local costs or substituted operation"
        );
        assert_eq!(
            state.cities[0].name,
            if route == "rename-city" {
                "New Rome"
            } else {
                "Rome"
            },
            "receipt replaces state, including trimmed server name"
        );
        assert_eq!(
            active.session.revision(),
            18,
            "authoritative revision applied"
        );
    }
}
