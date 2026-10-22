//! Read-only canonical Strings.txt imports; no SLIC execution or external paths.
use crate::gameplay::{GameplayError, asset_error};
use alloc::collections::{BTreeMap, BTreeSet};
use std::{
    io,
    path::{Path, PathBuf},
};

/// Bounds recursive imports and rejects source cycles rather than hanging loading.
const MAX_IMPORT_DEPTH: usize = 16;
/// An installation cannot make a UI label read expand without bound.
const MAX_IMPORTED_FILES: usize = 64;

/// Loads the actual game's import manifest, including addon/advisor/tooltip labels.
///
/// # Errors
/// Reports invalid imports, cycles, malformed entries and contextual read failures.
pub(super) fn load(root: &Path) -> Result<BTreeMap<String, String>, GameplayError> {
    load_with_reader(root, |path| std::fs::read(path))
}

/// Fully injected source reader for parser/privacy regressions; never writes files.
pub(super) fn load_with_reader(
    root: &Path,
    read: impl FnMut(&Path) -> io::Result<Vec<u8>>,
) -> Result<BTreeMap<String, String>, GameplayError> {
    let mut loader = Loader {
        directory: root.join("ctp2_data/english/gamedata"),
        read,
        labels: BTreeMap::new(),
        active: BTreeSet::new(),
        loaded: BTreeSet::new(),
    };
    loader.file("Strings.txt", 0)?;
    Ok(loader.labels)
}
/// Import bookkeeping owns only text/indices, never installation caches or SDL.
struct Loader<Reader> {
    /// Only this authorized installation subdirectory may be read.
    directory: PathBuf,
    /// Fileless tests supply their entire virtual installation through this callback.
    read: Reader,
    /// First base definition wins, matching `StringDB::InsertStr/GetNameStr`.
    labels: BTreeMap<String, String>,
    /// Current recursive chain detects cycles before another read.
    active: BTreeSet<String>,
    /// Repeated imports do not create exponential work or replace earlier labels.
    loaded: BTreeSet<String>,
}
impl<Reader: FnMut(&Path) -> io::Result<Vec<u8>>> Loader<Reader> {
    /// Consumes paired IDs/text and inline imports in the source's declared order.
    fn file(&mut self, name: &str, depth: usize) -> Result<(), GameplayError> {
        if depth > MAX_IMPORT_DEPTH || !safe_name(name) {
            return Err(asset_error(format!(
                "invalid or excessive string import: {name}"
            )));
        }
        let key = name.to_ascii_lowercase();
        if self.active.contains(&key) {
            return Err(asset_error(format!("cyclic string import: {name}")));
        }
        if self.loaded.contains(&key) {
            return Ok(());
        }
        if self.loaded.len().saturating_add(self.active.len()) >= MAX_IMPORTED_FILES {
            return Err(asset_error("too many imported string files".to_owned()));
        }
        self.active.insert(key.clone());
        let path = self.directory.join(name);
        let bytes = (self.read)(&path)
            .map_err(|error| asset_error(format!("{}: {error}", path.display())))?;
        let tokens = super::tokenize(&crate::gameplay::assets::text::decode(&bytes))
            .map_err(|error| asset_error(format!("{name}: {error}")))?;
        let mut entries = tokens.into_iter();
        while let Some(id) = entries.next() {
            // The configured installation can include this standalone marker.
            // It is not in the original source grammar. Our selected-language
            // loader never merges implicit defaults, so no value/read is needed.
            if id == "DoNotImportDefaults" {
                continue;
            }
            let value = entries
                .next()
                .ok_or_else(|| asset_error(format!("unpaired string ID {id:?} in {name}")))?;
            if id.eq_ignore_ascii_case("import") {
                self.file(&value, depth.saturating_add(1))?;
            } else {
                if id.is_empty() || id.chars().any(char::is_whitespace) {
                    return Err(asset_error(format!("invalid string ID {id:?} in {name}")));
                }
                self.labels.entry(id).or_insert(value);
            }
        }
        self.active.remove(&key);
        self.loaded.insert(key);
        Ok(())
    }
}
/// Simple basenames work on both Windows and Linux, with no absolute/drive escape.
fn safe_name(name: &str) -> bool {
    name.len() <= 128
        && name.to_ascii_lowercase().ends_with(".txt")
        && !name.contains(['/', '\\', ':'])
        && !name.chars().any(char::is_control)
        && !name.starts_with('.')
}
