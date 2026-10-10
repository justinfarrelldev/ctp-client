//! Tests for the gameplay SDL runtime and pure hit testing.

use std::path::PathBuf;

use sdl2::event::Event;

use super::{
    Camera, HeadlessSdlEnvironment, click_in_end_turn_button, handle_left_click,
    handle_right_click, run_headless_smoke_for_test, run_sdl_game_on_existing_window,
    top_menu_click_action,
};
use crate::gameplay::ui_model::{InGameAction, InGameUiModel, MenuCategory};
use crate::gameplay::{GameplayOutcome, SinglePlayerLaunchConfig};

#[test]
fn dropdown_hit_testing_preserves_disabled_row_indexes() {
    let mut ui = InGameUiModel {
        open_menu: Some(MenuCategory::Options),
        ..InGameUiModel::default()
    };
    let save_row = ui
        .dropdown_row_rect(MenuCategory::Options, 7)
        .expect("Save Game row should be present");
    let x = save_row.x.saturating_add(4);
    let y = save_row.y.saturating_add(4);

    let (consumed, action) = top_menu_click_action(x, y, &mut ui);

    assert!(
        consumed,
        "disabled Save Game row should still consume the click"
    );
    assert_eq!(
        action, None,
        "disabled Save Game must not fall through to Restart/New Game because enabled rows were filtered"
    );
    assert_eq!(
        ui.open_menu, None,
        "selecting any row should close the menu"
    );
}

#[test]
fn open_dropdown_consumes_outside_end_turn_click_without_action() {
    let mut ui = InGameUiModel {
        open_menu: Some(MenuCategory::Options),
        ..InGameUiModel::default()
    };

    let (consumed, action) = top_menu_click_action(960, 648, &mut ui);

    assert!(
        consumed,
        "a weakly-modal CTP2 dropdown should consume outside clicks instead of activating end-turn/terrain"
    );
    assert_eq!(
        action, None,
        "outside click should only cancel the dropdown"
    );
    assert_eq!(
        ui.open_menu, None,
        "outside click should close the dropdown"
    );
}

#[test]
fn top_menu_buttons_open_original_categories() {
    let mut ui = InGameUiModel::default();

    let (consumed, action) = top_menu_click_action(405, 12, &mut ui);

    assert!(
        consumed,
        "clicking the Options menu button should be consumed"
    );
    assert_eq!(action, None, "opening a menu should not fire a row action");
    assert_eq!(
        ui.open_menu,
        Some(MenuCategory::Options),
        "Options button should use original 80px LDL positioning"
    );
}

#[test]
fn enabled_dropdown_row_returns_action_from_exact_row() {
    let mut ui = InGameUiModel {
        open_menu: Some(MenuCategory::Options),
        ..InGameUiModel::default()
    };
    let quit_index = ui
        .items_for(MenuCategory::Options)
        .iter()
        .position(|item| item.label == "Quit")
        .expect("Quit row should be present");
    let quit_row = ui
        .dropdown_row_rect(MenuCategory::Options, quit_index)
        .expect("Quit row should have a rectangle");

    let (consumed, action) = top_menu_click_action(
        quit_row.x.saturating_add(4),
        quit_row.y.saturating_add(4),
        &mut ui,
    );

    assert!(consumed, "enabled dropdown row should consume the click");
    assert_eq!(
        action,
        Some(InGameAction::Quit),
        "Quit row should return Quit"
    );
    assert_eq!(
        ui.open_menu, None,
        "row selection should close the dropdown"
    );
}

#[test]
fn original_end_turn_button_geometry_is_on_control_panel() {
    assert!(
        click_in_end_turn_button(570, 650),
        "original controlpanel.ldl TurnButton is at panel-relative 563,60"
    );
    assert!(
        !click_in_end_turn_button(960, 648),
        "old placeholder far-right end-turn hitbox must not remain active"
    );
}

#[test]
fn lower_control_panel_click_does_not_move_selected_unit() {
    let mut state = crate::gameplay::model::GameState::new(24, 18, 1, 77);
    state.popups.clear();
    let start = state
        .selected_unit()
        .expect("selected unit should exist")
        .position;
    let mut ui = InGameUiModel::default();
    let mut camera = Camera::centered_on_selected(&state);

    let outcome = handle_left_click(120, 610, &mut state, &mut ui, &mut camera, None);

    assert_eq!(outcome, None, "panel click should not close gameplay");
    assert_eq!(
        state
            .selected_unit()
            .expect("selected unit should still exist")
            .position,
        start,
        "clicking the lower UI should not issue a terrain move"
    );
}

#[test]
fn side_panel_click_does_not_move_selected_unit() {
    let mut state = crate::gameplay::model::GameState::new(24, 18, 1, 78);
    state.popups.clear();
    let start = state
        .selected_unit()
        .expect("selected unit should exist")
        .position;
    let mut ui = InGameUiModel::default();
    let mut camera = Camera::centered_on_selected(&state);

    let outcome = handle_left_click(900, 100, &mut state, &mut ui, &mut camera, None);

    assert_eq!(outcome, None, "side panel click should not close gameplay");
    assert_eq!(
        state
            .selected_unit()
            .expect("selected unit should still exist")
            .position,
        start,
        "clicking the side/radar UI should not issue a terrain move"
    );
}

