//! Explicit create-only queue persistence; all tests inject a completely fake store.
use crate::gameplay::ui_model::{
    InGameUiModel,
    management::saved::{self, QueueFile, codec, custom, save},
};
use std::{
    fs,
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
};
#[cfg(test)]
mod tests;

/// One explicit no-clobber effect with readback; never means overwrite permission.
pub(in crate::gameplay::runtime) trait QueueWriter {
    /// Publishes full bytes only to a new portable basename, returns verified bytes.
    fn create(&mut self, name: &str, bytes: &[u8]) -> Result<Vec<u8>, String>;
}
/* Source: ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::{SaveNameResponse,
 * Save,SaveQueryCallback} writes custom U/B/W/CAP/INF and restores old mode.
 * Source CivPaths.cpp::GetSavePath(C3SAVEDIR_QUEUES) is replaced here only for
 * new client-created templates: data-local/ctp-client/queues, never installation
 * assets. This is not a city/server/game-save operation. Correct CAP/INF mapping,
 * bounded complete encoding, checked I/O and retain-on-error fix source bugs.
 * Stage private bytes in the same directory, sync and persist_noclobber; never
 * truncate a target, follow an existing target symlink, or delete someone else's
 * file on failure. tempfile3.20 documents no clobber on all platforms, generally
 * atomic Linux/Windows publication (not universal crash/directory durability).
 * Leaf directories reject static symlinks; case collisions are rejected before
 * publication, and a racing existing exact target remains protected atomically.
 * External directory/case renames cannot be certified as a globally locked CAS;
 * ambiguity is rejected on later read, never permission to choose/overwrite one.
 */
/// Actual user-profile backend; never instantiated by an effectful test.
struct NativeWriter {
    /// OS data-local home; no caller-selected game installation destination.
    home: PathBuf,
}
impl QueueWriter for NativeWriter {
    fn create(&mut self, name: &str, bytes: &[u8]) -> Result<Vec<u8>, String> {
        if !save::name_valid(name) || bytes.len() > codec::MAX_BYTES {
            return Err("Invalid bounded local queue filename/data".to_owned());
        }
        let profile=super::profile::open(&self.home)?;
        let root=profile.root();
        let entries = fs::read_dir(root)
            .map_err(|error| format!("Queue directory cannot be inspected: {error}"))?;
        let mut count = 0_usize;
        for entry in entries.take(saved::MAX_FILES.saturating_add(1_usize)) {
            let entry =
                entry.map_err(|error| format!("Queue directory entry is unavailable: {error}"))?;
            let existing = entry
                .file_name()
                .into_string()
                .map_err(|_name| "Queue directory has a non-Unicode filename".to_owned())?;
            if existing==super::profile::LOCK_NAME {continue;}
            count=count.saturating_add(1_usize);
            if existing.to_lowercase() == name.to_lowercase() {
                return Err("A queue/file with this name already exists; overwrite requires its separate confirmation workflow".to_owned());
            }
        }
        if count >= saved::MAX_FILES {
            return Err("Saved queue directory has reached its safe file-count limit".to_owned());
        }
        let mut temporary = tempfile::NamedTempFile::new_in(root)
            .map_err(|error| format!("Cannot stage local queue: {error}"))?;
        temporary
            .write_all(bytes)
            .map_err(|error| format!("Cannot write complete local queue: {error}"))?;
        temporary
            .as_file()
            .sync_all()
            .map_err(|error| format!("Cannot sync local queue: {error}"))?;
        let mut file = temporary
            .persist_noclobber(root.join(name))
            .map_err(|error| format!("Queue was not published without clobber: {}", error.error))?;
        file.seek(SeekFrom::Start(0_u64))
            .map_err(|error| format!("Saved queue readback could not seek: {error}"))?;
        let mut readback = Vec::new();
        file.take(
            u64::try_from(codec::MAX_BYTES.saturating_add(1_usize))
                .map_err(|error| error.to_string())?,
        )
        .read_to_end(&mut readback)
        .map_err(|error| format!("Saved queue readback failed; retain your draft: {error}"))?;
        Ok(readback)
    }
}
/// Production session executes only an explicit currently owned local approval.
pub(in crate::gameplay::runtime) fn service(ui: &mut InGameUiModel) -> bool {
    if !pending(ui) {
        return false;
    }
    let Some(home) = dirs::data_local_dir() else {
        fail(ui, "No user data-local directory is available");
        return true;
    };
    service_with(ui, &mut NativeWriter { home })
}
/// True only when this exact source `TextQuery` owns a pending approved request.
fn pending(ui: &InGameUiModel) -> bool {
    ui.operation
        .as_ref()
        .and_then(|dialog| dialog.queue_save.as_ref())
        .is_some_and(|capture| capture.name.is_some())
}
/// Fileless injection point: one exact effect, then verified catalog/parent success.
pub(in crate::gameplay::runtime) fn service_with(
    ui: &mut InGameUiModel,
    writer: &mut impl QueueWriter,
) -> bool {
    if !pending(ui) {
        return false;
    }
    let Some(capture) = ui
        .operation
        .as_ref()
        .and_then(|dialog| dialog.queue_save.clone())
    else {
        return false;
    };
    if !capture.current(&ui.management) {
        fail(
            ui,
            "This pending Save belongs to an old local template; no file was changed",
        );
        return true;
    }
    let Some(name) = capture.name.as_ref().filter(|name| save::name_valid(name)) else {
        fail(ui, "The approved filename is invalid; no file was changed");
        return true;
    };
    let bytes = match codec::encode(&capture.template) {
        Ok(text) => text.into_bytes(),
        Err(reason) => {
            fail(ui, &reason);
            return true;
        }
    };
    match writer.create(name, &bytes) {
        Ok(readback) if readback == bytes => {
            let id = format!("profile:{name}");
            let cached = QueueFile::from_bytes(id.clone(), name.clone(), &readback);
            if cached.parsed.as_ref() != Ok(&capture.template) {
                fail(
                    ui,
                    "Saved bytes could not be parsed back exactly; retain your draft",
                );
                return true;
            }
            ui.management.saved_queues.files.insert(id, cached);
            ui.management.saved_queues.error = None;
            ui.operation = None;
            if custom::cancel(&mut ui.management) {
                ui.status_text = format!("Saved local queue {name}");
            } else {
                "Saved local queue; original editor parent is unavailable".clone_into(&mut ui.status_text);
            }
        }
        Ok(_readback) => fail(
            ui,
            "Saved queue readback differed; retain your draft. No existing file will be overwritten",
        ),
        Err(reason) => fail(ui, &reason),
    }
    true
}
/// Retain editor/name on error and require another explicit approval, never retry blindly.
fn fail(ui: &mut InGameUiModel, reason: &str) {
    reason.clone_into(&mut ui.status_text);
    if let Some(capture) = ui
        .operation
        .as_mut()
        .and_then(|dialog| dialog.queue_save.as_mut())
    {
        capture.name = None;
    }
}
