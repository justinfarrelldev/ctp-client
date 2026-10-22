//! Viewer-safe manager lists and values; no invented economy, contacts or names.

use alloc::collections::{BTreeMap, BTreeSet};

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
pub(super) fn owned_units(state: &GameState) -> Vec<&ServerUnitProjection> {
    let own: BTreeSet<_> = state
        .units
        .iter()
        .filter(|unit| unit.alive && unit.owner == PlayerId::Human)
        .map(|unit| unit.id)
        .collect();
    state.viewer.as_ref().map_or_else(Vec::new, |viewer| {
        viewer
            .units
            .iter()
            .filter(|source| own.contains(&UnitId::from_server_id(&source.id)))
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
    if path == "Advisor.List" {
        return super::advisors::rows(ui, state);
    }
    if super::selectors::list_path(view.kind, path) {
        let owner = path.strip_suffix(".window.listbox").unwrap_or(path);
        return super::selectors::options(ui, state, view, owner);
    }
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
                    .filter_map(|item| {
                        item["id"].as_str().map(|id| {
                            (
                                item["entry_id"].as_str().unwrap_or(id).to_owned(),
                                label(ui, id),
                            )
                        })
                    })
                    .collect()
            }),
        ManagerKind::Build if path.starts_with("ItemsBox.") => buildables(ui, state, view, path),
        ManagerKind::Science => super::rows::advances(ui, state, true),
        ManagerKind::Research => super::rows::advances(ui, state, false),
        ManagerKind::National => super::national::rows(ui, state, view, path),
        ManagerKind::Unit => unit_rows(ui, state, path),
        ManagerKind::Trade => super::trade::rows(state, view, path),
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

        ManagerKind::Statistics if path == "TabGroup.Tab1.TabPanel.List" => viewer
            .interface
            .statistics
            .iter()
            .rev()
            .map(|sample| {
                (
                    sample.turn.to_string(),
                    format!(
                        "Turn {}: score {}  cities {}  units {}  population {}",
                        sample.turn,
                        sample.score,
                        sample.city_count,
                        sample.unit_count,
                        sample.population
                    ),
                )
            })
            .collect(),
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
    if state
        .viewer
        .as_ref()
        .is_some_and(|viewer| viewer.interface.supports("atomic_production_queue"))
    {
        if let Some(rules) = state
            .viewer
            .as_ref()
            .and_then(|viewer| viewer.interface.rules.as_ref())
        {
            for category in ["units", "production"] {
                if let Some(items) = rules[category].as_array() {
                    for item in items {
                        let item_kind = item["kind"].as_str().unwrap_or(if category == "units" {
                            "unit"
                        } else {
                            ""
                        });
                        if item_kind == kind {
                            if let Some(id) = item["id"].as_str() {
                                records.insert(id.to_owned(), label(ui, id));
                            }
                        }
                    }
                }
            }
        }
    }
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
    city.facts["name"]
        .as_str()
        .filter(|name| !name.is_empty())
        .unwrap_or("—")
        .to_owned()
}

/// Replaces source placeholder numbers only with known current viewer facts.
pub(super) fn text(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    path: &str,
) -> Option<String> {
    if let Some(text) = super::advisors::text(ui, state, view, path) {
        return Some(text);
    }
    if let Some(text) = articles::text(ui, view, path)
        .or_else(|| super::trade::text(view, path))
        .or_else(|| super::national::text(state, view, path))
    {
        return Some(text);
    }
    let viewer = state.viewer.as_ref()?;
    match view.kind {
        ManagerKind::Unit => super::units::text(ui, state, path),
        ManagerKind::City if path == "GovernorBox.Pulldown" => Some(
            city(state, view)
                .and_then(|city| city.governor.as_ref())
                .map_or_else(|| "—".to_owned(), |governor| label(ui, &governor.priority)),
        ),
        ManagerKind::Statistics if path == "TabGroup.Tab1.TabPanel.Total.Value" => Some(
            viewer
                .interface
                .statistics
                .last()
                .map_or_else(|| "—".to_owned(), |sample| sample.score.to_string()),
        ),
        ManagerKind::Statistics
            if path.ends_with("Rank.Value") || path.ends_with("Difficulty.Value") =>
        {
            Some("—".to_owned())
        }
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
                "CurrentResearch.Turn.Value" => Some(research.forecast.as_ref().map_or_else(
                    || "—".to_owned(),
                    |forecast| forecast.label(viewer.revision),
                )),
                _ => None,
            }
        }
        ManagerKind::Build if path == "ItemDescription" => {
            let id = articles::selected_description(view, state)?;
            Some(label(ui, id))
        }
        ManagerKind::Empire => {
            super::government::text(ui, state, view, path).or_else(|| empire_value(ui, state, path))
        }
        _ => None,
    }
}

/// Known policy/government values; leave undisclosed economic subtotals unknown.
fn empire_value(_ui: &ManagementUi, state: &GameState, path: &str) -> Option<String> {
    match path {
        "TabGroup.Tab1.TabPanel.CitiesValue" => Some(owned_city_ids(state).len().to_string()),
        "TabGroup.Tab1.TabPanel.PopulationValue" => {
            let viewer = state.viewer.as_ref()?;
            let owned = owned_city_ids(state);
            Some(
                viewer
                    .cities
                    .iter()
                    .filter(|city| owned.contains(&city.id))
                    .fold(0_u64, |total, city| {
                        total.saturating_add(u64::from(city.population))
                    })
                    .to_string(),
            )
        }
        "TabGroup.Tab1.TabPanel.HappinessValue" => Some("—".to_owned()),
        _ => None,
    }
}
