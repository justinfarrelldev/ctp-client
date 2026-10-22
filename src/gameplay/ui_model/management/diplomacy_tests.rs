//! Exact contacted-nation proposal/reply choices, never first-contact execution.
use super::{ManagementUi, ManagerIntent, ManagerKind};
use crate::gameplay::{assets::layout::LayoutBook, model::GameState, server::ServerProjection};
use serde_json::json;
use std::path::Path;
/// Server diplomacyActions suppresses duplicate trade while an incoming trade is pending.
fn fixture() -> (ManagementUi, GameState) {
    let viewer:ServerProjection=serde_json::from_value(json!({"revision":6_i64,"turn":2_u32,"contract":{"version":2_u32},"diplomacy":{"relations":[{"target_player_id":"other-1","treaty":"peace","regard":"neutral"},{"target_player_id":"other-2","treaty":"peace","regard":"neutral"}],"proposals":[{"id":"offer-1","from_player_id":"other-1","to_player_id":"human","kind":"trade"}]},"legal_actions":[{"route":"respond-agreement","entity_id":"offer-1","target_id":"other-1","fields":["proposal_id","accept"],"payload":{"proposal_id":"offer-1","accept":true}},{"route":"propose-agreement","entity_id":"human","target_id":"other-1","fields":["proposal_id","target_player_id","kind"],"payload":{"proposal_id":"client-union-human-other-1","target_player_id":"other-1","kind":"union"}},{"route":"propose-agreement","entity_id":"human","target_id":"other-2","fields":["proposal_id","target_player_id","kind"],"payload":{"proposal_id":"client-trade-human-other-2","target_player_id":"other-2","kind":"trade"}},{"route":"propose-agreement","entity_id":"human","target_id":"other-2","fields":["proposal_id","target_player_id","kind"],"payload":{"proposal_id":"client-union-human-other-2","target_player_id":"other-2","kind":"union"}}]})).expect("actual viewer action shapes");
    let state = GameState::from_server_projection(&viewer, 1, 1);
    let mut ui = ManagementUi::new(
        LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"))
            .expect("installed layouts"),
    )
    .expect("cached source");
    ui.open(ManagerKind::Diplomacy, &state, None);
    (ui, state)
}
#[test]
fn diplomacy_reference_uses_the_tall_original_window_source_list_and_aligned_lower_controls() {
    let (ui, state) = fixture();
    let widgets = ui.widgets(&state);
    let root = widgets
        .iter()
        .find(|widget| widget.id.is_empty())
        .expect("source root");
    assert_eq!(
        (root.rect.width, root.rect.height, root.rect.y),
        (601_u32, 558_u32, 28_i32),
        "public reference revision, not old short centered dialog"
    );
    let list = widgets
        .iter()
        .find(|widget| widget.id == "DiplomacyTabs.Intelligence.TabPanel.List")
        .expect("intelligence list");
    assert_eq!(
        (
            list.node.integer("list-content-width"),
            list.node.integer("list-content-height")
        ),
        (Some(470_i32), Some(396_i32)),
        "all declared source content pixels, excluding frame expansion"
    );
    let close = widgets
        .iter()
        .find(|widget| widget.id == "CloseButton")
        .expect("source close");
    for name in [
        "DetailsButton",
        "MessageButton",
        "WarButton",
        "EmbargoButton",
    ] {
        let button = widgets
            .iter()
            .find(|widget| widget.id == format!("DiplomacyTabs.Intelligence.TabPanel.{name}"))
            .expect("all source controls retained");
        assert_eq!(
            button.rect.y, close.rect.y,
            "source lower action row aligns across parent coordinate systems"
        );
    }
}
#[test]
fn intelligence_uses_source_columns_and_received_relation_facts_without_inventing_nations_strength_or_embassies()
 {
    let (ui, mut state) = fixture();
    state
        .viewer
        .as_mut()
        .expect("viewer")
        .diplomacy
        .as_mut()
        .expect("relations")
        .facts["relations"][0]["trade"] = json!(true);
    let widgets = ui.widgets(&state);
    let prefix = "DiplomacyTabs.Intelligence.TabPanel.List/other-1/cell/";
    for (name, text) in [
        ("Nation", "ID other-1"),
        ("Regard", "Neutral"),
        ("Treaties", "Peace, Trade"),
        ("Flag", "—"),
        ("Strength", "—"),
        ("Embassy", "—"),
    ] {
        assert!(
            widgets
                .iter()
                .any(|widget| widget.id == format!("{prefix}{name}") && widget.label == text),
            "known current source column {name}; unknown is not zero or guessed artwork"
        );
    }
    let nation = widgets
        .iter()
        .find(|widget| widget.id == format!("{prefix}Nation"))
        .expect("nation identity");
    let regard = widgets
        .iter()
        .find(|widget| widget.id == format!("{prefix}Regard"))
        .expect("regard");
    assert!(
        regard.rect.x > nation.rect.x,
        "sequential original header-width columns cannot overprint"
    );
    assert_eq!(
        nation.rect.height, 25_u32,
        "actual runtime-created IntelligenceListItem height"
    );
}

