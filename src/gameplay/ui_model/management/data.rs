//! Viewer-safe manager lists and values; no invented economy, contacts or names.

use alloc::collections::BTreeMap;

use super::{ManagementUi, ManagerKind, ManagerView, articles};
use crate::gameplay::model::{CityId, GameState, PlayerId, UnitId};
use crate::gameplay::server::interface::LegalAction;
use crate::gameplay::server::{ServerCityProjection, ServerUnitProjection};

/// Finds exact owned city identities using the authenticated render-owner mapping.
pub(super) fn owned_city_ids(state: &GameState) -> Vec<String> {
    state.viewer.as_ref().map_or_else(Vec::new, |viewer| {
        viewer
            .cities
            .iter()
            .filter(|source| {
                !source.stale
                    && state.cities.iter().any(|city| {
                        city.id == CityId::from_server_id(&source.id)
                            && city.owner == PlayerId::Human
                    })
            })
            .map(|source| source.id.clone())
            .collect()
    })
}

/// Selected city facts must still belong to the viewer after a receipt/resync.
pub(super) fn city<'state>(
    state: &'state GameState,
    view: &ManagerView,
) -> Option<&'state ServerCityProjection> {
    let id = view.city_id.as_deref()?;
    owned_city_ids(state)
        .iter()
        .any(|owned| owned == id)
        .then_some(())?;
    state
        .viewer
        .as_ref()?
        .cities
        .iter()
        .find(|source| source.id == id)
}

/// Exact living owned units, not every visible foreign actor.
fn owned_units(state: &GameState) -> Vec<&ServerUnitProjection> {
    state.viewer.as_ref().map_or_else(Vec::new, |viewer| {
        viewer
            .units
            .iter()
            .filter(|source| {
                state.units.iter().any(|unit| {
                    unit.alive
                        && unit.owner == PlayerId::Human
                        && unit.id == UnitId::from_server_id(&source.id)
                })
            })
            .collect()
    })
}

/// Selected action candidates for the current city, with no first-target guessing.
pub(super) fn city_actions<'state>(
    state: &'state GameState,
    view: &ManagerView,
    route: &str,
) -> Vec<&'state LegalAction> {
    let Some(city) = city(state, view) else {
        return Vec::new();
    };
    state.viewer.as_ref().map_or_else(Vec::new, |viewer| {
        viewer
            .interface
            .legal_actions
            .iter()
            .filter(|action| {
                action.route == route && action.entity_id == city.id && action.http_supported()
            })
            .collect()
    })
}

/// Returns one available list's stable IDs and display labels.
pub(super) fn rows(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    path: &str,
) -> Vec<(String, String)> {
    if view.kind == ManagerKind::Library {
        return super::index::rows(ui, state);
    }
    let Some(viewer) = state.viewer.as_ref() else {
        return Vec::new();
    };
    match view.kind {
        ManagerKind::Build if path == "QueueGroup.QueueList" => city(state, view)
            .and_then(|city| city.facts["queue"].as_array())
            .map_or_else(Vec::new, |queue| {
                queue
                    .iter()
                    .filter_map(|item| item["id"].as_str().map(|id| (id.to_owned(), label(ui, id))))
                    .collect()
            }),
        ManagerKind::Build if path.starts_with("ItemsBox.") => buildables(ui, state, view, path),
        ManagerKind::Science | ManagerKind::Research => viewer
            .interface
            .research
            .as_ref()
            .map_or_else(Vec::new, |research| {
                research
                    .available
                    .iter()
                    .map(|id| (id.clone(), label(ui, id)))
                    .collect()
            }),
        ManagerKind::National => owned_city_ids(state)
            .into_iter()
            .map(|id| {
                let title = viewer
                    .cities
                    .iter()
                    .find(|city| city.id == id)
                    .map_or_else(|| id.clone(), city_name);
                (id, title)
            })
            .collect(),
        ManagerKind::Unit => unit_rows(ui, state, path),
        ManagerKind::Trade => trade_rows(viewer, path),
        ManagerKind::Diplomacy => viewer
            .diplomacy
            .as_ref()
            .and_then(|diplomacy| diplomacy.facts["relations"].as_array())
            .map_or_else(Vec::new, |relations| {
                relations
                    .iter()
                    .filter_map(|relation| {
                        let id = relation["target_player_id"].as_str()?;
                        Some((
                            id.to_owned(),
                            format!("{id}      {}", relation["treaty"].as_str().unwrap_or("—")),
                        ))
                    })
                    .collect()
            }),

        ManagerKind::Army => owned_units(state)
            .iter()
            .map(|unit| {
                (
                    unit.id.clone(),
                    format!("{} [{}]", label(ui, &unit.type_id), unit.id),
                )
            })
            .collect(),
        ManagerKind::City
        | ManagerKind::Empire
        | ManagerKind::Statistics
        | ManagerKind::Graphics
        | ManagerKind::Sound
        | ManagerKind::Build
        | ManagerKind::Library => Vec::new(),
    }
}

/// Exact tactical identities or source-style type counts, never foreign units.
fn unit_rows(ui: &ManagementUi, state: &GameState, path: &str) -> Vec<(String, String)> {
    if path.contains("Tactical") {
        return owned_units(state)
            .iter()
            .map(|unit| {
                (
                    unit.id.clone(),
                    format!("{} [{}]", label(ui, &unit.type_id), unit.id),
                )
            })
            .collect();
    }
    let mut counts = BTreeMap::<String, usize>::new();
    for unit in owned_units(state) {
        let count = counts.entry(unit.type_id.clone()).or_default();
        *count = count.saturating_add(1);
    }
    counts
        .into_iter()
        .map(|(id, count)| {
            let title = format!("{}      {count}", label(ui, &id));
            (id, title)
        })
        .collect()
}

