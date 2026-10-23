//! Saved queue catalog boundaries use injected metadata/bytes, never real files.
use super::{Entry, QueueReader, service_with};
use crate::gameplay::ui_model::{InGameUiModel, management::{ManagerKind, ManagerView, saved}};
use alloc::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Pure metadata/byte provider, with an observable bounded request trace.
#[derive(Default)]
struct Reader {
    /// Exact source names, not any real directory contents.
    entries: Vec<Entry>,
    /// Explicit fake bytes or I/O failure by flat filename.
    bytes: BTreeMap<String, Result<Vec<u8>, String>>,
    /// Metadata reads, so unchanged frames can prove no repeated scanning.
    lists: usize,
    /// Only paths requested through the validated boundary.
    reads: Vec<PathBuf>,
}
impl QueueReader for Reader {
    fn list(&mut self, _directory: &Path) -> Result<Vec<Entry>, String> {self.lists=self.lists.saturating_add(1_usize);Ok(self.entries.clone())}
    fn read(&mut self, path: &Path, maximum: usize) -> Result<Vec<u8>, String> {
        if maximum != saved::codec::MAX_BYTES { return Err("Source read was not safely bounded".to_owned()); }self.reads.push(path.to_path_buf());
        self.bytes.get(path.file_name().and_then(|name|name.to_str()).unwrap_or("")).cloned().unwrap_or_else(||Err("fake read failure".to_owned()))
    }
}
/// Original source inline browser with one pending explicit read request.
fn ui() -> InGameUiModel {
    let mut ui=InGameUiModel::default();
    ui.management.view=Some(ManagerView {kind:ManagerKind::Build,city_id:None,selections:BTreeMap::from([(saved::MODE.to_owned(),"true".to_owned()),(saved::REFRESH.to_owned(),"true".to_owned())]),scroll:BTreeMap::new()});
    ui
}
/// Exact synthetic source, with no physical path operations.
fn sources() -> Vec<(String,PathBuf)> {vec![("memory".to_owned(),PathBuf::from("/memory/queues"))]}
/// A regular local file record, not a symbolic link or folder.
fn entry(name: &str) -> Entry {Entry {name:name.to_owned(),regular:true}}

#[test]
fn build_saved_reader_scans_once_on_explicit_request_and_retains_exact_cached_preview() {
    let mut ui=ui();let mut reader=Reader {entries:vec![entry("alpha")],bytes:BTreeMap::from([("alpha".to_owned(),Ok(b"U UNIT_SETTLER".to_vec()))]),..Reader::default()};
    ui.management.view.as_mut().expect("view").selections.insert(saved::custom::MODE.to_owned(),"true".to_owned());
    ui.management.saved_queues.custom=saved::custom::State {epoch:42_u64,draft:Some(saved::custom::Draft {epoch:42_u64,version:9_u64,next:1_u64,parent:ui.management.view.clone().expect("exact custom parent"),catalog:BTreeMap::from([("UNIT_SETTLER".to_owned(),saved::Item::Unit("UNIT_SETTLER".to_owned()))]),entries:vec![saved::custom::Entry {id:"custom:42:1".to_owned(),content:"UNIT_SETTLER".to_owned()}]})};
    let authoring=ui.management.saved_queues.custom.clone();
    assert!(service_with(&mut ui,&sources(),&mut reader),"explicit original Load request serviced");
    assert_eq!(ui.management.saved_queues.custom,authoring,"read-only catalog refresh cannot erase local work, parent or generation");
    assert_eq!(reader.reads,vec![PathBuf::from("/memory/queues/alpha")],"only exact safe file");
    assert_eq!(ui.management.saved_queues.files.get("memory:alpha").expect("exact source key").parsed.as_ref().expect("full preview").items.len(),1_usize,"final line intact");
    assert!(!service_with(&mut ui,&sources(),&mut reader),"unchanged frame never scans files");assert_eq!(reader.lists,1_usize,"one directory snapshot");
}

#[test]
fn build_saved_reader_rejects_unsafe_names_and_case_collisions_before_reading_any_bytes() {
    for names in [vec!["../secret"],vec!["bad\\path"],vec!["NUL.txt"],vec!["a","A"]] {
        let mut ui=ui();let mut reader=Reader {entries:names.into_iter().map(entry).collect(),..Reader::default()};
        assert!(service_with(&mut ui,&sources(),&mut reader),"request completes with honest failure");
        assert!(ui.management.saved_queues.error.is_some() && ui.management.saved_queues.files.is_empty(),"no fabricated partial/empty success");
        assert!(reader.reads.is_empty(),"unsafe/colliding names never cross byte boundary");
    }
    for name in ["", ".hidden", "..", "CON", "CON .txt", "com1.txt", "LPT9", "bad:", "name.", "name ", "bad\nname", "a/b", "a\u{202e}b"] {assert!(!saved::valid_name(name),"unsafe name rejected: {name:?}");}
    assert!(saved::valid_name("日本 100%.queue"),"portable Unicode/spaces/percent are literal filenames");
}

#[test]
fn build_saved_reader_skips_hidden_and_nonregular_entries_without_opening_them() {
    let mut ui=ui();let mut reader=Reader {entries:vec![entry(".hidden"),Entry {name:"link".to_owned(),regular:false}],..Reader::default()};
    service_with(&mut ui,&sources(),&mut reader);
    assert!(reader.reads.is_empty() && ui.management.saved_queues.files.is_empty(),"no following symlinks/folders/hidden files");
    assert!(ui.management.saved_queues.error.is_none(),"known eligible catalog is empty, not failed I/O");
}

#[test]
fn build_saved_reader_read_failure_and_oversize_are_unavailable_records_not_valid_empty_templates() {
    for bytes in [Err("fake permission failure".to_owned()),Ok(vec![b'X';saved::codec::MAX_BYTES.saturating_add(1_usize)]),Ok(vec![0xff_u8])] {
        let mut ui=ui();let mut reader=Reader {entries:vec![entry("bad")],bytes:BTreeMap::from([("bad".to_owned(),bytes)]),..Reader::default()};
        service_with(&mut ui,&sources(),&mut reader);
        assert!(ui.management.saved_queues.files.get("memory:bad").expect("visible error record").parsed.is_err(),"failed bytes never become an empty destructive load");
    }
}

#[test]
fn build_saved_reader_bounds_catalog_and_does_nothing_after_browser_canceled_or_never_requested() {
    let mut ui=ui();let mut reader=Reader {entries:(0_usize..saved::MAX_FILES.saturating_add(1_usize)).map(|index|entry(&format!("queue{index}"))).collect(),..Reader::default()};
    service_with(&mut ui,&sources(),&mut reader);
    assert!(ui.management.saved_queues.error.is_some() && reader.reads.is_empty(),"oversized catalog rejected before byte reads");
    ui.management.view.as_mut().expect("view").selections.remove(saved::MODE);ui.management.view.as_mut().expect("view").selections.insert(saved::REFRESH.to_owned(),"true".to_owned());
    assert!(!service_with(&mut ui,&sources(),&mut reader),"canceled/hidden request cannot access files");assert_eq!(reader.lists,1_usize,"no hidden scan");
}