#[test]
fn source_proposal_button_requires_explicit_contact_and_groups_only_that_contacts_current_proposals_and_replies()
 {
    let (mut ui, mut state) = fixture();
    let button = "DiplomacyTabs.Intelligence.TabPanel.MessageButton";
    assert!(
        ui.activate_id(button, &state).is_none(),
        "never default to the first contacted nation"
    );
    ui.apply_local(ManagerIntent::Select(
        "contact".to_owned(),
        "other-1".to_owned(),
    ));
    let form = ui
        .activate_id(button, &state)
        .and_then(|intent| match intent {
            ManagerIntent::Dialog(dialog) => Some(dialog),
            _ => None,
        })
        .expect("source proposal workspace");
    assert_eq!(
        form.choices.len(),
        3_usize,
        "one canonical union offer plus explicit accept/reject incoming trade"
    );
    assert!(form.choices.iter().any(|choice|choice.command.payload == json!({"proposal_id":"offer-1","accept":false})),"reachable exact rejection, not only acceptance");
    assert!(
        form.choices
            .iter()
            .all(|choice| choice.command.payload["target_player_id"].as_str() != Some("other-2")),
        "different contacted target is not smuggled into this workspace"
    );
    state.viewer.as_mut().expect("viewer").revision = 7_i64;
    assert!(
        form.choices
            .iter()
            .all(|choice| !choice.command.valid_for(&state)),
        "every choice is bound to its captured current snapshot"
    );
    state
        .viewer
        .as_mut()
        .expect("viewer")
        .diplomacy
        .as_mut()
        .expect("relations")
        .facts["relations"] = json!([]);
    assert!(
        ui.activate_id(button, &state).is_none(),
        "disappeared contact does not retarget the old workspace to another nation"
    );
}
#[test]
fn exact_selected_contact_opens_source_details_tabs_without_private_or_unexpanded_slic_data_and_returns_to_same_parent()
 {
    let (mut ui, state) = fixture();
    let button = "DiplomacyTabs.Intelligence.TabPanel.DetailsButton";
    assert!(
        ui.activate_id(button, &state).is_none(),
        "details requires explicit current contact"
    );
    ui.apply_local(ManagerIntent::Select(
        "contact".to_owned(),
        "other-2".to_owned(),
    ));
    let parent = ui.view.clone();
    assert_eq!(
        ui.activate_id(button, &state),
        Some(ManagerIntent::ContactDetails("other-2".to_owned())),
        "not the first contact or an own-city namespace"
    );
    assert!(
        ui.open_contact_details(&state, "other-2"),
        "revalidate current identity before navigation"
    );
    assert_eq!(
        ui.view.as_ref().map(|view| view.kind),
        Some(ManagerKind::DiplomacyDetails),
        "original related source root"
    );
    assert!(
        ui.view.as_ref().is_some_and(|view| view.city_id.is_none()),
        "foreign player identity never occupies a selected-city slot"
    );
    let report = ui
        .widgets(&state)
        .into_iter()
        .find(|widget| widget.id == "IntelReportLabel")
        .expect("source intelligence heading");
    assert!(
        report.label.contains("ID other-2"),
        "identity stays an explicit ID without inventing a civilization"
    );
    for tab in ["Tab1", "Tab2", "Tab3"] {
        ui.apply_local(ManagerIntent::Select(
            "tab:TabGroup".to_owned(),
            tab.to_owned(),
        ));
        let widgets = ui.widgets(&state);
        if tab == "Tab2" {
            for name in [
                "EmpireSizeLabel",
                "PopulationLabel",
                "PollutionLabel",
                "CapitalLabel",
                "LargestCityLabel",
                "GovTypeLabel",
                "MilStateLabel",
            ] {
                let field = widgets
                    .iter()
                    .find(|widget| widget.id == format!("TabGroup.Tab2.TabPanel.{name}"))
                    .expect("source domestic metric");
                assert!(
                    field.label.ends_with('—'),
                    "unknown {name} is never zero, own data or visible-city-derived foreign accounting"
                );
            }
        }
        assert!(
            widgets
                .iter()
                .all(|widget| !widget.label.contains('{') && !widget.label.contains("%d")),
            "private/unexpanded SLIC is never exposed on {tab}"
        );
        assert!(
            widgets
                .iter()
                .any(|widget| widget.id.ends_with("/unavailable")
                    && widget.intent.is_none()
                    && widget.label.to_lowercase().contains("unavailable")),
            "unknown foreign lists are explicitly unavailable, not empty known sets on {tab}"
        );
        assert!(
            !widgets.iter().any(|widget| widget.id.contains("/other-1")),
            "the foreign nation's contact list must not be replaced with our known relations"
        );
    }
    ui.apply_local(ManagerIntent::Close);
    assert_eq!(
        ui.view, parent,
        "Close restores exact parent selections and local offsets"
    );
    assert!(ui.parents.is_empty(), "no navigation loop/window leak");
    assert!(
        !ui.open_contact_details(&state, "hidden-or-stale"),
        "forged contact cannot open private global data"
    );
    assert_eq!(
        ui.view, parent,
        "failed navigation leaves the parent intact"
    );
}

