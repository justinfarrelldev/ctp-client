//! Original DipWizard review of one immutable, explicitly selected server intent.
use super::{ManagerKind, ManagerView};
use crate::gameplay::{
    assets::layout::LayoutNode,
    model::GameState,
    ui_model::management::{ManagementUi, ManagerCommand, ManagerIntent},
};
/// Shared manager scope keeps captured intent/details private to this UI subsystem.
type Binding = (Option<ManagerIntent>, Option<String>);
/// Source's tone icon would falsely claim an unprojected selected tone. Retain
/// source tone controls/art in Stage0 but never display a fabricated active tone.
pub(in crate::gameplay::ui_model::management) fn configure(root: &mut LayoutNode) {
    if let Some(icon) = super::super::fidelity::node(root, "Details.ToneIcon") {
        icon.fields.remove("image0");
        icon.fields.insert(
            "reference-data-unavailable".to_owned(),
            "Diplomatic tone is not projected/accepted by this server contract".to_owned(),
        );
    }
}
/// Only a current exact canonical choice may open; opening executes nothing.
pub(in crate::gameplay::ui_model::management) fn open(
    ui: &mut ManagementUi,
    state: &GameState,
    command: ManagerCommand,
) -> bool {
    if !matches!(
        command.route.as_str(),
        "propose-agreement" | "respond-agreement"
    ) || !command.valid_for(state)
    {
        return false;
    }
    let Some(contact) = target(state, &command) else {
        return false;
    };
    let incoming = command.route == "respond-agreement";
    let proposal = command.payload["proposal_id"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    ui.open_related(ManagerKind::DiplomacyReview, state);
    if let Some(view) = ui.view.as_mut() {
        view.city_id = None;
        view.selections.insert("contact".to_owned(), contact);
        view.selections.insert(
            "review-mode".to_owned(),
            if incoming { "incoming" } else { "outgoing" }.to_owned(),
        );
        view.selections.insert(
            "review-stage".to_owned(),
            if incoming { "3" } else { "1" }.to_owned(),
        );
        view.selections.insert("proposal".to_owned(), proposal);
        view.selections
            .insert("tab:Stage1.Tabs".to_owned(), "Treaty".to_owned());
    }
    ui.diplomatic_review = Some(command);
    true
}
/// A proposal ID may identify only one exact counterparty. Contradictory action
/// metadata cannot select the first match or show one player while sending another.
fn target(state: &GameState, command: &ManagerCommand) -> Option<String> {
    command.payload["proposal_id"]
        .as_str()
        .filter(|id| !id.is_empty())?;
    if command.route == "propose-agreement"
        && (!crate::gameplay::server::commands::shape(
            &command.payload,
            &["proposal_id", "target_player_id", "kind"],
            &[],
        ) || command.payload["kind"].as_str().is_none())
    {
        return None;
    }
    let targets = state
        .viewer
        .as_ref()?
        .interface
        .legal_actions
        .iter()
        .filter(|action| {
            action.route == command.route
                && if command.route == "respond-agreement" {
                    action.payload["proposal_id"] == command.payload["proposal_id"]
                } else {
                    action.payload == command.payload
                }
        })
        .map(|action| action.target_id.as_str())
        .collect::<alloc::collections::BTreeSet<_>>();
    if targets.len() != 1_usize {
        return None;
    }
    let contact = targets.first()?;
    if !super::details::known(state, contact)
        || (command.route == "propose-agreement"
            && command.payload["target_player_id"].as_str() != Some(contact))
    {
        return None;
    }
    Some((*contact).to_owned())
}
/// Original stages/button groups are mutually exclusive, not layered together.
pub(in crate::gameplay::ui_model::management) fn hidden(view: &ManagerView, path: &str) -> bool {
    if view.kind != ManagerKind::DiplomacyReview {
        return false;
    }
    let incoming = view
        .selections
        .get("review-mode")
        .is_some_and(|mode| mode == "incoming");
    let stage = if incoming {
        "3"
    } else {
        view.selections
            .get("review-stage")
            .map_or("1", String::as_str)
    };
    if let Some(part) = path
        .split('.')
        .next()
        .and_then(|part| part.strip_prefix("Stage"))
    {
        return part != stage;
    }
    (incoming && path.starts_with("CreateButtons"))
        || (!incoming && path.starts_with("ViewButtons"))
}
/// Changes only the current proposal's explicitly mutable typed boolean.
fn response(state: &GameState, command: &ManagerCommand, accept: bool) -> Option<ManagerIntent> {
    if command.route != "respond-agreement" || !command.valid_for(state) {
        return None;
    }
    let mut choice = command.clone();
    choice.payload["accept"] = serde_json::Value::Bool(accept);
    choice
        .valid_for(state)
        .then_some(ManagerIntent::Submit(choice))
}
/// Native source callbacks share exact current metadata with accessible controls.
pub(in crate::gameplay::ui_model::management) fn binding(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    path: &str,
) -> Option<Binding> {
    if view.kind != ManagerKind::DiplomacyReview {
        return None;
    }
    if path == "CreateButtons.CancelButton" {
        return Some((Some(ManagerIntent::Close), None));
    }
    let Some(command) = ui.diplomatic_review.as_ref() else {
        return Some((
            None,
            Some("No current diplomatic proposal is captured; reopen current choices".to_owned()),
        ));
    };
    let position = view
        .selections
        .get("review-stage")
        .and_then(|value| value.parse::<u8>().ok())
        .unwrap_or(1_u8);
    let intent=match path {
        "ViewButtons.AcceptButton"=>response(state,command,true),
        "ViewButtons.RejectButton"=>response(state,command,false),
        "CreateButtons.SendButton"=> (command.route == "propose-agreement" && command.valid_for(state)).then(||ManagerIntent::Submit(command.clone())),
        "CreateButtons.BackButton"=>position.checked_sub(1_u8).map(|previous|ManagerIntent::Select("review-stage".to_owned(),previous.to_string())),
        "CreateButtons.NextButton"=>(position<2_u8).then(||ManagerIntent::Select("review-stage".to_owned(),position.saturating_add(1_u8).to_string())),
        "CheckIntelligenceButton"=>view.selections.get("contact").filter(|id|super::details::known(state,id)).map(|id|ManagerIntent::ContactDetails(id.clone())),
        "ViewButtons.CounterOrThreatenButton"=>return Some((None,Some("The current server contract does not authorize counteroffers or threats".to_owned()))),
        _ if path.starts_with("Stage0.Tones.") || path == "Stage0.Nations"=>return Some((None,Some("This review preserves the exact selected target; diplomatic tone is not an accepted server parameter".to_owned()))),
        _=>return None,
    };
    let reason=intent.is_none().then(||"This exact review is stale, unavailable or not currently authorized; reopen current choices".to_owned());
    Some((intent, reason))
}
/// Canonical kind is a received fact, not inferred from ID/name/foreign statistics.
fn kind<'state>(state: &'state GameState, command: &'state ManagerCommand) -> Option<&'state str> {
    if command.route == "propose-agreement" {
        return command.payload["kind"].as_str();
    }
    if state.viewer.as_ref()?.revision != command.revision {
        return None;
    }
    state.viewer.as_ref()?.diplomacy.as_ref()?.facts["proposals"]
        .as_array()?
        .iter()
        .find(|proposal| proposal["id"] == command.payload["proposal_id"])?["kind"]
        .as_str()
}
/// Source details never expose raw SLIC, claim an executed response, or invent
/// civilization/portrait/tone/exchange data unavailable from the authority.
pub(in crate::gameplay::ui_model::management) fn text(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    path: &str,
) -> Option<String> {
    if view.kind != ManagerKind::DiplomacyReview {
        return None;
    }
    let Some(command) = ui.diplomatic_review.as_ref() else {
        return (path.starts_with("Details.") || matches!(path, "Stage3.Text" | "Stage0.Nations"))
            .then(|| "Unavailable — no current proposal selected".to_owned());
    };
    let incoming = command.route == "respond-agreement";
    let contact = view.selections.get("contact").map_or("—", String::as_str);
    match path {
        "Details.Year" => Some(
            state
                .viewer
                .as_ref()
                .and_then(|viewer| {
                    viewer
                        .interface
                        .calendar
                        .as_ref()
                        .filter(|calendar| calendar.turn == viewer.turn)
                })
                .map_or_else(
                    || "—".to_owned(),
                    crate::gameplay::server::v2::Calendar::label,
                ),
        ),
        "Details.Recipient" => Some(format!(
            "{} ID {contact}",
            if incoming { "From" } else { "To" }
        )),
        "Stage0.Nations" => Some(format!("ID {contact}")),
        "Details.Proposal" => Some(kind(state, command).map_or_else(
            || "Agreement details unavailable".to_owned(),
            |kind| {
                format!(
                    "{} {kind} agreement",
                    if incoming { "Received" } else { "Propose" }
                )
            },
        )),
        "Details.Exchange" => {
            Some("Compound exchanges are not authorized by this server contract".to_owned())
        }
        "Details.Response" | "Stage3.Text" => Some(
            if !command.valid_for(state) {
                "Review stale; reopen current choices"
            } else if incoming {
                "Choose Accept or Reject; no response has been submitted"
            } else {
                "Not sent; confirm this exact proposal with Send"
            }
            .to_owned(),
        ),
        _ => None,
    }
}
/// One user-selected canonical offer appears in its original Treaty list.
pub(in crate::gameplay::ui_model::management) fn rows(
    ui: &ManagementUi,
    view: &ManagerView,
    path: &str,
) -> Vec<(String, String)> {
    if view.kind != ManagerKind::DiplomacyReview || path != "Stage1.Tabs.Treaty.TabPanel.List" {
        return Vec::new();
    }
    ui.diplomatic_review
        .as_ref()
        .and_then(|command| command.payload["proposal_id"].as_str())
        .map_or_else(Vec::new, |id| vec![(id.to_owned(), row_label(ui))])
}
/// Unsupported request/offer/exchange panes are explicitly unavailable, not
/// known-empty sets, and never contain fabricated selectable server commands.
pub(in crate::gameplay::ui_model::management) fn unavailable(
    ui: &ManagementUi,
    view: &ManagerView,
    path: &str,
    rect: crate::gameplay::ui_model::MenuRect,
    widgets: &mut Vec<crate::gameplay::ui_model::management::ManagerWidget>,
) {
    if view.kind != ManagerKind::DiplomacyReview
        || !path.ends_with(".TabPanel.List")
        || path == "Stage1.Tabs.Treaty.TabPanel.List"
    {
        return;
    }
    let mut node = ui
        .row_templates
        .get("DipWizProposalItem")
        .and_then(|item| item.children.first())
        .cloned()
        .unwrap_or_default();
    node.children.clear();
    node.fields.insert("wordwrap".to_owned(), "true".to_owned());
    widgets.push(crate::gameplay::ui_model::management::ManagerWidget {id:format!("{path}/unavailable"),label:"Unavailable in this server contract".to_owned(),rect:crate::gameplay::ui_model::MenuRect {height:rect.height.min(60_u32),..rect},node,intent:None,unavailable:Some("Only exact currently advertised peace/trade/union proposals and explicit received-proposal responses are authorized; custom requests/offers/exchanges are not".to_owned()),selected:false});
}
/// Fixed canonical kind label; list selection never rewrites the captured command.
pub(in crate::gameplay::ui_model::management) fn row_label(ui: &ManagementUi) -> String {
    ui.diplomatic_review
        .as_ref()
        .and_then(|command| command.payload["kind"].as_str())
        .map_or_else(|| "—".to_owned(), |kind| format!("{kind} agreement"))
}
