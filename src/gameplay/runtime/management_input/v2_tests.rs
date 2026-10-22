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
fn explicit_diplomacy_rejection_posts_false_for_the_same_offer_and_waits_for_authority() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("isolated response gateway");
    let port = listener.local_addr().expect("ephemeral port").port();
    let initial = json!({"revision":17_i64,"turn":3_u32,"contract":{"version":2_u32},"diplomacy":{"relations":[{"target_player_id":"other","treaty":"peace","regard":"neutral"}],"proposals":[{"id":"offer-1","from_player_id":"other","to_player_id":"account-7","kind":"trade"}]},"legal_actions":[{"route":"respond-agreement","entity_id":"offer-1","target_id":"other","fields":["proposal_id","accept"],"payload":{"proposal_id":"offer-1","accept":true}}]});
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
    let mut model = super::tests::ui();
    model.management.open(
        crate::gameplay::ui_model::management::ManagerKind::Diplomacy,
        &state,
        None,
    );
    super::apply(
        ManagerIntent::Select("contact".to_owned(), "other".to_owned()),
        &mut state,
        &mut model,
        Some(&mut active),
    );
    let open = model
        .management
        .activate_id("DiplomacyTabs.Intelligence.TabPanel.MessageButton", &state)
        .expect("reachable original proposal button");
    super::apply(open, &mut state, &mut model, Some(&mut active));
    let form = model
        .operation
        .take()
        .expect("explicit source workspace choices");
    let reject = form
        .choices
        .into_iter()
        .find(|choice| choice.command.payload["accept"].as_bool() == Some(false))
        .expect("reject, not implicit acceptance")
        .command;
    super::apply(
        ManagerIntent::Submit(reject),
        &mut state,
        &mut model,
        Some(&mut active),
    );
    let request = worker
        .join()
        .expect("isolated worker")
        .expect("one response envelope");
    assert!(
        request.line.starts_with("POST /game/44/respond-agreement "),
        "actual registered route"
    );
    assert_eq!(
        payload_after_validating_envelope(&request.body),
        json!({"proposal_id":"offer-1","accept":false}),
        "same authorized proposal, explicit false and no unrelated target fields"
    );
    let received = &state
        .viewer
        .as_ref()
        .expect("queued authority")
        .diplomacy
        .as_ref()
        .expect("diplomacy")
        .facts;
    assert_eq!(
        received["proposals"].as_array().map(Vec::len),
        Some(1_usize),
        "queuing is not local deletion of the proposal"
    );
    assert_eq!(
        received["relations"][0]["treaty"],
        json!("peace"),
        "no speculative acceptance/trade treaty"
    );
}

#[test]
fn national_selected_city_mayor_and_quoted_rush_keep_exact_envelopes_and_only_received_state() {
    for (route, control, payload) in [
        (
            "set-governor",
            "GovernorSelect.Toggle",
            json!({"city_id":"c2","enabled":false}),
        ),
        ("rush-buy", "RushBuyButton", json!({"city_id":"c2"})),
    ] {
        let listener = TcpListener::bind("127.0.0.1:0").expect("isolated gateway");
        let port = listener.local_addr().expect("ephemeral port").port();
        let (target, fields) = if route == "set-governor" {
            ("disabled", json!(["city_id", "enabled"]))
        } else {
            ("UNIT_HOPLITE", json!(["city_id"]))
        };
        let initial = json!({"revision":17_i64,"turn":3_u32,"contract":{"version":2_u32},"tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"plains"},{"id":"b","x":2_u16,"y":1_u16,"terrain":"plains"}],"cities":[{"id":"c1","name":"Rome","owner_id":"account-7","tile_id":"a","population":1_u32},{"id":"c2","name":"Athens","owner_id":"account-7","tile_id":"b","population":2_u32,"governor":{"enabled":true,"priority":"production"},"queue":[{"entry_id":"c2:1","id":"UNIT_HOPLITE","kind":"unit","cost":20_i64}],"rush_buy_quote":{"revision":17_i64,"status":"available","cost":30_i64}}],"diplomacy":{"gold":100_i32},"rules":{"governor_priorities":["production","science"]},"legal_actions":[{"route":route,"entity_id":"c2","target_id":target,"fields":fields,"payload":payload}]});
        let mut returned = initial.clone();
        returned["revision"] = json!(18_i64);
        returned["planning_status"] = json!("queued");
        returned["cities"][1]["rush_buy_quote"]["revision"] = json!(18_i64);
        let worker = thread::spawn(move || respond(&listener, &returned));
        let mut active = authority(port, initial);
        let mut state = state_from_authority_projection(
            &active,
            active.session.projection.as_ref().expect("viewer"),
        );
        state.popups.clear();
        let mut model = super::tests::ui();
        model.management.open(
            crate::gameplay::ui_model::management::ManagerKind::National,
            &state,
            None,
        );
        super::apply(
            ManagerIntent::Select("city".to_owned(), "c2".to_owned()),
            &mut state,
            &mut model,
            Some(&mut active),
        );
        let tab = model
            .management
            .activate_id("TabGroup.Tab2.TabButton", &state)
            .expect("original status tab");
        super::apply(tab, &mut state, &mut model, Some(&mut active));
        let command = model
            .management
            .activate_id(&format!("TabGroup.Tab2.TabPanel.{control}"), &state)
            .expect("exact selected city control");
        super::apply(command, &mut state, &mut model, Some(&mut active));
        let request = worker.join().expect("HTTP worker").expect("one request");
        assert!(
            request.line.starts_with(&format!("POST /game/44/{route} ")),
            "registered selected-city route"
        );
        assert_eq!(
            payload_after_validating_envelope(&request.body),
            payload,
            "no first-city fallback or client economics"
        );
        let viewer = state.viewer.as_ref().expect("received queued snapshot");
        assert!(
            viewer.cities[1]
                .governor
                .as_ref()
                .expect("received mayor")
                .enabled,
            "queued request is not completed mayor change"
        );
        assert_eq!(
            viewer.diplomacy.as_ref().expect("received treasury").gold,
            100_i32,
            "no optimistic rush deduction"
        );
        assert_eq!(
            viewer.cities[1].facts["queue"][0]["entry_id"],
            json!("c2:1"),
            "queued receipt does not pop/rebuild production"
        );
    }
}

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
