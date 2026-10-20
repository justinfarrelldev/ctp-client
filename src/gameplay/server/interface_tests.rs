//! Viewer contracts mirror visibility.go; fixtures are in memory, never live HTTP.

use super::ServerProjection;

#[test]
fn viewer_interface_retains_exact_legal_actions_rules_and_management_data() {
    let projection: ServerProjection = serde_json::from_str(r#"{
        "revision":12,"turn":3,"tiles":[],"units":[],"cities":[],
        "research":{"target_id":"ADVANCE_BALLISTICS","progress":20,"available":["ADVANCE_TOOLMAKING"]},
        "rules":{"content_hash":"retail","units":[{"id":"UNIT_HOPLITE","attack":10}]},
        "legal_actions":[{"command":"fortify","route":"fortify","entity_id":"hoplite-2","target_id":"tile-4-4","fields":["unit_id"],"payload":{"unit_id":"hoplite-2"}}],
        "city_reports":[{"city_id":"rome","happiness":75}],
        "planning_status":"open"
    }"#).expect("server viewer fixture");
    let action = projection
        .interface
        .action("fortify", "hoplite-2", Some("tile-4-4"))
        .expect("exact legal action");
    assert_eq!(
        action.payload["unit_id"], "hoplite-2",
        "wire identity preserved"
    );
    assert!(
        projection
            .interface
            .action("fortify", "hoplite-1", None)
            .is_none(),
        "no identity substitution"
    );
    assert!(
        projection
            .interface
            .action("disband", "hoplite-2", None)
            .is_none(),
        "do not invent unavailable orders"
    );
    assert_eq!(
        projection
            .interface
            .research
            .as_ref()
            .expect("research")
            .target_id,
        "ADVANCE_BALLISTICS",
        "research retained"
    );
    assert_eq!(
        projection.interface.rules.as_ref().expect("rules")["content_hash"],
        "retail",
        "rules retained"
    );
    assert_eq!(
        projection.interface.city_reports.len(),
        1_usize,
        "reports retained"
    );
}

#[test]
fn viewer_actor_city_and_tile_details_are_not_silently_discarded() {
    let projection: ServerProjection = serde_json::from_str(r#"{"revision":1,"turn":1,"tiles":[{"id":"a","x":0,"y":0,"terrain":"plains","planning":{"cost":2,"domains":["land"],"accessible":true,"enemy_zoc":false}}],"units":[{"id":"g","fortified":true,"attack":10,"health":8}],"cities":[{"id":"r","population":3,"queue":[{"id":"UNIT_HOPLITE","kind":"unit","cost":80}]}]}"#).expect("source facts");
    assert_eq!(
        projection.units[0_usize].facts["fortified"], true,
        "fortification retained"
    );
    assert_eq!(
        projection.units[0_usize].facts["health"], 8_i32,
        "health retained"
    );
    assert_eq!(
        projection.cities[0_usize].facts["queue"][0_usize]["id"], "UNIT_HOPLITE",
        "queue retained"
    );
    assert_eq!(
        projection.tiles[0_usize]
            .planning
            .as_ref()
            .expect("planning")
            .cost,
        2_i64,
        "viewer planning cost retained"
    );
}

#[test]
fn advertised_engine_only_routes_are_not_misrepresented_as_http_commands() {
    let action = super::interface::LegalAction {
        route: "expel".to_owned(),
        ..super::interface::LegalAction::default()
    };
    assert!(
        !action.http_supported(),
        "visibility.go advertises expel, but http.go has no expel registration"
    );
    let fortify = super::interface::LegalAction {
        route: "fortify".to_owned(),
        ..super::interface::LegalAction::default()
    };
    assert!(
        fortify.http_supported(),
        "registered route remains executable"
    );
}

#[test]
fn go_nil_management_slices_are_empty_not_decode_errors() {
    let projection: ServerProjection = serde_json::from_str(r#"{"revision":1,"turn":1,"tiles":null,"units":null,"cities":null,"legal_actions":null,"city_reports":null,"trade_routes":null,"embassy_player_ids":null}"#).expect("Go nil slices");
    assert!(
        projection.interface.legal_actions.is_empty(),
        "nil actions are empty"
    );
    assert!(
        projection.interface.city_reports.is_empty(),
        "nil reports are empty"
    );
    assert!(
        projection.interface.trade_routes.is_empty(),
        "nil routes are empty"
    );
}
