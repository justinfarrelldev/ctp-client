//! Tests for the gameplay SDL runtime and pure hit testing.

use alloc::sync::Arc;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};

use super::{
    Camera, HeadlessSdlEnvironment, MovementInputState, RuntimeAuthority, click_in_end_turn_button,
    handle_event, handle_left_click, handle_left_release, handle_mouse_motion, handle_right_click,
    run_headless_smoke_for_test, run_sdl_game_on_existing_window, server_tile_ids_for_path,
    top_menu_click_action,
};
use crate::gameplay::model::{FogState, MapPoint, TerrainKind};
use crate::gameplay::server::{
    ServerClient, ServerConfig, ServerGameplaySession, ServerProjection, ServerTileProjection,
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
    let mut movement = MovementInputState::default();

    let outcome = handle_left_click(
        120,
        610,
        &mut state,
        &mut ui,
        &mut camera,
        &mut movement,
        None,
    );

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
    let mut movement = MovementInputState::default();

    let outcome = handle_left_click(
        900,
        100,
        &mut state,
        &mut ui,
        &mut camera,
        &mut movement,
        None,
    );

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
    let mut movement = MovementInputState::default();

    let outcome = handle_left_click(
        570,
        650,
        &mut state,
        &mut ui,
        &mut camera,
        &mut movement,
        None,
    );

    assert_eq!(outcome, None, "popup click should only dismiss the popup");
    assert_eq!(state.turn, 1, "popup dismissal must not advance the turn");
    assert!(state.popups.is_empty(), "popup should be dismissed");
}

#[test]
fn keyboard_shortcut_with_popup_dismisses_without_command_action() {
    let mut state = crate::gameplay::model::GameState::new(24, 18, 1, 83);
    let selected_id = state
        .selected_unit()
        .expect("selected unit should exist")
        .id;
    let mut ui = InGameUiModel::default();
    let mut camera = Camera::centered_on_selected(&state);
    let mut movement = MovementInputState::default();

    let outcome = handle_event(
        &Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: Some(Keycode::B),
            scancode: None,
            keymod: Mod::NOMOD,
            repeat: false,
        },
        &mut state,
        &mut ui,
        &mut camera,
        &mut movement,
        None,
    );

    assert_eq!(outcome, None, "popup key should not close gameplay");
    assert!(
        state.popups.is_empty(),
        "first key should only dismiss popup"
    );
    assert!(
        state
            .units
            .iter()
            .any(|unit| unit.id == selected_id && unit.alive),
        "B must not found a city while a modal popup is visible"
    );
    assert!(state.cities.is_empty(), "no city should be founded");
}

#[test]
fn plain_left_click_on_map_tile_does_not_move_unit() {
    let mut state = crate::gameplay::model::GameState::new(24, 18, 1, 84);
    state.popups.clear();
    let start = state
        .selected_unit()
        .expect("selected unit should exist")
        .position;
    let target = empty_passable_tile(&state, start);
    let mut ui = InGameUiModel::default();
    let mut camera = Camera::centered_on_selected(&state);
    let mut movement = MovementInputState::default();
    let (x, y) = camera.tile_to_screen(target);

    let down = handle_left_click(x, y, &mut state, &mut ui, &mut camera, &mut movement, None);
    let up = handle_left_release(x, y, &mut state, &ui, &mut camera, &mut movement, None);

    assert_eq!(
        down, None,
        "plain terrain mouse-down should not close gameplay"
    );
    assert_eq!(up, None, "plain terrain mouse-up should not close gameplay");
    assert_eq!(
        state
            .selected_unit()
            .expect("selected unit should still exist")
            .position,
        start,
        "plain left-clicking terrain should not issue a move order in CTP2"
    );
    assert_eq!(
        movement.active_unit(),
        None,
        "terrain clicks should not activate a unit"
    );
}

#[test]
fn plain_left_click_with_authority_does_not_submit_move_request() {
    let (port, request_seen, handle) = spawn_authoritative_request_probe();
    let mut state = crate::gameplay::model::GameState::new(24, 18, 1, 89);
    state.popups.clear();
    let start = state
        .selected_unit()
        .expect("selected unit should exist")
        .position;
    let target = empty_passable_tile(&state, start);
    let mut ui = InGameUiModel::default();
    let mut camera = Camera::centered_on_selected(&state);
    let mut movement = MovementInputState::default();
    let client = ServerClient::from_config(&ServerConfig {
        base_url: format!("http://127.0.0.1:{port}"),
        required: true,
    })
    .expect("test server URL should parse");
    let mut authority = RuntimeAuthority {
        client,
        session: ServerGameplaySession {
            game_id: 44,
            session_id: 11,
            account_id: 7,
            projection: Some(ServerProjection::default()),
        },
        empire_count: 1,
        seed: 89,
    };
    let (x, y) = camera.tile_to_screen(target);

    let down = handle_left_click(
        x,
        y,
        &mut state,
        &mut ui,
        &mut camera,
        &mut movement,
        Some(&mut authority),
    );
    let up = handle_left_release(
        x,
        y,
        &mut state,
        &ui,
        &mut camera,
        &mut movement,
        Some(&mut authority),
    );

    handle.join().expect("listener worker should finish");
    assert_eq!(down, None, "plain terrain down should not close gameplay");
    assert_eq!(up, None, "plain terrain up should not close gameplay");
    assert!(
        !request_seen.load(Ordering::SeqCst),
        "plain terrain left-clicks must not submit authoritative movement"
    );
}

