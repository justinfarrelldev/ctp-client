//! Source Save-name -> existing-file Overwrite question, all storage injected.
use super::{Harness, begin_row_press, buildable_fixture, input, key, manager_event};
use crate::gameplay::runtime::{
    operation_input,
    saved_queues::{
        overwrites::{QueueOverwrite, service_with as replace_with},
        writes::{CreateResult, QueueWriter, service_with},
    },
};
use crate::gameplay::ui_model::management::saved::save;
use std::sync::mpsc::TryRecvError;

/// Exact fake replacement effects; no real file is opened or mutated.
#[derive(Default)]
struct Replace {
    /// Captured name/old/new versions.
    calls: Vec<(String, Vec<u8>, Vec<u8>)>,
    /// Simulated lock/version/write failure.
    failure: bool,
}
impl QueueOverwrite for Replace {
    fn replace(&mut self, name: &str, expected: &[u8], bytes: &[u8]) -> Result<Vec<u8>, String> {
        self.calls
            .push((name.to_owned(), expected.to_vec(), bytes.to_vec()));
        if self.failure {
            Err("mock stale/busy/failed overwrite".to_owned())
        } else {
            Ok(bytes.to_vec())
        }
    }
}
use sdl2::{event::Event, keyboard::Keycode};

/// Same actual source/focused Return button activation as accessible semantics.
fn activate(harness: &mut Harness, id: &str) {
    harness.ui.management.focused = Some(id.to_owned());
    manager_event(harness, &key(Keycode::RETURN));
}
/// Public source local template and original Save-name input, no canonical edit.
fn fixture() -> Harness {
    let mut harness = buildable_fixture();
    activate(&mut harness, "NormalModeButtons.CustomButton");
    begin_row_press(&mut harness, "ItemsBox.UnitsList/UNIT_SETTLER");
    activate(&mut harness, "ItemsBox.AddButton");
    activate(&mut harness, "CustomModeButtons.SaveButton");
    input(
        &mut harness,
        &Event::TextInput {
            timestamp: 0_u32,
            window_id: 0_u32,
            text: "existing %s".to_owned(),
        },
    );
    input(&mut harness, &key(Keycode::RETURN));
    harness
}
/// Existing-file boundary with no actual directory/file mutation.
struct Existing;
impl QueueWriter for Existing {
    fn create(&mut self, name: &str, bytes: &[u8]) -> Result<CreateResult, String> {
        if name != "existing %s" || bytes != b"U UNIT_SETTLER\n" {
            return Err("wrong source capture".to_owned());
        }
        Ok(CreateResult::Existing(
            b"# old version\r\nU UNIT_WARRIOR\r\n".to_vec(),
        ))
    }
}
/* Released EditQueue.cpp::SaveNameResponse probes an existing exact queue name,
 * schedules ConfirmOverwriteQueueAction::Execute with original localized
 * str_ldl_EditQueueReallyOverwrite/QueryOverwiteQueue, then SaveQueryCallback.
 * messagebox.ldl supplies original320x174 source question/OK/Cancel, no field.
 */
#[test]
fn build_overwrite_existing_name_opens_original_explicit_question_not_error_or_implicit_replace() {
    let mut harness = fixture();
    let draft = harness.ui.management.saved_queues.custom.draft.clone();
    let canonical = harness.state.viewer.clone();
    assert!(
        service_with(&mut harness.ui, &mut Existing),
        "existing profile name inspected"
    );
    let question = harness
        .ui
        .operation
        .as_ref()
        .expect("source overwrite question");
    assert!(
        question.input.is_none(),
        "existing Save-name must transition to original Overwrite Query, not an input error/no-op"
    );
    assert!(
        question.choices.is_empty() && question.confirmation_focus.is_none(),
        "no implicit command/affirmative"
    );
    assert_eq!(
        harness.ui.management.saved_queues.custom.draft, draft,
        "not saved/replaced before explicit approval"
    );
    assert_eq!(harness.state.viewer, canonical, "no game edit");
    input(&mut harness, &key(Keycode::RETURN));
    assert!(
        harness
            .ui
            .operation
            .as_ref()
            .and_then(|dialog| dialog.queue_overwrite.as_ref())
            .is_some_and(|capture| !capture.approved),
        "Enter without explicit focus never overwrites"
    );
    input(&mut harness, &key(Keycode::ESCAPE));
    assert!(
        harness.ui.operation.is_none(),
        "Cancel only overwrite question"
    );
    assert_eq!(
        harness.ui.management.saved_queues.custom.draft, draft,
        "same unsaved original template"
    );
}

