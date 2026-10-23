//! Original Delete queue question/native input with entirely fake profile storage.
use super::{Harness, begin_row_press, buildable_fixture, input, key, manager_event};
use crate::gameplay::runtime::{
    operation_input,
    saved_queues::deletes::{QueueDelete, service_with},
};
use crate::gameplay::ui_model::management::saved::{self, QueueFile};
use sdl2::keyboard::Keycode;
use std::sync::mpsc::TryRecvError;

/// Exact in-memory effect; no actual profile files are opened or removed.
#[derive(Default)]
struct Store {
    /// Captured exact name/content-version authorization.
    calls: Vec<(String, Vec<u8>)>,
    /// Injected storage failure.
    failure: bool,
}
impl QueueDelete for Store {
    fn delete(&mut self, name: &str, expected: &[u8]) -> Result<(), String> {
        self.calls.push((name.to_owned(), expected.to_vec()));
        if self.failure {
            Err("fake delete denied".to_owned())
        } else {
            Ok(())
        }
    }
}

/// Shared actual source semantic/focused Return activation.
fn activate(harness: &mut Harness, id: &str) {
    harness.ui.management.focused = Some(id.to_owned());
    manager_event(harness, &key(Keycode::RETURN));
}
/// Source inline file browser, second exact profile file, no directory operations.
fn fixture() -> Harness {
    let mut harness = buildable_fixture();
    activate(&mut harness, "ItemsBox.LoadSaveButton");
    harness
        .ui
        .management
        .view
        .as_mut()
        .expect("view")
        .selections
        .remove(saved::REFRESH);
    for name in ["00-other", "literal %s"] {
        let id = format!("profile:{name}");
        harness.ui.management.saved_queues.files.insert(
            id.clone(),
            QueueFile::from_bytes(id, name.to_owned(), b"U UNIT_SETTLER\n"),
        );
    }
    begin_row_press(&mut harness, "LoadBox.QueuesList/profile:literal %s");
    harness
}
/* Source: released ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::DeleteCallback
 * -> MessageBoxDialog::Query(QueryDeleteQueue) -> DeleteQueryCallback, source
 * editqueue.ldl::LoadBox.DeleteButton and messagebox.ldl original OK/Cancel.
 * The key str_ldl_EditQueueReallyDelete is absent in this installed English
 * variant, so the original C++ fallback is required, not invented UI wording.
 */
#[test]
fn build_delete_source_question_cancel_and_unfocused_enter_keep_exact_file_parent_and_all_game_facts()
 {
    let mut harness = fixture();
    harness.ui.management.focused = Some("LoadBox.DeleteButton".to_owned());
    let parent = harness.ui.management.clone();
    let facts = harness.state.viewer.clone();
    activate(&mut harness, "LoadBox.DeleteButton");
    let dialog = harness
        .ui
        .operation
        .as_ref()
        .expect("original profile Delete Query must open");
    assert!(
        dialog
            .confirmation
            .iter()
            .any(|widget| widget.node.name == "MessageDisplay"
                && widget.label == "Permanently delete Queue literal %s?"),
        "source fallback substitutes filename once, not its percent contents"
    );
    assert!(
        dialog.choices.is_empty() && dialog.input.is_none(),
        "local destructive file question, not a city command/name field"
    );
    input(&mut harness, &key(Keycode::RETURN));
    assert!(harness.ui.operation.is_some(), "no implicit affirmative");
    input(&mut harness, &key(Keycode::ESCAPE));
    assert!(harness.ui.operation.is_none(), "Cancel question only");
    assert_eq!(
        harness.ui.management, parent,
        "exact original file/preview/parent remains"
    );
    assert_eq!(harness.state.viewer, facts, "no game mutation");
    assert_eq!(
        harness.jobs.as_ref().expect("jobs").try_recv(),
        Err(TryRecvError::Empty),
        "no game requests"
    );
}