/// Spawns a short-lived fake command listener that records unexpected requests.
fn spawn_authoritative_request_probe() -> (u16, Arc<AtomicBool>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("test listener should bind");
    listener
        .set_nonblocking(true)
        .expect("listener should support nonblocking accept");
    let port = listener
        .local_addr()
        .expect("listener should expose address")
        .port();
    let request_seen = Arc::new(AtomicBool::new(false));
    let worker_seen = Arc::clone(&request_seen);
    let handle =
        thread::spawn(move || watch_for_unexpected_authoritative_request(listener, worker_seen));
    (port, request_seen, handle)
}

/// Watches for an unexpected command request without blocking successful no-request tests.
fn watch_for_unexpected_authoritative_request(listener: TcpListener, worker_seen: Arc<AtomicBool>) {
    let deadline = Instant::now()
        .checked_add(Duration::from_millis(200))
        .expect("deadline should be representable");
    while Instant::now() < deadline {
        match listener.accept() {
            Ok((mut stream, _address)) => {
                worker_seen.store(true, Ordering::SeqCst);
                let mut request = [0_u8; 1024];
                let read_result = stream.read(&mut request);
                assert!(
                    read_result.is_ok(),
                    "unexpected command request should be readable"
                );
                let response = serde_json::json!({
                    "revision": 2_i32,
                    "receipt_id": "unexpected-move",
                    "event_cursor": 1_i32,
                    "viewer_projection": {"revision": 2_i32, "turn": 1_i32}
                })
                .to_string();
                let message = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                    response.len(),
                    response
                );
                stream
                    .write_all(message.as_bytes())
                    .expect("unexpected command response should write");
                break;
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => assert_eq!(
                error.kind(),
                ErrorKind::WouldBlock,
                "test listener should only report WouldBlock while idle"
            ),
        }
    }
    drop(listener);
    drop(worker_seen);
}

#[test]
fn left_click_on_owned_unit_activates_without_moving() {
    let mut state = crate::gameplay::model::GameState::new(24, 18, 1, 85);
    state.popups.clear();
    let selected = state
        .selected_unit()
        .expect("selected unit should exist")
        .clone();
    let mut ui = InGameUiModel::default();
    let mut camera = Camera::centered_on_selected(&state);
    let mut movement = MovementInputState::default();
    let (x, y) = camera.tile_to_screen(selected.position);

    let outcome = handle_left_click(x, y, &mut state, &mut ui, &mut camera, &mut movement, None);

    assert_eq!(outcome, None, "unit activation should not close gameplay");
    assert_eq!(
        movement.active_unit(),
        Some(selected.id),
        "clicking an owned unit should activate it for path planning"
    );
    assert_eq!(
        state
            .selected_unit()
            .expect("selected unit should remain selected")
            .position,
        selected.position,
        "activation alone must not move the unit"
    );
}

#[test]
fn first_unit_click_drag_release_moves_along_computed_path() {
    let mut state = crate::gameplay::model::GameState::new(24, 18, 1, 86);
    state.popups.clear();
    make_world_passable_for_movement_test(&mut state);
    let start = state
        .selected_unit()
        .expect("selected unit should exist")
        .position;
    let detour = start
        .offset(1, 0, state.width, state.height)
        .expect("detour target should be in bounds");
    let target = start
        .offset(1, 1, state.width, state.height)
        .expect("diagonal target should be in bounds");
    let mut ui = InGameUiModel::default();
    let mut camera = Camera::centered_on_selected(&state);
    let mut movement = MovementInputState::default();
    let (start_x, start_y) = camera.tile_to_screen(start);
    let (detour_x, detour_y) = camera.tile_to_screen(detour);
    let (target_x, target_y) = camera.tile_to_screen(target);

    let _activate_and_drag_start = handle_left_click(
        start_x,
        start_y,
        &mut state,
        &mut ui,
        &mut camera,
        &mut movement,
        None,
    );
    handle_mouse_motion(detour_x, detour_y, &state, &camera, &mut movement);
    handle_mouse_motion(target_x, target_y, &state, &camera, &mut movement);

    assert_eq!(
        movement.planned_path(),
        &[target],
        "CTP2 movement should recompute the path from the unit origin to the current destination instead of preserving cursor detours"
    );

    let outcome = handle_left_release(
        target_x,
        target_y,
        &mut state,
        &ui,
        &mut camera,
        &mut movement,
        None,
    );

    assert_eq!(outcome, None, "drag release should not close gameplay");
    assert_eq!(
        state
            .selected_unit()
            .expect("selected unit should still exist")
            .position,
        target,
        "local fallback should execute only after a completed computed path"
    );
}