#[test]
fn gameplay_popup_consumes_end_turn_click_without_advancing_turn() {
    let mut state = crate::gameplay::model::GameState::new(24, 18, 1, 79);
    let mut ui = InGameUiModel::default();
    let mut camera = Camera::centered_on_selected(&state);

    let outcome = handle_left_click(570, 650, &mut state, &mut ui, &mut camera, None);

    assert_eq!(outcome, None, "popup click should only dismiss the popup");
    assert_eq!(state.turn, 1, "popup dismissal must not advance the turn");
    assert!(state.popups.is_empty(), "popup should be dismissed");
}

#[test]
fn right_click_with_open_dropdown_cancels_only_dropdown() {
    let mut state = crate::gameplay::model::GameState::new(24, 18, 1, 80);
    state.popups.clear();
    let mut ui = InGameUiModel {
        open_menu: Some(MenuCategory::Options),
        ..InGameUiModel::default()
    };
    let mut camera = Camera::centered_on_selected(&state);

    let outcome = handle_right_click(570, 650, &mut state, &mut ui, &mut camera, None);

    assert_eq!(
        outcome, None,
        "dropdown cancellation should not close gameplay"
    );
    assert_eq!(
        ui.open_menu, None,
        "right-click outside should close the dropdown"
    );
    assert_eq!(
        state.turn, 1,
        "dropdown cancellation must not advance the turn"
    );
}

#[test]
fn right_click_popup_consumes_without_context_action() {
    let mut state = crate::gameplay::model::GameState::new(24, 18, 1, 81);
    let mut ui = InGameUiModel::default();
    let mut camera = Camera::centered_on_selected(&state);

    let outcome = handle_right_click(120, 610, &mut state, &mut ui, &mut camera, None);

    assert_eq!(
        outcome, None,
        "popup right-click should only dismiss the popup"
    );
    assert!(state.popups.is_empty(), "popup should be dismissed");
    assert!(
        state.city_build_menu.is_none(),
        "popup right-click must not leak into context-menu city actions"
    );
}

#[test]
fn right_click_lower_panel_does_not_open_context_actions() {
    let mut state = crate::gameplay::model::GameState::new(24, 18, 1, 82);
    state.popups.clear();
    let mut ui = InGameUiModel::default();
    let mut camera = Camera::centered_on_selected(&state);

    let outcome = handle_right_click(120, 610, &mut state, &mut ui, &mut camera, None);

    assert_eq!(outcome, None, "panel right-click should not close gameplay");
    assert!(
        state.city_build_menu.is_none(),
        "right-clicking the lower UI must not open map context actions"
    );
}

#[test]
fn headless_gameplay_smoke_opens_and_closes() {
    let outcome = run_headless_smoke_for_test(&SinglePlayerLaunchConfig {
        installation_root: PathBuf::from("/tmp/ctp2"),
        empire_count: 2,
        leader_name: "Tester".to_owned(),
        seed: 42,
        server: crate::gameplay::server::ServerConfig::disabled_for_test(),
    })
    .expect("headless gameplay should run");

    assert_eq!(outcome, GameplayOutcome::Closed);
}

#[test]
fn headless_gameplay_reuses_existing_sdl_window() {
    let guard =
        crate::sdl_test_lock::lock_sdl_for_test().expect("SDL test lock should be available");
    let environment = HeadlessSdlEnvironment::apply();
    {
        let sdl = sdl2::init().expect("SDL should initialize with dummy driver");
        let video = sdl.video().expect("video subsystem should initialize");
        let window = video
            .window("menu-owned", 800, 600)
            .position_centered()
            .build()
            .expect("dummy window should be created");
        let mut canvas = window
            .into_canvas()
            .software()
            .build()
            .expect("dummy canvas should be created");
        let existing_window_id = canvas.window().id();
        let events = sdl.event().expect("event subsystem should initialize");
        events
            .push_event(Event::Quit { timestamp: 0 })
            .expect("quit event should be queued");
        let mut event_pump = sdl.event_pump().expect("event pump should initialize");

        let outcome = run_sdl_game_on_existing_window(
            &SinglePlayerLaunchConfig {
                installation_root: PathBuf::from("/tmp/ctp2"),
                empire_count: 2,
                leader_name: "Tester".to_owned(),
                seed: 42,
                server: crate::gameplay::server::ServerConfig::disabled_for_test(),
            },
            &mut canvas,
            &mut event_pump,
        )
        .expect("gameplay should run on a menu-owned window");

        assert_eq!(outcome, GameplayOutcome::Closed);
        assert_eq!(
            canvas.window().id(),
            existing_window_id,
            "gameplay must reuse the menu-owned SDL window"
        );
    }
    environment.restore();
    drop(guard);
}
