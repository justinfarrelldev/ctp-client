//! Original existing-name Overwrite Query, immutable local work and raw file version.
use super::super::{ManagementUi, ManagerIntent, confirmation};
use super::{QueueFile, codec, prompt, save};
use crate::gameplay::ui_model::operations::OperationDialog;

/// Existing-file authority is separate from opening Save-name or observing existence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capture {
    /// Exact local draft lifetime/version/template and approved literal basename.
    pub save: save::Capture,
    /// Complete observed original profile bytes; empty is a valid explicit version.
    pub expected: Vec<u8>,
    /// Explicit original affirmative queued for one replacement, not actual success.
    pub approved: bool,
}
impl Capture {
    /// Source owner, captured local template and exact cached profile version all match.
    #[must_use]
    pub fn current(&self, ui: &ManagementUi) -> bool {
        if !self.save.current(ui) || self.expected.len() > codec::MAX_BYTES {
            return false;
        }
        let Some(name) = self
            .save
            .name
            .as_ref()
            .filter(|name| save::name_valid(name))
        else {
            return false;
        };
        let key = format!("profile:{name}");
        let Some(file) = ui.saved_queues.files.get(&key) else {
            return false;
        };
        if file.id != key {
            return false;
        }
        if &file.name != name {
            return false;
        }
        if file.bytes.as_ref() != Some(&self.expected) {
            return false;
        }
        let folded = name.to_lowercase();
        ui.saved_queues
            .files
            .values()
            .filter(|candidate| {
                candidate.id.starts_with("profile:") && candidate.name.to_lowercase() == folded
            })
            .count()
            == 1_usize
    }
}
/*
 * Released ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::SaveNameResponse
 * probes exact existing filename, schedules ConfirmOverwriteQueueAction::Execute,
 * uses str_ldl_EditQueueReallyOverwrite/"QueryOverwiteQueue" (original identifier
 * spelling), then SaveQueryCallback -> Save -> SetMode(m_oldMode) only on Yes.
 * MessageBoxDialog.cpp::Query/LeftButtonActionCallback/RightButtonActionCallback
 * and messagebox.ldl original320x174/uptg29/font/OK/Cancel define this question;
 * no TextField, implicit affirmative or destructive Don'tShow replay here.
 * Client owns template/name/raw bytes rather than static saveFileName/mutable
 * list pointers. Current local + observed file version and cooperative backend
 * lock/check/staged replacement/readback must succeed before leaving Custom.
 * Installed assets are read-only; byte content version is not an inode identity
 * or OS-global CAS against hostile/uncooperative external directory changes.
 */
/// Exact existing snapshot becomes a Question, never implicit permission to replace.
#[must_use]
pub fn dialog(
    ui: &mut ManagementUi,
    save: save::Capture,
    expected: Vec<u8>,
) -> Option<OperationDialog> {
    if !save.current(ui) || expected.len() > codec::MAX_BYTES {
        return None;
    }
    let name = save
        .name
        .as_ref()
        .filter(|name| super::save::name_valid(name))?;
    let id = format!("profile:{name}");
    let key = "str_ldl_EditQueueReallyOverwrite";
    let localized = ui.book.label(key);
    let format = if localized == key {
        "Overwrite queue %s?"
    } else {
        localized
    };
    let message = prompt(format, &[name])?;
    let widgets = confirmation::controls(
        ui,
        &message,
        ManagerIntent::Select("queue-overwrite".to_owned(), "approve".to_owned()),
        false,
    )?;
    // A locked native observation is a fresh per-file fact, not a global directory success.
    ui.saved_queues.files.insert(
        id.clone(),
        QueueFile::from_bytes(id, name.clone(), &expected),
    );
    let capture = Capture {
        save,
        expected,
        approved: false,
    };
    if !capture.current(ui) {
        return None;
    }
    Some(OperationDialog::queue_overwrite(&message, widgets, capture))
}
