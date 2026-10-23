//! Explicit approved profile replacement, immutable raw version and local template.
use crate::gameplay::ui_model::{
    InGameUiModel,
    management::saved::{QueueFile, codec, custom},
};
use std::path::PathBuf;
#[cfg(test)]
mod tests;
/// One conditional local replacement, never authority to create/delete another target.
pub(in crate::gameplay::runtime) trait QueueOverwrite {
    /// Under shared profile lock, compare raw bytes and publish complete new bytes.
    fn replace(&mut self, name: &str, expected: &[u8], bytes: &[u8]) -> Result<Vec<u8>, String>;
}
/// OS profile adapter; tests only use injected in-memory storage.
struct NativeOverwrite {
    /// Trusted OS data-local home; no arbitrary installation/destination path.
    home: PathBuf,
}
impl QueueOverwrite for NativeOverwrite {
    fn replace(&mut self, name: &str, expected: &[u8], bytes: &[u8]) -> Result<Vec<u8>, String> {
        let profile = super::profile::open(&self.home)?;
        profile.publish(name, bytes, super::profile::Publication::Replace(expected))
    }
}
/* Released EditQueue.cpp::ConfirmOverwriteQueueAction::Execute/SaveQueryCallback
 * -> Save -> SetMode(oldMode) follows original MessageBoxDialog Query only after
 * explicit affirmative. Profile namespace/name/complete raw version and original
 * local draft/template are immutable here; no filename/static-list callback race.
 * Shared profile lock and before/after-stage raw checks protect cooperating client
 * writers; same-directory sync/persist/readback is not an OS-global CAS against
 * hostile external writers or a universal crash-directory durability guarantee.
 * All actual failures retain question/work and do not fake successful replacement
 * or touch installed assets/canonical gameplay. No effectful filesystem tests.
 */
/// Executes one current explicitly approved source Overwrite; untouched frames do no I/O.
pub(in crate::gameplay::runtime) fn service(ui: &mut InGameUiModel) -> bool {
    if !pending(ui) {
        return false;
    }
    let Some(home) = dirs::data_local_dir() else {
        fail(
            ui,
            "No user profile directory is available; no queue was replaced",
        );
        return true;
    };
    service_with(ui, &mut NativeOverwrite { home })
}
/// Pending is local authorization only, not proof of persistence.
fn pending(ui: &InGameUiModel) -> bool {
    ui.operation
        .as_ref()
        .and_then(|dialog| dialog.queue_overwrite.as_ref())
        .is_some_and(|capture| capture.approved)
}
/// Same effect contract for actual native dispatch and entirely fake storage tests.
pub(in crate::gameplay::runtime) fn service_with(
    ui: &mut InGameUiModel,
    storage: &mut impl QueueOverwrite,
) -> bool {
    if !pending(ui) {
        return false;
    }
    let Some(capture) = ui
        .operation
        .as_ref()
        .and_then(|dialog| dialog.queue_overwrite.clone())
    else {
        return false;
    };
    if !capture.current(&ui.management) {
        fail(
            ui,
            "The approved local template or profile version changed; no queue was replaced",
        );
        return true;
    }
    let Some(name) = capture.save.name.as_ref() else {
        fail(ui, "The approved profile filename is unavailable");
        return true;
    };
    let bytes = match codec::encode(&capture.save.template) {
        Ok(text) => text.into_bytes(),
        Err(reason) => {
            fail(ui, &reason);
            return true;
        }
    };
    match storage.replace(name, &capture.expected, &bytes) {
        Ok(readback) if readback == bytes => {
            let id = format!("profile:{name}");
            let file = QueueFile::from_bytes(id.clone(), name.clone(), &readback);
            if file.parsed.as_ref() != Ok(&capture.save.template) {
                fail(
                    ui,
                    "Replacement readback could not be parsed exactly; retain local work",
                );
                return true;
            }
            ui.management.saved_queues.files.insert(id, file);
            ui.operation = None;
            if custom::cancel(&mut ui.management) {
                ui.status_text = format!("Overwrote local queue {name}");
            } else {
                "Queue replaced; original editor parent is unavailable"
                    .clone_into(&mut ui.status_text);
            }
        }
        Ok(_readback) => fail(
            ui,
            "Replacement readback differs; retain local work and verify the stored file",
        ),
        Err(reason) => fail(ui, &reason),
    }
    true
}
/// No silent retry or new expected bytes; explicit original question remains captured.
fn fail(ui: &mut InGameUiModel, reason: &str) {
    reason.clone_into(&mut ui.status_text);
    if let Some(capture) = ui
        .operation
        .as_mut()
        .and_then(|dialog| dialog.queue_overwrite.as_mut())
    {
        capture.approved = false;
    }
}
