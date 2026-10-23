//! Fileless lower-panel presentation with no synthetic economic facts.

use super::ManagementUi;
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::model::GameState;
use crate::gameplay::server::ServerProjection;
use crate::gameplay::ui_model::{ControlPanelTab, InGameUiModel};
use serde_json::json;
use std::path::Path;

/// Current viewer and installed source controls, never a running game server.
fn fixture(value: serde_json::Value) -> (InGameUiModel, GameState) {
    let projection: ServerProjection = serde_json::from_value(value).expect("viewer fixture");
    let state = GameState::from_server_projection(&projection, 1_u8, 1_u64);
    let ui = InGameUiModel {
        management: ManagementUi::new(
            LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2")).expect("source"),
        )
        .expect("panel"),
        ..InGameUiModel::default()
    };
    (ui, state)
}

#[test]
fn research_entry_nonunit_tabs_keep_source_navigation_distinct_from_commands_and_empty_city_art() {
    let (mut ui, state) = fixture(json!({"revision":1_i64,"turn":1_u32}));
    for (tab, id) in [
        (
            ControlPanelTab::Domestic,
            "AdvanceProgress.IconBorder.IconButton",
        ),
        (ControlPanelTab::City, "CitySelect.Pulldown.autobutton"),
        (ControlPanelTab::Messages, "MessageListBox/up"),
        (ControlPanelTab::TileImprovements, "TerraformOceanButton"),
    ] {
        ui.active_tab = tab;
        let widgets = ui.management.panel_widgets(&state, &ui);
        assert!(
            widgets.iter().any(|widget| widget.id == id),
            "source body control {id}"
        );
        assert!(
            widgets.iter().all(|widget| widget
                .intent
                .as_ref()
                .is_none_or(|intent| matches!(intent, &super::ManagerIntent::Select(..)) || (tab==ControlPanelTab::Domestic && matches!(intent,&super::ManagerIntent::Open(super::ManagerKind::Research))))),
            "only local palette selection or source read-only research navigation; painting cannot grant commands"
        );
    }
    ui.active_tab = ControlPanelTab::City;
    let empty = ui.management.panel_widgets(&state, &ui);
    assert!(
        !empty
            .iter()
            .any(|widget| widget.node.field("image40").is_some()),
        "no fabricated city production"
    );
    assert!(
        empty
            .iter()
            .find(|widget| widget.id == "BuildProgress.Title")
            .is_some_and(|widget| widget.label.is_empty()),
        "empty city title"
    );
}

#[test]
fn research_entry_semantic_descriptors_bind_only_actual_domestic_portrait_and_radial_controls() {
    let (mut ui,state)=fixture(json!({"revision":7_i64,"turn":1_u32,"research":{"target_id":"","available":["ADVANCE_TOOLMAKING"]}}));
    ui.active_tab=ControlPanelTab::Domestic;
    let widgets=ui.management.panel_widgets(&state,&ui);
    let actionable:Vec<_>=widgets.into_iter().filter(|widget|widget.id.contains("IconButton")&&widget.intent.is_some()).collect();
    assert_eq!(actionable.len(),2_usize,"only source portrait/radial callbacks, no decorative descendant or generic city callback");
    for widget in actionable {
        assert!(matches!(widget.id.as_str(),"AdvanceProgress.IconBorder.IconButton"|"AdvanceProgress.IconBorder.IconButton.RadialButton"),"exact source control");
        assert_eq!(widget.intent,Some(super::ManagerIntent::Open(super::ManagerKind::Research)),"semantic intent is independent of absent city and absent current research");
    }
}

#[test]
fn missing_projected_city_names_do_not_turn_machine_identities_into_captions() {
    let (mut ui, state) = fixture(
        json!({"revision":1_i64,"turn":1_u32,"tiles":[{"id":"a","x":2_u16,"y":2_u16,"terrain":"TERRAIN_PLAINS"}],"cities":[{"id":"client-city-settler-95-1","owner_id":"human","tile_id":"a","population":1_u32}]}),
    );
    ui.active_tab = ControlPanelTab::City;
    let widgets = ui.management.panel_widgets(&state, &ui);
    assert_eq!(
        widgets
            .iter()
            .find(|widget| widget.id == "CitySelect.Pulldown/selected")
            .map(|widget| widget.label.as_str()),
        Some("—"),
        "missing canonical name is unknown, not the machine identity"
    );
}

#[test]
fn domestic_values_and_city_portraits_follow_exact_viewer_owned_facts() {
    let (mut ui, state) = fixture(
        json!({"revision":3_i64,"turn":2_u32,"diplomacy":{"government_id":"GOVERNMENT_TYRANNY","science_percent":50_i64,"public_works_percent":20_i64},"research":{"target_id":"ADVANCE_BALLISTICS","progress":7_i64},"tiles":[{"id":"t1","x":1_u16,"y":1_u16,"terrain":"TERRAIN_PLAINS"},{"id":"t2","x":2_u16,"y":1_u16,"terrain":"TERRAIN_PLAINS"}],"cities":[{"id":"city-a","owner_id":"human","tile_id":"t1","population":3_u32,"name":"Rome","queue":[{"id":"UNIT_HOPLITE","kind":"unit","cost":20_i64}]},{"id":"city-b","owner_id":"human","tile_id":"t2","population":2_u32},{"id":"foreign","owner_id":"rival","tile_id":"t2","population":99_u32,"name":"SECRET"}]}),
    );
    ui.active_tab = ControlPanelTab::Domestic;
    let widgets = ui.management.panel_widgets(&state, &ui);
    for (id, value) in [
        ("PopulationValue", "5"),
        ("GovernmentValue", "Tyranny"),
        ("ScienceValue", "50%"),
        ("PublicWorksValue", "20%"),
        ("PollutionValue", "—"),
        ("AdvanceProgress.Title", "Ballistics"),
    ] {
        assert_eq!(
            widgets
                .iter()
                .find(|widget| widget.id == id)
                .map(|widget| widget.label.as_str()),
            Some(value),
            "public source value {id}"
        );
    }
    ui.active_tab = ControlPanelTab::City;
    ui.selected_city = Some(crate::gameplay::model::CityId::from_server_id("city-a"));
    let city = ui.management.panel_widgets(&state, &ui);
    assert!(
        city.iter()
            .any(|widget| widget.id == "BuildProgress.Title" && widget.label == "Hoplite"),
        "head production name"
    );
    assert!(
        city.iter()
            .any(|widget| widget.id == "BuildProgress.IconBorder.IconButton"
                && widget.node.field("image40") == Some("upup008l.rim")),
        "exact Hoplite art"
    );
    assert!(
        city.iter()
            .any(|widget| widget.id == "CitySelect.Pulldown/selected" && widget.label == "Rome"),
        "real city caption"
    );
    assert!(
        !city.iter().any(|widget| widget.label.contains("SECRET")),
        "no foreign city in own selector"
    );
}
