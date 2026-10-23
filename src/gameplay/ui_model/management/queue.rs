//! Atomic production editor requests; entry identity is never a content topic.
use serde_json::{Value, json};
use super::{ManagerCommand, ManagerView, data};
use crate::gameplay::model::GameState;

/// Builds one bounded, revision-checked replacement without mutating real state.
pub(super) fn request(state:&GameState, view:&ManagerView, path:&str)->Option<ManagerCommand> {
    let viewer=state.viewer.as_ref()?;
    let city=data::city(state,view)?;
    if !viewer.interface.legal_actions.iter().any(|action| action.route == "edit-production" && action.entity_id == city.id) { return None; }
    let mut entries=match city.facts["queue"].as_array() {
        Some(items) => items.iter().map(|item| item["entry_id"].as_str().map(|id| json!({"entry_id":id}))).collect::<Option<Vec<_>>>()?,
        None => Vec::new(),
    };
    let selected=view.selections.get("queue").and_then(|id| entries.iter().position(|entry| entry["entry_id"].as_str() == Some(id)));
    match path {
        "ItemsBox.AddButton" | "ItemsBox.InsertButton" => {
            let id=view.selections.get("buildable")?;
            if !crate::gameplay::server::commands::known_content(viewer,id) || entries.len() >= 256 { return None; }
            let index=if path == "ItemsBox.InsertButton" { selected? } else { entries.len() };
            entries.insert(index,json!({"item_id":id}));
        }
        "QueueGroup.RemoveButton" => { entries.remove(selected?); }
        "QueueGroup.UpButton" => { let index=selected?; entries.swap(index,index.checked_sub(1)?); }
        "QueueGroup.DownButton" => { let index=selected?; let next=index.checked_add(1).filter(|next| *next < entries.len())?; entries.swap(index,next); }
        "QueueGroup.ClearButton" => { if entries.is_empty() { return None; } entries.clear(); }
        _ => return None,
    }
    replacement(state,&city.id,&entries)
}

/// Validated complete custom selection; never accepts client-defined cost/effects.
pub(super) fn replacement(state:&GameState, city_id:&str, entries:&[Value])->Option<ManagerCommand> {
    let command=ManagerCommand { revision:state.viewer.as_ref()?.revision, route:"edit-production".to_owned(),payload:json!({"city_id":city_id,"entries":entries}) };
    command.valid_for(state).then_some(command)
}

/* Source: ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::ShowSelectedInfo/
 * Library and editqueue.ldl::{QueueGroup.QueueList,ItemDescription,LibraryButton}
 * resolve the selected queue's content, never a guessed neighboring row. Legacy
 * snapshots without entry_id may show their explicit content read-only; typed
 * entries never fall back to matching content IDs. Editing still requires exact
 * stable entry identities in request(), so this cannot grant a legacy command.
 */
/// Resolves exact typed entry content, or explicit read-only legacy content.
pub(super) fn content<'view>(state:&'view GameState, view:&ManagerView, entry:&str)->Option<&'view str> {
    data::city(state,view)?.facts["queue"].as_array()?.iter().find(|item| item["entry_id"].as_str().map_or_else(|| item["id"].as_str() == Some(entry), |id| id == entry)).and_then(|item| item["id"].as_str())
}
