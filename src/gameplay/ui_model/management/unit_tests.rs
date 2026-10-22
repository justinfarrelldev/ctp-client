//! Unit manager source columns/readiness retain canonical identity and receipt values.
use super::{ManagementUi, ManagerIntent, ManagerKind};
use crate::gameplay::{assets::layout::LayoutBook, model::GameState, server::ServerProjection};
use serde_json::json;
use std::path::Path;

/// Own actors, a foreign visible actor, source-compatible public rules and actions.
fn fixture() -> (ManagementUi, GameState) {
    let viewer:ServerProjection=serde_json::from_value(json!({"revision":4_i64,"turn":1_u32,"contract":{"version":2_u32},"tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"plains"}],"units":[{"id":"u1","owner_id":"human","type_id":"UNIT_SETTLER","tile_id":"a","health":10_i16,"max_health":10_i16},{"id":"u2","owner_id":"human","type_id":"UNIT_SETTLER","tile_id":"a","health":5_i16,"max_health":10_i16,"asleep":true},{"id":"foreign","owner_id":"other","type_id":"UNIT_HOPLITE","tile_id":"a","health":10_i16,"max_health":10_i16}],"rules":{"units":[{"id":"UNIT_SETTLER","role":"settler","movement":100_i64,"defense":10_i64,"health":10_i64}]},"diplomacy":{"gold":100_i32,"readiness":"war"},"legal_actions":[{"route":"set-readiness","entity_id":"human","target_id":"peace","fields":["readiness"],"payload":{"readiness":"peace"}},{"route":"set-readiness","entity_id":"human","target_id":"alert","fields":["readiness"],"payload":{"readiness":"alert"}},{"route":"disband","entity_id":"u1","payload":{"unit_id":"u1"}},{"route":"disband","entity_id":"u2","payload":{"unit_id":"u2"}}]})).expect("viewer fixture");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
    (
        ManagementUi::new(LayoutBook::load(&root).expect("read-only source layouts"))
            .expect("cache"),
        GameState::from_server_projection(&viewer, 1, 1),
    )
}
#[test]
fn reference_unit_panes_have_bounded_counter_wrapped_advice_and_community_source_geometry() {
    let (mut ui, state) = fixture();
    ui.open(ManagerKind::Unit, &state, None);
    let widgets = ui.widgets(&state);
    let counter = widgets
        .iter()
        .find(|widget| widget.id == "TotalUnits")
        .expect("counter");
    assert_eq!(
        counter.rect.width, 30_u32,
        "implicit value box must not inherit the whole 576-pixel window and erase its value"
    );
    let advice = widgets
        .iter()
        .find(|widget| widget.id == "Advisor.Advice")
        .expect("advice text");
    assert_eq!(
        advice.node.field("wordwrap"),
        Some("true"),
        "original HyperTextBox intrinsically wraps, it is not a centered clipped single line"
    );
    assert_eq!(
        advice.node.field("just"),
        Some("left"),
        "source hypertext starts at the left edge"
    );
    let list = ui.layouts[&ManagerKind::Unit]
        .child("Tabs")
        .and_then(|tabs| tabs.child("Stats"))
        .and_then(|tab| tab.child("TabPanel"))
        .and_then(|panel| panel.child("List"))
        .expect("source list");
    assert_eq!(
        (
            list.integer("xpix"),
            list.integer("ypix"),
            list.integer("widthpix"),
            list.integer("heightpix")
        ),
        (Some(18_i32), Some(35_i32), Some(454_i32), Some(200_i32)),
        "public reference revision's list rectangle, not old retail geometry"
    );
    assert_eq!(
        widgets
            .iter()
            .find(|widget| widget.id == "Advisor.Title")
            .map(|widget| widget.label.as_str()),
        Some("MILITARY ADVISOR"),
        "canonical addon string imports, not a raw key"
    );
}

#[test]
fn unit_type_and_tactical_identity_selections_are_independent_and_receipt_safe() {
    let (mut ui, state) = fixture();
    ui.open(ManagerKind::Unit, &state, None);
    ui.apply_local(
        ui.activate_id("Tabs.Stats.TabPanel.List/UNIT_SETTLER", &state)
            .expect("type row"),
    );
    assert_eq!(
        ui.view
            .as_ref()
            .expect("manager")
            .selections
            .get("unit-type")
            .map(String::as_str),
        Some("UNIT_SETTLER"),
        "statistics select a type, not a guessed instance"
    );
    ui.reconcile(&state);
    assert!(
        ui.view
            .as_ref()
            .expect("manager")
            .selections
            .contains_key("unit-type"),
        "same authoritative type survives a receipt"
    );
    let mut projection = state.viewer.clone().expect("viewer");
    projection.revision = 5;
    projection.units.retain(|unit| unit.owner_id != "human");
    projection.units[0].type_id = "UNIT_SETTLER".to_owned();
    let replaced = GameState::from_server_projection(&projection, 1, 1);
    ui.reconcile(&replaced);
    assert!(
        !ui.view
            .as_ref()
            .expect("manager")
            .selections
            .contains_key("unit-type"),
        "a foreign surviving type cannot retain an own disband target"
    );
}

#[test]
fn unit_stats_use_original_columns_icons_and_known_pinned_values_not_flattened_strings() {
    let (mut ui, state) = fixture();
    ui.open(ManagerKind::Unit, &state, None);
    let widgets = ui.widgets(&state);
    for (column, expected) in [
        ("UnitName", "Settler"),
        ("Count", "2"),
        ("UnitType", "Settler"),
        ("Support", "0"),
        ("A", "0"),
        ("R", "0"),
        ("D", "10"),
        ("M", "100"),
    ] {
        let id = format!("Tabs.Stats.TabPanel.List/UNIT_SETTLER/cell/{column}");
        assert_eq!(
            widgets
                .iter()
                .find(|widget| widget.id == id)
                .map(|widget| widget.label.as_str()),
            Some(expected),
            "source statistics column {column}"
        );
    }
    let icon = widgets
        .iter()
        .find(|widget| widget.id.ends_with("UNIT_SETTLER/cell/UnitIcon"))
        .expect("source small icon");
    assert_eq!(
        icon.node.field("image00"),
        Some("upup002b.rim"),
        "use original SmallIcon, not a stretched portrait"
    );
    let row = widgets
        .iter()
        .find(|widget| widget.id == "Tabs.Stats.TabPanel.List/UNIT_SETTLER")
        .expect("type row");
    assert_eq!(
        icon.rect.x,
        row.rect.x.saturating_add(2),
        "public UnitStatsItem icon is inset two native pixels"
    );
    assert_eq!(
        widgets
            .iter()
            .find(|widget| widget.id == "TotalUnits")
            .map(|widget| widget.label.as_str()),
        Some("2"),
        "only living own units"
    );
    assert_eq!(
        widgets
            .iter()
            .find(|widget| widget.id == "State")
            .map(|widget| widget.label.as_str()),
        Some("War"),
        "current received readiness"
    );
}
#[test]
fn readiness_is_a_quantized_revision_bound_source_slider_not_a_local_policy_mutation() {
    let (mut ui, mut state) = fixture();
    ui.open(ManagerKind::Unit, &state, None);
    let before = state.clone();
    let widgets = ui.widgets(&state);
    let slider = widgets
        .iter()
        .find(|widget| widget.id == "Slider")
        .expect("source slider");
    let thumb = widgets
        .iter()
        .find(|widget| widget.id == "Slider/thumb")
        .expect("source atomic thumb");
    assert_eq!(
        thumb.node.integer("value-current"),
        Some(2_i32),
        "war is original index two"
    );
    assert_eq!(
        thumb
            .rect
            .x
            .saturating_add(i32::try_from(thumb.rect.width).expect("width")),
        slider
            .rect
            .x
            .saturating_add(i32::try_from(slider.rect.width).expect("width")),
        "native war thumb lands at right endpoint"
    );
    let intent = ui
        .activate_id("Slider/value/0", &state)
        .expect("authorized peace step");
    assert!(
        matches!(intent, ManagerIntent::SetValue(_, 0_i32, 4_i64)),
        "explicit captured revision"
    );
    let request = ui
        .value_command("Slider", 0, &state)
        .expect("exact readiness command");
    assert_eq!(request.route, "set-readiness", "registered source callback");
    assert_eq!(
        request.payload,
        json!({"readiness":"peace"}),
        "no invented actor or companion rates"
    );
    assert!(
        ui.value_command("Slider", 3, &state).is_none(),
        "no fourth readiness"
    );
    assert_eq!(
        state, before,
        "preview and command construction are view-only"
    );
    state.viewer.as_mut().expect("viewer").revision = 5;
    assert!(
        !request.valid_for(&state),
        "stale source gesture cannot silently rebase"
    );
}
#[test]
fn unit_tactical_rows_and_disband_choose_the_exact_authorized_unit_not_the_first_type_member() {
    let (mut ui, state) = fixture();
    ui.open(ManagerKind::Unit, &state, None);
    ui.apply_local(ManagerIntent::Select(
        "tab:Tabs".to_owned(),
        "Tactical".to_owned(),
    ));
    let widgets = ui.widgets(&state);
    assert!(
        widgets
            .iter()
            .any(|widget| widget.id.ends_with("List/u2/cell/Order") && widget.label == "Sentinel"),
        "received sleeping state is an actual tactical order"
    );
    assert!(
        ui.activate_id("DisbandButton", &state).is_none(),
        "no implicit first target"
    );
    let row = widgets
        .iter()
        .find(|widget| widget.id.ends_with("List/u2"))
        .expect("exact second actor");
    ui.apply_local(ui.activate_id(&row.id, &state).expect("row selection"));
    let dialog = ui
        .activate_id("DisbandButton", &state)
        .expect("explicit confirmation");
    assert!(
        matches!(dialog,ManagerIntent::Dialog(form) if form.choices.len() == 1 && form.choices[0].command.payload == json!({"unit_id":"u2"})),
        "source query carries only the chosen canonical unit"
    );
}
