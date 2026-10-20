//! Manager interactions use in-memory snapshots, never a live server or file writes.

#![expect(
    clippy::default_numeric_fallback,
    reason = "Synthetic JSON number literals represent Go int fields, not machine arithmetic"
)]

use super::management::{ManagementUi, ManagerIntent, ManagerKind};
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::model::GameState;
use crate::gameplay::server::ServerProjection;
use serde_json::json;
use std::path::Path;

/// Authoritative fixture with distinct acting/queue/city identities.
fn fixture() -> GameState {
    let projection: ServerProjection = serde_json::from_value(json!({
        "revision": 17, "turn": 3,
        "tiles": [{"id":"tile-a","x":0,"y":0,"terrain":"TERRAIN_PLAINS"}],
        "units": [{"id":"guard","type_id":"UNIT_HOPLITE","owner_id":"human","tile_id":"tile-a"}],
        "cities": [{"id":"city-a","owner_id":"human","tile_id":"tile-a","population":3,"stored_production":40,
            "queue":[{"id":"UNIT_HOPLITE","kind":"unit","cost":80}]}],
        "research":{"target_id":"ADVANCE_BALLISTICS","progress":25,"available":["ADVANCE_TOOLMAKING","ADVANCE_AGRICULTURE"]},
        "legal_actions":[
            {"route":"queue-production","entity_id":"city-a","target_id":"UNIT_SETTLER","payload":{"city_id":"city-a","item":{"id":"UNIT_SETTLER","kind":"unit","cost":330}}},
            {"route":"queue-production","entity_id":"city-a","target_id":"IMPROVE_GRANARY","payload":{"city_id":"city-a","item":{"id":"IMPROVE_GRANARY","kind":"building","cost":140}}},
            {"route":"cancel-production","entity_id":"city-a","target_id":"UNIT_HOPLITE","payload":{"city_id":"city-a","item_id":"UNIT_HOPLITE"}},
            {"route":"rush-buy","entity_id":"city-a","target_id":"UNIT_HOPLITE","payload":{"city_id":"city-a"}}
        ]
    })).expect("viewer fixture");
    let mut state = GameState::from_server_projection(&projection, 1, 42);
    state.popups.clear();
    state
}

/// Read-only installed layout resolution, shared by all manager test cases.
fn ui() -> ManagementUi {
    ManagementUi::new(
        LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"))
            .expect("installed source layouts"),
    )
    .expect("resolve every manager root")
}

#[test]
fn every_manager_opens_original_layout_and_has_semantic_close() {
    let state = fixture();
    let mut management = ui();
    for kind in ManagerKind::ALL {
        management.open(kind, &state, None);
        let widgets = management.widgets(&state);
        assert!(!widgets.is_empty(), "{kind:?} has source controls");
        assert!(
            widgets
                .iter()
                .all(|widget| widget.intent.is_none() || widget.unavailable.is_none()),
            "{kind:?}: enabled controls must not retain disabled paint/reasons"
        );
        assert!(
            widgets
                .iter()
                .any(|widget| widget.intent == Some(ManagerIntent::Close)),
            "{kind:?} has a working Close"
        );
        let label = if kind == ManagerKind::Research {
            "Cancel"
        } else {
            "Close"
        };
        assert!(
            management.activate(label, &state).is_some(),
            "{kind:?} semantic dismissal available"
        );
    }
}

#[test]
fn build_selection_is_independent_and_never_emulates_unsafe_queue_reorder() {
    let state = fixture();
    let before = &state;
    let mut management = ui();
    management.open(ManagerKind::Build, &state, Some("city-a"));
    assert!(
        management.activate("Add >", &state).is_none(),
        "Add disabled without buildable selection"
    );
    let select_queue = management
        .activate_id("QueueGroup.QueueList/UNIT_HOPLITE", &state)
        .expect("select queue row");
    assert!(
        management.apply_local(select_queue),
        "queue selection is view-only"
    );
    let select_unit = management
        .activate_id("ItemsBox.UnitsList/UNIT_SETTLER", &state)
        .expect("select exact unit");
    assert!(
        management.apply_local(select_unit),
        "buildable selection is view-only"
    );
    let add = management
        .activate_id("ItemsBox.AddButton", &state)
        .expect("Add selected buildable");
    let command = add.into_command().expect("Add emits server command");
    assert_eq!(
        command.revision, 17_i64,
        "command bound to displayed viewer revision"
    );
    assert_eq!(command.route, "queue-production", "registered route");
    assert_eq!(
        command.payload,
        json!({"city_id":"city-a","item":{"id":"UNIT_SETTLER","kind":"unit","cost":330}}),
        "exact server metadata, not locally reconstructed costs"
    );
    let view = management.view.as_ref().expect("open");
    assert_eq!(
        view.selections.get("queue").map(String::as_str),
        Some("UNIT_HOPLITE"),
        "queue selection survives buildable change"
    );
    assert_eq!(
        view.selections.get("buildable").map(String::as_str),
        Some("UNIT_SETTLER"),
        "separate buildable selection"
    );
    for id in [
        "ItemsBox.InsertButton",
        "QueueGroup.UpButton",
        "QueueGroup.DownButton",
        "QueueGroup.ClearButton",
    ] {
        let control = management
            .widgets(&state)
            .into_iter()
            .find(|widget| widget.id == id)
            .expect("retained original control");
        assert!(
            control.intent.is_none() && control.unavailable.is_some(),
            "{id} has explicit contract blocker, never destructive cancel/re-add"
        );
    }
    assert_eq!(
        &state, before,
        "selection and command creation never mutate authoritative state"
    );
}

