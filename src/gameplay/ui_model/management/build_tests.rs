//! Released EditQueue queue-head/price controls with injected owner-only facts.
use super::{ManagementUi, ManagerIntent, ManagerKind};
use crate::gameplay::{assets::layout::LayoutBook, model::{GameState, NetworkStatus}, server::{ServerCityProjection, ServerProjection}};
use serde_json::json;
use std::path::Path;

/*
 * Original behavior/style provenance:
 * ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::UpdateButtons only enables
 * Rush Buy for selected queue index0, known nonnegative GetOvertimeCost, funds
 * and not AlreadyBoughtFront. RushBuy repeats the queue-head checks; it does not
 * buy whichever content name is selected elsewhere. UpdateButtons shows the
 * actual head price even if unaffordable, and --- for no/future/bought selection.
 * ctp2/ctp2_data/english/uidata/layouts/editqueue.ldl::BuildEditorWindow.
 * {RushBuyButton,RushBuyCost,QueueGroup.QueueList} supplies original controls,
 * fonts and positions. Tests exercise shared descriptors, no scaling or I/O.
 * The server quote/revision/legal-city envelope replaces local price arithmetic
 * and CityData mutation; equal content IDs are deliberately different entries.
 */

/// Explicit non-first owned city and two equal-content, distinct queue identities.
fn fixture() -> (ManagementUi, GameState) {
    let projection: ServerProjection = serde_json::from_value(json!({
        "revision":7_i64,"turn":1_u32,"contract":{"version":2_u32},
        "tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"plains"},{"id":"b","x":2_u16,"y":1_u16,"terrain":"plains"}],
        "cities":[{"id":"c1","owner_id":"human","tile_id":"a","name":"Other city","queue":[]},{"id":"c2","owner_id":"human","tile_id":"b","name":"Rome","queue":[{"entry_id":"c2:head","id":"UNIT_HOPLITE","kind":"unit","cost":80_i64},{"entry_id":"c2:duplicate","id":"UNIT_HOPLITE","kind":"unit","cost":80_i64},{"entry_id":"c2:granary","id":"IMPROVE_GRANARY","kind":"building","cost":140_i64}],"rush_buy_quote":{"revision":7_i64,"status":"available","cost":30_i64}}],
        "diplomacy":{"gold":100_i32},"rules":{"units":[{"id":"UNIT_SETTLER","production_cost":330_i64}]},
        "legal_actions":[{"route":"rush-buy","entity_id":"c2","target_id":"UNIT_HOPLITE","payload":{"city_id":"c2"}},{"route":"queue-production","entity_id":"c2","target_id":"UNIT_SETTLER","payload":{"city_id":"c2","item":{"id":"UNIT_SETTLER","kind":"unit","cost":330_i64}}}]
    })).expect("typed own-city/quote/queue fixture");
    let state = GameState::from_server_projection(&projection, 1_u8, 1_u64);
    let book = LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2")).expect("read-only original source layouts");
    let mut ui = ManagementUi::new(book).expect("original Build controls");
    ui.open(ManagerKind::Build, &state, Some("c2"));
    (ui, state)
}

/// Change only the explicitly identified fixture city; never the first city.
fn city(state: &mut GameState) -> &mut ServerCityProjection {
    state.viewer.as_mut().expect("viewer").cities.iter_mut().find(|city| city.id == "c2").expect("exact chosen city")
}

/// Select using the same queue-entry descriptor as native and accessible input.
fn select(ui: &mut ManagementUi, state: &GameState, entry: &str) {
    let intent = ui.activate_id(&format!("QueueGroup.QueueList/{entry}"), state).expect("exact visible queue entry");
    assert!(ui.apply_local(intent), "local queue selection, not a production order");
}

/// Original numeric price box, not the neighboring city panel's separate caption.
fn price(ui: &ManagementUi, state: &GameState) -> String {
    ui.widgets(state).into_iter().find(|widget| widget.id == "RushBuyCost").expect("source price control").label
}

