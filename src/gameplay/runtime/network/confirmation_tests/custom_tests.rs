//! Original local Custom mode; all input is native/semantic, all transport inert.
use super::{begin_row_press, buildable_fixture, key, manager_event, row_event};
use crate::gameplay::ui_model::management::ManagerKind;
use sdl2::keyboard::Keycode;
use std::sync::mpsc::TryRecvError;

/* Source: ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::{CustomButton,
 * SetMode,UpdateChoiceLists,UpdateQueueList} clears a separate custom list and
 * nulls CityData, hiding SingleCityChooser/normal buttons and showing the custom
 * controls. editqueue.ldl::BuildEditorWindow.{ItemsBox.ModeLabel,QueueGroup.
 * QueueLabel,CustomModeButtons,NormalModeButtons,SingleCityChooser} owns layout.
 * A local template is never a city command or a copy of a paid canonical head.
 */
#[test]
fn build_custom_mode_starts_empty_local_editor_and_cancel_restores_exact_source_city_parent_without_a_command(
) {
    let mut harness = buildable_fixture();
    let parent = harness.ui.management.view.clone();
    let facts = harness.state.viewer.clone();
    let intent = harness
        .ui
        .management
        .activate_id("NormalModeButtons.CustomButton", &harness.state)
        .expect("original Custom callback must open actual local editor");
    crate::gameplay::runtime::management_input::apply(
        intent,
        &mut harness.state,
        &mut harness.ui,
        Some(&mut harness.authority),
    );
    let widgets = harness.ui.management.widgets(&harness.state);
    assert!(
        harness.ui.operation.is_none(),
        "not the raw server queue input form"
    );
    assert!(
        widgets
            .iter()
            .any(|widget| widget.id == "CustomModeButtons"),
        "source custom controls shown"
    );
    assert!(
        !widgets
            .iter()
            .any(|widget| widget.id == "NormalModeButtons" || widget.id == "SingleCityChooser"),
        "source old-city controls hidden"
    );
    assert!(!widgets.iter().any(|widget|matches!(widget.intent.clone(),Some(crate::gameplay::ui_model::management::ManagerIntent::Select(role,_)) if role=="queue")),"custom list starts empty, not canonical production; empty source ranger may still exist");
    assert_eq!(
        harness
            .ui
            .management
            .view
            .as_ref()
            .and_then(|view| view.city_id.as_deref()),
        None,
        "source custom CityData is null"
    );
    assert!(
        harness
            .ui
            .management
            .activate_id("RushBuyButton", &harness.state)
            .is_none(),
        "no local paid-head operation"
    );
    harness.ui.management.focused = Some("CustomModeButtons.CancelButton".to_owned());
    manager_event(&mut harness, &key(Keycode::RETURN));
    assert_eq!(
        harness.ui.management.view, parent,
        "exact original non-first city/selection parent"
    );
    assert_eq!(
        harness.ui.management.view.as_ref().map(|view| view.kind),
        Some(ManagerKind::Build),
        "Cancel exits custom, not entire manager"
    );
    assert_eq!(
        harness.state.viewer, facts,
        "no canonical queue/production/gold mutation"
    );
    assert_eq!(
        harness.jobs.as_ref().expect("jobs").try_recv(),
        Err(TryRecvError::Empty),
        "no hidden city/turn request"
    );
}

/// Same native focused button path as an accessible source ID activation.
fn activate(harness: &mut super::Harness, id: &str) {
    harness.ui.management.focused = Some(id.to_owned());
    manager_event(harness, &key(Keycode::RETURN));
}

/// Actual first down/up plus matching SDL count2 for one exact source row.
fn double_click(harness:&mut super::Harness,id:&str) {
    begin_row_press(harness,id);let second=row_event(harness,id,2_u8,true);manager_event(harness,&second);
}

