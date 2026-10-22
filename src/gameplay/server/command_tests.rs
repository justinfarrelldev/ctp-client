//! Pure command-shape and phase regressions for the current server gateway.
use super::{ServerProjection, commands};
use serde_json::{Value, json};

/// Complete replacement selections reference this city's entries, not content IDs.
fn viewer() -> ServerProjection {
    serde_json::from_value(json!({"revision":8,"turn":2,"planning_status":"open",
        "contract":{"version":2,"features":{"atomic_production_queue":true}},
        "world":{"min_x":0,"min_y":0,"width":20,"height":10},
        "rules":{"units":[{"id":"UNIT_WARRIOR"}],"governor_priorities":["growth","science"]},
        "cities":[{"id":"c","owner_id":"human","name":"Rome","queue":[{"entry_id":"c:1","id":"UNIT_WARRIOR"},{"entry_id":"c:2","id":"UNIT_WARRIOR"}]}],
        "legal_actions":[{"route":"round-complete","payload":{}},{"route":"edit-production","entity_id":"c","payload":{"city_id":"c","entries":[{"entry_id":"c:1"},{"entry_id":"c:2"}]}},{"route":"rename-city","entity_id":"c","payload":{"city_id":"c","name":"Rome"}},{"route":"set-governor","entity_id":"c","payload":{"city_id":"c","enabled":true,"priority":"growth"}}]
    })).expect("viewer fixture")
}

#[test]
fn v2_commands_validate_parameter_shapes_without_requiring_suggested_values() {
    let projection = viewer();
    for (route, payload) in [
        (
            "edit-production",
            json!({"city_id":"c","entries":[{"entry_id":"c:2"},{"item_id":"UNIT_WARRIOR"},{"entry_id":"c:1"}]}),
        ),
        ("edit-production", json!({"city_id":"c","entries":[]})),
        ("rename-city", json!({"city_id":"c","name":"  New Rome  "})),
        (
            "set-governor",
            json!({"city_id":"c","enabled":true,"priority":"science"}),
        ),
        ("set-governor", json!({"city_id":"c","enabled":false})),
    ] {
        assert!(
            commands::valid(&projection, route, &payload),
            "valid selected operation {route}: {payload}"
        );
    }
    for payload in [
        json!({"city_id":"c","entries":null}),
        json!({"city_id":"c"}),
        json!({"city_id":"c","entries":[{"entry_id":"foreign:1"}]}),
        json!({"city_id":"c","entries":[{"entry_id":"c:1"},{"entry_id":"c:1"}]}),
        json!({"city_id":"c","entries":[{"item_id":"FORGED"}]}),
        json!({"city_id":"c","entries":[{"item_id":"UNIT_WARRIOR","cost":1_i64}]}),
    ] {
        assert!(
            !commands::valid(&projection, "edit-production", &payload),
            "invalid queue is rejected: {payload}"
        );
    }
    assert!(
        !commands::valid(
            &projection,
            "rename-city",
            &json!({"city_id":"c","name":"\n"})
        ),
        "empty/control name rejects"
    );
    assert!(
        !commands::valid(
            &projection,
            "set-governor",
            &json!({"city_id":"c","enabled":null})
        ),
        "null is not disable"
    );
}

#[test]
fn v2_end_turn_uses_exact_phase_action_and_never_acts_while_waiting() {
    let mut projection = viewer();
    assert_eq!(
        projection
            .interface
            .end_turn()
            .map(|action| action.route.as_str()),
        Some("round-complete"),
        "planning closes"
    );
    projection.interface.planning_status.clear();
    projection.interface.legal_actions[0].route = "pass".to_owned();
    assert_eq!(
        projection
            .interface
            .end_turn()
            .map(|action| action.route.as_str()),
        Some("pass"),
        "sequential/conflict passes"
    );
    projection.interface.legal_actions.clear();
    assert!(
        projection.interface.end_turn().is_none(),
        "waiting/terminal has no executable action"
    );
    assert!(
        !commands::valid(
            &projection,
            "rename-city",
            &json!({"city_id":"c","name":"Other"})
        ),
        "cached capability is not authorization"
    );
}

#[test]
fn an_authorized_current_proposal_may_be_explicitly_rejected_without_changing_identity_or_shape() {
    let mut projection = viewer();
    projection
        .interface
        .legal_actions
        .push(super::interface::LegalAction {
            route: "respond-agreement".to_owned(),
            entity_id: "proposal-7".to_owned(),
            target_id: "other".to_owned(),
            fields: vec!["proposal_id".to_owned(), "accept".to_owned()],
            payload: json!({"proposal_id":"proposal-7","accept":true}),
            ..super::interface::LegalAction::default()
        });
    assert!(
        commands::valid(
            &projection,
            "respond-agreement",
            &json!({"proposal_id":"proposal-7","accept":false})
        ),
        "server respondAgreement accepts either explicit boolean for this exact authorized received proposal"
    );
    for payload in [
        json!({"proposal_id":"proposal-7","accept":null}),
        json!({"proposal_id":"proposal-7","accept":"false"}),
        json!({"proposal_id":"proposal-7"}),
        json!({"proposal_id":"foreign-proposal","accept":false}),
        json!({"proposal_id":"proposal-7","accept":false,"target_player_id":"other"}),
    ] {
        assert!(
            !commands::valid(&projection, "respond-agreement", &payload),
            "reject malformed/forged response {payload}"
        );
    }
    projection
        .interface
        .legal_actions
        .last_mut()
        .expect("response action")
        .fields
        .clear();
    assert!(
        !commands::valid(
            &projection,
            "respond-agreement",
            &json!({"proposal_id":"proposal-7","accept":false})
        ),
        "fixed legacy suggestion without mutable accept cannot be edited"
    );
    assert!(
        commands::valid(
            &projection,
            "respond-agreement",
            &json!({"proposal_id":"proposal-7","accept":true})
        ),
        "exact typed legacy authorization still works"
    );
}

#[test]
fn v2_route_manifest_covers_current_server_commands_but_rejects_inventions() {
    for route in [
        "edit-production",
        "sleep",
        "wake",
        "rename-city",
        "disband-city",
        "set-governor",
        "set-research-goal",
        "set-movement-order",
        "cancel-movement-order",
        "pillage",
        "expel",
        "reform",
        "sue",
        "upgrade-unit",
        "bombard",
    ] {
        let action = super::interface::LegalAction {
            route: route.to_owned(),
            ..super::interface::LegalAction::default()
        };
        assert!(action.http_supported(), "registered route {route}");
    }
    assert!(
        !commands::valid(&viewer(), "invent-reward", &Value::Null),
        "never invent authority"
    );
}
