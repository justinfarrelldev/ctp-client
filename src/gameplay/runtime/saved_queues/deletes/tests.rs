//! Exact profile content-version deletion through an entirely in-memory effect.
use super::{QueueDelete, service_with};
use crate::gameplay::ui_model::{
    InGameUiModel,
    management::{
        ManagerKind, ManagerView,
        saved::{self, QueueFile, delete},
    },
    operations::OperationDialog,
};
use alloc::collections::BTreeMap;

/// Fake locked compare/remove contract, never an actual directory or filesystem.
#[derive(Default)]
struct Store {
    /// Actual mock storage contents, distinct from cached UI previews.
    files: BTreeMap<String, Vec<u8>>,
    /// Exact requested basename and complete raw version.
    calls: Vec<(String, Vec<u8>)>,
    /// Injected permission/lock failure, not a real OS effect.
    failure: bool,
}
impl QueueDelete for Store {
    fn delete(&mut self, name: &str, expected: &[u8]) -> Result<(), String> {
        self.calls.push((name.to_owned(), expected.to_vec()));
        if self.failure {
            return Err("fake locked/permission error".to_owned());
        }
        if self.files.get(name).map(Vec::as_slice) != Some(expected) {
            return Err("missing/changed current raw version".to_owned());
        }
        self.files.remove(name);
        Ok(())
    }
}
/// Local file identity and raw snapshot, including a non-normalized source comment.
fn fixture() -> (InGameUiModel, Store) {
    let mut ui = InGameUiModel::default();
    let name = "target";
    let key = "profile:target";
    let bytes = b"# exact version\r\nU UNIT_SETTLER\r\n";
    ui.management.view = Some(ManagerView {
        kind: ManagerKind::Build,
        city_id: Some("c2".to_owned()),
        selections: BTreeMap::from([
            (saved::MODE.to_owned(), "true".to_owned()),
            ("saved-queue".to_owned(), key.to_owned()),
            ("saved-preview".to_owned(), "preview:0".to_owned()),
        ]),
        scroll: BTreeMap::new(),
    });
    for (id, filename) in [
        (key, name),
        ("profile:other", "other"),
        ("retail:target", name),
    ] {
        ui.management.saved_queues.files.insert(
            id.to_owned(),
            QueueFile::from_bytes(id.to_owned(), filename.to_owned(), bytes),
        );
    }
    let capture = delete::Capture {
        key: key.to_owned(),
        name: name.to_owned(),
        expected: bytes.to_vec(),
        approved: true,
    };
    ui.operation = Some(OperationDialog::queue_delete("source", Vec::new(), capture));
    let store = Store {
        files: BTreeMap::from([
            (name.to_owned(), bytes.to_vec()),
            ("other".to_owned(), bytes.to_vec()),
        ]),
        ..Store::default()
    };
    (ui, store)
}
#[test]
fn build_delete_storage_success_removes_exact_profile_version_only_and_never_selects_neighbor_or_installed_file()
 {
    let (mut ui, mut store) = fixture();
    let expected = ui
        .operation
        .as_ref()
        .expect("question")
        .queue_delete
        .as_ref()
        .expect("capture")
        .expected
        .clone();
    assert!(service_with(&mut ui, &mut store), "approved exact effect");
    assert_eq!(
        store.calls,
        vec![("target".to_owned(), expected)],
        "full raw CRLF/comment version, not normalized template"
    );
    assert!(
        !store.files.contains_key("target") && store.files.contains_key("other"),
        "one named profile entry only"
    );
    assert!(
        !ui.management
            .saved_queues
            .files
            .contains_key("profile:target")
            && ui
                .management
                .saved_queues
                .files
                .contains_key("retail:target"),
        "installation snapshot not changed"
    );
    let view = ui.management.view.as_ref().expect("same browser");
    assert_eq!(view.city_id.as_deref(), Some("c2"), "city scope unchanged");
    assert!(
        !view.selections.contains_key("saved-queue")
            && !view.selections.contains_key("saved-preview"),
        "no first/neighbor file choice"
    );
    assert!(
        ui.operation.is_none(),
        "only real success dismisses question"
    );
    assert!(!service_with(&mut ui, &mut store), "no duplicate request");
}
#[test]
fn build_delete_storage_failure_changed_cache_or_backend_readonly_case_ambiguity_and_missing_target_do_not_rebase()
 {
    for mode in 0_u8..7_u8 {
        let (mut ui, mut store) = fixture();
        match mode {
            0_u8 => {
                ui.management
                    .saved_queues
                    .files
                    .get_mut("profile:target")
                    .expect("file")
                    .bytes = Some(b"changed cache".to_vec());
            }
            1_u8 => {
                store
                    .files
                    .insert("target".to_owned(), b"changed storage".to_vec());
            }
            2_u8 => {
                store.files.remove("target");
            }
            3_u8 => store.failure = true,
            4_u8 => {
                ui.operation
                    .as_mut()
                    .expect("question")
                    .queue_delete
                    .as_mut()
                    .expect("capture")
                    .key = "retail:target".to_owned();
            }
            5_u8 => {
                ui.management.saved_queues.files.insert(
                    "profile:TARGET".to_owned(),
                    QueueFile::from_bytes(
                        "profile:TARGET".to_owned(),
                        "TARGET".to_owned(),
                        b"same",
                    ),
                );
            }
            _ => {
                ui.management.saved_queues.files.remove("profile:target");
            }
        }
        let before = ui.management.clone();
        service_with(&mut ui, &mut store);
        assert_eq!(
            ui.management, before,
            "mode{mode}: never optimistically remove cache/preview/parent"
        );
        assert!(
            ui.operation
                .as_ref()
                .and_then(|dialog| dialog.queue_delete.as_ref())
                .is_some_and(|capture| !capture.approved),
            "error resets approval only, not captured bytes"
        );
        assert_eq!(
            store.calls.len(),
            usize::from(matches!(mode, 1_u8..=3_u8)),
            "only current cache/exact profile identity reaches backend"
        );
        assert!(!service_with(&mut ui, &mut store), "no blind error retry");
    }
}
#[test]
fn build_delete_storage_explicit_empty_file_is_known_version_not_failed_read_or_empty_default() {
    let (mut ui, mut store) = fixture();
    let file = ui
        .management
        .saved_queues
        .files
        .get_mut("profile:target")
        .expect("file");
    file.bytes = Some(Vec::new());
    ui.operation
        .as_mut()
        .expect("form")
        .queue_delete
        .as_mut()
        .expect("capture")
        .expected = Vec::new();
    store.files.insert("target".to_owned(), Vec::new());
    assert!(
        service_with(&mut ui, &mut store),
        "explicit known empty file may be deleted"
    );
    assert!(
        !store.files.contains_key("target"),
        "empty content version matched"
    );
}
