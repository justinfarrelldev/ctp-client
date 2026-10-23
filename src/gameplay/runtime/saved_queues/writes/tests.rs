//! In-memory create/readback model; no `NativeWriter`, filesystem or network access.
use super::{CreateResult, QueueWriter, service_with};
use crate::gameplay::ui_model::{
    InGameUiModel,
    management::{
        ManagerKind, ManagerView,
        saved::{Item, Template, custom, save},
    },
    operations::OperationDialog,
};
use alloc::collections::BTreeMap;

/// Mock actual create-new semantics; keeps original bytes on any name collision.
#[derive(Default)]
struct Store {
    /// Only in-memory queue files.
    files: BTreeMap<String, Vec<u8>>,
    /// Exact requested effect bodies.
    calls: Vec<(String, Vec<u8>)>,
    /// Simulated permission/write/read failure, not a real disk error.
    failure: bool,
    /// Corrupt readback, never a successful verified receipt.
    mismatch: bool,
}
impl QueueWriter for Store {
    fn create(&mut self, name: &str, bytes: &[u8]) -> Result<CreateResult, String> {
        self.calls.push((name.to_owned(), bytes.to_vec()));
        if self.failure {
            return Err("fake disk failure".to_owned());
        }
        if let Some(existing)=self.files.get(name){return Ok(CreateResult::Existing(existing.clone()));}
        if self
            .files
            .keys()
            .any(|existing| existing.to_lowercase() == name.to_lowercase())
        {
            return Err("Existing name requires separate overwrite confirmation".to_owned());
        }
        self.files.insert(name.to_owned(), bytes.to_vec());
        if self.mismatch {
            Ok(CreateResult::Created(b"BAD".to_vec()))
        } else {
            Ok(CreateResult::Created(bytes.to_vec()))
        }
    }
}
/// Captured original empty parent and separate valid authoring template.
fn fixture() -> InGameUiModel {
    let parent = ManagerView {
        kind: ManagerKind::Build,
        city_id: Some("not-first".to_owned()),
        selections: BTreeMap::new(),
        scroll: BTreeMap::new(),
    };
    let mut ui = InGameUiModel::default();
    ui.management.view = Some(ManagerView {
        kind: ManagerKind::Build,
        city_id: None,
        selections: BTreeMap::from([(custom::MODE.to_owned(), "true".to_owned())]),
        scroll: BTreeMap::new(),
    });
    ui.management.saved_queues.custom = custom::State {
        epoch: 2_u64,
        draft: Some(custom::Draft {
            epoch: 2_u64,
            version: 3_u64,
            next: 1_u64,
            parent,
            catalog: BTreeMap::from([(
                "UNIT_SETTLER".to_owned(),
                Item::Unit("UNIT_SETTLER".to_owned()),
            )]),
            entries: vec![custom::Entry {
                id: "custom:2:1".to_owned(),
                content: "UNIT_SETTLER".to_owned(),
            }],
        }),
    };
    let template = Template {
        items: vec![Item::Unit("UNIT_SETTLER".to_owned())],
    };
    ui.operation = Some(OperationDialog::queue_name(
        "source name",
        Vec::new(),
        save::Capture {
            epoch: 2_u64,
            version: 3_u64,
            template,
            name: Some("日本 100%.queue".to_owned()),
        },
    ));
    ui
}

#[test]
fn build_save_store_exact_once_verified_bytes_refresh_profile_catalog_and_restore_only_original_parent()
 {
    let mut ui = fixture();
    let mut store = Store::default();
    let parent = ui
        .management
        .saved_queues
        .custom
        .draft
        .as_ref()
        .expect("draft")
        .parent
        .clone();
    assert!(
        service_with(&mut ui, &mut store),
        "one explicit pending save"
    );
    assert_eq!(
        store.calls,
        vec![("日本 100%.queue".to_owned(), b"U UNIT_SETTLER\n".to_vec())],
        "literal exact basename/source bytes"
    );
    assert!(
        ui.operation.is_none() && ui.management.saved_queues.custom.draft.is_none(),
        "only actual verified completion leaves Custom"
    );
    assert_eq!(
        ui.management.view,
        Some(parent),
        "exact original non-first parent"
    );
    assert_eq!(
        ui.management
            .saved_queues
            .files
            .get("profile:日本 100%.queue")
            .expect("exact profile key")
            .parsed
            .as_ref()
            .expect("full parse")
            .items,
        vec![Item::Unit("UNIT_SETTLER".to_owned())],
        "subsequent browser can preview actual saved data"
    );
    assert!(
        !service_with(&mut ui, &mut store),
        "unchanged frame cannot repeat a save"
    );
    assert_eq!(store.calls.len(), 1_usize, "no automatic retry");
}

#[test]
fn build_save_store_existing_case_name_error_or_bad_readback_preserves_draft_name_and_all_original_bytes()
 {
    for mode in 0_u8..3_u8 {
        let mut ui = fixture();
        let draft = ui.management.saved_queues.custom.draft.clone();
        let mut store = Store {
            failure: mode == 1_u8,
            mismatch: mode == 2_u8,
            ..Store::default()
        };
        if mode == 0_u8 {
            store
                .files
                .insert("日本 100%.QUEUE".to_owned(), b"original".to_vec());
        }
        assert!(
            service_with(&mut ui, &mut store),
            "request consumes one effect result"
        );
        assert_eq!(
            ui.management.saved_queues.custom.draft, draft,
            "no loss of unsaved local work"
        );
        assert!(
            ui.management.saved_queues.files.is_empty(),
            "no fabricated success catalog"
        );
        assert!(
            ui.operation
                .as_ref()
                .and_then(|dialog| dialog.queue_save.as_ref())
                .is_some_and(|capture| capture.name.is_none()),
            "failure clears pending, not editor"
        );
        if mode == 0_u8 {
            assert_eq!(
                store.files.get("日本 100%.QUEUE"),
                Some(&b"original".to_vec()),
                "never overwrite an existing case alias"
            );
        }
        assert!(
            !service_with(&mut ui, &mut store),
            "failed effect never retries blindly"
        );
    }
}

#[test]
fn build_save_store_stale_epoch_version_template_or_unsafe_name_never_crosses_effect_boundary() {
    for mode in 0_u8..5_u8 {
        let mut ui = fixture();
        match mode {
            0_u8 => {
                ui.management
                    .saved_queues
                    .custom
                    .draft
                    .as_mut()
                    .expect("draft")
                    .epoch = 4_u64;
            }
            1_u8 => {
                ui.management
                    .saved_queues
                    .custom
                    .draft
                    .as_mut()
                    .expect("draft")
                    .version = 4_u64;
            }
            2_u8 => ui
                .management
                .saved_queues
                .custom
                .draft
                .as_mut()
                .expect("draft")
                .entries
                .clear(),
            3_u8 => {
                ui.operation
                    .as_mut()
                    .expect("form")
                    .queue_save
                    .as_mut()
                    .expect("capture")
                    .name = Some("../outside".to_owned());
            }
            _ => ui.operation = None,
        }
        let before = ui.management.clone();
        let mut store = Store::default();
        service_with(&mut ui, &mut store);
        assert!(
            store.calls.is_empty(),
            "mode{mode}: no stale/unsafe/canceled I/O"
        );
        assert_eq!(ui.management, before, "no parent/draft mutation");
    }
}
