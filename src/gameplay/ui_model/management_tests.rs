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
fn library_uses_a_real_edit_field_and_never_draws_obsolete_search_or_missing_title_keys() {
    let state = fixture();
    let mut management = ui();
    management.open(ManagerKind::Library, &state, None);
    let widgets = management.widgets(&state);
    assert!(
        !widgets.iter().any(|widget| widget.id == "SearchLabel"),
        "obsolete negative-position search caption is not part of the reference UI"
    );
    assert!(
        widgets
            .iter()
            .any(|widget| widget.id == "Title" && widget.label == "GREAT LIBRARY"),
        "missing retail uppercase key falls back to the installed localized title"
    );
    assert!(
        widgets
            .iter()
            .any(|widget| widget.id == "Category" && widget.label == "ADVANCES"),
        "original localized uppercase category"
    );
    let field = widgets
        .iter()
        .find(|widget| widget.id == "SearchWord")
        .expect("source field");
    assert_eq!(
        field.rect,
        super::MenuRect {
            x: 126_i32,
            y: 81_i32,
            width: 138_u32,
            height: 20_u32
        },
        "original text-field geometry"
    );
    assert_eq!(
        field.node.field("textfocused"),
        Some("true"),
        "native painting knows which local field owns input"
    );
}

#[test]
fn selected_queue_description_follows_last_interaction_without_losing_buildable_selection() {
    let state = fixture();
    let mut management = ui();
    management.open(ManagerKind::Build, &state, Some("city-a"));
    let left = management
        .activate_id("ItemsBox.UnitsList/UNIT_SETTLER", &state)
        .expect("settler");
    assert!(management.apply_local(left), "select buildable");
    let queue = management
        .activate_id("QueueGroup.QueueList/UNIT_HOPLITE", &state)
        .expect("queue");
    assert!(management.apply_local(queue), "select queue");
    let view = management.view.as_ref().expect("view");
    assert_eq!(
        view.selections.get("description").map(String::as_str),
        Some("UNIT_HOPLITE"),
        "queue click controls the description"
    );
    assert_eq!(
        view.selections.get("buildable").map(String::as_str),
        Some("UNIT_SETTLER"),
        "left selection is independent"
    );
    assert_eq!(
        management.activate_id("LibraryButton", &state),
        Some(ManagerIntent::Article("UNIT_HOPLITE".to_owned())),
        "library opens the described topic, not a first item or stale left selection"
    );
}

#[test]
fn library_historical_tab_changes_content_not_game_state_and_missing_panels_are_disabled() {
    let state = fixture();
    let mut management = ui();
    management.open(ManagerKind::Library, &state, None);
    let history = management
        .activate_id("Tabs.Historical.TabButton", &state)
        .expect("installed history");
    assert!(management.apply_local(history), "select historical panel");
    let widgets = management.widgets(&state);
    assert!(
        widgets.iter().any(|widget| widget
            .id
            .starts_with("Tabs.Historical.TabPanel.HistoricalText/word-")),
        "actual historical text is painted"
    );
    assert!(
        !widgets.iter().any(|widget| widget
            .id
            .starts_with("Tabs.Gameplay.TabPanel.GameplayText/word-")),
        "inactive body is not layered over history"
    );
    let tree = widgets
        .iter()
        .find(|widget| widget.id == "Tabs.TechTreeTab.TabButton")
        .expect("retained source tab");
    assert!(
        tree.intent.is_none() && tree.unavailable.is_some(),
        "unimplemented chart is not a fake-enabled blank panel"
    );
    management
        .library
        .catalog
        .articles
        .get_mut("ADVANCE_BALLISTICS")
        .expect("topic")
        .sections
        .remove(&crate::gameplay::assets::library::LibrarySection::Historical);
    assert!(
        management
            .activate_id("Tabs.Historical.TabButton", &state)
            .is_none(),
        "source disables unavailable article sections"
    );
    assert_eq!(
        management.library.current.as_deref(),
        Some("ADVANCE_BALLISTICS"),
        "tab navigation never selects a research goal"
    );
}