#[test]
fn research_change_uses_available_goal_even_when_gateway_suggestions_omit_it() {
    let state = fixture();
    let mut management = ui();
    management.open(ManagerKind::Research, &state, None);
    assert!(
        management
            .activate_id("Background.BackButton", &state)
            .is_none(),
        "OK requires explicit choice"
    );
    let choose = management
        .activate_id("Background.SciAdvanceList/ADVANCE_AGRICULTURE", &state)
        .expect("available advance");
    assert!(
        management.apply_local(choose),
        "select without changing research"
    );
    let intent = management
        .activate_id("Background.BackButton", &state)
        .expect("confirm selection");
    let command = intent
        .into_command()
        .expect("research confirmation submits");
    assert_eq!(command.route, "set-research", "gateway route");
    assert_eq!(
        command.payload,
        json!({"advance_id":"ADVANCE_AGRICULTURE"}),
        "selected target, not existing Ballistics"
    );
    assert!(command.valid_for(&state), "viewer offers available advance");
    let mut extra = command.clone();
    extra.payload["city_id"] = json!("unrelated-city");
    assert!(
        !extra.valid_for(&state),
        "research cannot smuggle unrelated fields through its specialized validator"
    );
    let mut changed = state.clone();
    changed.viewer.as_mut().expect("viewer").revision = 18;
    assert!(
        !command.valid_for(&changed),
        "stale confirmation is rejected before dispatch"
    );
}

#[test]
fn statistics_menu_entries_open_their_distinct_original_tabs_and_shortcuts_agree() {
    let mut state = fixture();
    let mut model = super::InGameUiModel {
        management: ui(),
        ..super::InGameUiModel::default()
    };
    for (label, tab, key, keymod) in [
        (
            "Ranking",
            "Tab3",
            sdl2::keyboard::Keycode::K,
            sdl2::keyboard::Mod::LCTRLMOD,
        ),
        (
            "Score",
            "Tab1",
            sdl2::keyboard::Keycode::F9,
            sdl2::keyboard::Mod::NOMOD,
        ),
        (
            "Wonders",
            "Tab2",
            sdl2::keyboard::Keycode::W,
            sdl2::keyboard::Mod::NOMOD,
        ),
    ] {
        let action = model
            .activate_menu_label(label)
            .expect("source stats entry");
        assert_eq!(
            super::InGameUiModel::action_for_key(key, keymod),
            action,
            "menu and original shortcut agree for {label}"
        );
        let sounds = model.apply_action(&mut state, action);
        assert!(
            sounds.is_empty(),
            "view-only stats navigation needs no world sound"
        );
        let view = model.management.view.as_ref().expect("statistics window");
        assert_eq!(
            view.kind,
            ManagerKind::Statistics,
            "real manager, not a status label"
        );
        assert_eq!(
            view.selections.get("tab:TabGroup").map(String::as_str),
            Some(tab),
            "{label} must select its own source tab"
        );
    }
}

#[test]
fn original_tabs_and_close_are_view_only_and_pointer_semantics_agree() {
    let state = fixture();
    let mut management = ui();
    management.open(ManagerKind::Build, &state, None);
    let buildings = management
        .activate_id("ItemsBox.BuildingsButton", &state)
        .expect("buildings category");
    assert!(management.apply_local(buildings), "local tab switch");
    let widgets = management.widgets(&state);
    assert!(
        widgets
            .iter()
            .any(|widget| widget.id == "ItemsBox.BuildingList/IMPROVE_GRANARY"),
        "building legal candidate visible"
    );
    assert!(
        !widgets
            .iter()
            .any(|widget| widget.id == "ItemsBox.UnitsList/UNIT_SETTLER"),
        "inactive list hidden"
    );
    for widget in widgets.iter().filter(|widget| widget.intent.is_some()) {
        assert_eq!(
            management.intent_at(
                widget.rect.x.saturating_add(1),
                widget.rect.y.saturating_add(1),
                &state
            ),
            widget.intent,
            "semantic/pointer agreement for {}",
            widget.id
        );
    }
    assert!(
        management.apply_local(ManagerIntent::Close),
        "close local view"
    );
    assert!(management.view.is_none(), "dialog closed");
}
