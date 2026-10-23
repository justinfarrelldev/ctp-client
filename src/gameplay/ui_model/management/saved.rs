//! Original saved-queue browser: cached local files, never local game execution.
use super::{ManagementUi, ManagerIntent, ManagerKind, ManagerView, confirmation, data};
use crate::gameplay::{model::GameState, server::ServerProjection, ui_model::operations::OperationDialog};
use alloc::collections::{BTreeMap, BTreeSet};

/// Bounded original U/B/W/CAP/INF file grammar.
pub mod codec;
#[cfg(test)]
mod tests;

/// Source browser mode marker, presentation-only and never sent to the server.
pub const MODE: &str = "build-load-mode";
/// One explicit scan request; session performs I/O outside input/model tests.
pub const REFRESH: &str = "build-load-refresh";
/// Bounded directory catalog, not an unbounded frame-time filesystem scan.
pub const MAX_FILES: usize = 128_usize;

/// Original saved-record categories, with no executable costs or effects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    /// Explicit original unit ID.
    Unit(String),
    /// Explicit original building ID.
    Building(String),
    /// Explicit original wonder ID.
    Wonder(String),
    /// CAP resolves only through one current pinned capitalization definition.
    Capitalization,
    /// INF resolves only through one current pinned infrastructure definition.
    Infrastructure,
}
impl Item {
    /// Borrows one exact grammar category/ID without cloning cached templates.
    #[expect(clippy::pattern_type_mismatch, reason = "Borrow owned enum IDs without duplicating every cached template during preview and current-kind validation")]
    fn definition(&self) -> (&'static str, Option<&str>) {
        match self {
            Self::Unit(id) => ("unit", Some(id.as_str())), Self::Building(id) => ("building", Some(id.as_str())), Self::Wonder(id) => ("wonder", Some(id.as_str())),
            Self::Capitalization => ("capitalization", None), Self::Infrastructure => ("infrastructure", None),
        }
    }
}
/// Known complete local template; empty is different from a failed file read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Template {
    /// Ordered new content, including intentional duplicates.
    pub items: Vec<Item>,
}
/// Immutable preview snapshot; selecting a name never re-reads a different file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueFile {
    /// Exact opaque local catalog key, distinct from a content/entity ID.
    pub id: String,
    /// Actual safe user filename, not a localized game identity.
    pub name: String,
    /// Complete bounded parse or an honest unavailable reason.
    pub parsed: Result<Template, String>,
}
impl QueueFile {
    /// Decodes only lossless UTF-8/ASCII records, retaining parse failures visibly.
    #[must_use]
    pub fn from_bytes(id: String, name: String, bytes: &[u8]) -> Self {
        let parsed = core::str::from_utf8(bytes).map_err(|_error| "Saved queue is not valid UTF-8/ASCII".to_owned()).and_then(codec::parse);
        Self { id, name, parsed }
    }
}
/// Last explicit local directory snapshot; no file I/O is performed by rendering.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QueueLibrary {
    /// Exact local keys; duplicate names in distinct sources remain distinct.
    pub files: BTreeMap<String, QueueFile>,
    /// Directory/provider failure is not a fabricated empty catalog.
    pub error: Option<String>,
}

/*
 * Released source: ctp2_source/ctp2_code/ui/interface/EditQueue.cpp constructor,
 * {LoadModeCallback,ExitLoadMode,QueueFileList,DisplayQueueContents,LoadCallback,
 * LoadQueryCallback}; CityData.cpp::LoadQueue -> gs/gameobj/bldque.cpp::Load.
 * editqueue.ldl::BuildEditorWindow.{LoadBox,LoadBox.QueuesList,LoadBox.Contents,
 * LoadBox.QueueName,LoadBox.LoadButton,LoadBox.CancelButton} and eq_LoadListItem/
 * eq_ContentsItem establish original styling/geometry. QueryLoadQueue uses
 * str_ldl_EditQueueReallyLoad and cached messagebox.ldl OK/Cancel controls.
 * Client reads bounded local previews once per explicit request, resolves only
 * current pinned kinds/IDs, captures one atomic revision-bound replacement and
 * retains known funded head. No costs/private buildability/local CityData edits.
 * Failed/invalid files cannot clear a queue before open as original Load does.
 * Custom/save and destructive delete are separate unfinished local workflows;
 * neither is substituted with a raw authoritative comma/JSON editor.
 */