#[test]
fn build_rush_requires_explicit_exact_queue_head_not_unselected_or_same_content_future_entry() {
    let (mut ui, state) = fixture();
    assert!(ui.activate_id("RushBuyButton", &state).is_none(), "city authorization alone cannot choose queue head for user");
    assert_eq!(price(&ui, &state), "---", "original no-selection price placeholder");
    select(&mut ui, &state, "c2:duplicate");
    assert!(ui.activate_id("RushBuyButton", &state).is_none(), "matching UNIT_HOPLITE content isn't the head entry identity");
    assert_eq!(price(&ui, &state), "---", "future entry isn't quoted as if it were head");
    select(&mut ui, &state, "c2:granary");
    assert!(ui.activate_id("RushBuyButton", &state).is_none(), "future building never rushes unrelated Hoplite");
    select(&mut ui, &state, "c2:head");
    let before = state.clone();
    let command = ui.activate_id("RushBuyButton", &state).and_then(ManagerIntent::into_command).expect("selected affordable authorized head");
    assert_eq!((command.revision, command.route.as_str()), (7_i64,"rush-buy"), "current exact route/revision");
    assert_eq!(command.payload, json!({"city_id":"c2"}), "server-defined head operation, no cost/content/entry execution metadata");
    assert_eq!(price(&ui, &state), "30", "received head price");
    let buildable = ui.activate_id("ItemsBox.UnitsList/UNIT_SETTLER", &state).expect("independent buildable row");
    assert!(ui.apply_local(buildable), "buildable description does not silently change queue entry");
    assert_eq!(ui.view.as_ref().expect("Build").selections.get("queue").map(String::as_str), Some("c2:head"), "original UpdateButtons still uses the independently selected head");
    assert_eq!(state, before, "preview/selection never spends gold, completes production or changes queue");
}

#[test]
fn build_rush_current_quote_states_show_original_price_vs_placeholder_without_guessing_unknown_costs() {
    let (mut ui, original) = fixture();
    for (mode, enabled, caption) in [(0_u8,true,"30"),(1_u8,false,"140"),(2_u8,false,"140"),(3_u8,false,"—"),(4_u8,false,"—"),(5_u8,false,"—"),(6_u8,false,"---"),(7_u8,false,"---"),(8_u8,false,"---"),(9_u8,false,"—"),(10_u8,true,"0"),(11_u8,false,"30"),(12_u8,false,"30")] {
        let mut state = original.clone();
        ui.open(ManagerKind::Build, &state, Some("c2"));
        select(&mut ui, &state, "c2:head");
        match mode {
            0_u8 => {},
            1_u8 => { let quote=city(&mut state).rush_buy_quote.as_mut().expect("quote");quote.status="insufficient-gold".to_owned();quote.cost=Some(140_i64); }
            2_u8 => city(&mut state).rush_buy_quote.as_mut().expect("quote").cost=Some(140_i64),
            3_u8 => city(&mut state).rush_buy_quote.as_mut().expect("quote").cost=None,
            4_u8 => city(&mut state).rush_buy_quote.as_mut().expect("quote").revision=6_i64,
            5_u8 => city(&mut state).rush_buy_quote=None,
            6_u8 => { let quote=city(&mut state).rush_buy_quote.as_mut().expect("quote");quote.status="completed".to_owned();quote.cost=Some(0_i64); }
            7_u8 => city(&mut state).facts["queue"]=json!([]),
            8_u8 => { let quote=city(&mut state).rush_buy_quote.as_mut().expect("quote");quote.status="not-rushable".to_owned();quote.cost=None; }
            9_u8 => city(&mut state).rush_buy_quote.as_mut().expect("quote").cost=Some(-1_i64),
            10_u8 => city(&mut state).rush_buy_quote.as_mut().expect("quote").cost=Some(0_i64),
            11_u8 => state.viewer.as_mut().expect("viewer").diplomacy=None,
            _ => state.network=NetworkStatus::Pending,
        }
        assert_eq!(price(&ui, &state), caption, "original selection/quote caption for case{mode}");
        assert_eq!(ui.activate_id("RushBuyButton", &state).is_some(), enabled, "missing/stale/null/completed/insufficient is never a free authorized purchase in case{mode}");
    }
}

