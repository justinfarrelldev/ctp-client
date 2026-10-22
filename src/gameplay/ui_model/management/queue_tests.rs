//! Stable-entry editor selections remain distinct from library content topics.
use super::{ManagementUi, ManagerKind, ManagerView, data, queue};
use crate::gameplay::{model::GameState, server::ServerProjection};
use alloc::collections::BTreeMap;
use serde_json::json;

#[test]
fn synthetic_test_content_is_explicitly_described_not_mistaken_for_retail_unit_names() {
    let ui = ManagementUi::default();
    for (id, label) in [
        ("UNIT_SYNTHETIC", "Synthetic test unit"),
        ("IMPROVE_SYNTHETIC", "Synthetic test building"),
        ("WONDER_SYNTHETIC", "Synthetic test wonder"),
    ] {
        assert_eq!(
            data::label(&ui, id),
            label,
            "explain received test records without pretending they are retail units/buildings or changing canonical ID"
        );
    }
}
#[test]
fn received_retail_catalog_populates_all_three_production_categories_and_keeps_exact_authority_ids()
{
    let viewer:ServerProjection=serde_json::from_value(json!({"revision":9_i64,"turn":2_u32,"contract":{"version":2_u32,"features":{"atomic_production_queue":true}},"tiles":[{"id":"tile","x":0_u16,"y":0_u16,"terrain":"TERRAIN_PLAINS"}],"cities":[{"id":"own-city","owner_id":"human","tile_id":"tile","queue":[]}],"rules":{"units":[{"id":"UNIT_SETTLER","production_cost":3_i64},{"id":"UNIT_WARRIOR","production_cost":4_i64}],"production":[{"id":"UNIT_SETTLER","kind":"unit","cost":3_i64},{"id":"IMPROVE_GRANARY","kind":"building","cost":5_i64},{"id":"WONDER_PYRAMIDS","kind":"wonder","cost":6_i64}]},"legal_actions":[{"route":"edit-production","entity_id":"own-city","fields":["city_id","entries"],"payload":{"city_id":"own-city","entries":[]}}]})).expect("actual projected catalog/action shapes with synthetic numeric test values");
    let state = GameState::from_server_projection(&viewer, 1_u8, 1_u64);
    let ui = ManagementUi::default();
    let mut view = ManagerView {
        kind: ManagerKind::Build,
        city_id: Some("own-city".to_owned()),
        selections: BTreeMap::new(),
        scroll: BTreeMap::new(),
    };
    for (path, ids) in [
        ("ItemsBox.UnitsList", vec!["UNIT_SETTLER", "UNIT_WARRIOR"]),
        ("ItemsBox.BuildingList", vec!["IMPROVE_GRANARY"]),
        ("ItemsBox.WonderList", vec!["WONDER_PYRAMIDS"]),
    ] {
        let rows = data::rows(&ui, &state, &view, path);
        for id in ids {
            assert!(
                rows.iter().any(|row| row.0 == id),
                "received {id} appears in its original source category {path}"
            );
            view.selections
                .insert("buildable".to_owned(), id.to_owned());
            let command = queue::request(&state, &view, "ItemsBox.AddButton")
                .expect("current authorized atomic add");
            assert_eq!(
                command.payload,
                json!({"city_id":"own-city","entries":[{"item_id":id}]}),
                "preserve server ID; never invent client cost/effects or relabel synthetic into retail"
            );
        }
        assert!(
            rows.iter().all(|row| !row.0.contains("SYNTHETIC")),
            "no client-injected fixture content"
        );
    }
    assert_eq!(
        state.viewer.as_ref().expect("authority").cities[0].facts["queue"],
        json!([]),
        "selection/submit intent is not locally executed production"
    );
}

#[test]
fn atomic_queue_edits_keep_duplicate_units_distinct_and_never_redefine_costs() {
    let projection:ServerProjection=serde_json::from_value(json!({"revision":9_i64,"turn":2_u32,"tiles":[{"id":"a","x":0_u16,"y":0_u16,"terrain":"plains"}],"cities":[{"id":"c","tile_id":"a","owner_id":"human","stored_production":40_i64,"queue":[{"entry_id":"c:1","id":"UNIT_WARRIOR"},{"entry_id":"c:2","id":"UNIT_WARRIOR"}]}],"rules":{"units":[{"id":"UNIT_WARRIOR"}]},"legal_actions":[{"route":"edit-production","entity_id":"c","payload":{"city_id":"c","entries":[{"entry_id":"c:1"},{"entry_id":"c:2"}]}}]})).expect("viewer");
    let state = GameState::from_server_projection(&projection, 1, 1);
    let view = ManagerView {
        kind: ManagerKind::Build,
        city_id: Some("c".to_owned()),
        selections: BTreeMap::from([
            ("queue".to_owned(), "c:2".to_owned()),
            ("buildable".to_owned(), "UNIT_WARRIOR".to_owned()),
        ]),
        scroll: BTreeMap::new(),
    };
    for (path, expected) in [
        ("QueueGroup.RemoveButton", json!([{"entry_id":"c:1"}])),
        (
            "QueueGroup.UpButton",
            json!([{"entry_id":"c:2"},{"entry_id":"c:1"}]),
        ),
        (
            "ItemsBox.InsertButton",
            json!([{"entry_id":"c:1"},{"item_id":"UNIT_WARRIOR"},{"entry_id":"c:2"}]),
        ),
        (
            "ItemsBox.AddButton",
            json!([{"entry_id":"c:1"},{"entry_id":"c:2"},{"item_id":"UNIT_WARRIOR"}]),
        ),
        ("QueueGroup.ClearButton", json!([])),
    ] {
        let command = queue::request(&state, &view, path).expect("enabled atomic operation");
        assert_eq!(command.route, "edit-production", "one server transaction");
        assert_eq!(
            command.payload,
            json!({"city_id":"c","entries":expected}),
            "exact stable selection for {path}"
        );
        assert!(command.valid_for(&state), "current authoritative shape");
    }
    assert!(
        queue::request(&state, &view, "QueueGroup.DownButton").is_none(),
        "tail cannot move down"
    );
    assert_eq!(
        state.viewer.as_ref().expect("source").cities[0].facts["stored_production"],
        40_i64,
        "constructing requests never changes production"
    );
}
