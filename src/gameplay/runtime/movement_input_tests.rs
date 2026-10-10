//! Source-aligned movement input tests for the SDL gameplay runtime.

use sdl2::event::Event;
use sdl2::mouse::MouseState;

use super::{Camera, MovementInputState, handle_event, handle_left_click, handle_left_release};
use crate::gameplay::model::{FogState, MapPoint, TerrainKind, UnitId};
use crate::gameplay::ui_model::InGameUiModel;

/// Empty SDL mouse state used for hover events.
const NO_MOUSE_BUTTONS: u32 = 0_u32;
/// SDL mouse-state bit for the left button.
const LEFT_MOUSE_BUTTON: u32 = 1_u32;
/// Multi-step movement allowance for tests that need to consume a full computed path.
const TEST_MULTI_STEP_MOVE_POINTS: i16 = 300_i16;

#[test]
fn mouse_motion_without_left_button_updates_ctp2_hover_path_for_active_unit() {
    let mut state = crate::gameplay::model::GameState::new(24, 18, 1, 91);
    state.popups.clear();
    make_world_passable_for_input_test(&mut state);
    let selected = state
        .selected_unit()
        .expect("generated game should select a human unit")
        .clone();
    let target = selected
        .position
        .offset(1, 1, state.width, state.height)
        .expect("diagonal hover target should be in bounds");
    let mut ui = InGameUiModel::default();
    let mut camera = Camera::centered_on_selected(&state);
    let mut movement = MovementInputState::default();
    let (start_x, start_y) = camera.tile_to_screen(selected.position);
    let (target_x, target_y) = camera.tile_to_screen(target);

    let down = handle_left_click(
        start_x,
        start_y,
        &mut state,
        &mut ui,
        &mut camera,
        &mut movement,
        None,
    );
    let up = handle_left_release(
        start_x,
        start_y,
        &mut state,
        &ui,
        &mut camera,
        &mut movement,
        None,
    );
    let hover = handle_event(
        &mouse_motion_event(target_x, target_y, NO_MOUSE_BUTTONS),
        &mut state,
        &mut ui,
        &mut camera,
        &mut movement,
        None,
    );

    assert_eq!(
        down, None,
        "activation mouse-down should not close gameplay"
    );
    assert_eq!(up, None, "activation mouse-up should not close gameplay");
    assert_eq!(hover, None, "hover pathing should not close gameplay");
    assert_eq!(
        movement.active_unit(),
        Some(selected.id),
        "the clicked unit should remain active for CTP2 path preview"
    );
    assert_eq!(
        movement.planned_path(),
        &[target],
        "CTP2 mode should update the drawable path on ordinary hover after activating a unit"
    );
    assert_eq!(
        movement.finish_drag(),
        None,
        "hover-only path previews must never submit a move order"
    );
}

#[test]
fn drag_release_applies_computed_path_around_blocker_in_local_fallback() {
    let mut state = crate::gameplay::model::GameState::new(24, 18, 1, 92);
    state.popups.clear();
    make_world_passable_for_input_test(&mut state);
    let selected = state
        .selected_unit()
        .expect("generated game should select a human unit")
        .clone();
    give_unit_multi_step_movement(&mut state, selected.id);
    let blocker = selected
        .position
        .offset(1, 0, state.width, state.height)
        .expect("blocker should be in bounds");
    let target = selected
        .position
        .offset(2, 0, state.width, state.height)
        .expect("drag target should be in bounds");
    set_terrain(&mut state, blocker, TerrainKind::Water);
    let mut ui = InGameUiModel::default();
    let mut camera = Camera::centered_on_selected(&state);
    let mut movement = MovementInputState::default();
    let (start_x, start_y) = camera.tile_to_screen(selected.position);
    let (target_x, target_y) = camera.tile_to_screen(target);

    let _activation = handle_left_click(
        start_x,
        start_y,
        &mut state,
        &mut ui,
        &mut camera,
        &mut movement,
        None,
    );
    let drag = handle_event(
        &mouse_motion_event(target_x, target_y, LEFT_MOUSE_BUTTON),
        &mut state,
        &mut ui,
        &mut camera,
        &mut movement,
        None,
    );

    assert_eq!(drag, None, "dragging a path should not close gameplay");
    assert!(
        !movement.planned_path().is_empty(),
        "drag planning should find a passable CTP2-style path around the blocked direct tile"
    );
    assert!(
        !movement.planned_path().contains(&blocker),
        "the computed movement path must not include impassable terrain"
    );

    let release = handle_left_release(
        target_x,
        target_y,
        &mut state,
        &ui,
        &mut camera,
        &mut movement,
        None,
    );

    assert_eq!(release, None, "movement release should not close gameplay");
    assert_eq!(
        state
            .selected_unit()
            .expect("selected unit should survive local fallback movement")
            .position,
        target,
        "local fallback must execute the same computed path that the drag preview displayed"
    );
}

/// Builds an SDL mouse-motion event with an explicit button mask.
fn mouse_motion_event(x: i32, y: i32, button_state: u32) -> Event {
    Event::MouseMotion {
        timestamp: 0,
        window_id: 0,
        which: 0,
        mousestate: MouseState::from_sdl_state(button_state),
        x,
        y,
        xrel: 0,
        yrel: 0,
    }
}

/// Makes every generated tile passable and visible so movement tests isolate input behavior.
fn make_world_passable_for_input_test(state: &mut crate::gameplay::model::GameState) {
    for tile in &mut state.tiles {
        tile.terrain = TerrainKind::Grassland;
        tile.fog = FogState::Visible;
    }
}

/// Gives one test unit enough movement points to traverse a short multi-tile path immediately.
fn give_unit_multi_step_movement(state: &mut crate::gameplay::model::GameState, unit_id: UnitId) {
    for unit in &mut state.units {
        if unit.id == unit_id {
            unit.movement_points = TEST_MULTI_STEP_MOVE_POINTS;
            unit.max_movement_points = TEST_MULTI_STEP_MOVE_POINTS;
        }
    }
}

/// Sets terrain at a map point and fails loudly if the generated point is invalid.
fn set_terrain(
    state: &mut crate::gameplay::model::GameState,
    point: MapPoint,
    terrain: TerrainKind,
) {
    let index = usize::from(point.y)
        .checked_mul(usize::from(state.width))
        .and_then(|row| row.checked_add(usize::from(point.x)))
        .expect("test point index should fit in usize");
    let tile = state
        .tiles
        .get_mut(index)
        .expect("test point should address an existing tile");
    tile.terrain = terrain;
}