#[test]
fn explicit_received_proposal_uses_original_review_accept_reject_and_intelligence_with_no_opening_execution()
 {
    let (mut ui, mut state) = fixture();
    ui.apply_local(ManagerIntent::Select(
        "contact".to_owned(),
        "other-1".to_owned(),
    ));
    let parent = ui.view.clone();
    let form = ui
        .activate_id("DiplomacyTabs.Intelligence.TabPanel.MessageButton", &state)
        .and_then(|intent| match intent {
            ManagerIntent::Dialog(dialog) => Some(dialog),
            _ => None,
        })
        .expect("explicit current choices");
    let selected = form
        .choices
        .into_iter()
        .find(|choice| {
            choice.command.route == "respond-agreement"
                && choice.command.payload["accept"].as_bool() == Some(false)
        })
        .expect("explicit rejection")
        .command;
    let before = state.viewer.clone();
    assert!(
        ui.open_diplomatic_review(&state, selected.clone()),
        "current exact received proposal opens source review"
    );
    assert_eq!(
        ui.view.as_ref().map(|view| view.kind),
        Some(ManagerKind::DiplomacyReview),
        "original DipWizard, not a synthetic form"
    );
    assert_eq!(
        state.viewer, before,
        "opening/review never executes the response"
    );
    let widgets = ui.widgets(&state);
    assert!(
        widgets.iter().any(|widget| widget.id == "Stage3.Text"),
        "original incoming review stage"
    );
    assert!(
        !widgets
            .iter()
            .any(|widget| widget.id.starts_with("Stage0.")
                || widget.id.starts_with("CreateButtons.")),
        "mutually exclusive source stages/button groups"
    );
    for (button, accept) in [
        ("ViewButtons.AcceptButton", true),
        ("ViewButtons.RejectButton", false),
    ] {
        let command = ui
            .activate_id(button, &state)
            .and_then(ManagerIntent::into_command)
            .expect("current exact source response");
        assert_eq!(
            command.payload,
            json!({"proposal_id":"offer-1","accept":accept}),
            "only disposition varies, never entity/shape"
        );
        assert_eq!(command.revision, selected.revision, "review never rebases");
    }
    assert_eq!(
        ui.activate_id("CheckIntelligenceButton", &state),
        Some(ManagerIntent::ContactDetails("other-1".to_owned())),
        "same explicit player namespace"
    );
    assert!(
        ui.open_contact_details(&state, "other-1"),
        "source related details"
    );
    ui.apply_local(ManagerIntent::Close);
    assert_eq!(
        ui.view.as_ref().map(|view| view.kind),
        Some(ManagerKind::DiplomacyReview),
        "exact captured review survives related details"
    );
    state.viewer.as_mut().expect("viewer").revision = 7_i64;
    assert!(
        ui.activate_id("ViewButtons.RejectButton", &state).is_none(),
        "stale review cannot submit a newly rebound proposal"
    );
    ui.apply_local(ManagerIntent::Close);
    assert_eq!(
        ui.view, parent,
        "cancel restores exact original parent contact"
    );
    assert!(
        ui.diplomatic_review.is_none(),
        "closed review releases its bounded captured command"
    );
}
#[test]
fn original_review_respects_fixed_legacy_disposition_and_removed_contact_closes_related_pages_without_retargeting()
 {
    let (mut ui, mut state) = fixture();
    state
        .viewer
        .as_mut()
        .expect("viewer")
        .interface
        .legal_actions
        .iter_mut()
        .find(|action| action.route == "respond-agreement")
        .expect("received offer")
        .fields
        .clear();
    let command = super::ManagerCommand {
        revision: 6_i64,
        route: "respond-agreement".to_owned(),
        payload: json!({"proposal_id":"offer-1","accept":true}),
    };
    ui.apply_local(ManagerIntent::Select(
        "contact".to_owned(),
        "other-1".to_owned(),
    ));
    assert!(
        ui.open_diplomatic_review(&state, command),
        "fixed advertised acceptance can be reviewed"
    );
    assert!(
        ui.activate_id("ViewButtons.AcceptButton", &state).is_some(),
        "exact fixed acceptance remains legal"
    );
    assert!(
        ui.activate_id("ViewButtons.RejectButton", &state).is_none(),
        "review cannot create mutable accept permission for fixed legacy action"
    );
    assert!(
        ui.open_contact_details(&state, "other-1"),
        "nested source intelligence"
    );
    let viewer = state.viewer.as_mut().expect("snapshot");
    viewer.revision = 7_i64;
    viewer.diplomacy.as_mut().expect("contacts").facts["relations"] =
        json!([{"target_player_id":"other-2","treaty":"peace","regard":"neutral"}]);
    ui.reconcile(&state);
    assert_eq!(
        ui.view.as_ref().map(|view| view.kind),
        Some(ManagerKind::Diplomacy),
        "both removed-target related pages close to main"
    );
    assert!(
        ui.diplomatic_review.is_none() && ui.parents.is_empty(),
        "captured stale command/stack released, no next contact chosen"
    );
    assert!(
        ui.view
            .as_ref()
            .is_some_and(|view| !view.selections.contains_key("contact")),
        "other-2 never silently substitutes"
    );
}

