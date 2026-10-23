//! Shared cooperative profile-write lock, never a game-installation capability.
use crate::gameplay::ui_model::management::saved::{MAX_FILES, codec, save};
use fs2::FileExt;
use std::{
    fs,
    io::{Read,Seek,SeekFrom,Write},
    path::{Path, PathBuf},
};

/// Reserved private advisory-lock basename, excluded from user template catalogs.
pub(super) const LOCK_NAME: &str = ".ctp-client-queue-write.lock";
/// Publication policy; replacement always carries explicitly approved complete bytes.
#[derive(Clone,Copy)]
pub(super) enum Publication<'bytes> {
    /// New basename only, never clobber an existing entry.
    Create,
    /// Replace only this currently matching raw version under cooperative lock.
    Replace(&'bytes [u8]),
}

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
    /* Released EditQueue.cpp::Save/SaveQueryCallback is replaced by staged,
     * checked publication; original truncating fopen and mutable data are not
     * ported. Shared lock serializes client writers; double byte checks prevent
     * stale cooperative overwrite, not hostile external global/inode CAS.
     */
    /// Stages/syncs complete bytes and reads back only after permitted publication.
    pub(super) fn publish(&self,name:&str,bytes:&[u8],mode:Publication<'_>)->Result<Vec<u8>,String>{
        if !save::name_valid(name)||bytes.len()>codec::MAX_BYTES{return Err("Invalid bounded queue publication".to_owned());}
        if let Publication::Replace(expected)=mode {if self.read(name)?!=expected{return Err("Approved profile queue version changed; not overwritten".to_owned());}}
        let mut temporary=tempfile::Builder::new().prefix(".ctp-client-queue-stage-").tempfile_in(&self.root).map_err(|error|format!("Cannot stage local queue: {error}"))?;
        temporary.write_all(bytes).map_err(|error|format!("Cannot write complete local queue: {error}"))?;
        temporary.as_file().sync_all().map_err(|error|format!("Cannot sync local queue: {error}"))?;
        let mut file=match mode {
            Publication::Create=>temporary.persist_noclobber(self.root.join(name)).map_err(|error|format!("Queue was not published without clobber: {}",error.error))?,
            Publication::Replace(expected)=>{
                if self.read(name)?!=expected{return Err("Approved profile queue changed during staging; not overwritten".to_owned());}
                temporary.persist(self.root.join(name)).map_err(|error|format!("Approved queue replacement failed: {}",error.error))?
            },
        };
        file.seek(SeekFrom::Start(0_u64)).map_err(|error|format!("Saved queue readback could not seek: {error}"))?;
        let mut readback=Vec::new();file.take(u64::try_from(codec::MAX_BYTES.saturating_add(1_usize)).map_err(|error|error.to_string())?).read_to_end(&mut readback).map_err(|error|format!("Saved queue readback failed; retain your draft: {error}"))?;
        Ok(readback)
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
            .take(MAX_FILES.saturating_mul(2_usize).saturating_add(2_usize))
        {
            let entry =
                entry.map_err(|error| format!("Cannot inspect profile queue entry: {error}"))?;
            let actual = entry
                .file_name()
                .into_string()
                .map_err(|_name| "Non-Unicode profile filename is ambiguous".to_owned())?;
            if actual.starts_with('.') {
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
