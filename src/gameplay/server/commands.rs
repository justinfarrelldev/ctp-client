//! Typed parameter validation for server-authorized operation shapes.
//!
//! This is an input guard, not a second engine: costs, eligibility and outcomes
//! remain server-validated. A registered route or feature alone grants nothing.

use alloc::collections::BTreeSet;
use serde_json::Value;
use super::{ServerProjection, interface::LegalAction};

/// Rejects unknown/forged fields and preserves exact entity authorization.
#[must_use]
pub fn valid(viewer: &ServerProjection, route: &str, payload: &Value) -> bool {
    if !(LegalAction { route: route.to_owned(), ..LegalAction::default() }).http_supported() {
        return false;
    }
    let actions=&viewer.interface.legal_actions;
    let authorized=actions.iter().any(|action| action.route == route && action.payload == *payload);
    match route {
        "edit-production" => queue(viewer,payload),
        "rename-city" => city_action(viewer,route,payload) && shape(payload,&["city_id","name"],&[]) && payload["name"].as_str().is_some_and(|name| !name.trim().is_empty() && name.chars().count() <= 128 && !name.chars().any(char::is_control)),
        "set-governor" => governor(viewer,payload),
        "set-research" => shape(payload,&["advance_id"],&[]) && viewer.interface.research.as_ref().is_some_and(|research| payload["advance_id"].as_str().is_some_and(|id| research.available.iter().any(|available| available == id))) && (viewer.interface.contract.is_none() || (viewer.interface.planning_status != "queued" && viewer.interface.end_turn().is_some())),
        "set-movement-order" => coordinate(viewer,route,payload,false),
        "move" if payload.get("target_x").is_some() || payload.get("target_y").is_some() => coordinate(viewer,route,payload,true),
        _ => authorized,
    }
}

/// Exact required/optional JSON shape; no executable display metadata allowed.
pub(in crate::gameplay) fn shape(payload: &Value, required: &[&str], optional: &[&str]) -> bool {
    payload.as_object().is_some_and(|fields| required.iter().all(|field| fields.contains_key(*field)) && fields.keys().all(|field| required.contains(&field.as_str()) || optional.contains(&field.as_str())))
}

/// A city is selectable only through the viewer's current authorized operation.
fn city_action(viewer: &ServerProjection, route: &str, payload: &Value) -> bool {
    payload["city_id"].as_str().is_some_and(|id| viewer.interface.legal_actions.iter().any(|action| action.route == route && action.entity_id == id) && viewer.cities.iter().any(|city| city.id == id && !city.stale))
}

/// Canonical content must exist in the pin; executable item metadata is forbidden.
#[must_use]
pub(in crate::gameplay) fn known_content(viewer: &ServerProjection, id: &str) -> bool {
    viewer.interface.rules.as_ref().is_some_and(|rules| ["units","production"].iter().any(|category| rules[*category].as_array().is_some_and(|entries| entries.iter().any(|entry| entry["id"].as_str() == Some(id)))))
}

/// Complete atomic replacement, selecting one existing identity or new content.
fn queue(viewer: &ServerProjection, payload: &Value) -> bool {
    if !city_action(viewer,"edit-production",payload) || !shape(payload,&["city_id","entries"],&[]) { return false; }
    let Some(entries)=payload["entries"].as_array().filter(|entries| entries.len() <= 256) else { return false; };
    let Some(city)=viewer.cities.iter().find(|city| payload["city_id"].as_str() == Some(city.id.as_str())) else { return false; };
    let mut selected=BTreeSet::new();
    entries.iter().all(|entry| {
        if shape(entry,&["entry_id"],&[]) {
            entry["entry_id"].as_str().is_some_and(|id| selected.insert(id) && city.facts["queue"].as_array().is_some_and(|queue| queue.iter().any(|item| item["entry_id"].as_str() == Some(id))))
        } else { shape(entry,&["item_id"],&[]) && entry["item_id"].as_str().is_some_and(|id| known_content(viewer,id)) }
    })
}

/// Owner production focus is a pinned string, not a local AI decision.
fn governor(viewer: &ServerProjection, payload: &Value) -> bool {
    if !city_action(viewer,"set-governor",payload) || !shape(payload,&["city_id","enabled"],&["priority"]) { return false; }
    let Some(enabled)=payload["enabled"].as_bool() else { return false; };
    payload.get("priority").map_or_else(
        || !enabled || viewer.cities.iter().any(|city| payload["city_id"].as_str() == Some(city.id.as_str()) && city.governor.is_some()),
        |priority| priority.as_str().is_some_and(|id| viewer.interface.rules.as_ref().and_then(|rules| rules["governor_priorities"].as_array()).is_some_and(|priorities| priorities.iter().any(|focus| focus.as_str() == Some(id)))),
    )
}

/// Public coordinate shape; hidden passability is never inferred or looked up.
fn coordinate(viewer: &ServerProjection, route: &str, payload: &Value, adjacent: bool) -> bool {
    if !shape(payload,&["unit_id","target_x","target_y"],&[]) { return false; }
    let Some(id)=payload["unit_id"].as_str() else { return false; };
    if !viewer.interface.action_intents.iter().chain(&viewer.interface.legal_actions).any(|action| action.route == route && action.entity_id == id) { return false; }
    let (Some(x),Some(y))=(payload["target_x"].as_u64(),payload["target_y"].as_u64()) else { return false; };
    let Some((width,height))=viewer.interface.world.as_ref().and_then(super::v2::World::dimensions) else { return false; };
    if x >= u64::from(width) || y >= u64::from(height) { return false; }
    if !adjacent { return true; }
    viewer.units.iter().find(|unit| unit.id == id).and_then(|unit| viewer.tiles.iter().find(|tile| tile.id == unit.tile_id)).is_some_and(|tile| {
        let dx=u64::from(tile.x).abs_diff(x);
        let dy=u64::from(tile.y).abs_diff(y);
        dx.max(dy) == 1
    })
}