#[test]
fn missing_review_capture_fails_closed_instead_of_turning_original_accept_into_generic_close_or_exposing_slic()
 {
    let (mut ui, mut state) = fixture();
    let mut duplicate = state
        .viewer
        .as_ref()
        .expect("viewer")
        .interface
        .legal_actions
        .iter()
        .find(|action| action.route == "respond-agreement")
        .expect("response")
        .clone();
    "other-2".clone_into(&mut duplicate.target_id);
    state
        .viewer
        .as_mut()
        .expect("viewer")
        .interface
        .legal_actions
        .push(duplicate);
    let command = super::ManagerCommand {
        revision: 6_i64,
        route: "respond-agreement".to_owned(),
        payload: json!({"proposal_id":"offer-1","accept":true}),
    };
    assert!(
        !ui.open_diplomatic_review(&state, command),
        "contradictory counterpart identities must not silently choose the first matching action"
    );
    ui.open(ManagerKind::DiplomacyReview, &state, None);
    let widgets = ui.widgets(&state);
    assert!(
        ui.activate_id("CreateButtons.SendButton", &state).is_none(),
        "no captured current command means no Send"
    );
    assert!(
        widgets.iter().all(|widget| !widget.label.contains('{')),
        "direct diagnostic/open API cannot expose raw private placeholders without a captured subject"
    );
    ui.apply_local(ManagerIntent::Select(
        "review-mode".to_owned(),
        "incoming".to_owned(),
    ));
    assert!(
        ui.activate_id("ViewButtons.AcceptButton", &state).is_none(),
        "missing capture must not fall through to generic Accept-as-Close binding"
    );
}
#[test]
fn outgoing_original_review_send_preserves_canonical_payload_and_does_not_invent_tone_exchange_or_foreign_identity()
 {
    let (mut ui, state) = fixture();
    ui.apply_local(ManagerIntent::Select(
        "contact".to_owned(),
        "other-2".to_owned(),
    ));
    let form = ui
        .activate_id("DiplomacyTabs.Intelligence.TabPanel.MessageButton", &state)
        .and_then(|intent| match intent {
            ManagerIntent::Dialog(dialog) => Some(dialog),
            _ => None,
        })
        .expect("canonical current choices");
    let selected = form
        .choices
        .into_iter()
        .find(|choice| choice.command.payload["kind"].as_str() == Some("union"))
        .expect("explicit union not first trade")
        .command;
    assert!(
        ui.open_diplomatic_review(&state, selected.clone()),
        "selected canonical union proposal"
    );
    assert_eq!(
        ui.activate_id("CreateButtons.SendButton", &state)
            .and_then(ManagerIntent::into_command),
        Some(selected),
        "exact canonical proposal id/target/kind and no tone/cost/exchange extras"
    );
    let widgets = ui.widgets(&state);
    assert!(
        widgets
            .iter()
            .any(|widget| widget.id == "Details.Recipient" && widget.label.contains("ID other-2")),
        "no invented civilization name"
    );
    assert!(
        widgets
            .iter()
            .all(|widget| !widget.label.contains('{') && !widget.id.starts_with("ViewButtons.")),
        "no raw SLIC or incoming buttons in outgoing stage"
    );
    let back = ui
        .activate_id("CreateButtons.BackButton", &state)
        .expect("source Back");
    ui.apply_local(back);
    let tones = ui.widgets(&state);
    assert!(
        tones.iter().any(|widget| widget.id == "Stage0.Tones.Tone0"
            && widget.intent.is_none()
            && widget.unavailable.is_some()),
        "all original tone controls are present but not executable without protocol"
    );
}