#[test]
fn build_overwrite_native_explicit_source_ok_submits_one_captured_version_and_only_verified_success_restores_parent()
 {
    let mut harness = fixture();
    let canonical = harness.state.viewer.clone();
    assert!(
        service_with(&mut harness.ui, &mut Existing),
        "existing source observation"
    );
    input(&mut harness, &key(Keycode::TAB));
    input(&mut harness, &key(Keycode::TAB));
    input(&mut harness, &key(Keycode::RETURN));
    input(&mut harness, &key(Keycode::RETURN));
    assert!(
        harness.ui.operation.is_some() && harness.ui.management.saved_queues.custom.draft.is_some(),
        "approval is not successful replacement"
    );
    let mut storage = Replace::default();
    assert!(
        replace_with(&mut harness.ui, &mut storage),
        "one explicit conditional effect"
    );
    assert_eq!(
        storage.calls,
        vec![(
            "existing %s".to_owned(),
            b"# old version\r\nU UNIT_WARRIOR\r\n".to_vec(),
            b"U UNIT_SETTLER\n".to_vec()
        )],
        "exact literal name/raw old snapshot/new captured template"
    );
    assert!(
        harness.ui.operation.is_none() && harness.ui.management.saved_queues.custom.draft.is_none(),
        "actual verified completion returns original mode"
    );
    assert_eq!(
        harness
            .ui
            .management
            .view
            .as_ref()
            .and_then(|view| view.city_id.as_deref()),
        Some("c2"),
        "not first city"
    );
    assert_eq!(
        harness.state.viewer, canonical,
        "no local canonical execution"
    );
    assert_eq!(
        harness.jobs.as_ref().expect("jobs").try_recv(),
        Err(TryRecvError::Empty),
        "no server/game command"
    );
    assert!(
        !replace_with(&mut harness.ui, &mut storage),
        "no duplicate storage effect"
    );
}

#[test]
fn build_overwrite_native_accessible_stale_snapshot_template_changed_or_failed_storage_never_rebases_question()
 {
    for mode in 0_u8..3_u8 {
        let mut harness = fixture();
        assert!(
            service_with(&mut harness.ui, &mut Existing),
            "original overwrite query"
        );
        let captured = harness
            .ui
            .operation
            .as_ref()
            .expect("query")
            .queue_overwrite
            .clone();
        if mode == 0_u8 {
            harness
                .ui
                .management
                .saved_queues
                .files
                .get_mut("profile:existing %s")
                .expect("profile")
                .bytes = Some(b"changed".to_vec());
        }
        if mode == 1_u8 {
            harness
                .ui
                .management
                .saved_queues
                .custom
                .draft
                .as_mut()
                .expect("draft")
                .version = 10_u64;
        }
        let accepted = operation_input::queue_delete::activate(&mut harness.ui, save::OK);
        assert_eq!(
            accepted,
            mode == 2_u8,
            "native/accessible same exact snapshot/template guard"
        );
        let mut storage = Replace {
            failure: true,
            ..Replace::default()
        };
        replace_with(&mut harness.ui, &mut storage);
        assert_eq!(
            storage.calls.len(),
            usize::from(mode == 2_u8),
            "only actually current explicit approval reaches backend"
        );
        let current = harness
            .ui
            .operation
            .as_ref()
            .expect("question retained")
            .queue_overwrite
            .as_ref()
            .expect("capture");
        assert!(!current.approved, "failure/stale never auto-retries");
        assert_eq!(
            Some(current.expected.clone()),
            captured.as_ref().map(|capture| capture.expected.clone()),
            "never silently replace expected bytes"
        );
        assert!(
            harness.ui.management.saved_queues.custom.draft.is_some(),
            "no lost local work"
        );
        assert!(
            !replace_with(&mut harness.ui, &mut storage),
            "no blind retry"
        );
    }
}
