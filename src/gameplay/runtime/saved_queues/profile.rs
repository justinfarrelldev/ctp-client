//! Shared cooperative profile-write lock, never a game-installation capability.
use crate::gameplay::ui_model::management::saved::{MAX_FILES, codec, save};
use fs2::FileExt;
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

/// Reserved private advisory-lock basename, excluded from user template catalogs.
pub(super) const LOCK_NAME: &str = ".ctp-client-queue-write.lock";
/// Owns one nonblocking exclusive lock through normal File drop on every exit.
pub(super) struct Profile {
    /// Validated client-owned leaf, not a caller supplied retail directory.
    root: PathBuf,
    /// RAII file ownership releases flock/LockFile; never duplicated or shared.
    _lock: fs::File,
}
impl Profile {
    /// All operations stay within the validated profile basename namespace.
    pub(super) const fn root(&self) -> &PathBuf {
        &self.root
    }
    /// Complete current bytes while namespace is locked; no case-neighbor fallback.
    pub(super) fn read(&self, name: &str) -> Result<Vec<u8>, String> {
        if !save::name_valid(name) {
            return Err("Invalid profile queue basename".to_owned());
        }
        let folded = name.to_lowercase();
        let mut matches = Vec::new();
        let mut count = 0_usize;
        for entry in fs::read_dir(&self.root)
            .map_err(|error| format!("Cannot inspect profile queues: {error}"))?
            .take(MAX_FILES.saturating_add(2_usize))
        {
            let entry =
                entry.map_err(|error| format!("Cannot inspect profile queue entry: {error}"))?;
            let actual = entry
                .file_name()
                .into_string()
                .map_err(|_name| "Non-Unicode profile filename is ambiguous".to_owned())?;
            if actual == LOCK_NAME {
                continue;
            }
            count = count.saturating_add(1_usize);
            if actual.to_lowercase() == folded {
                matches.push(actual);
            }
        }
        if count > MAX_FILES
            || matches.len() != 1_usize
            || matches.first().is_none_or(|actual| actual != name)
        {
            return Err("The exact profile filename is missing, case-ambiguous or over the safe catalog limit".to_owned());
        }
        let path = self.root.join(name);
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("Cannot inspect selected profile queue: {error}"))?;
        if !super::regular(metadata.file_type()) {
            return Err("Selected profile queue is not a regular non-symlink file".to_owned());
        }
        let file = fs::File::open(path)
            .map_err(|error| format!("Cannot open selected profile queue: {error}"))?;
        let opened = file
            .metadata()
            .map_err(|error| format!("Cannot inspect open profile queue: {error}"))?;
        if !super::regular(opened.file_type()) {
            return Err("Opened selected queue is not a regular file".to_owned());
        }
        let mut bytes = Vec::new();
        file.take(
            u64::try_from(codec::MAX_BYTES.saturating_add(1_usize))
                .map_err(|error| error.to_string())?,
        )
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Cannot read complete selected profile queue: {error}"))?;
        if bytes.len() > codec::MAX_BYTES {
            return Err("Selected profile queue exceeds safe snapshot limit".to_owned());
        }
        Ok(bytes)
    }
}
/* Local integration boundary (not retail behavior): EditQueue.cpp::Save and
 * DeleteQueryCallback use C3SAVEDIR_QUEUES. New client-owned queues live only
 * under data-local/ctp-client/queues, never installed assets. All client writes
 * use the same nonblocking advisory lock to avoid stale check/mutate races
 * between cooperating client processes; no UI thread waits for another writer.
 * fs2 0.4.3 FileExt docs/tests specify flock on Unix, LockFile on Windows,
 * release on owning File drop and advisory-only semantics. No cloned handles.
 * Revalidate full bytes while locked. This does not lock hostile external tools,
 * promise an inode CAS, or certify arbitrary external directory/symlink races.
 * Static unsafe directories/lock symlinks/nonregular files fail without writes.
 */
/// Acquires profile namespace before any save/delete target is inspected/mutated.
pub(super) fn open(home: &Path) -> Result<Profile, String> {
    let client = home.join("ctp-client");
    ensure_directory(&client)?;
    let root = client.join("queues");
    ensure_directory(&root)?;
    let lock_path = root.join(LOCK_NAME);
    match fs::symlink_metadata(&lock_path) {
        Ok(metadata) if !super::regular(metadata.file_type()) => {
            return Err("Profile lock is not a regular non-symlink file".to_owned());
        }
        Ok(_metadata) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("Cannot inspect profile write lock: {error}")),
    }
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path)
        .map_err(|error| format!("Cannot open profile write lock: {error}"))?;
    let metadata = lock
        .metadata()
        .map_err(|error| format!("Cannot inspect open profile lock: {error}"))?;
    if !super::regular(metadata.file_type()) {
        return Err("Opened profile lock is not a regular file".to_owned());
    }
    FileExt::try_lock_exclusive(&lock).map_err(|error| {
        format!("Profile queue writer is busy/unavailable; retry explicitly: {error}")
    })?;
    Ok(Profile { root, _lock: lock })
}
/// Reject unsafe client-owned leaf directories; never create installation paths.
fn ensure_directory(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
                Ok(())
            } else {
                Err("Client queue location is not a safe non-symlink directory".to_owned())
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir_all(path)
                .map_err(|failure| format!("Cannot create client queue directory: {failure}"))?;
            let metadata = fs::symlink_metadata(path)
                .map_err(|failure| format!("Cannot inspect client queue directory: {failure}"))?;
            if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
                Ok(())
            } else {
                Err("Created queue location is not a safe directory".to_owned())
            }
        }
        Err(error) => Err(format!("Cannot inspect client queue directory: {error}")),
    }
}
