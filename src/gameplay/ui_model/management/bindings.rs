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
    if let Some(intent) = super::advisors::binding(view, path) {
        return enabled(intent);
    }
    if let Some(binding) = super::selectors::binding(state, view, path) {
        return binding;
    }
    if let Some(binding) = super::national::binding(state, view, path) {
        return binding;
    }
    if let Some(binding) = super::trade::binding(state, view, path) {
        return binding;
    }
    if let Some(binding) = super::government::binding(state, view, path) {
        return binding;
    }
    let name = node.name.as_str();
    if let Some(binding) = governor_binding(state, view, path, name) {
        return binding;
    }
    if matches!(
        name,
        "CloseButton" | "OkButton" | "AcceptButton" | "CancelButton"
    ) {
        return enabled(ManagerIntent::Close);
    }
    match view.kind {
        ManagerKind::Statistics
            if path.contains("RankSelect") && matches!(name, "Pulldown" | "autobutton") =>
        {
            history_metric(view)
        }
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
        ManagerKind::National | ManagerKind::City if name == "DisbandButton" => {
            let city = view.selections.get("city").or(view.city_id.as_ref());
            dialog(state.viewer.as_ref().and_then(|viewer| {
                crate::gameplay::ui_model::operations::OperationDialog::actions(
                    state,
                    "Disband City?",
                    viewer
                        .interface
                        .legal_actions
                        .iter()
                        .filter(|action| {
                            action.route == "disband-city" && city == Some(&action.entity_id)
                        })
                        .cloned(),
                )
            }))
        }
        ManagerKind::National if name == "BuildQueueButton" => {
            selected_city_navigation(view, ManagerKind::Build)
        }
        ManagerKind::National if name == "CityManagerButton" => {
            selected_city_navigation(view, ManagerKind::City)
        }
        ManagerKind::City if name == "BuildQueueButton" => {
            enabled(ManagerIntent::Open(ManagerKind::Build))
        }
        ManagerKind::Unit if name == "DisbandButton" => dialog(super::units::disband(state, view)),
        ManagerKind::Diplomacy if path == "DiplomacyTabs.Intelligence.TabPanel.DetailsButton" => {
            diplomatic_details(state, view)
        }
        ManagerKind::Diplomacy if path == "DiplomacyTabs.Intelligence.TabPanel.MessageButton" => {
            diplomatic_workspace(state, view)
        }
        ManagerKind::Empire if name == "DetailsButton" => enabled(ManagerIntent::Select(
            "empire-details".to_owned(),
            view.selections
                .get("empire-details")
                .is_none_or(|details| details != "true")
                .to_string(),
        )),
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

/// Details is view-only, but still requires the exact current selected contact.
fn diplomatic_details(
    state: &GameState,
    view: &ManagerView,
) -> (Option<ManagerIntent>, Option<String>) {
    view.selections
        .get("contact")
        .filter(|id| super::diplomacy::details::known(state, id))
        .map_or_else(
            || disabled("Select a current known contact for intelligence details"),
            |id| enabled(ManagerIntent::ContactDetails(id.clone())),
        )
}
/// The original proposal entry opens explicit current options for one known
/// contact. Opening neither accepts a treaty nor selects the first proposal.
fn diplomatic_workspace(
    state: &GameState,
    view: &ManagerView,
) -> (Option<ManagerIntent>, Option<String>) {
    let Some(contact) = view.selections.get("contact").filter(|id| !id.is_empty()) else {
        return disabled("Select a contacted nation before opening proposals or replies");
    };
    let Some(viewer) = state.viewer.as_ref() else {
        return disabled("No current authoritative diplomacy snapshot");
    };
    if !viewer
        .diplomacy
        .as_ref()
        .and_then(|diplomacy| diplomacy.facts["relations"].as_array())
        .is_some_and(|relations| {
            relations
                .iter()
                .any(|relation| relation["target_player_id"].as_str() == Some(contact))
        })
    {
        return disabled("The selected contact is not present in this viewer revision");
    }
    dialog(
        crate::gameplay::ui_model::operations::OperationDialog::actions(
            state,
            "Diplomatic proposals and replies",
            viewer
                .interface
                .legal_actions
                .iter()
                .filter(|action| {
                    matches!(
                        action.route.as_str(),
                        "propose-agreement" | "respond-agreement"
                    ) && action.target_id == *contact
                })
                .cloned(),
        ),
    )
}
/// Original toggle dispatches immediately; the pulldown chooses pinned priorities.
fn governor_binding(
    state: &GameState,
    view: &ManagerView,
    path: &str,
    name: &str,
) -> Option<(Option<ManagerIntent>, Option<String>)> {
    if view.kind != ManagerKind::City {
        return None;
    }
    if path == "GovernorBox.Toggle" {
        return Some(
            data::city(state, view)
                .and_then(|city| {
                    crate::gameplay::ui_model::operations::OperationDialog::toggle_governor(
                        state, &city.id,
                    )
                })
                .map_or_else(
                    || disabled("No authorized mayor toggle"),
                    |command| enabled(ManagerIntent::Submit(command)),
                ),
        );
    }
    (path.starts_with("GovernorBox.") && matches!(name, "Pulldown" | "autobutton")).then(|| {
        dialog(data::city(state, view).and_then(|city| {
            crate::gameplay::ui_model::operations::OperationDialog::governor(state, &city.id)
        }))
    })
}

/// Cycles actual own-history metrics, never a foreign or fabricated graph.
fn history_metric(view: &ManagerView) -> (Option<ManagerIntent>, Option<String>) {
    let metrics = crate::gameplay::ui_model::history::METRICS;
    let current = view
        .selections
        .get("history-metric")
        .map_or("score", String::as_str);
    let next = metrics
        .iter()
        .position(|metric| *metric == current)
        .unwrap_or(0)
        .saturating_add(1)
        .rem_euclid(metrics.len());
    enabled(ManagerIntent::Select(
        "history-metric".to_owned(),
        metrics[next].to_owned(),
    ))
}

/// Independent buildable/queue selection is required before issuing an order.
fn build(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    path: &str,
) -> (Option<ManagerIntent>, Option<String>) {
    if matches!(
        path,
        "ItemsBox.LoadSaveButton" | "NormalModeButtons.CustomButton"
    ) {
        return dialog(super::data::city(state, view).and_then(|city| {
            crate::gameplay::ui_model::operations::OperationDialog::input(
                state,
                &city.id,
                crate::gameplay::ui_model::operations::InputKind::Queue,
            )
        }));
    }
    if let Some(command) = super::queue::request(state, view, path) {
        return enabled(ManagerIntent::Submit(command));
    }
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
        "LibraryButton" => super::articles::selected_description(view, state)
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
        "SetGoalButton" => dialog(state.viewer.as_ref().and_then(|viewer| {
            let goal = viewer
                .interface
                .research
                .as_ref()
                .map_or("", |research| research.goal_id.as_str());
            let title = format!(
                "Research Goal: {}",
                if goal.is_empty() {
                    "None"
                } else {
                    ui.book.label(goal)
                }
            );
            crate::gameplay::ui_model::operations::OperationDialog::actions(
                state,
                &title,
                viewer
                    .interface
                    .legal_actions
                    .iter()
                    .filter(|action| {
                        action.route == "set-research-goal"
                            && (ui.library.current.as_deref() == Some(action.target_id.as_str())
                                || action.payload["advance_id"].as_str() == Some(""))
                    })
                    .cloned(),
            )
        })),
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

/// Input/target selection is local until explicit valid confirmation.
fn dialog(
    value: Option<crate::gameplay::ui_model::operations::OperationDialog>,
) -> (Option<ManagerIntent>, Option<String>) {
    value.map_or_else(
        || disabled("No currently authorized choice for this operation"),
        |value| enabled(ManagerIntent::Dialog(Box::new(value))),
    )
}

/// Enabled controls carry an actual executable intent, never only a painted flag.
const fn enabled(intent: ManagerIntent) -> (Option<ManagerIntent>, Option<String>) {
    (Some(intent), None)
}

/// Disabled controls retain an actionable explanation for inspection/accessibility.
fn disabled(reason: &str) -> (Option<ManagerIntent>, Option<String>) {
    (None, Some(reason.to_owned()))
}
