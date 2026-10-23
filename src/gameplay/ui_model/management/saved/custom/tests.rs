//! Fileless exact local editing, scope/version/identity and pinned-kind guards.
use super::{apply, binding, template, Action, Edit};
use crate::gameplay::{
    model::GameState,
    ui_model::management::{ManagementUi, ManagerIntent, ManagerKind, ManagerView},
};
use alloc::collections::{BTreeMap,BTreeSet};
use serde_json::json;

/// Public pinned rules, deliberately no city, quote, production permission or money.
fn fixture() -> (ManagementUi, GameState) {
    let viewer=serde_json::from_value(json!({"revision":7_i64,"turn":1_u32,"tiles":[],"cities":[],"rules":{"units":[{"id":"U_A"},{"id":"U_B"},{"id":"U_CANT","cant_build":true},{"id":"U_POP","has_pop_and_can_build":true},{"id":"CONFLICT"}],"production":[{"id":"B_A","kind":"building"},{"id":"W_A","kind":"wonder"},{"id":"CAPITALIZATION","kind":"capitalization"},{"id":"INFRASTRUCTURE","kind":"infrastructure"},{"id":"CONFLICT","kind":"building"}]}})).expect("complete public rules fixture");
    let state = GameState::from_server_projection(&viewer, 1_u8, 1_u64);
    let ui = ManagementUi {
        view: Some(ManagerView {
            kind: ManagerKind::Build,
            city_id: None,
            selections: BTreeMap::new(),
            scroll: BTreeMap::new(),
        }),
        ..ManagementUi::default()
    };
    (ui, state)
}
/// Source independent row selection, not a private queue mutation.
fn select(ui: &mut ManagementUi, role: &str, id: &str) {
    assert!(
        ui.apply_local(ManagerIntent::Select(role.to_owned(), id.to_owned())),
        "local source row selection"
    );
}
/// Actual source button descriptor's immutable local operation.
fn intent(ui: &ManagementUi, state: &GameState, path: &str) -> Option<Action> {
    match binding(ui, state, ui.view.as_ref()?, path)?.0? {
        ManagerIntent::Custom(action) => Some(action),
        _ => None,
    }
}
/// Same typed callback dispatcher as native/accessibility.
fn activate(ui: &mut ManagementUi, state: &GameState, path: &str) {
    let action = intent(ui, state, path).expect("exact enabled local control");
    assert!(apply(ui, state, action), "valid local action");
}

#[test]
fn build_custom_pinned_categories_duplicate_units_nonunit_filter_exact_entry_and_stale_version_are_local(
) {
    let (mut ui, state) = fixture();
    let facts = state.viewer.clone();
    assert!(apply(&mut ui, &state, Action::Begin), "begin");
    let draft = ui.saved_queues.custom.draft.as_ref().expect("draft");
    assert!(
        !draft.catalog.contains_key("U_CANT")
            && !draft.catalog.contains_key("U_POP")
            && !draft.catalog.contains_key("CONFLICT"),
        "only explicit true exclusions; ambiguous kinds not silently chosen"
    );
    select(&mut ui, "buildable", "U_B");
    let captured = intent(&ui, &state, "ItemsBox.AddButton").expect("captured add");
    activate(&mut ui, &state, "ItemsBox.AddButton");
    assert!(
        !apply(&mut ui, &state, captured),
        "immutable local version cannot replay on a newer draft"
    );
    activate(&mut ui, &state, "ItemsBox.AddButton");
    let entries = ui
        .saved_queues
        .custom
        .draft
        .as_ref()
        .expect("draft")
        .entries
        .clone();
    assert_eq!(
        entries.len(),
        2_usize,
        "unit duplicates intentionally allowed"
    );
    assert_ne!(
        entries[0_usize].id, entries[1_usize].id,
        "content identity is not entry identity"
    );
    select(&mut ui, "queue", &entries[1_usize].id);
    activate(&mut ui, &state, "QueueGroup.RemoveButton");
    assert_eq!(
        ui.saved_queues
            .custom
            .draft
            .as_ref()
            .expect("draft")
            .entries,
        vec![entries[0_usize].clone()],
        "remove exact duplicate, not first same content"
    );
    select(&mut ui, "build-category", "Buildings");
    select(&mut ui, "buildable", "B_A");
    activate(&mut ui, &state, "ItemsBox.AddButton");
    assert!(
        ui.view
            .as_ref()
            .is_some_and(|view| !view.selections.contains_key("buildable")),
        "queued building leaves candidates and executable choice"
    );
    assert!(
        intent(&ui, &state, "ItemsBox.AddButton").is_none(),
        "no repeated non-unit add through hidden choice"
    );
    let candidates=super::rows(&ui,ui.view.as_ref().expect("view"),"ItemsBox.BuildingList").expect("source category");
    assert!(!candidates.iter().any(|row|row.0=="B_A") && candidates.iter().any(|row|row.0=="CAPITALIZATION") && candidates.iter().any(|row|row.0=="INFRASTRUCTURE"),"actual source candidate rows filter queued non-unit, keeping explicit conversion definitions without a city");
    assert_eq!(
        template(ui.saved_queues.custom.draft.as_ref().expect("draft"))
            .expect("typed output")
            .items,
        vec![
            super::super::Item::Unit("U_B".to_owned()),
            super::super::Item::Building("B_A".to_owned())
        ],
        "source category semantics and ordered projection"
    );
    assert_eq!(state.viewer, facts, "no game authority needed or changed");
}

