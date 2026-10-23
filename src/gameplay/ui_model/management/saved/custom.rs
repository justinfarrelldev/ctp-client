//! Separate original local Custom editor; no city commands or filesystem effects.
use super::super::{data, ManagementUi, ManagerIntent, ManagerKind, ManagerView};
use super::{Item, Template};
use crate::gameplay::model::GameState;
use alloc::collections::{BTreeMap, BTreeSet};
#[cfg(test)]
mod tests;
/// Original Custom mode marker, never an authoritative city/planning flag.
pub const MODE: &str = "build-custom-mode";
/// View projection namespace; entry and content IDs remain separate.
const CONTENT: &str = "custom-content:";
/// Exact local entry identity, independent of identical content and server entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Opaque generation/counter key, never a queue position.
    pub id: String,
    /// Exact pinned content ID, not a cost or permission.
    pub content: String,
}
/// Original `m_customBuildList`, completely separate from canonical city facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft {
    /// Unique editor lifetime, retained across Library round trips.
    pub epoch: u64,
    /// Local immutable-intent revision, unrelated to remote gameplay revisions.
    pub version: u64,
    /// Monotonic identity allocation; removed identities cannot be reused.
    pub next: u64,
    /// Exact prior city/category/selections/rangers for source Custom Cancel.
    pub parent: ManagerView,
    /// Pinned public candidate categories only; no private buildability engine.
    pub catalog: BTreeMap<String, Item>,
    /// Ordered local template including intentional repeated units/imports.
    pub entries: Vec<Entry>,
}
/// Monotonic editor lifetime owner; read-only catalog refresh must preserve it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct State {
    /// Last allocated lifetime, not reset on Cancel/receipt/file refresh.
    pub epoch: u64,
    /// Active local authoring work, never received gameplay state.
    pub draft: Option<Draft>,
}
/// Exact local operation shared by real pointer/keyboard/accessibility dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    /// Append exact chosen content.
    Add(String),
    /// Insert chosen content before one exact existing local entry.
    Insert(String, String),
    /// Remove only this exact local entry, without an ordinary source Query.
    Remove(String),
    /// Move exact local entry one position up.
    Up(String),
    /// Move exact local entry one position down.
    Down(String),
    /// Clear local list without city Clear confirmation or server action.
    Clear,
}
/// Source callback intent; a stale descriptor cannot target a newer local draft.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Original `CustomButton` starts an empty independent template.
    Begin,
    /// Original CustomModeButtons.CancelButton restores the exact prior mode.
    Cancel,
    /// Local edit scoped to the exact authoring generation/version.
    Edit {
        /// Captured draft lifetime.
        epoch: u64,
        /// Captured local edit version.
        version: u64,
        /// Exact operation/identities.
        edit: Edit,
    },
    /// Original `LoadCustom` imports a complete cached file, not a city command.
    Import {
        /// Captured draft lifetime.
        epoch: u64,
        /// Captured local edit version.
        version: u64,
        /// Exact source+filename catalog key.
        file: String,
        /// Exact fully validated preview snapshot; never a later changed file.
        items: Vec<String>,
    },
}
/*
 * Released source: ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::CustomButton
 * -> SetMode(EDIT_QUEUE_MODE_CUSTOM), {UpdateChoiceLists,UpdateQueueList,
 * UpdateButtons}, {Add,InsertInQueue,Remove,Up,Down,ClearButton}, LoadCallback ->
 * LoadCustom. Source sets m_cityData=NULL and starts a separate empty custom list;
 * unit duplicates are allowed, queued buildings/wonders/CAP/INF leave candidates.
 * editqueue.ldl::BuildEditorWindow.{ItemsBox.ModeLabel,QueueGroup.QueueLabel,
 * QueueGroup.QueueList,ItemsBox lists/buttons,NormalModeButtons,CustomModeButtons,
 * SingleCityChooser,RushBuyButton,GotoCityButton} supplies unchanged styles/bounds.
 * Client difference: only known public pinned kinds/IDs, not private CanBuild or
 * projected city ETA/cost; missing original CantBuild/HasPopAndCanBuild flags leave
 * full All Items fidelity unaccepted. No inferred false flags, gameplay/paid-head
 * mutation, filesystem writes, first-row selection or duplicate-content removal.
 * Immutable local version protects authoring without binding it to remote ticks.
 * Save uses the separate source TextQuery/create-only persistence workflow;
 * never substitute an enabled raw authoritative city queue action.
 */
