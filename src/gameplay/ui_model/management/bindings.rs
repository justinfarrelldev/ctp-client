//! Original callbacks mapped to explicit local navigation or exact server orders.
//!
//! An unbound or server-unsupported original control is disabled with a reason;
//! merely drawing its original art must never make a fake command look enabled.

use serde_json::json;

use super::{ManagementUi, ManagerCommand, ManagerIntent, ManagerKind, ManagerView, data};
use crate::gameplay::assets::layout::LayoutNode;
use crate::gameplay::model::GameState;

/// Returns a shared semantic intent plus an optional explicit disabled reason.
pub(super) fn binding(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    path: &str,
    node: &LayoutNode,
) -> (Option<ManagerIntent>, Option<String>) {
    let name = node.name.as_str();
    if matches!(
        name,
        "CloseButton" | "OkButton" | "AcceptButton" | "CancelButton"
    ) {
        return enabled(ManagerIntent::Close);
    }
    match view.kind {
        ManagerKind::Build => build(ui, state, view, path),
        ManagerKind::Science
            if matches!(
                path,
                "ChangeResearch"
                    | "CurrentResearch.AdvanceProgress.IconBorder.IconButton"
                    | "CurrentResearch.AdvanceProgress.IconBorder.IconButton.RadialButton"
            ) =>
        {
            enabled(ManagerIntent::Open(ManagerKind::Research))
        }
        ManagerKind::Research if path == "Background.BackButton" => research(state, view),
        ManagerKind::Library => library(ui, state, name),
        ManagerKind::National if name == "BuildQueueButton" => {
            selected_city_navigation(view, ManagerKind::Build)
        }
        ManagerKind::National if name == "CityManagerButton" => {
            selected_city_navigation(view, ManagerKind::City)
        }
        ManagerKind::City if name == "BuildQueueButton" => {
            enabled(ManagerIntent::Open(ManagerKind::Build))
        }
        ManagerKind::Unit if name == "DisbandButton" => {
            selected_action(state, view, "disband", "unit")
        }
        ManagerKind::Trade if name == "CreateRouteButton" => {
            selected_action(state, view, "create-trade-route", "trade")
        }
        ManagerKind::Diplomacy if name == "MessageButton" => disabled(
            "Select a specific authoritative diplomatic proposal; a contact alone does not identify a proposal",
        ),
        ManagerKind::Empire if name == "DetailsButton" => {
            enabled(ManagerIntent::Open(ManagerKind::National))
        }
        _ => {
            if node
                .field("objecttype")
                .is_some_and(|kind| kind.to_ascii_lowercase().contains("button"))
                || name.ends_with("Button")
            {
                let label = node.field("text").map_or(name, |key| ui.book.label(key));
                disabled(&format!(
                    "{label}: original operation not yet integrated; no command will be substituted"
                ))
            } else {
                (None, None)
            }
        }
    }
}

/// Independent buildable/queue selection is required before issuing an order.
fn build(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    path: &str,
) -> (Option<ManagerIntent>, Option<String>) {
    match path {
        "ItemsBox.UnitsButton" => enabled(ManagerIntent::Select(
            "build-category".to_owned(),
            "Units".to_owned(),
        )),
        "ItemsBox.BuildingsButton" => enabled(ManagerIntent::Select(
            "build-category".to_owned(),
            "Buildings".to_owned(),
        )),
        "ItemsBox.WondersButton" => enabled(ManagerIntent::Select(
            "build-category".to_owned(),
            "Wonders".to_owned(),
        )),
        "ItemsBox.AddButton" => {
            let selected = view.selections.get("buildable");
            let action = data::city_actions(state, view, "queue-production")
                .into_iter()
                .find(|action| selected == Some(&action.target_id));
            advertised(state, action)
        }
        "QueueGroup.RemoveButton" => {
            if data::city(state, view)
                .and_then(|city| city.facts["stored_production"].as_i64())
                .is_some_and(|stored| stored > 0)
            {
                return disabled(
                    "Server cancellation clears stored production even for future entries (QUEUE-01); unsafe cancellation is unavailable while production is stored",
                );
            }
            let selected = view.selections.get("queue");
            let action = data::city_actions(state, view, "cancel-production")
                .into_iter()
                .find(|action| selected == Some(&action.target_id));
            advertised(state, action)
        }
        "RushBuyButton" => {
            let candidates = data::city_actions(state, view, "rush-buy");
            advertised(state, (candidates.len() == 1).then(|| candidates[0]))
        }
        "ItemsBox.InsertButton"
        | "QueueGroup.UpButton"
        | "QueueGroup.DownButton"
        | "QueueGroup.ClearButton" => disabled(
            "Server has no atomic indexed queue-edit operation; cancel/re-add would destroy production (QUEUE-01/02)",
        ),
        "ItemsBox.LoadSaveButton" | "NormalModeButtons.CustomButton" => disabled(
            "Applying a custom queue requires a safe authoritative queue-edit contract (QUEUE-02/03)",
        ),
        "LibraryButton" => super::articles::selected_description(view)
            .filter(|id| ui.library.catalog.article(id).is_some())
            .map_or_else(
                || disabled("Select an installed buildable/queue topic first"),
                |id| enabled(ManagerIntent::Article(id.to_owned())),
            ),
        "GotoCityButton" => enabled(ManagerIntent::Open(ManagerKind::City)),
        _ => (None, None),
    }
}

