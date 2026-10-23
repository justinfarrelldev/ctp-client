//! Identity removal cannot silently select another duplicate queue entry.
use super::{ManagementUi, ManagerKind, ManagerView};
use crate::gameplay::{model::GameState, server::ServerProjection};
use alloc::collections::BTreeMap;
use serde_json::json;

#[test]
fn receipts_drop_removed_entry_selection_but_preserve_independent_buildable_and_city() {
    let projection:ServerProjection=serde_json::from_value(json!({"revision":2_i64,"turn":1_u32,"tiles":[{"id":"a","x":0_u16,"y":0_u16,"terrain":"plains"}],"cities":[{"id":"c","tile_id":"a","owner_id":"human","queue":[{"entry_id":"c:2","id":"UNIT_WARRIOR"}]}]})).expect("receipt");
    let state = GameState::from_server_projection(&projection, 1, 1);
    let mut ui = ManagementUi {
        view: Some(ManagerView {
            kind: ManagerKind::Build,
            city_id: Some("c".to_owned()),
            selections: BTreeMap::from([
                ("queue".to_owned(), "c:1".to_owned()),
                ("description".to_owned(), "c:1".to_owned()),
                ("buildable".to_owned(), "UNIT_WARRIOR".to_owned()),
            ]),
            scroll: BTreeMap::new(),
        }),
        ..ManagementUi::default()
    };
    ui.reconcile(&state);
    let view = ui.view.as_ref().expect("manager remains open");
    assert_eq!(
        view.city_id.as_deref(),
        Some("c"),
        "unchanged exact city remains"
    );
    assert_eq!(
        view.selections.get("buildable").map(String::as_str),
        Some("UNIT_WARRIOR"),
        "independent content choice remains"
    );
    assert!(
        !view.selections.contains_key("queue"),
        "removed entry does not select c:2"
    );
    assert_eq!(view.selections.get("description").map(String::as_str),Some("UNIT_WARRIOR"),"released ShowSelectedInfo falls back only to independent explicit left content, never removed c:1 or neighboring c:2 entry identity");
}