#[test]
fn build_custom_insert_reorder_boundaries_clear_and_lifetime_never_reuse_local_entry_or_target_neighbor(
) {
    let (mut ui, state) = fixture();
    assert!(apply(&mut ui, &state, Action::Begin), "begin");
    for id in ["U_A", "U_B"] {
        select(&mut ui, "buildable", id);
        activate(&mut ui, &state, "ItemsBox.AddButton");
    }
    let target = ui
        .saved_queues
        .custom
        .draft
        .as_ref()
        .expect("draft")
        .entries[1_usize]
        .id
        .clone();
    select(&mut ui, "queue", &target);
    select(&mut ui, "buildable", "U_A");
    activate(&mut ui, &state, "ItemsBox.InsertButton");
    assert_eq!(
        ui.saved_queues
            .custom
            .draft
            .as_ref()
            .expect("draft")
            .entries
            .iter()
            .map(|entry| entry.content.as_str())
            .collect::<Vec<_>>(),
        vec!["U_A", "U_A", "U_B"],
        "insert before exact selected local identity"
    );
    assert_eq!(
        ui.view
            .as_ref()
            .and_then(|view| view.selections.get("queue")),
        Some(&target),
        "retained target rather than new/neighbor entry"
    );
    assert!(
        intent(&ui, &state, "QueueGroup.DownButton").is_none(),
        "last boundary"
    );
    activate(&mut ui, &state, "QueueGroup.UpButton");
    activate(&mut ui, &state, "QueueGroup.UpButton");
    assert!(
        intent(&ui, &state, "QueueGroup.UpButton").is_none(),
        "first boundary"
    );
    activate(&mut ui, &state, "QueueGroup.DownButton");
    let old = intent(&ui, &state, "QueueGroup.RemoveButton").expect("old generation remove");
    // Snapshot before clear/Cancel/new lifetime; an iterator would borrow changed work.
    let mut prior_ids = BTreeSet::new();
    for entry in &ui.saved_queues.custom.draft.as_ref().expect("draft").entries { prior_ids.insert(entry.id.clone()); }
    activate(&mut ui, &state, "QueueGroup.ClearButton");
    assert!(
        intent(&ui, &state, "QueueGroup.ClearButton").is_none(),
        "empty list doesn't enable a no-op"
    );
    assert!(apply(&mut ui, &state, Action::Cancel), "source old mode");
    assert!(apply(&mut ui, &state, Action::Begin), "new lifetime");
    select(&mut ui, "buildable", "U_A");
    activate(&mut ui, &state, "ItemsBox.AddButton");
    let entry = &ui
        .saved_queues
        .custom
        .draft
        .as_ref()
        .expect("draft")
        .entries[0_usize];
    assert!(
        !prior_ids.contains(&entry.id),
        "never reuse previous lifetime IDs"
    );
    assert!(
        !apply(&mut ui, &state, old),
        "old lifetime cannot remove a new entry"
    );
}