/// Research suggestions are intentionally incomplete when a target already exists.
/// The engine accepts any currently available advance through set-research; use
/// the actual available list, not an unrelated first legal-action suggestion.
fn research(state: &GameState, view: &ManagerView) -> (Option<ManagerIntent>, Option<String>) {
    let Some(advance) = view.selections.get("advance") else {
        return disabled("Select an available advance first");
    };
    let Some(viewer) = state.viewer.as_ref() else {
        return disabled("No authoritative viewer snapshot");
    };
    let command = ManagerCommand {
        revision: viewer.revision,
        route: "set-research".to_owned(),
        payload: json!({"advance_id":advance}),
    };
    if command.valid_for(state) {
        enabled(ManagerIntent::Submit(command))
    } else {
        disabled("The selected advance is no longer available in this viewer revision")
    }
}

/// Navigation requiring an explicit National Manager row selection.
fn selected_city_navigation(
    view: &ManagerView,
    kind: ManagerKind,
) -> (Option<ManagerIntent>, Option<String>) {
    if view.selections.contains_key("city") {
        enabled(ManagerIntent::Open(kind))
    } else {
        disabled("Select an owned city first")
    }
}

/// A selected row must identify exactly one eligible action, never a first target.
fn selected_action(
    state: &GameState,
    view: &ManagerView,
    route: &str,
    key: &str,
) -> (Option<ManagerIntent>, Option<String>) {
    let selected = view.selections.get(key);
    let candidates: Vec<_> = state.viewer.as_ref().map_or_else(Vec::new, |viewer| {
        viewer
            .interface
            .legal_actions
            .iter()
            .filter(|action| {
                action.route == route
                    && action.http_supported()
                    && selected.is_some_and(|id| action.entity_id == *id || action.target_id == *id)
            })
            .collect()
    });
    advertised(state, (candidates.len() == 1).then(|| candidates[0]))
}

/// Local category switching does not change rules, research or discovery state.
fn library(
    ui: &ManagementUi,
    state: &GameState,
    name: &str,
) -> (Option<ManagerIntent>, Option<String>) {
    if let Some(category) = crate::gameplay::ui_model::library::LibraryCategory::from_button(name) {
        return enabled(ManagerIntent::LibraryCategory(category));
    }
    match name {
        "SearchWord" => enabled(ManagerIntent::Focus("SearchWord".to_owned())),
        "SortByAgeButton" if super::index::can_sort(ui, state) => {
            enabled(ManagerIntent::LibrarySort)
        }
        "SortByAgeButton" => disabled(
            "Age sorting requires a supported category and current server age metadata; search and unknown ages are not guessed",
        ),
        "BackButton" if !ui.library.back.is_empty() => enabled(ManagerIntent::LibraryBack),
        "ForwardButton" if !ui.library.forward.is_empty() => enabled(ManagerIntent::LibraryForward),
        "BackButton" | "ForwardButton" => {
            disabled("No article in this direction of browser history")
        }
        "SetGoalButton" => disabled(
            "A persistent authoritative future research-goal contract is not exposed; Change Research selects only currently available advances",
        ),
        _ => (None, None),
    }
}

/// Wraps only a registered viewer-advertised action.
fn advertised(
    state: &GameState,
    action: Option<&crate::gameplay::server::interface::LegalAction>,
) -> (Option<ManagerIntent>, Option<String>) {
    action.and_then(|action| ManagerCommand::advertised(state, action)).map_or_else(
        || disabled("Select an eligible entity/item; the server does not currently authorize this operation"),
        |command| enabled(ManagerIntent::Submit(command)))
}

/// Enabled controls carry an actual executable intent, never only a painted flag.
const fn enabled(intent: ManagerIntent) -> (Option<ManagerIntent>, Option<String>) {
    (Some(intent), None)
}

/// Disabled controls retain an actionable explanation for inspection/accessibility.
fn disabled(reason: &str) -> (Option<ManagerIntent>, Option<String>) {
    (None, Some(reason.to_owned()))
}
