//! Original cursor art, edge thresholds and modal/fog-safe feedback regressions.

use super::{CursorKind, EdgeScroll, cursor_for_frame, edge_direction};
use crate::gameplay::model::{FogState, GameState, TerrainKind};
use crate::gameplay::runtime::movement::begin_or_activate_unit_drag;
use crate::gameplay::runtime::{Camera, MovementInputState};
use crate::gameplay::ui_model::InGameUiModel;

#[test]
fn original_screen_edges_and_vertical_corner_priority_are_exact() {
    let cases: [((i32, i32), Option<CursorKind>); 12] = [
        ((0, 200), Some(CursorKind::ScrollLeft)),
        ((3, 200), Some(CursorKind::ScrollLeft)),
        ((4, 200), None),
        ((796, 200), None),
        ((797, 200), Some(CursorKind::ScrollRight)),
        ((400, 2), Some(CursorKind::ScrollUp)),
        ((400, 3), None),
        ((400, 598), Some(CursorKind::ScrollDown)),
        ((0, 0), Some(CursorKind::ScrollUp)),
        ((799, 599), Some(CursorKind::ScrollDown)),
        ((-1, 200), None),
        ((800, 200), None),
    ];
    for (point, expected) in cases {
        assert_eq!(
            edge_direction(Some(point)),
            expected,
            "civ3_main.cpp edge {point:?}"
        );
    }
    assert_eq!(
        edge_direction(None),
        None,
        "unfocused/outside window never scrolls"
    );
}

#[test]
fn movement_cursor_uses_legal_path_and_never_peeks_at_hidden_enemy() {
    let mut state = GameState::new(24, 18, 1, 93);
    state.popups.clear();
    for tile in &mut state.tiles {
        tile.terrain = TerrainKind::Grassland;
        tile.fog = FogState::Visible;
    }
    let origin = state.selected_unit().expect("starter").position;
    let target = origin
        .offset(1, 0, state.width, state.height)
        .expect("neighbor");
    let camera = Camera::centered_on_selected(&state);
    let mut movement = MovementInputState::default();
    assert!(
        begin_or_activate_unit_drag(&mut state, &mut movement, origin),
        "activate"
    );
    movement.preview_path_to(&state, target);
    let mut ui = InGameUiModel {
        pointer: Some(camera.tile_click_center_to_screen(target)),
        hovered_tile: Some(target),
        ..InGameUiModel::default()
    };
    assert_eq!(
        cursor_for_frame(&state, &ui, &movement),
        CursorKind::Move,
        "legal path"
    );
    let index = usize::from(target.y)
        .saturating_mul(usize::from(state.width))
        .saturating_add(usize::from(target.x));
    state.tiles[index].terrain = TerrainKind::Water;
    movement.preview_path_to(&state, target);
    assert_eq!(
        cursor_for_frame(&state, &ui, &movement),
        CursorKind::NoMove,
        "blocked path"
    );
    state.tiles[index].terrain = TerrainKind::Grassland;
    let mut enemy = state.selected_unit().expect("starter").clone();
    enemy.id = crate::gameplay::model::UnitId::from_server_id("visible-enemy");
    enemy.owner = crate::gameplay::model::PlayerId::Empire(2);
    enemy.position = target;
    state.units.push(enemy);
    movement.preview_path_to(&state, target);
    assert_eq!(
        cursor_for_frame(&state, &ui, &movement),
        CursorKind::Assault,
        "visible enemy changes move feedback"
    );
    state.tiles[index].fog = FogState::Hidden;
    assert_eq!(
        cursor_for_frame(&state, &ui, &movement),
        CursorKind::Move,
        "hidden enemy must not affect cursor"
    );
    ui.pointer = Some((20_i32, 10_i32));
    assert_eq!(
        cursor_for_frame(&state, &ui, &movement),
        CursorKind::Hand,
        "UI uses original hand, not map order"
    );
    ui.pointer = Some((1_i32, 200_i32));
    assert_eq!(
        cursor_for_frame(&state, &ui, &movement),
        CursorKind::ScrollLeft,
        "edge has priority"
    );
    state.popups.push(crate::gameplay::model::GameplayPopup {
        title: String::new(),
        body: String::new(),
        kind: crate::gameplay::model::PopupKind::Info,
    });
    assert_eq!(
        cursor_for_frame(&state, &ui, &movement),
        CursorKind::Hand,
        "modal suppresses scroll"
    );
}