#[test]
fn build_custom_native_double_click_space_exact_duplicate_remove_reorder_and_clear_send_no_city_command_even_pending(
) {
    let mut harness = buildable_fixture();
    let facts = harness.state.viewer.clone();
    activate(&mut harness, "NormalModeButtons.CustomButton");
    double_click(&mut harness, "ItemsBox.UnitsList/UNIT_SETTLER");
    assert_eq!(
        harness
            .ui
            .management
            .saved_queues
            .custom
            .draft
            .as_ref()
            .expect("draft")
            .entries
            .len(),
        1_usize,
        "actual source double-click Add is local"
    );
    begin_row_press(&mut harness, "ItemsBox.UnitsList/UNIT_SETTLER");
    manager_event(&mut harness, &key(Keycode::SPACE));
    let entries = harness
        .ui
        .management
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
        "source focused-list Space appends another local unit"
    );
    assert_ne!(
        entries[0_usize].id, entries[1_usize].id,
        "duplicate entry identity"
    );
    harness.state.network = crate::gameplay::model::NetworkStatus::Pending;
    let row = format!("QueueGroup.QueueList/{}", entries[1_usize].id);
    begin_row_press(&mut harness, &row);
    activate(&mut harness, "QueueGroup.UpButton");
    assert_eq!(
        harness
            .ui
            .management
            .saved_queues
            .custom
            .draft
            .as_ref()
            .expect("draft")
            .entries[0_usize]
            .id,
        entries[1_usize].id,
        "local edit isn't blocked/inferred execution by unrelated pending gameplay"
    );
    double_click(&mut harness, &row);
    assert_eq!(
        harness
            .ui
            .management
            .saved_queues
            .custom
            .draft
            .as_ref()
            .expect("draft")
            .entries,
        vec![entries[0_usize].clone()],
        "source double-click Remove only exact duplicate entry"
    );
    activate(&mut harness, "QueueGroup.ClearButton");
    assert!(
        harness.ui.operation.is_none(),
        "original custom Clear does not show city destructive Query"
    );
    assert!(
        harness
            .ui
            .management
            .saved_queues
            .custom
            .draft
            .as_ref()
            .expect("draft")
            .entries
            .is_empty(),
        "local list cleared only"
    );
    assert_eq!(
        harness.state.viewer, facts,
        "canonical queue/stored production/gold remain exact"
    );
    assert_eq!(
        harness.jobs.as_ref().expect("jobs").try_recv(),
        Err(TryRecvError::Empty),
        "no edit-production/cancel/pass request"
    );
}

#[test]
fn build_custom_library_receipt_return_preserves_local_entry_description_and_never_restores_vanished_original_city(
) {
    let mut harness = buildable_fixture();
    activate(&mut harness, "NormalModeButtons.CustomButton");
    begin_row_press(&mut harness, "ItemsBox.UnitsList/UNIT_SETTLER");
    activate(&mut harness, "ItemsBox.AddButton");
    let local = harness
        .ui
        .management
        .saved_queues
        .custom
        .draft
        .as_ref()
        .expect("draft")
        .entries[0_usize]
        .id
        .clone();
    begin_row_press(&mut harness, &format!("QueueGroup.QueueList/{local}"));
    activate(&mut harness, "LibraryButton");
    assert_eq!(
        harness.ui.management.view.as_ref().map(|view| view.kind),
        Some(ManagerKind::Library),
        "local opaque entry resolves actual installed content"
    );
    assert_eq!(
        harness.ui.management.library.current.as_deref(),
        Some("UNIT_SETTLER"),
        "not local/city entry ID article"
    );
    let mut received = harness.state.viewer.clone().expect("viewer");
    received.revision = 8_i64;
    received.cities.retain(|city| city.id != "c2");
    harness.authority.session.projection = Some(received.clone());
    crate::gameplay::runtime::sync::apply(
        &mut harness.authority,
        &mut harness.state,
        &mut harness.ui,
        &mut crate::gameplay::runtime::MovementInputState::default(),
        super::Outcome::Receipt(Box::new(received)),
    );
    manager_event(&mut harness, &key(Keycode::ESCAPE));
    let returned = harness.ui.management.view.as_ref().expect("custom parent");
    assert_eq!(
        returned.selections.get("queue"),
        Some(&local),
        "unrelated received city facts don't clear local identities"
    );
    assert!(
        harness
            .ui
            .management
            .activate_id("LibraryButton", &harness.state)
            .is_some(),
        "local description article survives receipt"
    );
    activate(&mut harness, "CustomModeButtons.CancelButton");
    assert_eq!(
        harness
            .ui
            .management
            .view
            .as_ref()
            .and_then(|view| view.city_id.as_deref()),
        None,
        "old city lost ownership/disappeared, never revive or choose first c1"
    );
    assert_eq!(
        harness.jobs.as_ref().expect("jobs").try_recv(),
        Err(TryRecvError::Empty),
        "no Library/receipt/Cancel gameplay request"
    );
}