#[test]
fn build_custom_import_invalid_changed_wrong_kind_or_hidden_file_never_replaces_existing_local_work(
) {
    let (mut ui, state) = fixture();
    assert!(apply(&mut ui, &state, Action::Begin), "begin");
    select(&mut ui, "buildable", "U_A");
    activate(&mut ui, &state, "ItemsBox.AddButton");
    assert!(
        ui.apply_local(ManagerIntent::Select(
            super::super::MODE.to_owned(),
            "true".to_owned()
        )),
        "browser"
    );
    ui.view
        .as_mut()
        .expect("view")
        .selections
        .remove(super::super::REFRESH);
    let id = "memory:exact";
    ui.saved_queues.files.insert(
        id.to_owned(),
        super::super::QueueFile::from_bytes(id.to_owned(), "exact".to_owned(), b"U U_B"),
    );
    select(&mut ui, "saved-queue", id);
    let captured = intent(&ui, &state, "LoadBox.LoadButton").expect("full valid preview");
    let before = ui.saved_queues.custom.draft.clone();
    for bytes in [
        b"B U_B".as_slice(),
        b"U U_B\nBAD".as_slice(),
        b"U U_A".as_slice(),
    ] {
        ui.saved_queues
            .files
            .get_mut(id)
            .expect("exact file")
            .parsed = core::str::from_utf8(bytes)
            .map_err(|error| error.to_string())
            .and_then(super::super::codec::parse);
        assert!(
            !apply(&mut ui, &state, captured.clone()),
            "no failed/changed preview replacement"
        );
        assert_eq!(
            ui.saved_queues.custom.draft, before,
            "complete old work preserved"
        );
    }
    ui.saved_queues.files.get_mut(id).expect("file").parsed =
        super::super::codec::parse("U U_B\nU U_B");
    activate(&mut ui, &state, "LoadBox.LoadButton");
    assert!(
        ui.view
            .as_ref()
            .is_some_and(|view| !super::super::active(view)),
        "source LoadCustom returns to items without city Query"
    );
    assert_eq!(
        template(ui.saved_queues.custom.draft.as_ref().expect("draft"))
            .expect("template")
            .items,
        vec![super::super::Item::Unit("U_B".to_owned()); 2_usize],
        "full imported duplicate template"
    );
}

#[test]
fn build_custom_unknown_category_unsafe_boundary_overflow_and_wrong_generation_cannot_mutate_local_work(
) {
    let (mut ui, state) = fixture();
    assert!(apply(&mut ui, &state, Action::Begin), "begin");
    for id in ["UNKNOWN", "U_CANT", "CONFLICT"] {
        select(&mut ui, "buildable", id);
        assert!(
            intent(&ui, &state, "ItemsBox.AddButton").is_none(),
            "unknown/excluded/ambiguous candidate unavailable"
        );
    }
    select(&mut ui, "buildable", "U_A");
    let action = intent(&ui, &state, "ItemsBox.AddButton").expect("captured normal counter");
    ui.saved_queues.custom.draft.as_mut().expect("draft").next = u64::MAX;
    let before = ui.saved_queues.custom.draft.clone();
    assert!(
        intent(&ui, &state, "ItemsBox.AddButton").is_none(),
        "exhausted identity disables current control"
    );
    assert!(!apply(&mut ui, &state, action), "no wrapping identity");
    assert_eq!(
        ui.saved_queues.custom.draft, before,
        "overflow not partial mutation"
    );
    ui.saved_queues.custom.draft.as_mut().expect("draft").next = 0_u64;
    ui.saved_queues
        .custom
        .draft
        .as_mut()
        .expect("draft")
        .version = u64::MAX;
    assert!(
        intent(&ui, &state, "ItemsBox.AddButton").is_none(),
        "exhausted version disables current control"
    );
    let epoch = ui.saved_queues.custom.draft.as_ref().expect("draft").epoch;
    assert!(
        !apply(
            &mut ui,
            &state,
            Action::Edit {
                epoch,
                version: u64::MAX,
                edit: Edit::Add("U_A".to_owned())
            }
        ),
        "no wrapping version"
    );
    let draft = ui.saved_queues.custom.draft.as_mut().expect("draft");
    draft.version = 0_u64;
    draft.next = 256_u64;
    draft.entries = (0_u64..256_u64)
        .map(|number| super::Entry {
            id: format!("custom:{epoch}:{number}"),
            content: "U_A".to_owned(),
        })
        .collect();
    assert!(
        intent(&ui, &state, "ItemsBox.AddButton").is_none(),
        "256 entries is a complete bounded template, never silent truncation"
    );
    assert!(
        intent(&ui, &state, "CustomModeButtons.SaveButton").is_none()
            && intent(&ui, &state, "RushBuyButton").is_none()
            && intent(&ui, &state, "GotoCityButton").is_none(),
        "unsupported file effect/city operations aren't enabled substitutes"
    );
}
