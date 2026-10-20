//! Source city popup journeys via the production dispatcher, no filesystem/network.

use super::{apply, consume};
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::model::GameState;
use crate::gameplay::runtime::MovementInputState;
use crate::gameplay::server::ServerProjection;
use crate::gameplay::ui_model::InGameUiModel;
use crate::gameplay::ui_model::management::{ManagementUi, ManagerIntent, ManagerKind};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use serde_json::json;
use std::path::Path;

/// Accurate viewer-owned identities; the production selection path stays offline.
fn fixture() -> (InGameUiModel, GameState) {
    let projection:ServerProjection=serde_json::from_value(json!({"revision":9_i64,"turn":3_u32,"tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"TERRAIN_PLAINS"},{"id":"b","x":2_u16,"y":1_u16,"terrain":"TERRAIN_PLAINS"}],"cities":[{"id":"first","owner_id":"human","tile_id":"a","population":1_u32},{"id":"second","owner_id":"human","tile_id":"b","population":1_u32},{"id":"foreign","owner_id":"rival","tile_id":"a","population":1_u32}]})).expect("public fixture");
    let mut state = GameState::from_server_projection(&projection, 2_u8, 1_u64);
    state.popups.clear();
    let mut ui = InGameUiModel {
        management: ManagementUi::new(
            LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2")).expect("source"),
        )
        .expect("templates"),
        ..InGameUiModel::default()
    };
    ui.management
        .open(ManagerKind::Build, &state, Some("first"));
    (ui, state)
}
/// A normal key event, without coordinate-only activation or a real clipboard.
fn key(code: Keycode) -> Event {
    Event::KeyDown {
        timestamp: 0_u32,
        window_id: 0_u32,
        keycode: Some(code),
        scancode: None,
        keymod: Mod::NOMOD,
        repeat: false,
    }
}
/// Open through the exact semantic source button shared with pointer clicks.
fn open(ui: &mut InGameUiModel, state: &mut GameState) {
    let intent = ui
        .management
        .activate_id("SingleCityChooser.Pulldown.autobutton", state)
        .expect("open dropdown");
    apply(intent, state, ui, None);
}

#[test]
fn escape_dismisses_city_popup_before_closing_its_manager_and_never_ends_the_turn() {
    let (mut ui, mut state) = fixture();
    let before = state.clone();
    open(&mut ui, &mut state);
    assert!(
        consume(
            &key(Keycode::ESCAPE),
            &mut state,
            &mut ui,
            &mut MovementInputState::default(),
            None
        ),
        "dropdown owns Escape"
    );
    assert!(
        ui.management.view.is_some(),
        "first Escape dismisses only the detached popup"
    );
    assert!(
        ui.management.dropdown_choices(&state).is_empty(),
        "popup dismissed"
    );
    assert!(
        consume(
            &key(Keycode::ESCAPE),
            &mut state,
            &mut ui,
            &mut MovementInputState::default(),
            None
        ),
        "manager owns second Escape"
    );
    assert!(ui.management.view.is_none(), "second Escape closes manager");
    assert_eq!(state, before, "no game command or world mutation");
}

#[test]
fn popup_pointer_selection_agrees_with_semantic_ids_and_outside_click_only_dismisses_popup() {
    let (mut ui, mut state) = fixture();
    let before = state.clone();
    open(&mut ui, &mut state);
    let row = ui
        .management
        .widgets(&state)
        .into_iter()
        .find(|widget| widget.id == "SingleCityChooser.Pulldown.window.listbox/second")
        .expect("owned row");
    let pointer = Event::MouseButtonDown {
        timestamp: 0_u32,
        window_id: 0_u32,
        which: 0_u32,
        mouse_btn: sdl2::mouse::MouseButton::Left,
        clicks: 1_u8,
        x: row.rect.x.saturating_add(1_i32),
        y: row.rect.y.saturating_add(1_i32),
    };
    assert!(
        consume(
            &pointer,
            &mut state,
            &mut ui,
            &mut MovementInputState::default(),
            None
        ),
        "popup owns source row pointer"
    );
    assert_eq!(
        ui.management
            .view
            .as_ref()
            .expect("view")
            .city_id
            .as_deref(),
        Some("second"),
        "pointer and semantic choice use same exact city"
    );
    open(&mut ui, &mut state);
    let behind = ui
        .management
        .widgets(&state)
        .into_iter()
        .find(|widget| widget.id == "SingleCityChooser.Previous")
        .expect("underlying source control");
    let outside = Event::MouseButtonDown {
        timestamp: 0_u32,
        window_id: 0_u32,
        which: 0_u32,
        mouse_btn: sdl2::mouse::MouseButton::Left,
        clicks: 1_u8,
        x: behind.rect.x.saturating_add(1_i32),
        y: behind.rect.y.saturating_add(1_i32),
    };
    assert!(
        consume(
            &outside,
            &mut state,
            &mut ui,
            &mut MovementInputState::default(),
            None
        ),
        "outside click is consumed while dismissing popup"
    );
    assert!(
        ui.management.dropdown_choices(&state).is_empty() && ui.management.view.is_some(),
        "outside click cannot close parent or pass into the world"
    );
    assert_eq!(
        ui.management
            .view
            .as_ref()
            .expect("view")
            .city_id
            .as_deref(),
        Some("second"),
        "dismissing a popup must not click the underlying Previous City control"
    );
    assert_eq!(state, before, "pointer navigation is presentation only");
}

#[test]
fn removed_highlight_is_rejected_instead_of_silently_selecting_the_first_remaining_city() {
    let (mut ui, mut state) = fixture();
    open(&mut ui, &mut state);
    assert!(
        consume(
            &key(Keycode::END),
            &mut state,
            &mut ui,
            &mut MovementInputState::default(),
            None
        ),
        "highlight second city"
    );
    state
        .viewer
        .as_mut()
        .expect("viewer")
        .cities
        .iter_mut()
        .find(|city| city.id == "second")
        .expect("second")
        .stale = true;
    let before = state.clone();
    assert!(
        consume(
            &key(Keycode::RETURN),
            &mut state,
            &mut ui,
            &mut MovementInputState::default(),
            None
        ),
        "stale acceptance owned by popup"
    );
    assert!(
        ui.status_text.contains("no longer"),
        "receipt-invalidated highlight must be reported, not silently replaced"
    );
    assert_eq!(
        ui.management
            .view
            .as_ref()
            .expect("view")
            .city_id
            .as_deref(),
        Some("first"),
        "stale highlighted city cannot redirect selection"
    );
    assert!(
        ui.management.dropdown_choices(&state).is_empty(),
        "stale popup dismissed"
    );
    assert_eq!(
        state, before,
        "rejecting stale selection never mutates authoritative facts"
    );
}

#[test]
fn many_city_popup_scrolls_to_keyboard_target_and_cancels_on_focus_loss() {
    let (mut ui, mut state) = fixture();
    let mut projection = state.viewer.clone().expect("viewer");
    for index in 2_u16..22_u16 {
        projection.tiles.push(serde_json::from_value(json!({"id":format!("tile-{index}"),"x":index,"y":1_u16,"terrain":"TERRAIN_PLAINS"})).expect("tile"));
        projection.cities.push(serde_json::from_value(json!({"id":format!("city-{index}"),"owner_id":"human","tile_id":format!("tile-{index}"),"population":1_u32})).expect("city"));
    }
    state = GameState::from_server_projection(&projection, 2_u8, 1_u64);
    state.popups.clear();
    let before = state.clone();
    open(&mut ui, &mut state);
    assert!(
        consume(
            &key(Keycode::END),
            &mut state,
            &mut ui,
            &mut MovementInputState::default(),
            None
        ),
        "End belongs to popup"
    );
    let view = ui.management.view.as_ref().expect("view");
    assert_eq!(
        view.selections
            .get("dropdown-highlight")
            .map(String::as_str),
        Some("city-21"),
        "exact last owned city highlighted"
    );
    assert_eq!(
        view.scroll.get("SingleCityChooser.Pulldown.window.listbox"),
        Some(&7_usize),
        "22 cities minus fifteen complete source rows"
    );
    assert!(
        ui.management
            .widgets(&state)
            .iter()
            .any(|widget| widget.id == "SingleCityChooser.Pulldown.window.listbox/city-21"),
        "keyboard target is actually visible"
    );
    let focus = Event::Window {
        timestamp: 0_u32,
        window_id: 0_u32,
        win_event: sdl2::event::WindowEvent::FocusLost,
    };
    assert!(
        !consume(
            &focus,
            &mut state,
            &mut ui,
            &mut MovementInputState::default(),
            None
        ),
        "native window event continues to outer focus cleanup"
    );
    assert!(
        ui.management.dropdown_choices(&state).is_empty(),
        "focus loss dismisses detached popup"
    );
    assert_eq!(state, before, "popup scrolling cannot mutate game state");
}

#[test]
fn popup_keyboard_selection_and_stale_identity_checks_use_current_owned_cities() {
    let (mut ui, mut state) = fixture();
    let before = state.clone();
    open(&mut ui, &mut state);
    assert!(
        consume(
            &key(Keycode::END),
            &mut state,
            &mut ui,
            &mut MovementInputState::default(),
            None
        ),
        "popup navigation"
    );
    assert!(
        consume(
            &key(Keycode::RETURN),
            &mut state,
            &mut ui,
            &mut MovementInputState::default(),
            None
        ),
        "popup acceptance"
    );
    assert_eq!(
        ui.management
            .view
            .as_ref()
            .expect("view")
            .city_id
            .as_deref(),
        Some("second"),
        "keyboard picks exact last owned identity, not foreign city"
    );
    apply(
        ManagerIntent::Select("city".to_owned(), "foreign".to_owned()),
        &mut state,
        &mut ui,
        None,
    );
    assert_eq!(
        ui.management
            .view
            .as_ref()
            .expect("view")
            .city_id
            .as_deref(),
        Some("second"),
        "foreign or stale semantic requests are rejected at dispatch"
    );
    assert_eq!(
        state, before,
        "view navigation leaves the entire authoritative snapshot unchanged"
    );
}
