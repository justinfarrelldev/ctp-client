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