#[test]
fn build_rush_receipt_reorder_never_retargets_another_head_even_when_content_ids_are_equal() {
    let (mut ui, mut state) = fixture();
    select(&mut ui, &state, "c2:head");
    let old = ui.activate_id("RushBuyButton", &state).and_then(ManagerIntent::into_command).expect("captured old current quote");
    state.viewer.as_mut().expect("viewer").revision=8_i64;
    let chosen=city(&mut state);
    chosen.rush_buy_quote.as_mut().expect("quote").revision=8_i64;
    chosen.facts["queue"].as_array_mut().expect("queue").swap(0_usize,1_usize);
    ui.reconcile(&state);
    assert!(!old.valid_for(&state), "old price-sensitive command revision never silently rebases");
    assert!(ui.activate_id("RushBuyButton", &state).is_none(), "selected old head now future; duplicate content doesn't pick new head");
    assert_eq!(price(&ui, &state), "---", "old selection doesn't inherit new head price");
    select(&mut ui, &state, "c2:duplicate");
    assert!(ui.activate_id("RushBuyButton", &state).is_some(), "explicit exact new head selection re-enables current quote");
}

#[test]
fn build_rush_requires_one_exact_owned_city_action_not_wrong_payload_target_or_ambiguous_suggestion() {
    let (mut ui, original) = fixture();
    for mode in 0_u8..6_u8 {
        let mut state=original.clone();
        ui.open(ManagerKind::Build, &state, Some("c2"));
        select(&mut ui, &state, "c2:head");
        match mode {
            0_u8 => state.viewer.as_mut().expect("viewer").interface.legal_actions.clear(),
            1_u8 => { let viewer=state.viewer.as_mut().expect("viewer");viewer.interface.legal_actions.push(viewer.interface.legal_actions[0].clone()); }
            2_u8 => state.viewer.as_mut().expect("viewer").interface.legal_actions[0].payload=json!({"city_id":"c1"}),
            3_u8 => state.viewer.as_mut().expect("viewer").interface.legal_actions[0].target_id="IMPROVE_GRANARY".to_owned(),
            4_u8 => city(&mut state).stale=true,
            _ => {
                city(&mut state).owner_id="foreign".to_owned();
                let id = crate::gameplay::model::CityId::from_server_id("c2");
                state.cities.retain(|city| city.id != id);
            },
        }
        assert!(ui.activate_id("RushBuyButton", &state).is_none(), "exact safe current owned-head purchase only for case{mode}");
    }
}

/// Current owner quote and city-scoped atomic-edit permission, not an implicit edit.
fn clear_fixture() -> (ManagementUi, GameState) {
    let (ui, mut state) = fixture();
    state.viewer.as_mut().expect("viewer").interface.legal_actions.push(serde_json::from_value(json!({"route":"edit-production","entity_id":"c2","fields":["city_id","entries"],"payload":{"city_id":"c2","entries":[]}})).expect("current queue-editor authorization"));
    (ui, state)
}

#[test]
fn build_clear_requires_original_query_before_any_atomic_queue_submission() {
    let (ui, state) = clear_fixture();
    let intent = ui.activate_id("QueueGroup.ClearButton", &state);
    assert!(matches!(intent, Some(ManagerIntent::Dialog(_))), "EditQueue::ClearButton opens MessageBoxDialog::Query, not immediate destructive Submit");
}

/// Extract only the explicit source query, never treat a direct Submit as confirmation.
fn clear_dialog(ui: &ManagementUi, state: &GameState) -> crate::gameplay::ui_model::operations::OperationDialog {
    let dialog = match ui.activate_id("QueueGroup.ClearButton", state) {
        Some(ManagerIntent::Dialog(dialog)) => Some(*dialog),
        _ => None,
    };
    dialog.expect("original source Clear query")
}

