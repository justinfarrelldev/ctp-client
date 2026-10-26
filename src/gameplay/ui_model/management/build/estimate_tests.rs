//! Fileless Build estimates use received effective rates, never a local economy.
use super::super::super::{ManagementUi, ManagerKind};
use crate::gameplay::{assets::layout::LayoutBook, model::GameState, server::ServerProjection};
use serde_json::json;
use std::path::Path;

/// Two identical content IDs deliberately have distinct queue identities/progress.
fn fixture() -> (ManagementUi, GameState) {
    let projection: ServerProjection = serde_json::from_value(json!({
        "revision":7_i64,"turn":4_u32,
        "tiles":[{"id":"t","x":1_u16,"y":1_u16,"terrain":"plains"}],
        "cities":[{"id":"c","owner_id":"human","tile_id":"t","stored_production":60_i64,
            "queue":[{"entry_id":"first","id":"UNIT_WARRIOR","kind":"unit","cost":150_i64},
                     {"entry_id":"second","id":"UNIT_WARRIOR","kind":"unit","cost":150_i64},
                     {"entry_id":"third","id":"UNIT_HOPLITE","kind":"unit","cost":175_i64}],
            "production_forecast":{"revision":7_i64,"status":"estimated","rate":14_i64,"turns":7_u32,"assumption":"constant-current-economy"}}],
        "rules":{"units":[{"id":"UNIT_WARRIOR"},{"id":"UNIT_HOPLITE"}]},
        "legal_actions":[{"route":"queue-production","entity_id":"c","target_id":"UNIT_HOPLITE","payload":{"city_id":"c","item":{"id":"UNIT_HOPLITE","kind":"unit","cost":175_i64}}}]
    })).expect("public owner projection");
    let state = GameState::from_server_projection(&projection, 1_u8, 1_u64);
    let mut ui = ManagementUi::new(LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2")).expect("source LDL")).expect("manager");
    ui.open(ManagerKind::Build, &state, Some("c"));
    (ui, state)
}

/// Read the actual source row descriptor, including its enforced clipping.
fn label(ui: &ManagementUi, state: &GameState, id: &str) -> String {
    ui.widgets(state).into_iter().find(|widget| widget.id == id).expect("source estimate cell").label
}

#[test]
fn build_estimates_distinguish_remaining_head_from_full_cost_duplicate_and_choice() {
    let (ui, state) = fixture();
    for (id, expected) in [
        ("QueueGroup.QueueList/first/cell/StaticThing.Turns", "7"),
        ("QueueGroup.QueueList/second/cell/StaticThing.Turns", "11"),
        ("QueueGroup.QueueList/third/cell/StaticThing.Turns", "13"),
        ("ItemsBox.UnitsList/UNIT_HOPLITE/cell/box.Time", "13"),
    ] {
        assert_eq!(label(&ui, &state, id), expected, "source per-item duration, not cumulative queue completion: {id}");
    }
}

#[test]
fn build_estimates_keep_entire_right_aligned_turn_field_inside_its_queue_row() {
    let (ui, state) = fixture();
    let widgets = ui.widgets(&state);
    let row = widgets.iter().find(|widget| widget.id == "QueueGroup.QueueList/first").expect("row");
    let cell = widgets.iter().find(|widget| widget.id == "QueueGroup.QueueList/first/cell/StaticThing.Turns").expect("turn field");
    assert!(cell.rect.x.saturating_add(i32::try_from(cell.rect.width).expect("small width")) <= row.rect.x.saturating_add(i32::try_from(row.rect.width).expect("small width")), "source's four-pixel wrapper offset must not clip the last digit");
}

#[test]
fn build_estimates_use_received_append_prices_instead_of_catalog_or_stored_progress() {
    let (mut ui, mut state) = fixture();
    let viewer = state.viewer.as_mut().expect("viewer");
    viewer.interface.contract = Some(serde_json::from_value(json!({"version":2_u32,"features":{"production_options":true}})).expect("contract"));
    viewer.interface.rules = Some(json!({"content_hash":"pin","units":[{"id":"UNIT_HOPLITE","production_cost":999_i64}]}));
    viewer.interface.legal_actions = serde_json::from_value(json!([{"route":"edit-production","entity_id":"c","payload":{"city_id":"c"}}])).expect("edit authorization");
    viewer.cities[0].facts["production_options"] = json!({"revision":7_i64,"city_id":"c","content_hash":"pin","operation":"append","status":"complete","items":[{"id":"UNIT_HOPLITE","kind":"unit","cost":140_i64}]});
    ui.reconcile(&state);
    assert_eq!(label(&ui, &state, "ItemsBox.UnitsList/UNIT_HOPLITE/cell/box.Time"), "10", "current append cost/rate, not public catalog999 or remaining80");
    assert_eq!(label(&ui, &state, "QueueGroup.QueueList/third/cell/StaticThing.Turns"), "13", "already queued175 keeps its exact received cost");
}

#[test]
fn build_estimates_preserve_authoritative_completed_head_without_inventing_future_rates() {
    let (ui, mut state) = fixture();
    let forecast = state.viewer.as_mut().expect("viewer").cities[0].production_forecast.as_mut().expect("forecast");
    "completed".clone_into(&mut forecast.status); forecast.turns = Some(0_u32); forecast.rate = None;
    assert_eq!(label(&ui, &state, "QueueGroup.QueueList/first/cell/StaticThing.Turns"), "0", "do not change server completed0 to source minimum1");
    assert_eq!(label(&ui, &state, "QueueGroup.QueueList/second/cell/StaticThing.Turns"), "---", "completion is not a future rate");
    state.viewer.as_mut().expect("viewer").cities[0].facts["queue"][0_usize]["kind"] = json!("capitalization");
    assert_eq!(label(&ui, &state, "QueueGroup.QueueList/first/cell/StaticThing.Turns"), "---", "continuous conversion never has a finite duration");
}

#[test]
fn build_estimates_validate_assumptions_costs_overflow_and_exact_entry_identity() {
    let (ui, mut state) = fixture();
    state.viewer.as_mut().expect("viewer").cities[0].facts["queue"][1_usize]["cost"] = json!(i64::MAX);
    assert_eq!(label(&ui, &state, "QueueGroup.QueueList/second/cell/StaticThing.Turns"), "658812288346769701", "ceiling division cannot overflow cost+rate-1");
    for cost in [json!(-1_i64), json!(0_i64), json!(null), json!("150")] {
        state.viewer.as_mut().expect("viewer").cities[0].facts["queue"][1_usize]["cost"] = cost;
        assert_eq!(label(&ui, &state, "QueueGroup.QueueList/second/cell/StaticThing.Turns"), "---", "missing/malformed/nonpositive cost never becomes free0");
    }
    let forecast = state.viewer.as_mut().expect("viewer").cities[0].production_forecast.as_mut().expect("forecast");
    "different-assumption".clone_into(&mut forecast.assumption);
    assert_eq!(label(&ui, &state, "ItemsBox.UnitsList/UNIT_HOPLITE/cell/box.Time"), "---", "unknown economic assumptions cannot supply a full-cost duration");
    state.viewer.as_mut().expect("viewer").cities[0].facts["queue"][1_usize]["entry_id"] = json!("first");
    assert_eq!(super::text(&state, ui.view.as_ref().expect("view"), "QueueGroup.QueueList", "first", "Turns").as_deref(), Some("---"), "ambiguous repeated identity cannot receive the head forecast");
}

#[test]
fn build_estimates_never_reuse_missing_stale_or_stalled_rates() {
    let (ui, mut state) = fixture();
    for (status, revision, rate) in [("estimated", 6_i64, Some(14_i64)), ("stalled", 7_i64, Some(0_i64)), ("unavailable", 7_i64, Some(14_i64)), ("estimated", 7_i64, None), ("estimated", 7_i64, Some(-1_i64))] {
        let forecast = state.viewer.as_mut().expect("viewer").cities[0].production_forecast.as_mut().expect("forecast");
        status.clone_into(&mut forecast.status); forecast.revision = revision; forecast.rate = rate;
        assert_eq!(label(&ui, &state, "ItemsBox.UnitsList/UNIT_HOPLITE/cell/box.Time"), "---", "unavailable duration is original ASCII, never an unsupported em dash glyph");
        assert_eq!(label(&ui, &state, "QueueGroup.QueueList/second/cell/StaticThing.Turns"), "---", "future entries require a valid current positive rate");
    }
}