#[test]
fn camera_pan_recomputes_stationary_pointer_tile_and_path() {
    let mut state = GameState::new(24, 18, 1, 42);
    state.popups.clear();
    for tile in &mut state.tiles {
        tile.terrain = TerrainKind::Grassland;
        tile.fog = FogState::Visible;
    }
    let origin = state.selected_unit().expect("starter").position;
    let mut camera = Camera::centered_on_selected(&state);
    let mut input = MovementInputState::default();
    assert!(
        begin_or_activate_unit_drag(&mut state, &mut input, origin),
        "activate"
    );
    let mut ui = InGameUiModel {
        pointer: Some((400_i32, 200_i32)),
        ..InGameUiModel::default()
    };
    super::refresh_map_pointer(&state, &mut ui, &camera, &mut input);
    let first = ui.hovered_tile;
    camera.pan_pixels(94, 0);
    super::refresh_map_pointer(&state, &mut ui, &camera, &mut input);
    assert_ne!(
        ui.hovered_tile, first,
        "stationary pointer sees a different map tile after pan"
    );
    assert_eq!(
        input.planned_path().last().copied(),
        ui.hovered_tile,
        "preview must follow the updated tile, not a stale destination"
    );
    ui.pointer = None;
    super::refresh_map_pointer(&state, &mut ui, &camera, &mut input);
    assert!(
        input.planned_path().is_empty(),
        "outside-window pointer cancels stale preview"
    );
}

#[test]
fn leaving_window_clears_software_cursor_and_map_feedback() {
    let mut state = GameState::new(24, 18, 1, 42);
    state.popups.clear();
    let mut ui = InGameUiModel {
        pointer: Some((1_i32, 100_i32)),
        hovered_tile: state.selected_unit().map(|unit| unit.position),
        ..InGameUiModel::default()
    };
    let mut camera = Camera::centered_on_selected(&state);
    let mut movement = MovementInputState::default();
    let event = sdl2::event::Event::Window {
        timestamp: 0,
        window_id: 1,
        win_event: sdl2::event::WindowEvent::Leave,
    };
    assert_eq!(
        crate::gameplay::runtime::handle_event(
            &event,
            &mut state,
            &mut ui,
            &mut camera,
            &mut movement,
            None
        ),
        None,
        "leaving window is not a game action"
    );
    assert_eq!(ui.pointer, None, "no stuck cursor after leaving");
    assert_eq!(ui.hovered_tile, None, "no stuck tile highlight");
}

#[test]
fn software_cursor_guard_restores_inherited_os_visibility() {
    let result =
        crate::gameplay::runtime::test_support::with_canvas((800_u32, 600_u32), |_canvas| {
            let sdl = sdl2::init()?;
            let mouse = sdl.mouse();
            let before = mouse.is_cursor_showing();
            {
                let _guard =
                    super::SoftwareCursorGuard::new().map_err(|error| error.to_string())?;
                if mouse.is_cursor_showing() {
                    return Err("OS cursor not hidden".to_owned());
                }
            }
            if mouse.is_cursor_showing() != before {
                return Err("inherited cursor visibility not restored".to_owned());
            }
            Ok(())
        });
    assert!(result.is_ok(), "cursor lifetime regression: {result:?}");
}

#[test]
fn cursor_guard_reasserts_hidden_native_cursor_after_focus_or_external_visibility_changes() {
    let result =
        crate::gameplay::runtime::test_support::with_canvas((800_u32, 600_u32), |_canvas| {
            let sdl = sdl2::init()?;
            let mouse = sdl.mouse();
            let guard = super::SoftwareCursorGuard::new().map_err(|error| error.to_string())?;
            mouse.show_cursor(true);
            guard.refresh();
            if mouse.is_cursor_showing() {
                return Err(
                    "focus handoff exposed the OS arrow alongside the software hand".to_owned(),
                );
            }
            Ok(())
        });
    assert!(result.is_ok(), "exclusive software cursor: {result:?}");
}

