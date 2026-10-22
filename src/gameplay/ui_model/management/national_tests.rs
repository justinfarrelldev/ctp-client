//! National list geometry/identities never substitute raw yields for effective accounts.
use super::{ManagementUi, ManagerIntent, ManagerKind};
use crate::gameplay::{assets::layout::LayoutBook, model::GameState, server::ServerProjection};
use serde_json::{Value, json};
use std::path::Path;
/// Accurate owned-city DTO fields; foreign visible city is never a management row.
fn fixture() -> (ManagementUi, GameState) {
    let viewer:ServerProjection=serde_json::from_value(json!({"revision":9_i64,"turn":2_u32,"contract":{"version":2_u32},"tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"plains"},{"id":"b","x":2_u16,"y":1_u16,"terrain":"plains"},{"id":"f","x":3_u16,"y":1_u16,"terrain":"plains"}],"units":[{"id":"u","owner_id":"human","tile_id":"a","type_id":"UNIT_SETTLER","health":10_i16,"max_health":10_i16}],"cities":[{"id":"c1","owner_id":"human","tile_id":"a","name":"Rome","population":3_u32,"happiness":70_i64,"crime_percent":5_i64},{"id":"c2","owner_id":"human","tile_id":"b","name":"Athens","population":2_u32,"happiness":65_i64,"crime_percent":10_i64,"yields":{"food":100_i64,"production":99_i64,"gold":98_i64,"science":97_i64},"specialists":{"scientist":1_i64},"governor":{"enabled":true,"priority":"production"},"queue":[{"entry_id":"c2:1","id":"UNIT_HOPLITE","kind":"unit","cost":20_i64}],"production_forecast":{"revision":9_i64,"status":"estimated","rate":7_i64,"turns":3_u32,"assumption":"current-economy"},"rush_buy_quote":{"revision":9_i64,"status":"available","cost":30_i64}},{"id":"foreign","owner_id":"other","tile_id":"f","name":"Not ours","population":9_u32}],"diplomacy":{"gold":100_i32},"rules":{"governor_priorities":["production","science"]},"legal_actions":[{"route":"set-governor","entity_id":"c2","target_id":"production","payload":{"city_id":"c2","enabled":true,"priority":"production"},"fields":["city_id","enabled","priority"]},{"route":"set-governor","entity_id":"c2","target_id":"science","payload":{"city_id":"c2","enabled":true,"priority":"science"},"fields":["city_id","enabled","priority"]},{"route":"set-governor","entity_id":"c2","target_id":"disabled","payload":{"city_id":"c2","enabled":false},"fields":["city_id","enabled"]},{"route":"rush-buy","entity_id":"c2","target_id":"UNIT_HOPLITE","payload":{"city_id":"c2"}}]})).expect("source-compatible viewer");
    let state = GameState::from_server_projection(&viewer, 3, 1);
    let mut ui = ManagementUi::new(
        LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"))
            .expect("installed layouts"),
    )
    .expect("cached UI");
    ui.open(ManagerKind::National, &state, None);
    (ui, state)
}
/// One exact canonical city selection, shared across resource/status/specialist panes.
fn tab(ui: &mut ManagementUi, state: &GameState, name: &str) {
    ui.apply_local(
        ui.activate_id(&format!("TabGroup.{name}.TabButton"), state)
            .expect("source tab"),
    );
}
/// Actual column text, not a concatenated row that can hide overlaps/missing cells.
fn value(ui: &ManagementUi, state: &GameState, list: &str, city: &str, column: u8) -> String {
    ui.widgets(state)
        .into_iter()
        .find(|widget| widget.id == format!("{list}/{city}/cell/column{column}"))
        .expect("original source item column")
        .label
}
#[test]
fn national_reference_has_three_tabs_eleven_resource_headers_and_all_four_source_bottom_actions() {
    let (ui, state) = fixture();
    let widgets = ui.widgets(&state);
    for name in ["Tab1", "Tab2", "Tab3"] {
        let button = widgets
            .iter()
            .find(|widget| widget.id == format!("TabGroup.{name}.TabButton"))
            .expect("source three-tab revision");
        assert!(
            button.rect.width < 240_u32,
            "native three-tab art, not stretched legacy two-tab image"
        );
    }
    let path = "TabGroup.Tab1.TabPanel.ResourceList";
    let list = widgets
        .iter()
        .find(|widget| widget.id == path)
        .expect("resource list");
    assert_eq!(
        (
            list.node.integer("list-content-width"),
            list.node.integer("list-content-height")
        ),
        (Some(458_i32), Some(238_i32)),
        "public source resource pane size"
    );
    for index in 0_u8..11_u8 {
        assert!(
            widgets
                .iter()
                .any(|widget| widget.id == format!("{path}.header.switch{index}")),
            "all original resource header icons including conversion/franchise"
        );
    }
    assert!(
        widgets
            .iter()
            .any(|widget| widget.id == format!("{path}.header.switch1") && widget.label == "Pop."),
        "reference uses a bounded population abbreviation, not a clipped full word"
    );
    for button in [
        "BuildQueueButton",
        "CityManagerButton",
        "DisbandButton",
        "CloseButton",
    ] {
        assert!(
            widgets.iter().any(|widget| widget.id == button),
            "source four-action row retains {button}"
        );
    }
    assert!(!widgets.into_iter().filter_map(|widget|widget.intent).any(|intent|matches!(intent,ManagerIntent::Select(key,id) if key == "city" && id == "foreign")),"foreign visible cities are not manageable");
}
#[test]
fn resource_columns_use_received_population_happiness_crime_and_never_label_raw_yields_as_effective_accounts()
 {
    let (ui, state) = fixture();
    let list = "TabGroup.Tab1.TabPanel.ResourceList";
    for (column, expected) in [(0_u8, "Athens"), (1_u8, "2"), (2_u8, "65"), (8_u8, "10")] {
        assert_eq!(
            value(&ui, &state, list, "c2", column),
            expected,
            "actual scalar source column {column}"
        );
    }
    for column in [3_u8, 4_u8, 5_u8, 6_u8, 7_u8, 9_u8, 10_u8] {
        assert_eq!(
            value(&ui, &state, list, "c2", column),
            "—",
            "base yields/forecast rates are not gross/net/pollution/conversion/franchise accounting"
        );
    }
    let widgets = ui.widgets(&state);
    let columns: Vec<_> = (0_u8..11_u8)
        .map(|column| {
            widgets
                .iter()
                .find(|widget| widget.id == format!("{list}/c2/cell/column{column}"))
                .expect("column")
                .rect
                .x
        })
        .collect();
    assert!(
        columns.windows(2).all(|pair| pair[0] < pair[1]),
        "independent source columns cannot overprint"
    );
}
#[test]
fn status_and_specialist_tabs_preserve_exact_selected_city_governor_queue_and_current_forecast() {
    let (mut ui, state) = fixture();
    ui.apply_local(ManagerIntent::Select("city".to_owned(), "c2".to_owned()));
    tab(&mut ui, &state, "Tab2");
    let list = "TabGroup.Tab2.TabPanel.StatusList";
    assert_eq!(
        value(&ui, &state, list, "c2", 1),
        ui.book.label("str_ldl_ND_ON"),
        "received active mayor"
    );
    assert_eq!(
        value(&ui, &state, list, "c2", 4),
        "3",
        "revision-bound engine estimate"
    );
    assert_eq!(
        value(&ui, &state, list, "c2", 3),
        ui.book.label("UNIT_HOPLITE"),
        "canonical head content, not queue entry ID"
    );
    let command = ui
        .activate_id("TabGroup.Tab2.TabPanel.GovernorSelect.Toggle", &state)
        .and_then(|intent| match intent {
            ManagerIntent::Submit(command) => Some(command),
            _ => None,
        })
        .expect("exact selected-city mayor toggle");
    assert_eq!(
        command.payload["city_id"],
        json!("c2"),
        "never default first city"
    );
    assert_eq!(
        command.payload["enabled"],
        json!(false),
        "current toggle goes off"
    );
    tab(&mut ui, &state, "Tab3");
    let specialist = "TabGroup.Tab3.TabPanel.SpecialistList";
    assert_eq!(
        value(&ui, &state, specialist, "c2", 8),
        "1",
        "received scientist count"
    );
    assert_eq!(
        value(&ui, &state, specialist, "c2", 5),
        "0",
        "complete v2 owned map omits a zero farmer count"
    );
    for column in [2_u8, 3_u8, 9_u8] {
        assert_eq!(
            value(&ui, &state, specialist, "c2", column),
            "—",
            "worker/slave/combat counts require their real accounting, not population subtraction or stack guessing"
        );
    }
    assert_eq!(
        ui.view
            .as_ref()
            .and_then(|view| view.selections.get("city"))
            .map(String::as_str),
        Some("c2"),
        "same canonical identity across panes"
    );
}
#[test]
fn selected_city_source_controls_show_mayor_priority_and_exact_current_price_not_a_first_city() {
    let (mut ui, state) = fixture();
    ui.apply_local(ManagerIntent::Select("city".to_owned(), "c2".to_owned()));
    tab(&mut ui, &state, "Tab2");
    let widgets = ui.widgets(&state);
    for (path, label) in [
        ("GovernorSelect.Toggle", "X"),
        ("GovernorSelect.Pulldown/selected", "Production"),
        ("RushBuyValue", "30"),
    ] {
        assert!(
            widgets.iter().any(
                |widget| widget.id == format!("TabGroup.Tab2.TabPanel.{path}")
                    && widget.label == label
            ),
            "source current-city control {path}"
        );
    }
    let command = ui
        .activate_id("TabGroup.Tab2.TabPanel.RushBuyButton", &state)
        .and_then(|intent| match intent {
            ManagerIntent::Submit(command) => Some(command),
            _ => None,
        })
        .expect("known affordable current quote and exact action");
    assert_eq!(
        command.payload,
        json!({"city_id":"c2"}),
        "no locally computed price or different city sent"
    );
}

#[test]
fn stale_unaffordable_or_absent_price_and_captured_selection_cannot_rush_or_toggle_another_city() {
    let (mut ui, mut state) = fixture();
    ui.apply_local(ManagerIntent::Select("city".to_owned(), "c2".to_owned()));
    tab(&mut ui, &state, "Tab2");
    state.viewer.as_mut().expect("viewer").revision = 10_i64;
    assert!(
        ui.activate_id("TabGroup.Tab2.TabPanel.RushBuyButton", &state)
            .is_none(),
        "cached legal payload alone must not silently buy at an unknown stale displayed price"
    );
    let current = state.viewer.as_mut().expect("viewer");
    current.revision = 9_i64;
    current.cities[1]
        .rush_buy_quote
        .as_mut()
        .expect("quote")
        .status = "insufficient-gold".to_owned();
    assert!(
        ui.activate_id("TabGroup.Tab2.TabPanel.RushBuyButton", &state)
            .is_none(),
        "unaffordable displayed quote is not an actionable purchase"
    );
    state.viewer.as_mut().expect("viewer").cities[1].rush_buy_quote = None;
    assert!(
        ui.activate_id("TabGroup.Tab2.TabPanel.RushBuyButton", &state)
            .is_none(),
        "missing price is not a free or silently unknown purchase"
    );
    let mut captured = state.viewer.as_ref().expect("viewer").clone();
    captured.cities[1].owner_id = "other".to_owned();
    let next = GameState::from_server_projection(&captured, 3, 1);
    ui.reconcile(&next);
    assert!(
        !ui.view
            .as_ref()
            .expect("manager")
            .selections
            .contains_key("city"),
        "captured target loses exact selection"
    );
    assert!(
        ui.activate_id("TabGroup.Tab2.TabPanel.GovernorSelect.Toggle", &next)
            .is_none(),
        "do not retarget surviving Rome"
    );
}

#[test]
fn native_resource_headers_sort_received_columns_stably_without_changing_selected_city() {
    let (mut ui, state) = fixture();
    ui.apply_local(ManagerIntent::Select("city".to_owned(), "c2".to_owned()));
    let header = "TabGroup.Tab1.TabPanel.ResourceList.header.switch1";
    for expected in [
        vec!["c2".to_owned(), "c1".to_owned()],
        vec!["c1".to_owned(), "c2".to_owned()],
    ] {
        ui.apply_local(
            ui.activate_id(header, &state)
                .expect("original population sort control"),
        );
        let cities: Vec<_> = ui
            .widgets(&state)
            .into_iter()
            .filter_map(|widget| match widget.intent {
                Some(ManagerIntent::Select(key, id)) if key == "city" => Some(id),
                _ => None,
            })
            .collect();
        assert_eq!(
            cities, expected,
            "real current values, not row text spacing or canonical identity order"
        );
        assert_eq!(
            ui.view
                .as_ref()
                .and_then(|view| view.selections.get("city"))
                .map(String::as_str),
            Some("c2"),
            "sort never silently retargets the selection"
        );
    }
    tab(&mut ui, &state, "Tab2");
    for _direction in 0_u8..2_u8 {
        ui.apply_local(
            ui.activate_id("TabGroup.Tab2.TabPanel.StatusList.header.switch2", &state)
                .expect("source priority sort"),
        );
        let cities: Vec<_> = ui
            .widgets(&state)
            .into_iter()
            .filter_map(|widget| match widget.intent {
                Some(ManagerIntent::Select(key, id)) if key == "city" => Some(id),
                _ => None,
            })
            .collect();
        assert_eq!(
            cities,
            vec!["c2".to_owned(), "c1".to_owned()],
            "unavailable priority remains last in both sort directions, never numeric zero"
        );
    }
}

#[test]
fn resource_warning_uses_only_actual_rioting_and_original_black_red_text_not_local_thresholds() {
    let (ui, mut state) = fixture();
    state.viewer.as_mut().expect("viewer").cities[1].facts["rioting"] = json!(true);
    let widgets = ui.widgets(&state);
    for (column, red) in [(0_u8, "255"), (2_u8, "255"), (1_u8, "0")] {
        let widget = widgets
            .iter()
            .find(|widget| {
                widget.id == format!("TabGroup.Tab1.TabPanel.ResourceList/c2/cell/column{column}")
            })
            .expect("source resource cell");
        assert_eq!(
            widget.node.field("fontcolorred"),
            Some(red),
            "actual canonical riot warning, not guessed happiness/food threshold"
        );
        assert_eq!(
            widget.node.field("fontcolorgreen"),
            Some("0"),
            "source black/red palette"
        );
    }
}

#[test]
fn empty_queue_is_not_unavailable_null_or_missing_legacy_production() {
    let (mut ui, mut state) = fixture();
    tab(&mut ui, &state, "Tab2");
    let list = "TabGroup.Tab2.TabPanel.StatusList";
    assert_eq!(
        value(&ui, &state, list, "c1", 3),
        ui.book.label("str_ldl_ND_EMPTY"),
        "complete v2 omitted queue is explicitly empty"
    );
    assert_eq!(
        value(&ui, &state, list, "c1", 4),
        ui.book.label("str_ldl_ND_NA"),
        "empty queue has source N/A, not a fabricated zero-turn forecast"
    );
    let current = state.viewer.as_mut().expect("viewer");
    current.cities[1].facts["queue"] = Value::Null;
    current.cities[1].production_forecast = None;
    for column in [3_u8, 4_u8] {
        assert_eq!(
            value(&ui, &state, list, "c2", column),
            "—",
            "null production facts remain unavailable"
        );
    }
    state.viewer.as_mut().expect("viewer").interface.contract = None;
    assert_eq!(
        value(&ui, &state, list, "c1", 3),
        "—",
        "legacy absence is not a known empty queue"
    );
}

#[test]
fn selected_city_priority_and_disband_are_explicit_revision_bound_forms_never_first_target_commands()
 {
    let (mut ui, mut state) = fixture();
    ui.apply_local(ManagerIntent::Select("city".to_owned(), "c2".to_owned()));
    tab(&mut ui, &state, "Tab2");
    let priority = ui
        .activate_id("TabGroup.Tab2.TabPanel.GovernorSelect.Pulldown", &state)
        .and_then(|intent| match intent {
            ManagerIntent::Dialog(dialog) => Some(dialog),
            _ => None,
        })
        .expect("explicit pinned priority chooser");
    assert!(
        priority.choices.iter().any(|choice| choice.command.payload
            == json!({"city_id":"c2","enabled":true,"priority":"science"})),
        "current selected-city legal priority, not initial Production or first city"
    );
    assert!(
        priority
            .choices
            .iter()
            .all(|choice| choice.command.revision == 9_i64
                && choice.command.payload["city_id"] == "c2"),
        "every alternative is bound to same exact selected city/receipt"
    );
    state
        .viewer
        .as_mut()
        .expect("viewer")
        .interface
        .legal_actions
        .push(crate::gameplay::server::interface::LegalAction {
            route: "disband-city".to_owned(),
            entity_id: "c2".to_owned(),
            fields: vec!["city_id".to_owned()],
            payload: json!({"city_id":"c2"}),
            ..crate::gameplay::server::interface::LegalAction::default()
        });
    let confirmation = ui
        .activate_id("DisbandButton", &state)
        .and_then(|intent| match intent {
            ManagerIntent::Dialog(dialog) => Some(dialog),
            _ => None,
        })
        .expect("irreversible action requires explicit confirmation");
    assert_eq!(
        confirmation.choices.len(),
        1_usize,
        "one exact current selected-city target"
    );
    let command = &confirmation.choices[0].command;
    assert_eq!(
        command.payload,
        json!({"city_id":"c2"}),
        "not Rome or a local city index"
    );
    state.viewer.as_mut().expect("viewer").revision = 10_i64;
    assert!(
        !command.valid_for(&state),
        "new receipt invalidates prior confirmation, never silently retargeting it"
    );
    state.viewer.as_mut().expect("viewer").cities[1]
        .governor
        .as_mut()
        .expect("mayor")
        .enabled = false;
    assert!(
        ui.activate_id("TabGroup.Tab2.TabPanel.GovernorSelect.Pulldown", &state)
            .is_none(),
        "source priority picker is disabled when the selected mayor is off"
    );
}

#[test]
fn absent_legacy_or_null_specialists_and_stale_forecasts_remain_unavailable() {
    let (mut ui, mut state) = fixture();
    tab(&mut ui, &state, "Tab3");
    state.viewer.as_mut().expect("viewer").cities[1].facts["specialists"] = Value::Null;
    assert_eq!(
        value(
            &ui,
            &state,
            "TabGroup.Tab3.TabPanel.SpecialistList",
            "c2",
            8
        ),
        "—",
        "null is not a zero-count map"
    );
    let viewer = state.viewer.as_mut().expect("viewer");
    viewer.interface.contract = None;
    let removed = viewer.cities[1]
        .facts
        .as_object_mut()
        .expect("facts")
        .remove("specialists");
    assert!(removed.is_some(), "remove prior explicit field");
    assert_eq!(
        value(
            &ui,
            &state,
            "TabGroup.Tab3.TabPanel.SpecialistList",
            "c2",
            5
        ),
        "—",
        "legacy missing count is not a v2 zero"
    );
    tab(&mut ui, &state, "Tab2");
    state.viewer.as_mut().expect("viewer").revision = 10_i64;
    assert_eq!(
        value(&ui, &state, "TabGroup.Tab2.TabPanel.StatusList", "c2", 4),
        "—",
        "no stale quote/turn estimate shown as current"
    );
}