#[test]
fn mouse_up_without_drag_path_submits_nothing() {
    let mut state = crate::gameplay::model::GameState::new(24, 18, 1, 87);
    state.popups.clear();
    let start = state
        .selected_unit()
        .expect("selected unit should exist")
        .position;
    let mut ui = InGameUiModel::default();
    let mut camera = Camera::centered_on_selected(&state);
    let mut movement = MovementInputState::default();
    let (x, y) = camera.tile_to_screen(start);

    let _activate_and_drag_start =
        handle_left_click(x, y, &mut state, &mut ui, &mut camera, &mut movement, None);
    let outcome = handle_left_release(x, y, &mut state, &ui, &mut camera, &mut movement, None);

    assert_eq!(
        outcome, None,
        "empty drag release should not close gameplay"
    );
    assert_eq!(
        state
            .selected_unit()
            .expect("selected unit should still exist")
            .position,
        start,
        "mouse-up without a planned path must not issue movement"
    );
}

#[test]
fn server_path_conversion_preserves_dragged_tile_order() {
    let projection = ServerProjection {
        tiles: vec![
            ServerTileProjection {
                id: "origin".to_owned(),
                x: 4,
                y: 4,
                terrain: "plains".to_owned(),
                ..ServerTileProjection::default()
            },
            ServerTileProjection {
                id: "east".to_owned(),
                x: 5,
                y: 4,
                terrain: "plains".to_owned(),
                ..ServerTileProjection::default()
            },
            ServerTileProjection {
                id: "south-east".to_owned(),
                x: 6,
                y: 5,
                terrain: "plains".to_owned(),
                ..ServerTileProjection::default()
            },
        ],
        ..ServerProjection::default()
    };

    let ids = server_tile_ids_for_path(
        Some(&projection),
        &[MapPoint::new(5, 4), MapPoint::new(6, 5)],
    )
    .expect("all dragged tiles should map to server ids");

    assert_eq!(
        ids,
        vec!["east".to_owned(), "south-east".to_owned()],
        "authoritative move payload should preserve the drag path, not collapse to one target"
    );
}

/// Makes every generated tile passable and visible so movement tests isolate input semantics.
fn make_world_passable_for_movement_test(state: &mut crate::gameplay::model::GameState) {
    for tile in &mut state.tiles {
        tile.terrain = TerrainKind::Grassland;
        tile.fog = FogState::Visible;
    }
}

/// Returns a passable tile that does not contain a unit.
fn empty_passable_tile(state: &crate::gameplay::model::GameState, excluded: MapPoint) -> MapPoint {
    for y in 0..state.height {
        for x in 0..state.width {
            let point = MapPoint::new(x, y);
            if point == excluded || state.units.iter().any(|unit| unit.position == point) {
                continue;
            }
            if state
                .tile(point)
                .is_some_and(|tile| tile.terrain.passable_by_land())
            {
                return point;
            }
        }
    }
    excluded
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
    let mut movement = MovementInputState::default();

    let outcome = handle_right_click(
        570,
        650,
        &mut state,
        &mut ui,
        &mut camera,
        &mut movement,
        None,
    );

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
    let mut movement = MovementInputState::default();

    let outcome = handle_right_click(
        120,
        610,
        &mut state,
        &mut ui,
        &mut camera,
        &mut movement,
        None,
    );

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
    let mut movement = MovementInputState::default();

    let outcome = handle_right_click(
        120,
        610,
        &mut state,
        &mut ui,
        &mut camera,
        &mut movement,
        None,
    );

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
fn headless_gameplay_reuses_existing_sdl_window_and_scales_to_viewport() {
    let guard =
        crate::sdl_test_lock::lock_sdl_for_test().expect("SDL test lock should be available");
    let environment = HeadlessSdlEnvironment::apply();
    {
        let sdl = sdl2::init().expect("SDL should initialize with dummy driver");
        let video = sdl.video().expect("video subsystem should initialize");
        let window = video
            .window("menu-owned", 1280, 720)
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
        assert_eq!(
            canvas.logical_size(),
            (super::DEFAULT_WINDOW_WIDTH, super::DEFAULT_WINDOW_HEIGHT),
            "gameplay should render into a scaled 1024x768 logical viewport instead of the physical top-left corner"
        );
    }
    environment.restore();
    drop(guard);
}