#[test]
fn scaled_window_mouse_events_stay_aligned_after_native_frame_composition() {
    let result =
        crate::gameplay::runtime::test_support::with_canvas((1280_u32, 720_u32), |canvas| {
            let sdl = sdl2::init()?;
            let mut events = sdl.event_pump()?;
            let creator = canvas.texture_creator();
            let mut frame = creator
                .create_texture_target(None, 800_u32, 600_u32)
                .map_err(|error| error.to_string())?;
            let mut textures =
                crate::gameplay::runtime::textures::GameplayTextures::without_assets_for_test();
            let state = GameState::new(24_u16, 18_u16, 1_u8, 42_u64);
            let ui = InGameUiModel::default();
            let camera = Camera::centered_on_selected(&state);
            for _frame in 0_u8..3_u8 {
                crate::gameplay::runtime::frame::draw_scaled_frame(
                    canvas,
                    &mut frame,
                    &mut textures,
                    &state,
                    &ui,
                    &camera,
                    &MovementInputState::default(),
                    None,
                )
                .map_err(|error| error.to_string())?;
                // Drain SDL's end-of-poll sentinel before injecting the next
                // frame; dropping an iterator at its first match leaves it queued.
                events.poll_iter().for_each(drop);
                sdl.event()?.push_event(sdl2::event::Event::MouseMotion {
                    timestamp: 0_u32,
                    window_id: canvas.window().id(),
                    which: 0_u32,
                    mousestate: sdl2::mouse::MouseState::from_sdl_state(0_u32),
                    x: 700_i32,
                    y: 420_i32,
                    xrel: 0_i32,
                    yrel: 0_i32,
                })?;
                let event = events
                    .poll_iter()
                    .find(|event| matches!(event, &sdl2::event::Event::MouseMotion { .. }))
                    .ok_or_else(|| "native pointer event missing".to_owned())?;
                let event = crate::gameplay::runtime::viewport::map_event(canvas, &event)
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "pointer unexpectedly outside viewport".to_owned())?;
                if !matches!(
                    event,
                    sdl2::event::Event::MouseMotion {
                        x: 450_i32,
                        y: 350_i32,
                        ..
                    }
                ) {
                    return Err(format!(
                        "expected logical hand (450,350) at window (700,420), got {event:?}"
                    ));
                }
            }
            Ok(())
        });
    assert!(result.is_ok(), "scaled cursor/input alignment: {result:?}");
}

#[test]
fn scroll_is_clock_driven_delayed_and_resets_outside_window() {
    let mut scroll = EdgeScroll::default();
    let mut camera = Camera::for_test_identity();
    let start = camera;
    let edge = Some((799_i32, 200_i32));
    scroll.update(&mut camera, edge, false, 0);
    scroll.update(&mut camera, edge, false, 299);
    assert_eq!(camera, start, "original default ScrollDelay=3 waits 300ms");
    scroll.update(&mut camera, edge, false, 300);
    assert_ne!(
        camera, start,
        "stationary edge pointer must scroll without new mouse events"
    );
    let first = camera;
    scroll.update(&mut camera, edge, false, 300);
    assert_eq!(camera, first, "duplicate timestamp cannot advance camera");
    scroll.update(&mut camera, None, false, 320);
    scroll.update(&mut camera, edge, false, 400);
    assert_eq!(camera, first, "returning starts a fresh delay");
    scroll.update(&mut camera, edge, true, 900);
    assert_eq!(camera, first, "modal blocks scrolling");
}

#[test]
fn scroll_retains_fractional_ticks_at_different_frame_rates() {
    let mut fast = EdgeScroll::default();
    let mut slow = EdgeScroll::default();
    let mut fast_camera = Camera::for_test_identity();
    let mut slow_camera = fast_camera;
    let edge = Some((799_i32, 200_i32));
    for now in (0_u64..=600_u64).step_by(8) {
        fast.update(&mut fast_camera, edge, false, now);
    }
    for now in (0_u64..=600_u64).step_by(20) {
        slow.update(&mut slow_camera, edge, false, now);
    }
    assert_eq!(
        fast_camera, slow_camera,
        "scroll distance must not depend on rendering FPS"
    );
}