#[test]
fn disappeared_contact_is_deselected_on_receipt_without_selecting_the_other_visible_nation() {
    let (mut ui, mut state) = fixture();
    ui.apply_local(ManagerIntent::Select(
        "contact".to_owned(),
        "other-1".to_owned(),
    ));
    let viewer = state.viewer.as_mut().expect("viewer");
    viewer.revision = 7_i64;
    viewer.diplomacy.as_mut().expect("relations").facts["relations"] =
        json!([{"target_player_id":"other-2","treaty":"peace","regard":"neutral"}]);
    ui.reconcile(&state);
    assert!(
        !ui.view
            .as_ref()
            .expect("open source manager")
            .selections
            .contains_key("contact"),
        "deleted/stale contact cannot alias the next known nation"
    );
    assert!(
        ui.activate_id("DiplomacyTabs.Intelligence.TabPanel.MessageButton", &state)
            .is_none(),
        "require a new explicit contact selection even with legal other-target offers"
    );
}

#[test]
fn source_proposal_workspace_labels_distinguish_trade_and_union_without_guessing_foreign_identity()
{
    let (mut ui, state) = fixture();
    ui.apply_local(ManagerIntent::Select(
        "contact".to_owned(),
        "other-2".to_owned(),
    ));
    let form = ui
        .activate_id("DiplomacyTabs.Intelligence.TabPanel.MessageButton", &state)
        .and_then(|intent| match intent {
            ManagerIntent::Dialog(dialog) => Some(dialog),
            _ => None,
        })
        .expect("current proposals");
    assert_eq!(
        form.choices.len(),
        2_usize,
        "two server authorized treaty kinds"
    );
    for kind in ["trade", "union"] {
        let choice = form
            .choices
            .iter()
            .find(|choice| choice.command.payload["kind"].as_str() == Some(kind))
            .expect("canonical kind");
        assert!(
            choice.label.contains(kind),
            "do not show indistinguishable propose-agreement labels"
        );
        assert!(
            choice.label.contains("ID other-2"),
            "unknown foreign name remains an explicit identifier, not a city-name substitution"
        );
    }
}
