//! Entirely in-memory conditional replacement/version/readback, never real files.
use super::{QueueOverwrite, service_with};
use crate::gameplay::ui_model::{
    InGameUiModel,
    management::{
        ManagerKind, ManagerView,
        saved::{Item, QueueFile, Template, custom, overwrite, save},
    },
    operations::OperationDialog,
};
use alloc::collections::BTreeMap;

/// Cooperative locked compare/publish fake, with exact raw-content version checks.
#[derive(Default)]
struct Store {
    /// Actual bytes, separate from cached UI snapshots.
    bytes: Vec<u8>,
    /// Exact name/old-version/new-bytes effect requests.
    calls: Vec<(String, Vec<u8>, Vec<u8>)>,
    /// Simulated busy-lock/write failure.
    failure: bool,
    /// Failed post-publication readback, never fake verified success.
    mismatch: bool,
}
impl QueueOverwrite for Store {
    fn replace(&mut self, name: &str, expected: &[u8], bytes: &[u8]) -> Result<Vec<u8>, String> {
        self.calls
            .push((name.to_owned(), expected.to_vec(), bytes.to_vec()));
        if self.failure || self.bytes != expected {
            return Err("mock changed/busy/failed profile publication".to_owned());
        }
        self.bytes = bytes.to_vec();
        if self.mismatch {
            Ok(b"wrong readback".to_vec())
        } else {
            Ok(bytes.to_vec())
        }
    }
}
/// Exact prior local parent/new ordered template and a complete invalid-UTF8 old file.
fn fixture() -> (InGameUiModel, Store) {
    let name = "literal %s";
    let raw = vec![0xff_u8, 0x0a_u8];
    let template = Template {
        items: vec![Item::Unit("UNIT_SETTLER".to_owned()); 2_usize],
    };
    let parent = ManagerView {
        kind: ManagerKind::Build,
        city_id: Some("c2".to_owned()),
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
            next: 2_u64,
            parent,
            catalog: BTreeMap::from([(
                "UNIT_SETTLER".to_owned(),
                Item::Unit("UNIT_SETTLER".to_owned()),
            )]),
            entries: vec![
                custom::Entry {
                    id: "custom:2:1".to_owned(),
                    content: "UNIT_SETTLER".to_owned(),
                },
                custom::Entry {
                    id: "custom:2:2".to_owned(),
                    content: "UNIT_SETTLER".to_owned(),
                },
            ],
        }),
    };
    for source in ["profile", "retail"] {
        let id = format!("{source}:{name}");
        ui.management
            .saved_queues
            .files
            .insert(id.clone(), QueueFile::from_bytes(id, name.to_owned(), &raw));
    }
    let capture = overwrite::Capture {
        save: save::Capture {
            epoch: 2_u64,
            version: 3_u64,
            template,
            name: Some(name.to_owned()),
        },
        expected: raw.clone(),
        approved: true,
    };
    ui.operation = Some(OperationDialog::queue_overwrite(
        "source",
        Vec::new(),
        capture,
    ));
    (
        ui,
        Store {
            bytes: raw,
            ..Store::default()
        },
    )
}
#[test]
fn build_overwrite_store_exact_raw_version_new_duplicate_template_readback_and_only_success_restores_parent()
 {
    let (mut ui, mut store) = fixture();
    let parent = ui
        .management
        .saved_queues
        .custom
        .draft
        .as_ref()
        .expect("draft")
        .parent
        .clone();
    assert!(service_with(&mut ui, &mut store), "exact approved effect");
    assert_eq!(
        store.calls,
        vec![(
            "literal %s".to_owned(),
            vec![0xff_u8, 0x0a_u8],
            b"U UNIT_SETTLER\nU UNIT_SETTLER\n".to_vec()
        )],
        "literal name/raw old version/full ordered body"
    );
    assert!(
        ui.operation.is_none() && ui.management.saved_queues.custom.draft.is_none(),
        "only successful verified publication leaves local Custom"
    );
    assert_eq!(
        ui.management.view,
        Some(parent),
        "exact non-first old city parent"
    );
    assert_eq!(
        ui.management
            .saved_queues
            .files
            .get("profile:literal %s")
            .expect("profile")
            .bytes,
        Some(b"U UNIT_SETTLER\nU UNIT_SETTLER\n".to_vec()),
        "verified profile snapshot"
    );
    assert_eq!(
        ui.management
            .saved_queues
            .files
            .get("retail:literal %s")
            .expect("retail")
            .bytes,
        Some(vec![0xff_u8, 0x0a_u8]),
        "no installed file/cache mutation"
    );
    assert!(
        !service_with(&mut ui, &mut store),
        "no duplicate replacement"
    );
}
#[test]
fn build_overwrite_store_stale_template_profile_snapshot_unsafe_name_or_missing_capture_never_reaches_storage()
 {
    for mode in 0_u8..5_u8 {
        let (mut ui, mut store) = fixture();
        match mode {
            0_u8 => {
                ui.management
                    .saved_queues
                    .custom
                    .draft
                    .as_mut()
                    .expect("draft")
                    .version = 8_u64;
            }
            1_u8 => {
                ui.management
                    .saved_queues
                    .files
                    .get_mut("profile:literal %s")
                    .expect("file")
                    .bytes = Some(b"new version".to_vec());
            }
            2_u8 => {
                ui.operation
                    .as_mut()
                    .expect("question")
                    .queue_overwrite
                    .as_mut()
                    .expect("capture")
                    .save
                    .name = Some("../asset".to_owned());
            }
            3_u8 => {
                ui.management
                    .saved_queues
                    .files
                    .remove("profile:literal %s");
            }
            _ => ui.operation = None,
        }
        let before = ui.management.clone();
        service_with(&mut ui, &mut store);
        assert!(
            store.calls.is_empty(),
            "mode{mode}: no stale/unsafe/unconfirmed replacement"
        );
        assert_eq!(
            ui.management, before,
            "no optimistic local/cache/parent changes"
        );
    }
}
#[test]
fn build_overwrite_store_backend_conflict_busy_or_bad_readback_keeps_original_captured_question_and_unsaved_work()
 {
    for mode in 0_u8..3_u8 {
        let (mut ui, mut store) = fixture();
        let before = ui.management.clone();
        let expected = ui
            .operation
            .as_ref()
            .expect("query")
            .queue_overwrite
            .as_ref()
            .expect("capture")
            .expected
            .clone();
        match mode {
            0_u8 => store.bytes = b"changed after capture".to_vec(),
            1_u8 => store.failure = true,
            _ => store.mismatch = true,
        }
        assert!(
            service_with(&mut ui, &mut store),
            "one approved attempt/result"
        );
        assert_eq!(ui.management, before, "never fake cache/parent success");
        let capture = ui
            .operation
            .as_ref()
            .expect("source query retained")
            .queue_overwrite
            .as_ref()
            .expect("capture");
        assert!(
            !capture.approved && capture.expected == expected,
            "reset approval, not old version/template"
        );
        assert!(!service_with(&mut ui, &mut store), "no blind retry");
    }
}