/// True only for the visible original inline browser mode.
pub(super) fn active(view: &ManagerView) -> bool {
    view.kind == ManagerKind::Build && view.selections.get(MODE).is_some_and(|value| value == "true")
}
/// Original source controls return presentation or captured confirmed commands.
pub(super) fn binding(ui: &ManagementUi, state: &GameState, view: &ManagerView, path: &str) -> Option<(Option<ManagerIntent>, Option<String>)> {
    if view.kind != ManagerKind::Build { return None; }
    match path {
        "ItemsBox.LoadSaveButton" => Some((Some(ManagerIntent::Select(MODE.to_owned(), "true".to_owned())), None)),
        "LoadBox.CancelButton" => Some((Some(ManagerIntent::Select(MODE.to_owned(), "false".to_owned())), None)),
        "LoadBox.DeleteButton" => Some(disabled("Original saved-file deletion confirmation/write backend is not implemented; no file is altered")),
        "NormalModeButtons.CustomButton" => Some(disabled("Original local custom-queue authoring/save is not implemented; this button is not a raw authoritative queue editor")),
        "LoadBox.LoadButton" => Some(load_dialog(ui, state, view).map_or_else(|reason| disabled(&reason), |dialog| (Some(ManagerIntent::Dialog(Box::new(dialog))), None))),
        _ => None,
    }
}
/// Exact cached filename selection, never a first-file fallback.
fn selected<'ui>(ui: &'ui ManagementUi, view: &ManagerView) -> Option<&'ui QueueFile> {
    ui.saved_queues.files.get(view.selections.get("saved-queue")?)
}
/// Source query captures a complete validated payload before any transport.
fn load_dialog(ui: &ManagementUi, state: &GameState, view: &ManagerView) -> Result<OperationDialog, String> {
    if !active(view) { return Err("Open the original saved-queue browser first".to_owned()); }
    if view.selections.get(REFRESH).is_some_and(|value| value == "true") { return Err("Saved queues are still being read".to_owned()); }
    if let Some(reason) = ui.saved_queues.error.as_ref() { return Err(reason.clone()); }
    let file = selected(ui, view).ok_or_else(|| "Select an exact saved queue first".to_owned())?;
    let template = file.parsed.as_ref().map_err(Clone::clone)?;
    let viewer = state.viewer.as_ref().ok_or_else(|| "Current server definitions are unavailable".to_owned())?;
    let items = resolve(viewer, template)?;
    let command = super::build::load_replacement(state, view, &items).ok_or_else(|| "A current owned city, exact editable queue and safe known funding/phase authorization are required".to_owned())?;
    let city = data::city(state, view).ok_or_else(|| "The chosen city is no longer available".to_owned())?;
    let key = "str_ldl_EditQueueReallyLoad";
    let format = ui.book.label(key);
    if format == key { return Err("Original load-confirmation string is unavailable".to_owned()); }
    let message = prompt(format, &[&file.name, &data::city_name(city)]).ok_or_else(|| "Original load-confirmation format is invalid".to_owned())?;
    let mut dialog = confirmation::question(ui, state, &message, command).ok_or_else(|| "Original current confirmation controls are unavailable".to_owned())?;
    dialog.close_load_on_submit = true;
    Ok(dialog)
}
/// Honest unavailable source control, never an enabled no-op.
fn disabled(reason: &str) -> (Option<ManagerIntent>, Option<String>) { (None, Some(reason.to_owned())) }

