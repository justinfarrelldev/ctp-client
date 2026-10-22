//! Original item enabling-advance mapping uses only projected rules/legal actions.
use super::{ManagementUi, research_goal};
use crate::gameplay::{model::GameState, server::ServerProjection};
use serde_json::{Value, json};
/// Deliberately synthetic numeric/test IDs, no original rule costs or engine copy.
fn fixture() -> (ManagementUi, GameState) {
    let mut data = json!({"revision":12_i64,"turn":4_u32,"contract":{"version":2_u32},"research":{"target_id":"ADVANCE_NOW","goal_id":"ADVANCE_OLD","available":["ADVANCE_NOW"]},"rules":{"units":[{"id":"UNIT_WARRIOR","prerequisites":["ADVANCE_GOAL_A","ADVANCE_GOAL_B"]}],"production":[{"id":"UNIT_WARRIOR","kind":"unit","prerequisites":["ADVANCE_GOAL_A","ADVANCE_GOAL_B"]},{"id":"IMPROVE_GRANARY","kind":"building","prerequisites":["ADVANCE_GOAL_B"]},{"id":"WONDER_PYRAMIDS","kind":"wonder","prerequisites":["ADVANCE_GOAL_C"]}],"governments":[{"id":"GOVERNMENT_TYRANNY","enable_advance_id":"ADVANCE_GOAL_C"}],"advances":[{"id":"ADVANCE_DIRECT"},{"id":"ADVANCE_GOAL_A"},{"id":"ADVANCE_GOAL_B"},{"id":"ADVANCE_GOAL_C"}]}});
    data["legal_actions"]=Value::Array(["ADVANCE_DIRECT","ADVANCE_GOAL_A","ADVANCE_GOAL_B","ADVANCE_GOAL_C",""].into_iter().map(|id|json!({"route":"set-research-goal","entity_id":"human","target_id":id,"fields":["advance_id"],"payload":{"advance_id":id}})).collect());
    let projection: ServerProjection =
        serde_json::from_value(data).expect("actual rules/action schema");
    (
        ManagementUi::default(),
        GameState::from_server_projection(&projection, 1_u8, 1_u64),
    )
}
#[test]
fn selected_unit_building_wonder_government_and_advance_offer_only_their_exact_advertised_enabling_goals()
 {
    let (mut ui, state) = fixture();
    for (article, goals) in [
        ("UNIT_WARRIOR", vec!["ADVANCE_GOAL_A", "ADVANCE_GOAL_B"]),
        ("IMPROVE_GRANARY", vec!["ADVANCE_GOAL_B"]),
        ("WONDER_PYRAMIDS", vec!["ADVANCE_GOAL_C"]),
        ("GOVERNMENT_TYRANNY", vec!["ADVANCE_GOAL_C"]),
        ("ADVANCE_DIRECT", vec!["ADVANCE_DIRECT"]),
    ] {
        ui.library.current = Some(article.to_owned());
        let dialog =
            research_goal::dialog(&ui, &state).expect("source item has current goal choices");
        assert_eq!(
            dialog.choices.len(),
            goals.len().saturating_add(1_usize),
            "only item enabling advances plus explicit current clear, not every legal advance"
        );
        for choice in dialog.choices {
            let id = choice.command.payload["advance_id"]
                .as_str()
                .expect("typed goal");
            assert!(
                id.is_empty() || goals.contains(&id),
                "{article} never picks unrelated/first advance: {id}"
            );
            assert_eq!(
                choice.command.route, "set-research-goal",
                "future goal is not immediate set-research"
            );
            assert_eq!(
                choice.command.payload,
                json!({"advance_id":id}),
                "no locally computed prerequisite path/cost/target parameters"
            );
            assert!(
                choice.command.valid_for(&state),
                "current advertised authority"
            );
        }
    }
    let research = state
        .viewer
        .as_ref()
        .and_then(|viewer| viewer.interface.research.as_ref())
        .expect("received research");
    assert_eq!(
        (research.target_id.as_str(), research.goal_id.as_str()),
        ("ADVANCE_NOW", "ADVANCE_OLD"),
        "opening/selecting goals never changes current research or goal locally"
    );
}
#[test]
fn goal_choices_do_not_borrow_unrelated_city_names_or_claim_missing_goal_is_known_empty() {
    let (mut ui, mut state) = fixture();
    ui.library.current = Some("UNIT_WARRIOR".to_owned());
    let viewer = state.viewer.as_mut().expect("viewer");
    viewer.cities.push(
        serde_json::from_value(json!({"id":"ADVANCE_GOAL_A","name":"UNRELATED CITY NAME"}))
            .expect("adversarial namespace collision"),
    );
    viewer.interface.research = None;
    let form = research_goal::dialog(&ui, &state).expect("current advertised goals");
    assert!(
        form.title.contains("Unavailable"),
        "missing current research is not known no goal"
    );
    assert!(
        form.choices
            .iter()
            .all(|choice| !choice.label.contains("UNRELATED CITY NAME")),
        "advance identities never lookup city namespace"
    );
}
#[test]
fn unknown_null_malformed_or_conflicting_item_prerequisites_never_invent_goal_choices() {
    for prerequisites in [
        Value::Null,
        json!([1_i64]),
        json!(["ADVANCE_GOAL_A", false]),
        json!(["ADVANCE_GOAL_C"]),
    ] {
        let (mut ui, mut state) = fixture();
        ui.library.current = Some("UNIT_WARRIOR".to_owned());
        let rules = state
            .viewer
            .as_mut()
            .expect("viewer")
            .interface
            .rules
            .as_mut()
            .expect("rules");
        rules["units"][0]["prerequisites"] = prerequisites;
        let dialog = research_goal::dialog(&ui, &state).expect("explicit clearing remains current");
        assert_eq!(
            dialog.choices.len(),
            1_usize,
            "corrupt/conflicting prerequisite data cannot silently select a goal"
        );
        assert_eq!(
            dialog.choices[0].command.payload,
            json!({"advance_id":""}),
            "only explicitly advertised clear remains"
        );
    }
    let (mut ui, mut state) = fixture();
    ui.library.current = Some("UNKNOWN_ARTICLE".to_owned());
    let dialog = research_goal::dialog(&ui, &state).expect("clear");
    assert_eq!(
        dialog.choices.len(),
        1_usize,
        "unknown article cannot infer enabling advances"
    );
    state.viewer.as_mut().expect("viewer").revision = 13_i64;
    assert!(
        !dialog.choices[0].command.valid_for(&state),
        "captured goal choice never rebases after a receipt"
    );
}