#[test]
fn build_delete_native_explicit_original_ok_one_effect_clears_only_confirmed_profile_preview_no_game_command()
 {
    let mut harness = fixture();
    let facts = harness.state.viewer.clone();
    activate(&mut harness, "LoadBox.DeleteButton");
    input(&mut harness, &key(Keycode::TAB));
    input(&mut harness, &key(Keycode::TAB));
    input(&mut harness, &key(Keycode::RETURN));
    input(&mut harness, &key(Keycode::RETURN));
    assert!(
        harness.ui.operation.is_some()
            && harness
                .ui
                .management
                .saved_queues
                .files
                .contains_key("profile:literal %s"),
        "approval alone isn't deletion"
    );
    let mut store = Store::default();
    assert!(
        service_with(&mut harness.ui, &mut store),
        "one current explicit local effect"
    );
    assert_eq!(
        store.calls,
        vec![("literal %s".to_owned(), b"U UNIT_SETTLER\n".to_vec())],
        "exact non-first literal filename/raw version"
    );
    assert!(
        harness.ui.operation.is_none()
            && harness
                .ui
                .management
                .saved_queues
                .files
                .contains_key("profile:00-other")
            && !harness
                .ui
                .management
                .saved_queues
                .files
                .contains_key("profile:literal %s"),
        "remove one profile file, never first"
    );
    assert!(
        harness
            .ui
            .management
            .view
            .as_ref()
            .is_some_and(|view| !view.selections.contains_key("saved-queue")
                && !view.selections.contains_key("saved-preview")),
        "no automatic neighbor selection"
    );
    assert_eq!(
        harness.state.viewer, facts,
        "canonical cities/production/gold unchanged"
    );
    assert_eq!(
        harness.jobs.as_ref().expect("jobs").try_recv(),
        Err(TryRecvError::Empty),
        "no game commands"
    );
    assert!(
        !service_with(&mut harness.ui, &mut store),
        "no duplicate effect"
    );
}

/// External/cache change to the exact selected raw byte version, never another file.
fn change_version(harness:&mut Harness,bytes:&[u8]) {
    harness.ui.management.saved_queues.files.get_mut("profile:literal %s").expect("file").bytes=Some(bytes.to_vec());
}

#[test]
fn build_delete_native_readonly_or_failed_snapshot_disabled_and_changed_preview_or_storage_error_keeps_question()
 {
    let mut harness = fixture();
    for mode in 0_u8..3_u8 {
        let (key, name) = if mode == 0_u8 {
            ("retail:installed", "installed")
        } else {
            ("profile:failed", "failed")
        };
        let mut file =
            QueueFile::from_bytes(key.to_owned(), name.to_owned(), b"BAD source template");
        if mode == 1_u8 {
            file.bytes = None;
        }
        harness
            .ui
            .management
            .saved_queues
            .files
            .insert(key.to_owned(), file);
        begin_row_press(&mut harness, &format!("LoadBox.QueuesList/{key}"));
        if mode < 2_u8 {
            assert!(
                harness
                    .ui
                    .management
                    .activate_id("LoadBox.DeleteButton", &harness.state)
                    .is_none(),
                "installation or failed-read bytes can't authorize deletion"
            );
        } else {
            assert!(
                harness
                    .ui
                    .management
                    .activate_id("LoadBox.DeleteButton", &harness.state)
                    .is_some(),
                "fully read invalid template may be explicitly deleted without parsing it as commands"
            );
        }
    }
    begin_row_press(&mut harness, "LoadBox.QueuesList/profile:literal %s");
    activate(&mut harness, "LoadBox.DeleteButton");
    let original = harness
        .ui
        .operation
        .as_ref()
        .expect("query")
        .queue_delete
        .clone();
    change_version(&mut harness,b"new bytes");
    assert!(
        !operation_input::queue_delete::activate(
            &mut harness.ui,
            crate::gameplay::ui_model::management::saved::save::OK
        ),
        "accessible approval cannot rebase changed preview"
    );
    assert_eq!(
        harness.ui.operation.as_ref().expect("query").queue_delete,
        original,
        "captured raw version immutable"
    );
    change_version(&mut harness,b"U UNIT_SETTLER\n");
    assert!(
        operation_input::queue_delete::activate(
            &mut harness.ui,
            crate::gameplay::ui_model::management::saved::save::OK
        ),
        "exact unchanged version current again"
    );
    let mut store = Store {
        failure: true,
        ..Store::default()
    };
    assert!(
        service_with(&mut harness.ui, &mut store),
        "one failed actual effect"
    );
    assert!(
        harness.ui.operation.is_some()
            && harness
                .ui
                .management
                .saved_queues
                .files
                .contains_key("profile:literal %s"),
        "I/O failure retains original question and file, not optimistic deletion"
    );
    assert!(!service_with(&mut harness.ui, &mut store), "no blind retry");
}
