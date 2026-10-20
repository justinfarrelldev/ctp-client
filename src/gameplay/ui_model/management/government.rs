//! Source government comparison and exact currently advertised enactment targets.
//!
//! Runtime government percentages are not original UI rank descriptors. Only the
//! published city threshold is comparable; undisclosed source ranks stay unknown.

use super::{
    ManagementUi, ManagerCommand, ManagerIntent, ManagerKind, ManagerView, data, selectors,
};
use crate::gameplay::model::GameState;
use crate::gameplay::server::interface::LegalAction;
use alloc::collections::BTreeSet;
use serde_json::Value;

/// Exact source comparison picker, independent of its localized caption.
pub(super) const PICKER: &str = "TabGroup.Tab2.TabPanel.CompareGovernment";

/// Current government identity is never inferred from an installed rules default.
fn current(state: &GameState) -> Option<&str> {
    state.viewer.as_ref()?.diplomacy.as_ref()?.facts["government_id"]
        .as_str()
        .filter(|id| !id.is_empty())
}

/// Finds a current viewer rule definition by its exact nonempty identity.
fn definition<'state>(state: &'state GameState, id: &str) -> Option<&'state Value> {
    if id.is_empty() {
        return None;
    }
    state.viewer.as_ref()?.interface.rules.as_ref()?["governments"]
        .as_array()?
        .iter()
        .find(|record| record["id"].as_str() == Some(id))
}

/// Exact registered capability; no arbitrary government payload is parameterized.
fn action<'state>(state: &'state GameState, id: &str) -> Option<&'state LegalAction> {
    if current(state) == Some(id) {
        return None;
    }
    definition(state, id)?;
    state
        .viewer
        .as_ref()?
        .interface
        .legal_actions
        .iter()
        .find(|action| {
            action.route == "set-government"
                && action.http_supported()
                && action.target_id == id
                && action.payload.as_object().is_some_and(|fields| {
                    fields.len() == 1_usize
                        && fields.get("government_id").and_then(Value::as_str) == Some(id)
                })
        })
}

/// Source rule order, deduplicated; neither current nor unauthorized forms appear.
pub(super) fn ids(state: &GameState) -> Vec<String> {
    let Some(records) = state
        .viewer
        .as_ref()
        .and_then(|viewer| viewer.interface.rules.as_ref())
        .and_then(|rules| rules["governments"].as_array())
    else {
        return Vec::new();
    };
    let mut seen = BTreeSet::new();
    records
        .iter()
        .filter_map(|record| record["id"].as_str())
        .filter(|id| !id.is_empty() && seen.insert((*id).to_owned()) && action(state, id).is_some())
        .map(str::to_owned)
        .collect()
}

/// Current eligibility is rechecked at actual pointer/semantic selection dispatch.
pub(super) fn select(ui: &mut ManagementUi, state: &GameState, id: &str) -> bool {
    if action(state, id).is_none() {
        return false;
    }
    let Some(view) = ui
        .view
        .as_mut()
        .filter(|view| view.kind == ManagerKind::Empire)
    else {
        return false;
    };
    view.selections
        .insert("government".to_owned(), id.to_owned());
    selectors::dismiss(view);
    true
}

/// Bind only the actual source Enact callback; comparisons themselves are local.
pub(super) fn binding(
    state: &GameState,
    view: &ManagerView,
    path: &str,
) -> Option<(Option<ManagerIntent>, Option<String>)> {
    if view.kind != ManagerKind::Empire || path != "TabGroup.Tab2.TabPanel.EnactGovernment" {
        return None;
    }
    let request = view
        .selections
        .get("government")
        .and_then(|id| action(state, id))
        .and_then(|action| ManagerCommand::advertised(state, action));
    Some(request.map_or_else(
        || {
            (
                None,
                Some("Select a currently eligible government to enact".to_owned()),
            )
        },
        |command| (Some(ManagerIntent::Submit(command)), None),
    ))
}

/// Current/comparison column values follow source semantics, never guessed ranks.
pub(super) fn text(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    path: &str,
) -> Option<String> {
    if matches!(
        path,
        "TabGroup.Tab1.TabPanel.CurrentGovernment" | "TabGroup.Tab2.TabPanel.CurrentGovernment"
    ) {
        return Some(current(state).map_or_else(|| "—".to_owned(), |id| data::label(ui, id)));
    }
    let (row, column) = path
        .strip_prefix("TabGroup.Tab2.TabPanel.")?
        .split_once('.')?;
    if !matches!(
        row,
        "Cities"
            | "Growth"
            | "Production"
            | "Research"
            | "Economic"
            | "Commerce"
            | "Military"
            | "Loyalty"
            | "MartialLaw"
            | "AntiPollution"
    ) || !matches!(column, "CurrentValue" | "CompareValue" | "Comparison")
    {
        return None;
    }
    let selected = view
        .selections
        .get("government")
        .filter(|id| action(state, id).is_some());
    if column != "CurrentValue" && selected.is_none() {
        return Some(String::new());
    }
    if row != "Cities" {
        return Some("—".to_owned());
    }
    let left = current(state).and_then(|id| threshold(state, id));
    let right = selected.and_then(|id| threshold(state, id));
    Some(match column {
        "CurrentValue" => left.map_or_else(|| "—".to_owned(), |value| value.to_string()),
        "CompareValue" => right.map_or_else(|| "—".to_owned(), |value| value.to_string()),
        "Comparison" => left.zip(right).map_or_else(
            || "—".to_owned(),
            |(left, right)| {
                match left.cmp(&right) {
                    core::cmp::Ordering::Less => "<",
                    core::cmp::Ordering::Equal => "=",
                    core::cmp::Ordering::Greater => ">",
                }
                .to_owned()
            },
        ),
        _ => return None,
    })
}

/// Go's omitted scalar is zero only inside an existing exact government record.
fn threshold(state: &GameState, id: &str) -> Option<i64> {
    definition(state, id)?
        .get("too_many_cities_threshold")
        .map_or(Some(0_i64), Value::as_i64)
}