/// Route lists retain projected endpoint identities without inventing goods/income.
fn trade_rows(
    viewer: &crate::gameplay::server::ServerProjection,
    path: &str,
) -> Vec<(String, String)> {
    if path.contains("Market") {
        return viewer
            .interface
            .legal_actions
            .iter()
            .filter(|action| action.route == "create-trade-route" && action.http_supported())
            .map(|action| {
                (
                    action.target_id.clone(),
                    format!("{} → {}", action.entity_id, action.target_id),
                )
            })
            .collect();
    }
    viewer
        .interface
        .trade_routes
        .iter()
        .filter_map(|route| {
            let id = route["id"].as_str()?;
            Some((
                id.to_owned(),
                format!(
                    "{} → {}",
                    route["source_city_id"].as_str().unwrap_or("—"),
                    route["destination_city_id"].as_str().unwrap_or("—")
                ),
            ))
        })
        .collect()
}

/// Original buildable candidates plus current queue types, sorted by localized name.
fn buildables(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    path: &str,
) -> Vec<(String, String)> {
    let kind = match path {
        "ItemsBox.UnitsList" => "unit",
        "ItemsBox.BuildingList" => "building",
        "ItemsBox.WonderList" => "wonder",
        _ => return Vec::new(),
    };
    let mut records = BTreeMap::new();
    for action in city_actions(state, view, "queue-production") {
        if action.payload["item"]["kind"].as_str() == Some(kind) {
            records.insert(action.target_id.clone(), label(ui, &action.target_id));
        }
    }
    if let Some(queue) = city(state, view).and_then(|city| city.facts["queue"].as_array()) {
        for item in queue {
            if item["kind"].as_str() == Some(kind) {
                if let Some(id) = item["id"].as_str() {
                    records.insert(id.to_owned(), label(ui, id));
                }
            }
        }
    }
    let mut rows: Vec<_> = records.into_iter().collect();
    rows.sort_by(|left, right| left.1.cmp(&right.1));
    rows
}

/// Original content names when available; explicit readable canonical fallback.
pub(super) fn label(ui: &ManagementUi, id: &str) -> String {
    let localized = ui.book.label(id);
    if localized != id {
        return localized.to_owned();
    }
    let name = [
        "UNIT_",
        "ADVANCE_",
        "IMPROVE_",
        "BUILDING_",
        "WONDER_",
        "GOVERNMENT_",
    ]
    .into_iter()
    .find_map(|prefix| id.strip_prefix(prefix))
    .unwrap_or(id);
    let mut label = name.to_ascii_lowercase().replace('_', " ");
    if let Some(first) = label.get_mut(..1) {
        first.make_ascii_uppercase();
    }
    label
}

/// Authoritative name if a newer projector provides it; never invent Rome.
pub(super) fn city_name(city: &ServerCityProjection) -> String {
    city.facts["name"].as_str().unwrap_or(&city.id).to_owned()
}

/// Replaces source placeholder numbers only with known current viewer facts.
pub(super) fn text(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    path: &str,
) -> Option<String> {
    if let Some(text) = articles::text(ui, view, path) {
        return Some(text);
    }
    let viewer = state.viewer.as_ref()?;
    match view.kind {
        ManagerKind::Science => {
            let research = viewer.interface.research.as_ref()?;
            match path {
                "CurrentResearch.AdvanceProgress.Title" => Some(label(ui, &research.target_id)),
                "CurrentResearch.Spent.Value" => Some(research.progress.to_string()),
                "CurrentResearch.TotalCost.Value" => Some(
                    viewer
                        .interface
                        .rules
                        .as_ref()
                        .and_then(|rules| rules["advances"].as_array())
                        .and_then(|advances| {
                            advances
                                .iter()
                                .find(|advance| advance["id"].as_str() == Some(&research.target_id))
                        })
                        .and_then(|advance| advance["cost"].as_i64())
                        .map_or_else(|| "—".to_owned(), |cost| cost.to_string()),
                ),
                "CurrentResearch.Turn.Value" => Some("—".to_owned()),
                _ => None,
            }
        }
        ManagerKind::Build | ManagerKind::City
            if path.contains("CityChooser") || path.contains("CitySpinner") =>
        {
            city(state, view).map(city_name)
        }
        ManagerKind::Build if path == "ItemDescription" => {
            let id = articles::selected_description(view)?;
            Some(label(ui, id))
        }
        ManagerKind::Empire => empire_value(ui, state, path),
        _ => None,
    }
}

/// Known policy/government values; leave undisclosed economic subtotals unknown.
fn empire_value(ui: &ManagementUi, state: &GameState, path: &str) -> Option<String> {
    let facts = &state.viewer.as_ref()?.diplomacy.as_ref()?.facts;
    let field = if path.contains("ScienceTax") {
        "science_percent"
    } else if path.contains("PublicWorks") {
        "public_works_percent"
    } else if path.contains("CurrentGovernment") {
        "government_id"
    } else {
        return None;
    };
    let value = facts.get(field)?;
    value
        .as_str()
        .map(|id| label(ui, id))
        .or_else(|| value.as_i64().map(|number| number.to_string()))
}
