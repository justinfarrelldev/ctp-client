//! Three-tab reference revision is a resolved-property adaptation, not copied art.
use super::{ManagementUi, ManagerKind};
use crate::gameplay::{assets::layout::LayoutBook, model::GameState, server::ServerProjection};
use serde_json::{Value, json};
use std::path::Path;
/// In-memory complete own no-city view proves empty source sums without guesses.
fn empty() -> (ManagementUi, GameState) {
    let viewer:ServerProjection=serde_json::from_value(json!({"revision":3_i64,"turn":1_u32,"contract":{"version":2_u32},"tiles":[],"cities":[],"units":[],"diplomacy":{"gold":100_i32}})).expect("empty complete v2 projection");
    let state = GameState::from_server_projection(&viewer, 1, 1);
    let mut ui = ManagementUi::new(
        LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"))
            .expect("installed layouts"),
    )
    .expect("cached source");
    ui.open(ManagerKind::Trade, &state, None);
    (ui, state)
}
#[test]
fn source_reference_trade_has_three_native_tabs_wider_root_and_six_advisor_rows() {
    let (ui, state) = empty();
    let widgets = ui.widgets(&state);
    let root = widgets
        .iter()
        .find(|widget| widget.id.is_empty())
        .expect("window");
    assert_eq!(
        (root.rect.x, root.rect.width),
        (209_i32, 591_u32),
        "source DoneInstantiating clamps wider main window while advisor clips at left"
    );
    for (tab, label) in [
        ("Market", "Market"),
        ("Summary", "Export"),
        ("Import", "Import"),
    ] {
        let button = widgets
            .iter()
            .find(|widget| widget.id == format!("TradeTabs.{tab}.TabButton"))
            .expect("all three original tab captions");
        assert_eq!(button.label, label, "source three-tab caption");
        assert!(
            button.rect.width < 240,
            "three-tab intrinsic art, not old two-tab width"
        );
    }
    for (name, y) in [
        ("Available", 229_i32),
        ("InUse", 242_i32),
        ("Routes", 255_i32),
        ("Profit", 268_i32),
        ("Transit", 281_i32),
        ("Piracy", 294_i32),
    ] {
        let value = widgets
            .iter()
            .find(|widget| widget.id == format!("Advisor.{name}"))
            .expect("source advisor value");
        assert_eq!(
            value.rect.y,
            root.rect.y.saturating_add(y),
            "source 13-pixel accounting rows"
        );
        assert_eq!(
            value.label, "0",
            "empty current owned-city sum is provably zero, not guessed private accounting"
        );
    }
    let advice = widgets
        .iter()
        .find(|widget| widget.id == "Advisor.Advice")
        .expect("source advice text");
    assert_eq!(
        advice.label,
        ui.book.label("MAXIMUM_TRADE_EFFICIENCY"),
        "no owned city means there cannot be any source opportunity; the original branch is fully known"
    );
}
#[test]
fn source_import_is_an_independent_received_destination_list_not_export_alias() {
    let (mut ui, _empty) = empty();
    let viewer:ServerProjection=serde_json::from_value(json!({"revision":4_i64,"turn":1_u32,"contract":{"version":2_u32},"tiles":[{"id":"t","x":1_u16,"y":1_u16,"terrain":"plains"},{"id":"f","x":2_u16,"y":1_u16,"terrain":"plains"}],"cities":[{"id":"own","owner_id":"human","tile_id":"t","name":"Athens"},{"id":"foreign","owner_id":"other","tile_id":"f","name":"Known supplier"}],"trade_routes":[{"id":"in","source_city_id":"foreign","destination_city_id":"own","good_id":"GOOD_PLAINS_ONE","gold":7_i64,"transport_cost":3_i64}]})).expect("received import facts");
    let mut state = GameState::from_server_projection(&viewer, 1, 1);
    ui.open(ManagerKind::Trade, &state, None);
    ui.apply_local(
        ui.activate_id("TradeTabs.Import.TabButton", &state)
            .expect("original Import navigation"),
    );
    let widgets = ui.widgets(&state);
    for (name, value) in [
        ("City", "Known supplier"),
        ("ToCity", "Athens"),
        ("Caravans", "—"),
    ] {
        assert!(
            widgets.iter().any(|widget| widget.id
                == format!("TradeTabs.Import.TabPanel.List/in/cell/{name}")
                && widget.label == value),
            "import {name} uses received endpoints; actual RuntimeRules does not expose resource bonuses, and transport cost is not that bonus"
        );
    }
    let current = state.viewer.as_mut().expect("viewer");
    current.interface.trade_routes[0]["good_id"] = Value::Null;
    assert!(
        ui.widgets(&state).iter().any(|widget| widget.id
            == "TradeTabs.Import.TabPanel.List/in/cell/Caravans"
            && widget.label == "—"),
        "missing good identity still cannot turn a route transport cost into a resource bonus"
    );
}