/// Visible local editor, including its optional source `LoadBox` overlay.
#[must_use]
pub fn active(view: &ManagerView) -> bool {
    view.kind == ManagerKind::Build
        && view
            .selections
            .get(MODE)
            .is_some_and(|value| value == "true")
}
/// Exact local content projection for article/image/description identity.
pub(in crate::gameplay::ui_model::management) fn content<'view>(
    view: &'view ManagerView,
    entry: &str,
) -> Option<&'view str> {
    active(view)
        .then(|| {
            view.selections
                .get(&format!("{CONTENT}{entry}"))
                .map(String::as_str)
        })
        .flatten()
}
/// Source callbacks are local, not command placeholders; invalid choices disabled.
pub(in crate::gameplay::ui_model::management) fn binding(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    path: &str,
) -> Option<(Option<ManagerIntent>, Option<String>)> {
    if view.kind != ManagerKind::Build {
        return None;
    }
    if path == "NormalModeButtons.CustomButton" {
        return Some(
            if ui.saved_queues.custom.epoch < u64::MAX
                && state
                    .viewer
                    .as_ref()
                    .and_then(|viewer| viewer.interface.rules.as_ref())
                    .is_some()
            {
                (Some(ManagerIntent::Custom(Action::Begin)), None)
            } else {
                disabled("Current public pinned candidate definitions are unavailable")
            },
        );
    }
    if !active(view) {
        return None;
    }
    let draft = ui.saved_queues.custom.draft.as_ref()?;
    let choice = view.selections.get("buildable").cloned();
    let selected = view.selections.get("queue").cloned();
    let edit=match path {
        "CustomModeButtons.CancelButton"=>return Some((Some(ManagerIntent::Custom(Action::Cancel)),None)),
        "CustomModeButtons.SaveButton"=>return Some(super::save::dialog(ui).map_or_else(||disabled("A complete serializable current local template and original Save-name controls are required"),|dialog|(Some(ManagerIntent::Dialog(Box::new(dialog))),None))),
        "RushBuyButton"|"GotoCityButton"=>return Some(disabled("A local custom template is not city production")),
        "ItemsBox.AddButton"=>choice.map(Edit::Add),
        "ItemsBox.InsertButton"=>choice.zip(selected).map(|(item,entry)|Edit::Insert(item,entry)),
        "QueueGroup.RemoveButton"=>selected.map(Edit::Remove),
        "QueueGroup.UpButton"=>selected.map(Edit::Up),
        "QueueGroup.DownButton"=>selected.map(Edit::Down),
        "QueueGroup.ClearButton"=>Some(Edit::Clear),
        "LoadBox.LoadButton"=>return Some(import_intent(ui,state,view,draft).map_or_else(||disabled("Select a complete valid saved template with exact known pinned kinds first"),|action|(Some(ManagerIntent::Custom(action)),None))),
        _=>return None,
    };
    Some(edit.filter(|edit| can_edit(draft, view, edit)).map_or_else(
        || disabled("Select an exact available local item/entry and valid list boundary"),
        |edit| {
            (
                Some(ManagerIntent::Custom(Action::Edit {
                    epoch: draft.epoch,
                    version: draft.version,
                    edit,
                })),
                None,
            )
        },
    ))
}
/// Honest unfinished/invalid source control reason.
fn disabled(reason: &str) -> (Option<ManagerIntent>, Option<String>) {
    (None, Some(reason.to_owned()))
}
/// Source `LoadCustom` uses complete validated cached bytes, never a destructive Query.
fn import_intent(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    draft: &Draft,
) -> Option<Action> {
    if !super::active(view)
        || view.selections.contains_key(super::REFRESH)
        || ui.saved_queues.error.is_some()
    {
        return None;
    }
    let file = view.selections.get("saved-queue")?;
    let template = ui.saved_queues.files.get(file)?.parsed.as_ref().ok()?;
    let ids = super::resolve(state.viewer.as_ref()?, template).ok()?;
    if ids.iter().any(|id| !draft.catalog.contains_key(id)) {
        return None;
    }
    draft.next.checked_add(u64::try_from(ids.len()).ok()?)?;
    draft.version.checked_add(1_u64)?;
    Some(Action::Import {
        epoch: draft.epoch,
        version: draft.version,
        file: file.clone(),
        items: ids,
    })
}
/// Original special conversion records share the source building category.
const fn category_of(item: &Item) -> &'static str {
    match *item { Item::Unit(_) => "Units", Item::Wonder(_) => "Wonders", Item::Building(_) | Item::Capitalization | Item::Infrastructure => "Buildings" }
}
/// Exact candidate membership, category and non-unit duplicate filtering.
fn available(draft: &Draft, view: &ManagerView, id: &str) -> bool {
    let Some(item) = draft.catalog.get(id) else {
        return false;
    };
    let category = category_of(item);
    view.selections
        .get("build-category")
        .map_or("Units", String::as_str)
        == category
        && (matches!(*item, Item::Unit(_))
            || !draft.entries.iter().any(|entry| entry.content == id))
}
/// Public list projection; no production turn forecast is invented for templates.
pub(in crate::gameplay::ui_model::management) fn rows(
    ui: &ManagementUi,
    view: &ManagerView,
    path: &str,
) -> Option<Vec<(String, String)>> {
    if !active(view) {
        return None;
    }
    let draft = ui.saved_queues.custom.draft.as_ref()?;
    if path == "QueueGroup.QueueList" {
        return Some(
            draft
                .entries
                .iter()
                .map(|entry| (entry.id.clone(), data::label(ui, &entry.content)))
                .collect(),
        );
    }
    let category = match path {
        "ItemsBox.UnitsList" => "Units",
        "ItemsBox.BuildingList" => "Buildings",
        "ItemsBox.WonderList" => "Wonders",
        _ => return None,
    };
    // No cloning up to256 entry/content projections per frame, nor N*queue scans.
    let queued = draft.entries.iter().map(|entry|entry.content.as_str()).collect::<BTreeSet<_>>();
    let mut rows = draft
        .catalog
        .iter()
        .filter(|pair|category_of(pair.1)==category && (matches!(*pair.1,Item::Unit(_)) || !queued.contains(pair.0.as_str())))
        .map(|pair| (pair.0.clone(), data::label(ui, pair.0)))
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.1.cmp(&right.1).then(left.0.cmp(&right.0)));
    Some(rows)
}
/// Original custom captions; source time column stays unavailable, not city ETA.
pub(in crate::gameplay::ui_model::management) fn text(
    ui: &ManagementUi,
    view: &ManagerView,
    path: &str,
) -> Option<String> {
    if !active(view) {
        return None;
    }
    match path {
        "ItemsBox.ModeLabel" => Some(ui.book.label("str_ldl_EditQueueAllItems").to_owned()),
        "QueueGroup.QueueLabel" => Some(ui.book.label("str_ldl_EditQueueCustomQueue").to_owned()),
        "RushBuyCost" => Some("---".to_owned()),
        _ => None,
    }
}
/// Local edits have exact choice/entry scopes and cannot touch a neighbor.
#[expect(
    clippy::pattern_type_mismatch,
    reason = "Borrow exact local action IDs without duplicating immutable action payloads"
)]
fn can_edit(draft: &Draft, view: &ManagerView, edit: &Edit) -> bool {
    if draft.version == u64::MAX
        || (matches!(*edit, Edit::Add(_) | Edit::Insert(..)) && draft.next == u64::MAX)
    {
        return false;
    }
    let index = |id: &str| {
        draft.entries.iter().position(|entry| {
            entry.id == id
                && view
                    .selections
                    .get("queue")
                    .is_some_and(|selected| selected == id)
        })
    };
    match edit {
        Edit::Add(id) => {
            draft.entries.len() < super::codec::MAX_ITEMS
                && view.selections.get("buildable") == Some(id)
                && available(draft, view, id)
        }
        Edit::Insert(id, entry) => {
            draft.entries.len() < super::codec::MAX_ITEMS
                && view.selections.get("buildable") == Some(id)
                && available(draft, view, id)
                && index(entry).is_some()
        }
        Edit::Remove(id) => index(id).is_some(),
        Edit::Up(id) => index(id).is_some_and(|position| position > 0_usize),
        Edit::Down(id) => {
            index(id).is_some_and(|position| position.saturating_add(1_usize) < draft.entries.len())
        }
        Edit::Clear => !draft.entries.is_empty(),
    }
}
/// Applies native and accessible local callbacks; never submits any server route.
#[must_use]
pub fn apply(ui: &mut ManagementUi, state: &GameState, action: Action) -> bool {
    if action == Action::Begin {
        return begin(ui, state);
    }
    if action == Action::Cancel {
        return cancel(ui);
    }
    let Some(view) = ui.view.as_mut().filter(|view| active(view)) else {
        return false;
    };
    let Some(draft) = ui.saved_queues.custom.draft.as_mut() else {
        return false;
    };
    let (epoch, version) = match action {
        Action::Edit { epoch, version, .. } | Action::Import { epoch, version, .. } => {
            (epoch, version)
        }
        Action::Begin | Action::Cancel => return false,
    };
    if draft.epoch != epoch || draft.version != version {
        return false;
    }
    let Some(next_version) = version.checked_add(1_u64) else {
        return false;
    };
    match action {
        Action::Edit { edit, .. } => {
            if !can_edit(draft, view, &edit) || !edit_list(draft, &edit) {
                return false;
            }
        }
        Action::Import { file, items, .. } => {
            let Some(cached) = ui
                .saved_queues
                .files
                .get(&file)
                .filter(|cached| view.selections.get("saved-queue") == Some(&cached.id))
            else {
                return false;
            };
            let Some(ids) = cached
                .parsed
                .as_ref()
                .ok()
                .and_then(|template| super::resolve(state.viewer.as_ref()?, template).ok())
            else {
                return false;
            };
            if ids != items || ids.iter().any(|id| !draft.catalog.contains_key(id)) {
                return false;
            }
            let mut replacement = Vec::with_capacity(ids.len());
            let mut counter = draft.next;
            for id in ids {
                let Some(next) = counter.checked_add(1_u64) else {
                    return false;
                };
                counter = next;
                replacement.push(Entry {
                    id: format!("custom:{epoch}:{counter}"),
                    content: id,
                });
            }
            draft.next = counter;
            draft.entries = replacement;
            super::super::build::select(view, super::MODE, "false");
        }
        Action::Begin | Action::Cancel => return false,
    }
    draft.version = next_version;
    project(view, draft);
    true
}
/// Allocates one local entry without reusing removed IDs or overflowing counters.
#[expect(
    clippy::pattern_type_mismatch,
    reason = "Borrow exact entry/content IDs while mutating a separate owned local list, avoiding clones"
)]
fn edit_list(draft: &mut Draft, edit: &Edit) -> bool {
    let index = |id: &str| draft.entries.iter().position(|entry| entry.id == id);
    match edit {
        Edit::Add(id) | Edit::Insert(id, _) => {
            let position = match edit {
                Edit::Insert(_, entry) => index(entry),
                _ => Some(draft.entries.len()),
            };
            let Some(position) = position else {
                return false;
            };
            let Some(next) = draft.next.checked_add(1_u64) else {
                return false;
            };
            draft.next = next;
            draft.entries.insert(
                position,
                Entry {
                    id: format!("custom:{}:{next}", draft.epoch),
                    content: id.clone(),
                },
            );
        }
        Edit::Remove(id) => {
            let Some(position) = index(id) else {
                return false;
            };
            draft.entries.remove(position);
        }
        Edit::Up(id) => {
            let Some(position) = index(id) else {
                return false;
            };
            let Some(before) = position.checked_sub(1_usize) else {
                return false;
            };
            draft.entries.swap(position, before);
        }
        Edit::Down(id) => {
            let Some(position) = index(id) else {
                return false;
            };
            let Some(after) = position
                .checked_add(1_usize)
                .filter(|after| *after < draft.entries.len())
            else {
                return false;
            };
            draft.entries.swap(position, after);
        }
        Edit::Clear => draft.entries.clear(),
    }
    true
}
/// Starts a public pinned local catalog, rejecting ambiguous kind identities.
fn begin(ui: &mut ManagementUi, state: &GameState) -> bool {
    let Some(parent) = ui
        .view
        .as_ref()
        .filter(|view| view.kind == ManagerKind::Build && !active(view))
        .cloned()
    else {
        return false;
    };
    let Some(rules) = state
        .viewer
        .as_ref()
        .and_then(|viewer| viewer.interface.rules.as_ref())
    else {
        return false;
    };
    let mut catalog = BTreeMap::new();
    let mut conflicts = BTreeSet::new();
    for category in ["units", "production"] {
        if let Some(definitions) = rules[category].as_array() {
            for definition in definitions {
                let Some(id) = definition["id"].as_str().filter(|id| !id.is_empty()) else {
                    continue;
                };
                let kind = definition["kind"]
                    .as_str()
                    .unwrap_or(if category == "units" { "unit" } else { "" });
                let item = match kind {
                    "unit" => {
                        if definition["cant_build"].as_bool() == Some(true)
                            || definition["has_pop_and_can_build"].as_bool() == Some(true)
                        {
                            continue;
                        }
                        Item::Unit(id.to_owned())
                    }
                    "building" => Item::Building(id.to_owned()),
                    "wonder" => Item::Wonder(id.to_owned()),
                    "capitalization" => Item::Capitalization,
                    "infrastructure" => Item::Infrastructure,
                    _ => continue,
                };
                if catalog.get(id).is_some_and(|prior| *prior != item) {
                    conflicts.insert(id.to_owned());
                }
                catalog.insert(id.to_owned(), item);
            }
        }
    }
    for id in conflicts {
        catalog.remove(&id);
    }
    let Some(epoch) = ui.saved_queues.custom.epoch.checked_add(1_u64) else {
        return false;
    };
    ui.saved_queues.custom.epoch = epoch;
    ui.saved_queues.custom.draft = Some(Draft {
        epoch,
        version: 0_u64,
        next: 0_u64,
        parent,
        catalog,
        entries: Vec::new(),
    });
    ui.view = Some(ManagerView {
        kind: ManagerKind::Build,
        city_id: None,
        selections: BTreeMap::from([(MODE.to_owned(), "true".to_owned())]),
        scroll: BTreeMap::new(),
    });
    ui.focused = None;
    true
}
/// Restores source old mode, never a first/neighboring current city.
#[must_use]
pub fn cancel(ui: &mut ManagementUi) -> bool {
    if !ui.view.as_ref().is_some_and(active) {
        return false;
    }
    let Some(draft) = ui.saved_queues.custom.draft.take() else {
        return false;
    };
    ui.view = Some(draft.parent);
    ui.focused = None;
    true
}
/// Rebuilds only local content projections and clears vanished/hidden selections.
fn project(view: &mut ManagerView, draft: &Draft) {
    view.selections
        .retain(|key, _value| !key.starts_with(CONTENT));
    for entry in &draft.entries {
        view.selections
            .insert(format!("{CONTENT}{}", entry.id), entry.content.clone());
    }
    for role in ["queue", "buildable"] {
        let valid = view.selections.get(role).is_none_or(|id| {
            if role == "queue" {
                draft.entries.iter().any(|entry| entry.id == *id)
            } else {
                available(draft, view, id)
            }
        });
        if !valid {
            view.selections.remove(role);
            if view
                .selections
                .get("description-source")
                .is_some_and(|source| source == role)
            {
                view.selections.remove("description");
                view.selections.remove("description-source");
            }
        }
    }
    super::super::build::description::sync(view);
}
/// File-format projection keeps category semantics; duplicate units stay ordered.
#[must_use]
pub fn template(draft: &Draft) -> Option<Template> {
    Some(Template {
        items: draft
            .entries
            .iter()
            .map(|entry| draft.catalog.get(&entry.content).cloned())
            .collect::<Option<Vec<_>>>()?,
    })
}
