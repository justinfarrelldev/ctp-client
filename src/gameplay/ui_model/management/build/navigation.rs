//! Original Build list keyboard selection and received queue-position visibility.
use super::super::{ManagementUi, ManagerIntent, ManagerKind, ManagerView, data, lists, tree};
use crate::gameplay::{model::GameState, ui_model::BuildListKey};

/*
 * Released source: ctp2_source/ctp2_code/ui/interface/EditQueue.cpp constructor
 * SetKeyboardActionControl(m_addButton), {ListCallback,QueueListCallback,Up,Down,
 * InsertInQueue}; ui/aui_common/aui_listbox.cpp::{HandleKey,SendKeyboardAction,
 * SelectItem}; ui/aui_ctp2/ctp2_listbox.cpp::EnsureItemVisible. Original
 * editqueue.ldl::BuildEditorWindow.ItemsBox lists/QueueGroup.QueueList keep source
 * pane/ranger/style/dimensions. Up/Down wraps only the focused list; Space maps
 * only left lists to the existing Add callback, never queue Remove.
 * Original HandleKey/SelectItem omits EnsureItemVisible; explicitly fix offscreen
 * keyboard selection instead of copying that defect. Queue edits stay immutable
 * intents: only actually received index changes trigger source ensure-visible,
 * not queued receipts, unrelated snapshots or an invented neighboring selection.
 */
/// Exact original current list controls, not arbitrary path-like focus strings.
const PATHS: [&str; 6] = ["ItemsBox.UnitsList", "ItemsBox.BuildingList", "ItemsBox.WonderList", "QueueGroup.QueueList", "LoadBox.QueuesList", "LoadBox.Contents"];
/// Presentation-only anchor keys; never serialized as authoritative commands.
const ANCHOR: [&str; 3] = ["queue-viewport-entry", "queue-viewport-city", "queue-viewport-index"];

/// Maps the source selection role to its one currently visible list.
fn role_path(view: &ManagerView, role: &str) -> Option<&'static str> {
    if view.kind != ManagerKind::Build { return None; }
    match role {
        "queue" => Some(PATHS[3_usize]),
        "saved-queue" if super::super::saved::active(view) => Some(PATHS[4_usize]),
        "saved-preview" if super::super::saved::active(view) => Some(PATHS[5_usize]),
        "buildable" => match view.selections.get("build-category").map_or("Units", String::as_str) {
            "Units" => Some(PATHS[0_usize]), "Buildings" => Some(PATHS[1_usize]), "Wonders" => Some(PATHS[2_usize]), _ => None,
        },
        _ => None,
    }
}

/// Source pointer selection gives keyboard focus to its exact current list.
pub(in crate::gameplay::ui_model::management) fn focus(ui: &mut ManagementUi, state: &GameState, role: &str, id: &str) -> bool {
    let Some(view) = ui.view.as_ref() else { return false; };
    let Some(path) = role_path(view, role) else { return false; };
    if !data::rows(ui, state, view, path).iter().any(|row| row.0 == id) { return false; }
    if !ui.widgets(state).iter().any(|widget| widget.id == path) { return false; }
    ui.focused = Some(path.to_owned());
    true
}

/// Resolves only an actual current list or a visible semantic row/cell in it.
fn focused(ui: &ManagementUi, state: &GameState) -> Option<&'static str> {
    let view = ui.view.as_ref().filter(|view| view.kind == ManagerKind::Build)?;
    let focus = ui.focused.as_deref()?;
    let path = PATHS.iter().copied().find(|path| focus == *path || focus.strip_prefix(path).is_some_and(|suffix| suffix.starts_with('/')))?;
    let role = match path { "QueueGroup.QueueList" => "queue", "LoadBox.QueuesList" => "saved-queue", "LoadBox.Contents" => "saved-preview", _ => "buildable" };
    if role_path(view, role) != Some(path) { return None; }
    let widgets = ui.widgets(state);
    if !widgets.iter().any(|widget| widget.id == path) { return None; }
    if focus != path && !widgets.iter().any(|widget| widget.id == focus && matches!(widget.intent.clone(), Some(ManagerIntent::Select(key, _)) if key == role)) { return None; }
    Some(path)
}