#[test]
fn build_clear_question_uses_cached_original_window_controls_labels_and_protected_funded_head() {
    let (ui, mut state) = clear_fixture();
    let dialog = clear_dialog(&ui, &state);
    let control = |id: &str| dialog.confirmation.iter().find(|widget| widget.id == id).expect("original question control");
    assert_eq!(control("MessageBoxDialog").rect, crate::gameplay::ui_model::MenuRect { x:240_i32,y:213_i32,width:320_u32,height:174_u32 }, "original centered320x174 question, not generic input popup/scaling tweak");
    assert_eq!(control("MessageBoxDialog.DialogBackground").node.field("image00"), Some("uptg29a.tga"), "actual original nine-piece question theme");
    assert_eq!(control("MessageBoxDialog.DialogBackground.MessageDisplay").label, ui.book.label("str_ldl_ClearQueueQuery"), "original localized question, not copied text");
    assert_eq!(control("MessageBoxDialog.DialogBackground.LeftButton").label, ui.book.label("str_ldl_MB_OK"), "source Query default affirmative label");
    assert_eq!(control("MessageBoxDialog.DialogBackground.RightButton").label, ui.book.label("str_ldl_MB_CANCEL"), "source cancel label");
    assert!(!dialog.confirmation.iter().any(|widget| widget.id.ends_with(".TextField")), "source Query hides text input");
    let suppress = control("MessageBoxDialog.DialogBackground.DontShowButton");
    assert!(suppress.intent.is_none() && suppress.unavailable.is_some(), "unsafe original preference must never auto-confirm destructive requests");
    assert!(dialog.confirmation_focus.is_none(), "no implicit first-candidate/destructive default");
    let command = dialog.activate(ui.book.label("str_ldl_MB_OK"), &state).expect("explicit semantic affirmative");
    assert_eq!(command.payload, json!({"city_id":"c2","entries":[]}), "known unfunded queue clear captures exact city/empty replacement");
    let quote = city(&mut state).rush_buy_quote.as_mut().expect("quote");
    quote.status="completed".to_owned();quote.cost=Some(0_i64);
    let funded = clear_dialog(&ui, &state).activate(ui.book.label("str_ldl_MB_OK"), &state).expect("explicit funded-head-safe clear");
    assert_eq!(funded.payload, json!({"city_id":"c2","entries":[{"entry_id":"c2:head"}]}), "source ClearAllButHead protects known funded head, not content alias");
    city(&mut state).facts["queue"].as_array_mut().expect("queue").truncate(1_usize);
    assert!(ui.activate_id("QueueGroup.ClearButton", &state).is_none(), "a lone funded head has nothing safely removable; don't submit a fake clear");
}

#[test]
fn build_clear_question_capture_stays_stale_and_missing_funding_is_not_assumed_unbought() {
    let (ui, original) = clear_fixture();
    let old = clear_dialog(&ui, &original);
    let mut state = original.clone();
    state.viewer.as_mut().expect("viewer").revision=8_i64;
    city(&mut state).rush_buy_quote.as_mut().expect("quote").revision=8_i64;
    assert!(old.activate(ui.book.label("str_ldl_MB_OK"), &state).is_none(), "do not rebase an open destructive confirmation to new queue/price facts");
    for mode in 0_u8..4_u8 {
        let mut missing=original.clone();
        match mode {
            0_u8 => city(&mut missing).rush_buy_quote=None,
            1_u8 => city(&mut missing).rush_buy_quote.as_mut().expect("quote").revision=6_i64,
            2_u8 => missing.viewer.as_mut().expect("viewer").interface.planning_status="queued".to_owned(),
            _ => city(&mut missing).facts["queue"]=json!([]),
        }
        assert!(ui.activate_id("QueueGroup.ClearButton", &missing).is_none(), "unknown/stale/queued/funded-head safety cannot fall through to immediate old Submit in mode{mode}");
    }
}