#[test]
fn article_and_index_scroll_limits_allow_immediate_recovery_from_extreme_input() {
    let state = fixture();
    let mut management = ui();
    let text = (0_u32..40_u32)
        .map(|line| format!("line-{line}"))
        .collect::<Vec<_>>()
        .join("\n");
    management
        .library
        .catalog
        .articles
        .get_mut("ADVANCE_BALLISTICS")
        .expect("topic")
        .sections
        .insert(
            crate::gameplay::assets::library::LibrarySection::Gameplay,
            crate::gameplay::assets::library::LibraryDocument::parse(&text),
        );
    management.open(ManagerKind::Library, &state, None);
    let path = "Tabs.Gameplay.TabPanel.GameplayText";
    assert!(
        management.scroll(path, i32::MAX, &state),
        "bounded article scroll"
    );
    let bottom = management.view.as_ref().expect("view").scroll[path];
    assert!(
        bottom > 0_usize && bottom < 40_usize,
        "offset is bounded by real line count"
    );
    assert!(
        management
            .widgets(&state)
            .iter()
            .any(|widget| widget.label == "line-39"),
        "last page remains visible"
    );
    assert!(
        management.scroll(path, -1_i32, &state),
        "one upward event works immediately"
    );
    assert_eq!(
        management.view.as_ref().expect("view").scroll[path],
        bottom.saturating_sub(1_usize),
        "not stuck at an enormous invisible offset"
    );
    assert!(
        management.scroll("IndexSheet", i32::MAX, &state),
        "empty original index clamps to zero"
    );
    assert_eq!(
        management.view.as_ref().expect("view").scroll["IndexSheet"],
        0_usize,
        "no phantom index pages"
    );
    assert!(
        !management.scroll("not-a-control", 1_i32, &state),
        "unknown path cannot acquire scroll state"
    );
}

#[test]
fn related_research_and_library_close_restore_parent_city_and_queue_selections() {
    let state = fixture();
    let mut management = ui();
    management.open(ManagerKind::Build, &state, Some("city-a"));
    let select = management
        .activate_id("ItemsBox.UnitsList/UNIT_SETTLER", &state)
        .expect("settler");
    assert!(
        management.apply_local(select),
        "select before library navigation"
    );
    let parent = management.view.clone();
    management.open_related(ManagerKind::Library, &state);
    management.open_related(ManagerKind::Science, &state);
    management.open_related(ManagerKind::Research, &state);
    assert!(
        management.apply_local(ManagerIntent::Close),
        "cancel research"
    );
    assert_eq!(
        management.view.as_ref().map(|view| view.kind),
        Some(ManagerKind::Science),
        "restore original research parent"
    );
    management.open_related(ManagerKind::Build, &state);
    assert_eq!(
        management.view, parent,
        "returning to an existing manager restores its exact selections"
    );
    assert!(
        management.parents.is_empty(),
        "navigation cycles do not grow duplicate windows"
    );
}

#[test]
fn library_variables_use_current_projected_rules_and_never_execute_source_expressions() {
    let mut state = fixture();
    state.viewer.as_mut().expect("viewer").interface.rules =
        Some(json!({"advances":[{"id":"ADVANCE_BALLISTICS","cost":731_i32}]}));
    let mut management = ui();
    management
        .library
        .catalog
        .articles
        .get_mut("ADVANCE_BALLISTICS")
        .expect("installed topic")
        .sections
        .insert(
            crate::gameplay::assets::library::LibrarySection::Prerequisites,
            crate::gameplay::assets::library::LibraryDocument::parse(
                "Cost: {AdvanceDB(Advance[0]).Cost}\nUnknown: {arbitrary_script()}",
            ),
        );
    management.open(ManagerKind::Library, &state, None);
    let controls = management.widgets(&state);
    assert!(
        controls.iter().any(|widget| widget.label == "731"),
        "current server cost, not retail 570 or an invented zero"
    );
    assert!(
        controls.iter().any(|widget| widget.label == "—"),
        "unsupported expressions remain unknown and are never evaluated"
    );
    assert!(
        controls
            .iter()
            .any(|widget| widget.node.field("image00") == Some("upap012l.rim")),
        "correct active original still picture"
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
