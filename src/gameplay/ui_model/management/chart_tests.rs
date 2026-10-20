//! Source-layout chart navigation uses only viewer-safe definitions and research.

use super::{ManagementUi, ManagerKind};
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::model::GameState;
use serde_json::json;
use std::path::Path;

/// Synthetic prerequisite relations, not a copied retail technology tree.
fn fixture() -> (ManagementUi, GameState) {
    let mut state = GameState::new(8_u16, 8_u16, 1_u8, 1_u64);
    state.viewer=Some(serde_json::from_value(json!({"revision":11_i64,"turn":2_u32,
        "research":{"target_id":"ADVANCE_BALLISTICS","advances":["ADVANCE_AGRICULTURE"],"available":["ADVANCE_BALLISTICS"]},
        "rules":{"advances":[
            {"id":"ADVANCE_AGRICULTURE","cost":13_i32},
            {"id":"ADVANCE_BALLISTICS","cost":17_i32,"prerequisites":["ADVANCE_AGRICULTURE"]},
            {"id":"ADVANCE_WRITING","cost":19_i32,"prerequisites":["ADVANCE_BALLISTICS"]}
        ]}})).expect("viewer contract"));
    let mut ui = ManagementUi::new(
        LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"))
            .expect("read-only source"),
    )
    .expect("layouts");
    ui.open(ManagerKind::Library, &state, None);
    (ui, state)
}

/// Activates one real enabled source descriptor, not a coordinate-only click.
fn activate(ui: &mut ManagementUi, state: &GameState, id: &str) {
    let intent = ui.activate_id(id, state).expect("enabled source control");
    assert!(ui.apply_local(intent), "local chart navigation");
}

#[test]
fn chart_neighbors_recenter_without_research_or_article_mutation_and_center_opens_article() {
    let (mut ui, state) = fixture();
    let before = state.clone();
    activate(&mut ui, &state, "Tabs.TechTreeTab.TabButton");
    let widgets = ui.widgets(&state);
    let center = widgets
        .iter()
        .find(|widget| widget.id == "Tabs.TechTreeTab.TabPanel/center/ADVANCE_BALLISTICS")
        .expect("center source button");
    assert_eq!(
        center.rect,
        crate::gameplay::ui_model::MenuRect {
            x: 455_i32,
            y: 370_i32,
            width: 124_u32,
            height: 20_u32
        },
        "Chart::Update geometry from original 440x200/124x20 template"
    );
    activate(
        &mut ui,
        &state,
        "Tabs.TechTreeTab.TabPanel/successors/ADVANCE_WRITING",
    );
    assert_eq!(
        ui.library.current.as_deref(),
        Some("ADVANCE_BALLISTICS"),
        "neighbor only recenters the chart, as the original callback does"
    );
    assert!(
        ui.widgets(&state)
            .iter()
            .any(|widget| widget.id == "Tabs.TechTreeTab.TabPanel/center/ADVANCE_WRITING"),
        "new center is visible"
    );
    activate(
        &mut ui,
        &state,
        "Tabs.TechTreeTab.TabPanel/center/ADVANCE_WRITING",
    );
    assert_eq!(
        ui.library.current.as_deref(),
        Some("ADVANCE_WRITING"),
        "original empty center callback is replaced by useful article navigation"
    );
    assert_eq!(
        ui.view
            .as_ref()
            .expect("view")
            .selections
            .get("tab:Tabs")
            .map(String::as_str),
        Some("Gameplay"),
        "article content opens explicitly"
    );
    assert_eq!(
        state, before,
        "no current/persistent research mutation or server call"
    );
}

#[test]
fn chart_follows_runtime_not_retail_prerequisites_and_disables_missing_rule_panels() {
    let (mut ui, mut state) = fixture();
    activate(&mut ui, &state, "Tabs.TechTreeTab.TabButton");
    let widgets = ui.widgets(&state);
    assert!(
        widgets
            .iter()
            .any(|widget| widget.id.ends_with("/prerequisites/ADVANCE_AGRICULTURE")),
        "actual synthetic prerequisite"
    );
    assert!(
        !widgets
            .iter()
            .any(|widget| widget.id.ends_with("/prerequisites/ADVANCE_TOOLMAKING")),
        "never substitute the installed retail tree"
    );
    state.viewer.as_mut().expect("viewer").interface.rules = None;
    assert!(
        ui.activate_id("Tabs.TechTreeTab.TabButton", &state)
            .is_none(),
        "missing definitions are not an empty enabled graph"
    );
    assert!(
        ui.widgets(&state).iter().any(|widget| widget
            .id
            .starts_with("Tabs.Gameplay.TabPanel.GameplayText/word-")),
        "stale unavailable tab falls back to real article content"
    );
}

#[test]
fn chart_paginates_more_than_four_neighbors_and_bounds_malformed_cycles() {
    let (mut ui, mut state) = fixture();
    let definitions = state
        .viewer
        .as_mut()
        .expect("viewer")
        .interface
        .rules
        .as_mut()
        .expect("rules")["advances"]
        .as_array_mut()
        .expect("definitions");
    for index in 0_u32..6_u32 {
        definitions.push(json!({"id":format!("ADVANCE_SYNTHETIC_{index}"),"prerequisites":["ADVANCE_BALLISTICS"]}));
    }
    definitions[1_usize]["prerequisites"] = json!([
        "ADVANCE_BALLISTICS",
        "ADVANCE_AGRICULTURE",
        "ADVANCE_AGRICULTURE"
    ]);
    activate(&mut ui, &state, "Tabs.TechTreeTab.TabButton");
    let first = ui.widgets(&state);
    assert_eq!(
        first
            .iter()
            .filter(|widget| widget
                .id
                .starts_with("Tabs.TechTreeTab.TabPanel/successors/ADVANCE_"))
            .count(),
        4_usize,
        "source page capacity"
    );
    assert_eq!(
        first
            .iter()
            .filter(|widget| widget
                .id
                .starts_with("Tabs.TechTreeTab.TabPanel/prerequisites/ADVANCE_"))
            .count(),
        1_usize,
        "self references and duplicates do not create recursive/duplicate nodes"
    );
    activate(&mut ui, &state, "Tabs.TechTreeTab.TabPanel/successors/next");
    assert!(
        ui.widgets(&state)
            .iter()
            .any(|widget| widget.id.ends_with("/successors/ADVANCE_SYNTHETIC_5")),
        "overflow neighbors remain accessible rather than silently truncated"
    );
    activate(
        &mut ui,
        &state,
        "Tabs.TechTreeTab.TabPanel/successors/previous",
    );
    assert!(
        ui.widgets(&state)
            .iter()
            .any(|widget| widget.id.ends_with("/successors/ADVANCE_WRITING")),
        "previous page restores exact identities"
    );
}
