//! Explicit read-only saved-queue requests, outside model/input/render execution.
use crate::gameplay::ui_model::{InGameUiModel, management::{ManagerKind, saved::{self, QueueFile}}};
use alloc::collections::{BTreeMap, BTreeSet};
use std::{fs, io::Read, path::{Path, PathBuf}};
#[cfg(test)]
mod tests;

/// Read-only directory metadata; symbolic links/folders are never opened as files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Entry {
    /// Exact local basename, validated before joining any path.
    pub name: String,
    /// True only for a regular non-symlink file.
    pub regular: bool,
}
/// Injectable metadata/byte boundary; tests never access actual user files.
pub(super) trait QueueReader {
    /// Returns a bounded directory snapshot; absent directories may be empty.
    fn list(&mut self, directory: &Path) -> Result<Vec<Entry>, String>;
    /// Reads at most maximum bytes, rejecting excess rather than truncating.
    fn read(&mut self, path: &Path, maximum: usize) -> Result<Vec<u8>, String>;
}
/// Accept only regular templates; devices/FIFOs/sockets must not be read or blocked on.
#[expect(clippy::filetype_is_file, reason = "Security requires regular files only; !is_dir would admit devices, sockets and FIFOs across supported platforms")]
fn regular(kind: fs::FileType) -> bool { kind.is_file() && !kind.is_symlink() }

/// Thin OS adapter; never writes/extracts/copies a file or follows a file symlink.
struct NativeReader;
impl QueueReader for NativeReader {
    fn list(&mut self, directory: &Path) -> Result<Vec<Entry>, String> {
        let metadata = match fs::symlink_metadata(directory) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(format!("Saved queue directory could not be inspected: {error}")),
        };
        if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() { return Err("Saved queue source is not a safe directory".to_owned()); }
        let directory = fs::read_dir(directory).map_err(|error| format!("Saved queue directory could not be read: {error}"))?;
        let mut entries = Vec::new();
        for result in directory.take(saved::MAX_FILES.saturating_add(1_usize)) {
            let entry = result.map_err(|error| format!("Saved queue entry could not be read: {error}"))?;
            let kind = entry.file_type().map_err(|error| format!("Saved queue file type is unavailable: {error}"))?;
            let name = entry.file_name().into_string().map_err(|_name| "Saved queue filename is not valid Unicode".to_owned())?;
            entries.push(Entry { name, regular: regular(kind) });
        }
        Ok(entries)
    }
    fn read(&mut self, path: &Path, maximum: usize) -> Result<Vec<u8>, String> {
        let metadata = fs::symlink_metadata(path).map_err(|error| format!("Saved queue file is unavailable: {error}"))?;
        if !regular(metadata.file_type()) { return Err("Saved queue is not a regular non-symlink file".to_owned()); }
        let file = fs::File::open(path).map_err(|error| format!("Saved queue could not be opened: {error}"))?;
        let limit = u64::try_from(maximum.saturating_add(1_usize)).map_err(|error| error.to_string())?;
        let mut bytes = Vec::new();
        file.take(limit).read_to_end(&mut bytes).map_err(|error| format!("Saved queue could not be read: {error}"))?;
        if bytes.len() > maximum { return Err("Saved queue exceeds the safe file-size limit".to_owned()); }
        Ok(bytes)
    }
}

/*
 * Source: ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::{UpdateFileLists,
 * DisplayQueueContents,LoadModeCallback}; gs/fileio/CivPaths.cpp::{CivPaths,
 * GetSavePath(C3SAVEDIR_QUEUES)} and default civpaths.txt use save/queues below
 * the original program directory. The original nested ctp2_program/ctp and
 * standalone program layouts are read-only sources; modern user-created queues
 * belong in the client data-local directory, not copyrighted installation assets.
 * Scan only after explicit Load. Original source path concatenation/fixed buffers,
 * hidden entries, invalid portable names and symlink reads are not reproduced.
 * Exact source+basename keys keep duplicate filenames distinct across roots.
 */
/// Services a pending source-browser scan; unchanged frames do no file I/O.
pub(super) fn service(ui: &mut InGameUiModel) -> bool {
    if !requested(ui) { return false; }
    let mut sources = Vec::new();
    if let Some(root) = dirs::data_local_dir() { sources.push(("profile".to_owned(), root.join("ctp-client/queues"))); }
    if let Some(root) = ui.management.book.installation_root() {
        sources.push(("retail".to_owned(), root.join("ctp2_program/ctp/save/queues")));
        sources.push(("standalone".to_owned(), root.join("ctp2_program/save/queues")));
    }
    service_with(ui, &sources, &mut NativeReader)
}
/// Strong mode/request ownership; canceled or hidden views cannot scan files.
fn requested(ui: &InGameUiModel) -> bool {
    ui.management.view.as_ref().is_some_and(|view| view.kind == ManagerKind::Build && view.selections.get(saved::MODE).is_some_and(|value| value == "true") && view.selections.get(saved::REFRESH).is_some_and(|value| value == "true"))
}
/// Injected session boundary; errors remain failures, never valid empty templates.
pub(super) fn service_with(ui: &mut InGameUiModel, sources: &[(String, PathBuf)], reader: &mut impl QueueReader) -> bool {
    if !requested(ui) { return false; }
    match scan(sources, reader) {
        Ok(files) => { ui.management.saved_queues.files = files; ui.management.saved_queues.error = None; },
        Err(reason) => {
            reason.clone_into(&mut ui.status_text);
            ui.management.saved_queues.files.clear();ui.management.saved_queues.error = Some(reason);
        }
    }
    if let Some(view) = ui.management.view.as_mut() { view.selections.remove(saved::REFRESH); }
    true
}
/// Validates all names/collisions in a source before opening any of its bytes.
fn scan(sources: &[(String, PathBuf)], reader: &mut impl QueueReader) -> Result<BTreeMap<String, QueueFile>, String> {
    if sources.is_empty() { return Err("No saved queue directory is available".to_owned()); }
    let mut files = BTreeMap::new();
    for origin in sources {
        let source = &origin.0;
        let directory = &origin.1;
        let entries = reader.list(directory)?;
        if entries.len() > saved::MAX_FILES || files.len().saturating_add(entries.len()) > saved::MAX_FILES { return Err("Saved queue catalog exceeds the safe file-count limit".to_owned()); }
        let mut names = BTreeSet::new();
        let mut eligible = Vec::new();
        for entry in entries {
            if !entry.regular { continue; }
            if entry.name.starts_with('.') && !entry.name.contains(['/', '\\']) { continue; }
            if !saved::valid_name(&entry.name) { return Err("Saved queue catalog contains an unsafe portable filename".to_owned()); }
            if !names.insert(entry.name.to_lowercase()) { return Err("Saved queue filenames collide by case in one source".to_owned()); }
            eligible.push(entry.name);
        }
        for name in eligible {
            let id = format!("{source}:{name}");
            if files.contains_key(&id) { return Err("Saved queue catalog sources contain a duplicate identity".to_owned()); }
            let file = match reader.read(&directory.join(&name), saved::codec::MAX_BYTES) {
                Ok(bytes) => QueueFile::from_bytes(id.clone(), name, &bytes),
                Err(reason) => QueueFile { id: id.clone(), name, parsed: Err(reason) },
            };
            files.insert(id, file);
        }
    }
    Ok(files)
}
