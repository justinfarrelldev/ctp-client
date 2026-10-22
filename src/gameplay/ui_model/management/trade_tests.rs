//! Trade rows retain source/destination/payload identity, not destination alone.
use super::{ManagementUi, ManagerIntent, ManagerKind};
use crate::gameplay::{assets::layout::LayoutBook, model::GameState, server::ServerProjection};
use serde_json::{Value, json};
use std::path::Path;

/// Read-only original layouts plus an accurate isolated visible viewer projection.
fn fixture() -> (ManagementUi, GameState) {
    let viewer:ServerProjection=serde_json::from_value(json!({"revision":7_i64,"turn":1_u32,"contract":{"version":2_u32},"tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"plains"},{"id":"b","x":2_u16,"y":1_u16,"terrain":"plains"},{"id":"d","x":3_u16,"y":1_u16,"terrain":"plains"}],"units":[{"id":"u","owner_id":"human","tile_id":"a","type_id":"UNIT_SETTLER","health":10_i16,"max_health":10_i16}],"cities":[{"id":"c1","name":"Rome","owner_id":"human","tile_id":"a"},{"id":"c2","name":"Athens","owner_id":"human","tile_id":"b"},{"id":"dest","name":"Known foreign city","owner_id":"other","tile_id":"d"}],"diplomacy":{"trade_transport_points":10_i64,"used_trade_transport_points":0_i64,"relations":[{"target_player_id":"other","treaty":"peace","trade":true}]},"legal_actions":[{"route":"create-trade-route","entity_id":"c1","target_id":"dest","payload":{"source_city_id":"c1","destination_city_id":"dest"}},{"route":"create-trade-route","entity_id":"c2","target_id":"dest","payload":{"source_city_id":"c2","destination_city_id":"dest"}},{"route":"create-trade-route","entity_id":"c1","target_id":"c2","payload":{"source_city_id":"c1","destination_city_id":"c2"}}],"trade_routes":[{"id":"out","source_city_id":"c1","destination_city_id":"c2","good_id":"GOOD_PLAINS_ONE","gold":12_i64,"transport_cost":3_i64},{"id":"incoming","source_city_id":"dest","destination_city_id":"c1","gold":99_i64}]})).expect("viewer fixture");
    let mut ui = ManagementUi::new(
        LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"))
            .expect("installed layouts"),
    )
    .expect("cache");
    let state = GameState::from_server_projection(&viewer, 3, 1);
    ui.open(ManagerKind::Trade, &state, None);
    (ui, state)
}
/// Accessible row selections, not positional pointer clicks.
fn selected_rows(ui: &ManagementUi, state: &GameState, key: &str) -> Vec<String> {
    ui.widgets(state)
        .into_iter()
        .filter_map(|widget| match widget.intent {
            Some(ManagerIntent::Select(kind, id)) if kind == key => Some(id),
            _ => None,
        })
        .collect()
}
#[test]
fn trade_defaults_and_local_destination_filters_match_source_without_mutating_authority() {
    let (mut ui, state) = fixture();
    let own = "TradeTabs.Market.TabPanel.Filters.OwnButton";
    assert!(
        ui.widgets(&state)
            .iter()
            .any(|widget| widget.id == own && widget.selected),
        "source default own-city radio"
    );
    assert_eq!(
        selected_rows(&ui, &state, "trade-market").len(),
        1,
        "only the owned destination by default"
    );
    assert!(
        ui.widgets(&state).iter().any(|widget| widget.id
            == "TradeTabs.Market.TabPanel.Filters.NumCities"
            && widget.label == "1"),
        "source SetNumCities(1), not a missing numeric placeholder"
    );
    ui.apply_local(
        ui.activate_id("TradeTabs.Market.TabPanel.Filters.AllButton", &state)
            .expect("local all-city filter"),
    );
    assert_eq!(
        selected_rows(&ui, &state, "trade-market").len(),
        3,
        "only explicit authorized candidates, never inferred cities"
    );
    ui.apply_local(
        ui.activate_id("TradeTabs.Market.TabPanel.Filters.FriendlyButton", &state)
            .expect("local peace filter"),
    );
    assert_eq!(
        selected_rows(&ui, &state, "trade-market").len(),
        3,
        "projected peace, not proximity, proves friendship"
    );
}
#[test]
fn same_destination_market_rows_have_distinct_exact_payloads_and_source_item_columns() {
    let (mut ui, state) = fixture();
    ui.apply_local(
        ui.activate_id("TradeTabs.Market.TabPanel.Filters.AllButton", &state)
            .expect("all filter"),
    );
    let rows = selected_rows(&ui, &state, "trade-market");
    assert_eq!(rows.len(), 3, "all pairs");
    let chosen = rows
        .iter()
        .find(|id| {
            serde_json::from_str::<Value>(id)
                .is_ok_and(|parts| parts[0] == "c2" && parts[1] == "dest")
        })
        .expect("exact second source to shared destination")
        .clone();
    ui.apply_local(ManagerIntent::Select(
        "trade-market".to_owned(),
        chosen.clone(),
    ));
    let command = ui
        .activate_id("TradeTabs.Market.TabPanel.CreateRouteButton", &state)
        .and_then(|intent| match intent {
            ManagerIntent::Submit(command) => Some(command),
            _ => None,
        })
        .expect("one exact current command");
    assert_eq!(
        command.payload,
        json!({"source_city_id":"c2","destination_city_id":"dest"}),
        "no first candidate or destination-only retargeting"
    );
    assert!(command.valid_for(&state), "current envelope authorization");
    let widgets = ui.widgets(&state);
    let prefix = format!("TradeTabs.Market.TabPanel.List/{chosen}/cell/");
    let city = widgets
        .iter()
        .find(|widget| widget.id == format!("{prefix}City"))
        .expect("source City column");
    let target = widgets
        .iter()
        .find(|widget| widget.id == format!("{prefix}ToCity"))
        .expect("source destination column");
    assert_eq!(city.label, "Athens", "localized projected city name");
    assert_eq!(target.label, "Known foreign city", "only received name");
    assert!(
        target.rect.x > city.rect.x,
        "sequential source columns do not overprint"
    );
    assert_eq!(
        city.rect.height, 20,
        "original item height, not legacy generic 18"
    );
    for name in ["Good", "Price", "Caravans", "Nation"] {
        assert!(
            widgets
                .iter()
                .any(|widget| widget.id == format!("{prefix}{name}") && widget.label == "—"),
            "unprojected {name} is unavailable, not guessed from rules or routes"
        );
    }
}
#[test]
fn summary_uses_outbound_route_identity_and_received_accounting_in_independent_columns() {
    let (mut ui, state) = fixture();
    ui.apply_local(
        ui.activate_id("TradeTabs.Summary.TabButton", &state)
            .expect("summary tab"),
    );
    assert_eq!(
        selected_rows(&ui, &state, "trade-route"),
        vec!["out".to_owned()],
        "inbound/foreign routes are not own sales"
    );
    let widgets = ui.widgets(&state);
    for (name, value) in [
        ("City", "Rome"),
        ("ToCity", "Athens"),
        ("Price", "12"),
        ("Caravans", "3"),
    ] {
        assert!(
            widgets.iter().any(|widget| widget.id
                == format!("TradeTabs.Summary.TabPanel.List/out/cell/{name}")
                && widget.label == value),
            "source {name} uses actual confirmed value"
        );
    }
    ui.apply_local(ManagerIntent::Select(
        "trade-route".to_owned(),
        "out".to_owned(),
    ));
    let break_route = ui
        .widgets(&state)
        .into_iter()
        .find(|widget| widget.id == "TradeTabs.Summary.TabPanel.BreakRouteButton")
        .expect("original break control retained");
    assert!(
        break_route.intent.is_none(),
        "no invented unsupported cancellation endpoint"
    );
    assert!(
        break_route.unavailable.is_some(),
        "explicit protocol gap, not fake enabled button"
    );
}
#[test]
fn reconciliation_drops_disappeared_market_payload_without_aliasing_a_retained_route() {
    let (mut ui, mut state) = fixture();
    let identity = selected_rows(&ui, &state, "trade-market")
        .into_iter()
        .next()
        .expect("one owned route candidate");
    ui.apply_local(ManagerIntent::Select("trade-market".to_owned(), identity));
    ui.apply_local(ManagerIntent::Select(
        "trade-route".to_owned(),
        "out".to_owned(),
    ));
    state
        .viewer
        .as_mut()
        .expect("viewer")
        .interface
        .legal_actions
        .clear();
    ui.reconcile(&state);
    let view = ui.view.as_ref().expect("view");
    assert!(
        !view.selections.contains_key("trade-market"),
        "old command payload disappears rather than binding another source"
    );
    assert_eq!(
        view.selections.get("trade-route").map(String::as_str),
        Some("out"),
        "independent still-observed route survives"
    );
}

#[test]
fn missing_or_targetless_relations_do_not_invent_friendly_destinations() {
    let (mut ui, mut state) = fixture();
    ui.apply_local(
        ui.activate_id("TradeTabs.Market.TabPanel.Filters.FriendlyButton", &state)
            .expect("friend filter"),
    );
    let viewer = state.viewer.as_mut().expect("viewer");
    viewer.diplomacy.as_mut().expect("diplomacy").facts["relations"] = json!([]);
    assert_eq!(
        selected_rows(&ui, &state, "trade-market").len(),
        1,
        "without a received treaty only own destinations are known friendly"
    );
    let updated_viewer = state.viewer.as_mut().expect("viewer");
    updated_viewer.cities[2].stale = true;
    updated_viewer.diplomacy.as_mut().expect("relations").facts["relations"] =
        json!([{"treaty":"peace"}]);
    assert_eq!(
        selected_rows(&ui, &state, "trade-market").len(),
        1,
        "None must not match None as an invented target-owner relationship"
    );
}

#[test]
fn routes_render_actual_good_piracy_and_unknown_zero_transport_semantics_without_quote_guesses() {
    let (mut ui, mut state) = fixture();
    ui.apply_local(
        ui.activate_id("TradeTabs.Summary.TabButton", &state)
            .expect("export"),
    );
    assert!(
        ui.widgets(&state).iter().any(|widget| widget.id
            == "TradeTabs.Summary.TabPanel.List/out/cell/GoodIcon"
            && widget.node.field("image00") == Some("upgi19.rim")),
        "received canonical good selects the original medium good icon"
    );
    let record = &mut state
        .viewer
        .as_mut()
        .expect("viewer")
        .interface
        .trade_routes[0];
    record["gold"] = Value::Null;
    record["transport_cost"] = Value::Null;
    record["pirating_army_id"] = json!("observed-pirate");
    for name in ["Price", "Caravans"] {
        assert!(
            ui.widgets(&state).iter().any(|widget| widget.id
                == format!("TradeTabs.Summary.TabPanel.List/out/cell/{name}")
                && widget.label == "—"),
            "explicit unavailable {name} is not zero"
        );
    }
    assert!(
        ui.widgets(&state).iter().any(|widget| widget.id
            == "TradeTabs.Summary.TabPanel.List/out/cell/Piracy"
            && widget.node.field("image00") == Some("upic02.tga")),
        "only projected piracy draws the source skull"
    );
    let removed = state
        .viewer
        .as_mut()
        .expect("viewer")
        .interface
        .trade_routes[0]
        .as_object_mut()
        .expect("route")
        .remove("transport_cost");
    assert!(
        removed.is_some(),
        "remove explicit null for documented v2 omitempty case"
    );
    assert!(
        ui.widgets(&state).iter().any(|widget| widget.id
            == "TradeTabs.Summary.TabPanel.List/out/cell/Caravans"
            && widget.label == "0"),
        "observed v2 route's integer omission is zero"
    );
    state.viewer.as_mut().expect("viewer").interface.contract = None;
    assert!(
        ui.widgets(&state).iter().any(|widget| widget.id
            == "TradeTabs.Summary.TabPanel.List/out/cell/Caravans"
            && widget.label == "—"),
        "legacy absence remains unknown"
    );
}

#[test]
fn quote_dependent_slider_keeps_source_thumb_but_is_explicitly_unavailable_and_does_not_sort_guessed_prices()
 {
    let (ui, state) = fixture();
    let widgets = ui.widgets(&state);
    let thumb = widgets
        .iter()
        .find(|widget| widget.id == "TradeTabs.Market.TabPanel.Filters.CitiesSlider/thumb")
        .expect("original quantized source thumb still rendered");
    assert_eq!(
        (thumb.rect.width, thumb.rect.height),
        (9_u32, 20_u32),
        "source native thumb dimensions"
    );
    assert!(
        thumb.intent.is_none()
            && thumb
                .unavailable
                .as_ref()
                .is_some_and(|reason| reason.contains("quotes")),
        "no claimed top-N profit ranking without authoritative quotes"
    );
}