/// Handled input may select only; only Space on a left list yields an Add intent.
pub(in crate::gameplay::ui_model::management) fn key(ui: &mut ManagementUi, state: &GameState, key: BuildListKey) -> (bool, Option<ManagerIntent>) {
    let Some(path) = focused(ui, state) else { return (false, None); };
    if key == BuildListKey::Activate {
        return (true, path.starts_with("ItemsBox.").then(|| ui.activate_id("ItemsBox.AddButton", state)).flatten());
    }
    let Some(view) = ui.view.as_ref() else { return (false, None); };
    let rows = data::rows(ui, state, view, path);
    if rows.is_empty() { return (true, None); }
    let role = match path { "QueueGroup.QueueList" => "queue", "LoadBox.QueuesList" => "saved-queue", "LoadBox.Contents" => "saved-preview", _ => "buildable" };
    let current = view.selections.get(role).and_then(|id| rows.iter().position(|row| row.0 == *id));
    let index = match key {
        BuildListKey::Up => current.unwrap_or(0_usize).checked_sub(1_usize).unwrap_or_else(|| rows.len().saturating_sub(1_usize)),
        BuildListKey::Down => current.map_or(0_usize, |index| index.saturating_add(1_usize).checked_rem(rows.len()).unwrap_or(0_usize)),
        BuildListKey::Activate => return (true, None),
    };
    let Some(id) = rows.get(index).map(|row| row.0.clone()) else { return (true, None); };
    if ui.apply_local(ManagerIntent::Select(role.to_owned(), id)) {
        ui.focused = Some(path.to_owned());
        ensure(ui, state, path, index, rows.len());
        ui.refresh_build_description(state);
    }
    (true, None)
}

/// Actual source pane capacity, not borders or guessed fixed row counts.
fn ensure(ui: &mut ManagementUi, state: &GameState, path: &str, index: usize, count: usize) {
    let Some(widget) = ui.widgets(state).into_iter().find(|widget| widget.id == path) else { return; };
    let Some(view) = ui.view.as_ref() else { return; };
    let page = tree::visible_rows(ui, view.kind, path, lists::content(&widget));
    if page == 0_usize { return; }
    let maximum = count.saturating_sub(page);
    let previous = view.scroll.get(path).copied();
    let offset = previous.unwrap_or(0_usize).min(maximum);
    let next = (if index < offset { index } else if index >= offset.saturating_add(page) { index.saturating_add(1_usize).saturating_sub(page) } else { offset }).min(maximum);
    if previous.is_some() || next > 0_usize {
        if let Some(current) = ui.view.as_mut() { current.scroll.insert(path.to_owned(), next); }
    }
}

/// Tracks stable entry positions, scrolling only after a real received move.
pub(in crate::gameplay::ui_model::management) fn track(ui: &mut ManagementUi, state: &GameState) {
    let Some(view) = ui.view.as_ref().filter(|view| view.kind == ManagerKind::Build) else { return; };
    let position = view.selections.get("queue").and_then(|id| {
        let city = data::city(state, view)?;
        let entries = city.facts["queue"].as_array()?;
        let index = entries.iter().position(|entry| entry["entry_id"].as_str() == Some(id))?;
        Some((id.clone(), city.id.clone(), index, entries.len()))
    });
    let Some((id, city, index, count)) = position else {
        if let Some(current) = ui.view.as_mut() { for key in ANCHOR { current.selections.remove(key); } }
        return;
    };
    let same = view.selections.get(ANCHOR[0_usize]) == Some(&id) && view.selections.get(ANCHOR[1_usize]) == Some(&city);
    let moved = same && view.selections.get(ANCHOR[2_usize]).and_then(|value| value.parse::<usize>().ok()).is_some_and(|previous| previous != index);
    if moved { ensure(ui, state, PATHS[3_usize], index, count); }
    if let Some(current) = ui.view.as_mut() {
        for (key, value) in [(ANCHOR[0_usize], id), (ANCHOR[1_usize], city), (ANCHOR[2_usize], index.to_string())] { current.selections.insert(key.to_owned(), value); }
    }
}
