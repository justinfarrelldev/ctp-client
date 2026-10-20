//! Populated source row columns use fileless viewer fixtures, never local rules.

use super::{ManagementUi, ManagerIntent, ManagerKind};
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::model::GameState;
use crate::gameplay::server::ServerProjection;
use serde_json::json;
use std::path::Path;

/// Exact public field names; movement deliberately uses an unscaled custom unit.
fn fixture() -> (ManagementUi, GameState) {
    let projection:ServerProjection=serde_json::from_value(json!({"revision":7_i64,"turn":4_u32,"tiles":[{"id":"tile","x":1_u16,"y":1_u16,"terrain":"TERRAIN_PLAINS"}],"cities":[{"id":"city","owner_id":"human","tile_id":"tile","population":3_u32,"queue":[{"id":"UNIT_HOPLITE","kind":"unit","cost":20_i64}],"stored_production":10_i64}],"research":{"advances":["ADVANCE_AGRICULTURE","ADVANCE_AGRICULTURE"],"target_id":"ADVANCE_BALLISTICS","progress":10_i64,"available":["ADVANCE_BALLISTICS"]},"rules":{"units":[{"id":"UNIT_SETTLER","attack":0_i64,"defense":10_i64,"armor":1_i64,"firepower":1_i64,"movement":3_i64}],"advances":[{"id":"ADVANCE_AGRICULTURE","cost":20_i64},{"id":"ADVANCE_BALLISTICS","cost":100_i64}]},"legal_actions":[{"route":"queue-production","entity_id":"city","target_id":"UNIT_SETTLER","payload":{"city_id":"city","item":{"id":"UNIT_SETTLER","kind":"unit","cost":100_i64}}}]})).expect("public projection fixture");
    let state = GameState::from_server_projection(&projection, 1_u8, 1_u64);
    let ui = ManagementUi::new(
        LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"))
            .expect("installed source"),
    )
    .expect("resolved templates");
    (ui, state)
}

#[test]
fn build_rows_use_installed_columns_and_keep_cell_clicks_bound_to_exact_parent_identity() {
    let (mut ui, state) = fixture();
    ui.open(ManagerKind::Build, &state, Some("city"));
    let widgets = ui.widgets(&state);
    let parent = widgets
        .iter()
        .find(|widget| widget.id == "ItemsBox.UnitsList/UNIT_SETTLER")
        .expect("buildable");
    let attack = widgets
        .iter()
        .find(|widget| widget.id == "ItemsBox.UnitsList/UNIT_SETTLER/cell/box.Attack")
        .expect("source attack column");
    assert_eq!(attack.label, "0", "explicit actual zero is not unavailable");
    assert_eq!(
        attack.rect.x,
        parent.rect.x.saturating_add(148_i32),
        "eq_UnitItem Attack position is source-defined"
    );
    assert_eq!(attack.rect.width, 21_u32, "source stat column width");
    let ranged = widgets
        .iter()
        .find(|widget| widget.id == "ItemsBox.UnitsList/UNIT_SETTLER/cell/box.Ranged")
        .expect("ranged");
    assert_eq!(
        ranged.label, "0",
        "Go RuntimeUnitDefinition.ranged has omitempty: absence means zero under this verified public contract"
    );
    let movement = widgets
        .iter()
        .find(|widget| widget.id == "ItemsBox.UnitsList/UNIT_SETTLER/cell/box.Movement")
        .expect("movement");
    assert_eq!(
        movement.label, "—",
        "do not divide custom movement by the retail 100-point scale"
    );
    let time = widgets
        .iter()
        .find(|widget| widget.id == "ItemsBox.UnitsList/UNIT_SETTLER/cell/box.Time")
        .expect("turns");
    assert_eq!(
        time.label, "—",
        "missing authoritative production rate cannot invent a turn forecast"
    );
    assert_eq!(
        ui.intent_at(
            attack.rect.x.saturating_add(2_i32),
            attack.rect.y.saturating_add(2_i32),
            &state
        ),
        parent.intent,
        "numeric cells select the exact parent item"
    );
    assert_eq!(
        ui.activate("Settler", &state),
        Some(ManagerIntent::Select(
            "buildable".to_owned(),
            "UNIT_SETTLER".to_owned()
        )),
        "template labels retain accessible row activation"
    );
}

#[test]
fn cached_source_item_height_controls_both_row_rectangles_and_page_capacity() {
    let (mut ui, mut state) = fixture();
    let viewer = state.viewer.as_mut().expect("viewer");
    viewer
        .interface
        .research
        .as_mut()
        .expect("research")
        .available = vec![
        "ADVANCE_TOOLMAKING".to_owned(),
        "ADVANCE_AGRICULTURE".to_owned(),
        "ADVANCE_BALLISTICS".to_owned(),
        "ADVANCE_IRON_WORKING".to_owned(),
    ];
    let item = ui
        .row_templates
        .get_mut("SciListItem")
        .expect("source item");
    item.children
        .iter_mut()
        .find(|child| child.name == "Name")
        .expect("name")
        .fields
        .insert("heightpix".to_owned(), "30".to_owned());
    ui.open(ManagerKind::Research, &state, None);
    let widgets = ui.widgets(&state);
    let rows: Vec<_> = widgets
        .iter()
        .filter(|widget| {
            widget.id.starts_with("Background.SciAdvanceList/ADVANCE_")
                && widget.node.field("row-container") == Some("true")
        })
        .collect();
    assert_eq!(
        rows.len(),
        3_usize,
        "114-pixel pane fits three 30-pixel source items, not six guessed 18-pixel rows"
    );
    assert!(
        rows.iter().all(|widget| widget.rect.height == 30_u32),
        "source item height controls both drawing and scroll capacity"
    );
}

#[test]
fn science_discovery_table_is_not_the_available_research_picker() {
    let (mut ui, state) = fixture();
    ui.open(ManagerKind::Science, &state, None);
    let widgets = ui.widgets(&state);
    assert_eq!(
        widgets
            .iter()
            .filter(|widget| widget.id == "AdvanceList/ADVANCE_AGRICULTURE")
            .count(),
        1_usize,
        "show known discoveries once"
    );
    assert!(
        !widgets
            .iter()
            .any(|widget| widget.id == "AdvanceList/ADVANCE_BALLISTICS"),
        "available target is not yet a discovered advance"
    );
    let own = widgets
        .iter()
        .find(|widget| widget.id == "AdvanceList/ADVANCE_AGRICULTURE/known/self")
        .expect("source own discovery block");
    assert_eq!(
        own.node.field("primitive"),
        Some("fill"),
        "draw real known-player color, not an inferred research forecast"
    );
    assert!(
        !widgets
            .iter()
            .any(|widget| widget.id.contains("/known/foreign")),
        "embassy identity alone does not disclose foreign research"
    );
    ui.open(ManagerKind::Research, &state, None);
    let choices = ui.widgets(&state);
    assert!(
        choices
            .iter()
            .any(|widget| widget.id == "Background.SciAdvanceList/ADVANCE_BALLISTICS"),
        "research picker still uses exact available targets"
    );
    assert!(
        !choices
            .iter()
            .any(|widget| widget.id == "Background.SciAdvanceList/ADVANCE_AGRICULTURE"),
        "do not offer already-known discoveries as new targets"
    );
}