/// Source file/preview lists retain exact filename namespace and stable order.
pub(super) fn rows(ui: &ManagementUi, view: &ManagerView, path: &str) -> Option<Vec<(String, String)>> {
    if view.kind != ManagerKind::Build || !matches!(path, "LoadBox.QueuesList" | "LoadBox.Contents") { return None; }
    if path == "LoadBox.QueuesList" {
        /* Client integration difference: profile and read-only retail sources
         * may contain the same filename. Keep both exact identities and expose
         * their source in that case; never silently choose the first root.
         */
        let mut counts = BTreeMap::new();
        for file in ui.saved_queues.files.values() { let count = counts.entry(file.name.as_str()).or_insert(0_usize); *count = count.saturating_add(1_usize); }
        let mut files = ui.saved_queues.files.values().map(|file| {
            let label = if counts.get(file.name.as_str()).is_some_and(|count| *count > 1_usize) { format!("{} [{}]", file.name, file.id.split(':').next().unwrap_or("local")) } else { file.name.clone() };
            (file.id.clone(), label)
        }).collect::<Vec<_>>();
        files.sort_by(|left, right| left.1.cmp(&right.1).then(left.0.cmp(&right.0)));
        return Some(files);
    }
    Some(selected(ui, view).map_or_else(Vec::new, |file| file.parsed.as_ref().map_or_else(
        |reason| vec![("preview:unavailable".to_owned(), format!("— {reason}"))],
        |template| template.items.iter().enumerate().map(|(index, item)| (format!("preview:{index}"), label(ui, item))).collect(),
    )))
}
/// Preview labels are reference text, never evidence of current buildability.
fn label(ui: &ManagementUi, item: &Item) -> String {
    let (kind, id) = item.definition();
    data::label(ui, id.unwrap_or(if kind == "capitalization" { "CAPITALIZATION" } else { "INFRASTRUCTURE" }))
}
/// Actual file name/status overrides the obsolete numeric source placeholder.
pub(super) fn text(ui: &ManagementUi, view: &ManagerView, path: &str) -> Option<String> {
    if view.kind != ManagerKind::Build || path != "LoadBox.QueueName" { return None; }
    Some(if view.selections.get(REFRESH).is_some_and(|value| value == "true") { "Reading saved queues…".to_owned() }
        else if let Some(reason) = ui.saved_queues.error.as_ref() { reason.clone() }
        else { selected(ui, view).map_or_else(String::new, |file| file.name.clone()) })
}

/// Resolves each grammar category through current pinned metadata, preserving order.
///
/// # Errors
/// Missing/conflicting IDs/kinds or ambiguous semantic specials are unavailable.
pub fn resolve(viewer: &ServerProjection, template: &Template) -> Result<Vec<String>, String> {
    if template.items.is_empty() { return Ok(Vec::new()); }
    let rules = viewer.interface.rules.as_ref().ok_or_else(|| "Current server content definitions are unavailable".to_owned())?;
    let mut output = Vec::with_capacity(template.items.len());
    for item in &template.items {
        let (kind, explicit) = item.definition();
        let mut matches = BTreeSet::new();
        let mut kinds = BTreeSet::new();
        for category in ["units", "production"] {
            if let Some(definitions) = rules[category].as_array() { for definition in definitions {
                let Some(id) = definition["id"].as_str() else { continue; };
                let projected = definition["kind"].as_str().unwrap_or(if category == "units" { "unit" } else { "" });
                if explicit == Some(id) { kinds.insert(projected); }
                if projected == kind && explicit.is_none_or(|wanted| wanted == id) { matches.insert(id); }
            } }
        }
        if matches.len() != 1_usize || (explicit.is_some() && (kinds.len() != 1_usize || !kinds.contains(kind))) {
            return Err("Saved queue contains an unknown, wrong-kind or ambiguous current definition".to_owned());
        }
        let id = matches.first().ok_or_else(|| "Saved queue definition is unavailable".to_owned())?;
        output.push((*id).to_owned());
    }
    Ok(output)
}

/// Substitutes only original %s/%% slots, not printf, SLIC or inserted contents.
#[must_use]
pub fn prompt(format: &str, arguments: &[&str]) -> Option<String> {
    let mut output = String::new();let mut fields = 0_usize;let mut characters = format.chars();
    while let Some(character) = characters.next() {
        if character != '%' { output.push(character); continue; }
        match characters.next()? { '%' => output.push('%'), 's' => {output.push_str(arguments.get(fields)?);fields=fields.checked_add(1_usize)?;}, _ => return None }
    }
    (fields == arguments.len()).then_some(output)
}

/// Portable flat filename for local queue storage; never a path or device name.
#[must_use]
pub fn valid_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 256_usize || name.starts_with('.') || name.ends_with(['.', ' ']) || name.chars().any(|character| character.is_control() || matches!(character, '/' | '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')) { return false; }
    let base = name.split('.').next().unwrap_or(name).trim_end_matches(' ').to_ascii_uppercase();
    !matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL") && !["COM", "LPT"].iter().any(|prefix| base.strip_prefix(prefix).is_some_and(|number| matches!(number, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")))
}
