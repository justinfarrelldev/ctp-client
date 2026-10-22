//! Original Science current-state/description callbacks, using received own facts.
use super::{ManagementUi, ManagerView, data};
use crate::gameplay::model::GameState;
/// Original constructor uses absolute `Move(-X,-Y)` for the unused advisor,
/// placing its whole control outside the clipped Science window.
pub(super) fn configure(root: &mut crate::gameplay::assets::layout::LayoutNode) {
    if let Some(button) = super::fidelity::node(root, "AdvisorButton") {
        for field in ["xpix", "ypix"] {
            let value = button.integer(field).unwrap_or(0_i32).saturating_neg();
            button.fields.insert(field.to_owned(), value.to_string());
        }
    }
}
/// Outside-parent source controls are absent from native/semantic focus trees.
pub(super) fn hidden(path: &str) -> bool {
    path == "AdvisorButton" || path.starts_with("AdvisorButton.")
}
/// The original clears current controls for no target or an already known advance.
/// This is a direct received-set membership check, not a research simulation.
pub(super) fn current_target(state: &GameState) -> Option<&str> {
    let research = state.viewer.as_ref()?.interface.research.as_ref()?;
    (!research.target_id.is_empty() && !research.advances.contains(&research.target_id))
        .then_some(research.target_id.as_str())
}
/// List selection updates the Gives/description box only, never active research.
/// Invalid/available-only selections cannot replace a known discovery selection.
pub(super) fn description_target<'data>(
    view: &'data ManagerView,
    state: &'data GameState,
) -> Option<&'data str> {
    let research = state.viewer.as_ref()?.interface.research.as_ref()?;
    view.selections
        .get("advance")
        .filter(|id| research.advances.contains(id))
        .map(String::as_str)
        .or_else(|| current_target(state))
}
/// Only the received active-target estimate is available, never a counterfactual.
/// Server forecasts already include their assumption; do not port engine's +1.
pub(super) fn turns(state: &GameState, id: &str) -> Option<u32> {
    if current_target(state) != Some(id) {
        return None;
    }
    let viewer = state.viewer.as_ref()?;
    viewer.interface.research.as_ref()?.forecast.as_ref()
        .filter(|forecast| forecast.revision == viewer.revision
            && matches!(forecast.status.as_str(), "estimated" | "completed"))
        .and_then(|forecast| forecast.turns)
}
/// Shared source labels for both radial and numeric turn controls; no target is
/// blank, unprojected research unavailable, and an active unknown estimate '-'.
pub(super) fn text(ui: &ManagementUi, state: &GameState, path: &str) -> Option<String> {
    if !matches!(
        path,
        "CurrentResearch.AdvanceProgress.Title"
            | "CurrentResearch.Spent.Value"
            | "CurrentResearch.TotalCost.Value"
            | "CurrentResearch.Turn.Value"
            | "CurrentResearch.AdvanceProgress.IconBorder.IconButton.RadialButton"
    ) {
        return None;
    }
    let Some(viewer) = state.viewer.as_ref() else {
        return Some("—".to_owned());
    };
    let Some(research) = viewer.interface.research.as_ref() else {
        return Some("—".to_owned());
    };
    let Some(target) = current_target(state) else {
        return Some(String::new());
    };
    Some(match path {
        "CurrentResearch.AdvanceProgress.Title" => data::label(ui, target),
        "CurrentResearch.Spent.Value" => research.progress.to_string(),
        "CurrentResearch.TotalCost.Value" => viewer
            .interface
            .rules
            .as_ref()
            .and_then(|rules| rules["advances"].as_array())
            .and_then(|advances| {
                advances
                    .iter()
                    .find(|advance| advance["id"].as_str() == Some(target))
            })
            .and_then(|advance| advance["cost"].as_i64())
            .filter(|cost| *cost >= 0_i64)
            .map_or_else(|| "—".to_owned(), |cost| cost.to_string()),
        "CurrentResearch.Turn.Value"
        | "CurrentResearch.AdvanceProgress.IconBorder.IconButton.RadialButton" => turns(state, target)
            .map_or_else(|| "-".to_owned(), |turns| turns.to_string()),
        _ => return None,
    })
}
