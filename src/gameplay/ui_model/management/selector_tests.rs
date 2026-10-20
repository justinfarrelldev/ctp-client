//! Source city controls use exact current owned identities, never guessed city order.

use super::{ManagementUi, ManagerIntent, ManagerKind};
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::model::GameState;
use crate::gameplay::server::ServerProjection;
use serde_json::json;
use std::path::Path;

/// Two owned cities and a visible foreign city; no files/network are changed.
fn fixture() -> (ManagementUi, GameState) {
    let projection:ServerProjection=serde_json::from_value(json!({"revision":9_i64,"turn":3_u32,"tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"TERRAIN_PLAINS"},{"id":"b","x":2_u16,"y":1_u16,"terrain":"TERRAIN_PLAINS"}],"cities":[{"id":"first","owner_id":"human","tile_id":"a","population":1_u32},{"id":"second","owner_id":"human","tile_id":"b","population":1_u32},{"id":"foreign","owner_id":"rival","tile_id":"a","population":1_u32}]})).expect("viewer fixture");
    let state = GameState::from_server_projection(&projection, 2_u8, 1_u64);
    let mut ui = ManagementUi::new(
        LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2")).expect("source"),
    )
    .expect("layouts");
    ui.open(ManagerKind::Build, &state, Some("first"));
    (ui, state)
}

#[test]
fn collapsed_city_selector_paints_one_caption_not_its_hidden_popup_or_duplicate_parent_labels() {
    let (ui, state) = fixture();
    let widgets = ui.widgets(&state);
    let captions = widgets
        .iter()
        .filter(|widget| widget.id.starts_with("SingleCityChooser") && widget.label == "first")
        .count();
    assert_eq!(
        captions, 1_usize,
        "ctp2_DropDown draws its selected item once, after the button"
    );
    assert!(
        !widgets
            .iter()
            .any(|widget| widget.id.starts_with("SingleCityChooser.Pulldown.window")),
        "popup window is detached and hidden until explicitly opened"
    );
    let button = widgets
        .iter()
        .find(|widget| widget.id == "SingleCityChooser.Pulldown.autobutton")
        .expect("source dropdown button");
    assert_eq!(
        button.rect.width, 180_u32,
        "RepositionButton uses actual dropdown width"
    );
    assert!(button.intent.is_some(), "owned-city chooser is functional");
}

#[test]
fn source_city_arrows_wrap_exact_owned_ids_and_drop_stale_queue_descriptions() {
    let (mut ui, state) = fixture();
    ui.apply_local(ManagerIntent::Select(
        "queue".to_owned(),
        "UNIT_HOPLITE".to_owned(),
    ));
    ui.apply_local(ManagerIntent::Select(
        "buildable".to_owned(),
        "UNIT_SETTLER".to_owned(),
    ));
    let next = ui
        .activate_id("SingleCityChooser.Next", &state)
        .expect("source next callback");
    assert_eq!(
        next,
        ManagerIntent::Select("city".to_owned(), "second".to_owned()),
        "visible foreign city never enters owned-city cycling"
    );
    ui.apply_local(next);
    let view = ui.view.as_ref().expect("view");
    assert_eq!(view.city_id.as_deref(), Some("second"), "exact chosen city");
    assert!(
        !view.selections.contains_key("queue") && !view.selections.contains_key("description"),
        "old city queue/description must not authorize a different city's payload"
    );
    assert_eq!(
        ui.activate_id("SingleCityChooser.Next", &state),
        Some(ManagerIntent::Select("city".to_owned(), "first".to_owned())),
        "next wraps"
    );
    assert_eq!(
        ui.activate_id("SingleCityChooser.Previous", &state),
        Some(ManagerIntent::Select("city".to_owned(), "first".to_owned())),
        "previous follows source order"
    );
}

#[test]
fn city_manager_uses_its_actual_citylist_source_path_not_a_guessed_cityspinner_path() {
    let (mut ui, state) = fixture();
    ui.open(ManagerKind::City, &state, Some("first"));
    assert_eq!(
        ui.activate_id("CityList.Next", &state),
        Some(ManagerIntent::Select(
            "city".to_owned(),
            "second".to_owned()
        )),
        "CityWindow's original city picker callbacks are functional"
    );
    let open = ui
        .activate_id("CityList.Pulldown.autobutton", &state)
        .expect("actual city picker");
    ui.apply_local(open);
    assert!(
        ui.widgets(&state)
            .iter()
            .any(|widget| widget.id == "CityList.Pulldown.window.listbox/second"),
        "exact CityWindow popup path"
    );
}

#[test]
fn city_dropdown_opens_only_current_owned_rows_and_selects_by_identity() {
    let (mut ui, state) = fixture();
    let open = ui
        .activate_id("SingleCityChooser.Pulldown.autobutton", &state)
        .expect("open chooser");
    ui.apply_local(open);
    let widgets = ui.widgets(&state);
    let choice = widgets
        .iter()
        .find(|widget| widget.id == "SingleCityChooser.Pulldown.window.listbox/second")
        .expect("source city item");
    assert_eq!(
        choice.intent,
        Some(ManagerIntent::Select(
            "city".to_owned(),
            "second".to_owned()
        )),
        "exact canonical choice"
    );
    assert!(
        !widgets.iter().any(|widget| widget.id.ends_with("/foreign")),
        "foreign city excluded"
    );
    ui.apply_local(choice.intent.clone().expect("select"));
    assert_eq!(
        ui.view.as_ref().expect("view").city_id.as_deref(),
        Some("second"),
        "selection bound to exact row"
    );
    assert!(
        !ui.widgets(&state)
            .iter()
            .any(|widget| widget.id.starts_with("SingleCityChooser.Pulldown.window")),
        "selection dismisses popup"
    );
}
