//! Exact approved profile-file deletion with full version checks and fakeable I/O.
use crate::gameplay::ui_model::InGameUiModel;
use std::{fs, path::PathBuf};
#[cfg(test)]
mod tests;
/// One local effect, never arbitrary paths or permission to delete installed assets.
pub(in crate::gameplay::runtime) trait QueueDelete {
    /// Under shared profile write lock, reject missing/changed/ambiguous raw bytes.
    fn delete(&mut self, name: &str, expected: &[u8]) -> Result<(), String>;
}
/// OS user-profile adapter; never instantiated by effectful tests.
struct NativeDelete {
    /// Trusted OS data-local home, not selected retail/standalone asset root.
    home: PathBuf,
}
impl QueueDelete for NativeDelete {
    fn delete(&mut self, name: &str, expected: &[u8]) -> Result<(), String> {
        let profile = super::profile::open(&self.home)?;
        let actual = profile.read(name)?;
        if actual != expected {
            return Err("Selected profile queue bytes changed; no file was deleted".to_owned());
        }
        fs::remove_file(profile.root().join(name))
            .map_err(|error| format!("Confirmed profile file could not be deleted: {error}"))
    }
}
/* Released EditQueue.cpp::DeleteQueryCallback removes selected C3 queue only
 * after explicit QueryDeleteQueue and updates file lists. Here exact profile
 * name + complete raw bytes are captured, cache/selection revalidated, then
 * current bytes checked inside shared nonblocking advisory profile-write lock.
 * All cooperating saves/deletes serialize; failures preserve question/catalog.
 * Never delete a retail/standalone installation entry, guess another case/name,
 * or optimistically clear the preview before actual successful remove. No game
 * command/local production mutation. This is not an OS-global inode CAS against
 * hostile/uncooperative external directory changes; profile.rs states limits.
 */
/// Actual session executes only one explicit still-current affirmative.
pub(in crate::gameplay::runtime) fn service(ui: &mut InGameUiModel) -> bool {
    if !pending(ui) {
        return false;
    }
    let Some(home) = dirs::data_local_dir() else {
        fail(
            ui,
            "No user profile directory is available; no file was deleted",
        );
        return true;
    };
    service_with(ui, &mut NativeDelete { home })
}
/// Pending is a local approval, not proof that storage accepted/executed deletion.
fn pending(ui: &InGameUiModel) -> bool {
    ui.operation
        .as_ref()
        .and_then(|dialog| dialog.queue_delete.as_ref())
        .is_some_and(|capture| capture.approved)
}
/// Shared fakeable effect boundary; tests record exact basename/raw version only.
pub(in crate::gameplay::runtime) fn service_with(
    ui: &mut InGameUiModel,
    storage: &mut impl QueueDelete,
) -> bool {
    if !pending(ui) {
        return false;
    }
    let Some(capture) = ui
        .operation
        .as_ref()
        .and_then(|dialog| dialog.queue_delete.clone())
    else {
        return false;
    };
    if !capture.current(&ui.management) {
        fail(
            ui,
            "Selected profile file/preview changed; no file was deleted",
        );
        return true;
    }
    match storage.delete(&capture.name, &capture.expected) {
        Ok(()) => {
            ui.management.saved_queues.files.remove(&capture.key);
            if let Some(view) = ui.management.view.as_mut() {
                if view.selections.get("saved-queue") == Some(&capture.key) {
                    view.selections.remove("saved-queue");
                    view.selections.remove("saved-preview");
                    view.scroll.remove("LoadBox.Contents");
                }
            }
            ui.operation = None;
            ui.status_text = format!("Deleted local queue {}", capture.name);
        }
        Err(reason) => fail(ui, &reason),
    }
    true
}
/// No blind retry/rebased bytes: keep original source question and captured version.
fn fail(ui: &mut InGameUiModel, reason: &str) {
    reason.clone_into(&mut ui.status_text);
    if let Some(capture) = ui
        .operation
        .as_mut()
        .and_then(|dialog| dialog.queue_delete.as_mut())
    {
        capture.approved = false;
    }
}
