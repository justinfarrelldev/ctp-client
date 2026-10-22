//! Source-attached advice panes share visibility, native geometry and local actions.
use super::{ManagementUi, ManagerIntent, ManagerKind};
use crate::gameplay::{assets::layout::LayoutBook, model::GameState, server::ServerProjection};
use serde_json::json;
use std::path::Path;

/// Accurate viewer fields; foreign actors and inbound routes are not own totals.
fn fixture() -> (ManagementUi, GameState) {
    let viewer:ServerProjection=serde_json::from_value(json!({"revision":6_i64,"turn":1_u32,"contract":{"version":2_u32},"tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"plains"}],"units":[{"id":"u1","type_id":"UNIT_SETTLER","owner_id":"human","tile_id":"a","health":10_i16,"max_health":10_i16,"upkeep":2_i64},{"id":"u2","type_id":"UNIT_SETTLER","owner_id":"human","tile_id":"a","health":10_i16,"max_health":10_i16,"upkeep":3_i64},{"id":"foreign","type_id":"UNIT_HOPLITE","owner_id":"other","tile_id":"a","health":10_i16,"max_health":10_i16,"upkeep":99_i64}],"cities":[{"id":"c","name":"Rome","owner_id":"human","tile_id":"a"}],"diplomacy":{"gold":100_i32,"trade_transport_points":10_i64,"used_trade_transport_points":3_i64},"trade_routes":[{"id":"out","source_city_id":"c","destination_city_id":"elsewhere","gold":8_i64},{"id":"incoming","source_city_id":"elsewhere","destination_city_id":"c","gold":99_i64}]})).expect("source-compatible viewer");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
    (
        ManagementUi::new(LayoutBook::load(&root).expect("read-only installed source"))
            .expect("manager cache"),
        GameState::from_server_projection(&viewer, 1, 1),
    )
}
#[test]
fn military_upkeep_never_confuses_gold_maintenance_with_readiness_production_support() {
    let (mut ui, mut state) = fixture();
    ui.open(ManagerKind::Unit, &state, None);
    let percent = ui
        .widgets(&state)
        .into_iter()
        .find(|widget| widget.id == "Advisor.UpkeepButton")
        .expect("upkeep toggle")
        .label;
    assert!(
        percent.contains("0%"),
        "v2 shield hunger is zero; five gold upkeep is not production support: {percent}"
    );
    ui.apply_local(
        ui.activate_id("Advisor.UpkeepButton", &state)
            .expect("source total toggle"),
    );
    let absolute = ui
        .widgets(&state)
        .into_iter()
        .find(|widget| widget.id == "Advisor.UpkeepButton")
        .expect("absolute upkeep")
        .label;
    assert!(
        absolute.contains('0') && !absolute.contains('%'),
        "source callback switches units without inventing a cost: {absolute}"
    );
    ui.apply_local(
        ui.activate_id("Advisor.UpkeepButton", &state)
            .expect("percentage callback"),
    );
    state.viewer.as_mut().expect("viewer").units[0].facts["shield_hunger"] = json!(5_i64);
    let unavailable = ui
        .widgets(&state)
        .into_iter()
        .find(|widget| widget.id == "Advisor.UpkeepButton")
        .expect("unknown ratio")
        .label;
    assert!(
        unavailable.contains('—'),
        "applied readiness/government/reduction budget is not projected: {unavailable}"
    );
    state.viewer.as_mut().expect("viewer").units[0].facts["needs_no_support"] = json!(true);
    let exempt = ui
        .widgets(&state)
        .into_iter()
        .find(|widget| widget.id == "Advisor.UpkeepButton")
        .expect("exempt unit")
        .label;
    assert!(
        exempt.contains("0%"),
        "explicit no-support units prove zero without a local readiness simulator: {exempt}"
    );
}

#[test]
fn source_advisors_attach_on_open_and_advice_toggles_only_the_attached_pane() {
    let (mut ui, state) = fixture();
    for (kind, button) in [
        (ManagerKind::Unit, "AdviceButton"),
        (ManagerKind::Trade, "ShowAdviceButton"),
    ] {
        ui.open(kind, &state, None);
        let widgets = ui.widgets(&state);
        let main = widgets
            .iter()
            .find(|widget| widget.id.is_empty())
            .expect("main window");
        let advice = widgets
            .iter()
            .find(|widget| widget.id == "Advisor")
            .expect("source shows the attached advice window by default");
        assert_eq!(
            advice.rect.x,
            main.rect.x.saturating_sub(224),
            "source Move(main.X-width,main.Y)"
        );
        assert_eq!(advice.rect.y, main.rect.y, "docked vertical origin");
        assert_eq!(
            (advice.rect.width, advice.rect.height),
            (224_u32, 350_u32),
            "native installed advisor dimensions"
        );
        let before = state.clone();
        let hide = ui
            .activate_id(button, &state)
            .expect("real source advice callback");
        assert!(ui.apply_local(hide), "hide is presentation only");
        assert!(
            ui.widgets(&state)
                .iter()
                .all(|widget| !widget.id.starts_with("Advisor")),
            "hidden advice does not paint or hit test"
        );
        assert_eq!(
            ui.view.as_ref().map(|view| view.kind),
            Some(kind),
            "main window stays open"
        );
        assert!(
            ui.apply_local(ui.activate_id(button, &state).expect("show callback")),
            "show advice again"
        );
        assert!(
            ui.widgets(&state)
                .iter()
                .any(|widget| widget.id == "Advisor.Background"),
            "original source advice artwork restored"
        );
        assert_eq!(state, before, "no authoritative counter changed");
        ui.apply_local(ManagerIntent::Close);
        assert!(
            ui.widgets(&state).is_empty(),
            "closing manager also closes its docked advice"
        );
    }
}
#[test]
fn source_trade_advice_values_are_actual_own_outbound_totals_not_guessed_zeroes() {
    let (mut ui, mut state) = fixture();
    ui.open(ManagerKind::Trade, &state, None);
    for (id, expected) in [
        ("Advisor.Available", "7"),
        ("Advisor.InUse", "3"),
        ("Advisor.Profit", "8"),
        ("Advisor.Routes", "1"),
    ] {
        assert_eq!(
            ui.widgets(&state)
                .iter()
                .find(|widget| widget.id == id)
                .map(|widget| widget.label.as_str()),
            Some(expected),
            "actual source field {id}"
        );
    }
    state
        .viewer
        .as_mut()
        .expect("viewer")
        .diplomacy
        .as_mut()
        .expect("diplomacy")
        .facts = json!({});
    state.viewer.as_mut().expect("viewer").interface.contract = None;
    assert_eq!(
        ui.widgets(&state)
            .iter()
            .find(|widget| widget.id == "Advisor.Available")
            .map(|widget| widget.label.as_str()),
        Some("—"),
        "omitted legacy transport is unknown"
    );
}
#[test]
fn source_military_advice_counts_exclude_foreign_actors_and_do_not_fabricate_rankings() {
    let (mut ui, state) = fixture();
    ui.open(ManagerKind::Unit, &state, None);
    let rows = ui.widgets(&state);
    assert!(
        rows.iter().any(
            |widget| widget.id == "Advisor.List/UNIT_SETTLER/cell/Percent"
                && widget.label == "100%"
        ),
        "two owned settlers, no foreign Hoplite in denominator"
    );
    assert!(
        rows.iter().any(
            |widget| widget.id == "Advisor.List/UNIT_SETTLER/cell/Total" && widget.label == "2"
        ),
        "actual own type count"
    );
    let name = rows
        .iter()
        .find(|widget| widget.id == "Advisor.List/UNIT_SETTLER/cell/Type")
        .expect("name column");
    let percent = rows
        .iter()
        .find(|widget| widget.id == "Advisor.List/UNIT_SETTLER/cell/Percent")
        .expect("percent column");
    assert_eq!(
        percent.rect.x,
        name.rect
            .x
            .saturating_add(i32::try_from(name.rect.width).expect("native width")),
        "source list columns arrange sequentially, not overprinted at x zero"
    );
    assert!(
        rows.iter()
            .any(|widget| widget.id == "Advisor.Advice" && widget.label.contains("unavailable")),
        "private military ranks/planner goals cannot become invented strategic advice"
    );
}
