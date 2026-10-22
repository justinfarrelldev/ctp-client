//! Original Library item-to-enabling-advance mapping, never a research planner.
use super::{ManagementUi, data};
use crate::gameplay::{model::GameState, ui_model::operations::OperationDialog};
use alloc::collections::BTreeSet;
use serde_json::Value;
/// Keeps only exact current advertised goal actions for the selected source
/// article or its projected enabling advances. Multiple prerequisites require an
/// explicit choice; the server owns all recursive research planning/execution.
pub(super) fn dialog(ui: &ManagementUi, state: &GameState) -> Option<OperationDialog> {
    let viewer = state.viewer.as_ref()?;
    let article = ui.library.current.as_deref();
    let goals = article
        .and_then(|id| targets(viewer.interface.rules.as_ref(), id))
        .unwrap_or_default();
    let current = viewer.interface.research.as_ref().map_or_else(
        || "Unavailable".to_owned(),
        |research| {
            if research.goal_id.is_empty() {
                "None".to_owned()
            } else {
                data::label(ui, &research.goal_id)
            }
        },
    );
    let mut form = OperationDialog::actions(
        state,
        &format!("Research Goal: {current}"),
        viewer
            .interface
            .legal_actions
            .iter()
            .filter(|action| {
                action.route == "set-research-goal"
                    && action.payload["advance_id"]
                        .as_str()
                        .is_some_and(|id| id.is_empty() || goals.contains(id))
            })
            .cloned(),
    )?;
    // Advance IDs never share the city-name lookup namespace used by generic
    // target forms; localized article names are presentation, not permission.
    for choice in &mut form.choices {
        if let Some(id) = choice.command.payload["advance_id"]
            .as_str()
            .filter(|id| !id.is_empty())
        {
            choice.label = format!("Research Goal: {}", data::label(ui, id));
        }
    }
    Some(form)
}
/// Normalizes only already-projected direct enabling IDs. Conflicting copies of
/// a unit's prerequisites in unit/production namespaces fail closed, not first-win.
fn targets(rules: Option<&Value>, id: &str) -> Option<BTreeSet<String>> {
    let Some(definitions) = rules else {
        return Some(BTreeSet::from([id.to_owned()]));
    };
    let mut found = None;
    for category in ["units", "production", "governments"] {
        if let Some(records) = definitions[category].as_array() {
            for record in records
                .iter()
                .filter(|record| record["id"].as_str() == Some(id))
            {
                let candidate = if category == "governments" {
                    let advance = record["enable_advance_id"].as_str()?;
                    if advance.is_empty() {
                        BTreeSet::new()
                    } else {
                        BTreeSet::from([advance.to_owned()])
                    }
                } else {
                    prerequisites(record)?
                };
                if found
                    .as_ref()
                    .is_some_and(|previous| *previous != candidate)
                {
                    return None;
                }
                found = Some(candidate);
            }
        }
    }
    // A direct advance remains exactly its advertised identity even for legacy
    // snapshots without item definitions. No legal action means no executable goal.
    Some(found.unwrap_or_else(|| BTreeSet::from([id.to_owned()])))
}
/// Null/malformed mixed arrays are unknown, never partially accepted or zeroed.
fn prerequisites(record: &Value) -> Option<BTreeSet<String>> {
    record["prerequisites"]
        .as_array()?
        .iter()
        .map(|value| {
            value
                .as_str()
                .filter(|id| !id.is_empty())
                .map(str::to_owned)
        })
        .collect()
}
