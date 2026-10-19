//! Input dispatch reaches the same confirmed-motion presenter used by SDL drawing.

use super::{Camera, MovementInputState, apply_ui_action};
use crate::gameplay::model::{FogState, GameState, TerrainKind};
use crate::gameplay::ui_model::{InGameAction, InGameUiModel};

#[test]
fn keyboard_movement_preserves_camera_and_starts_confirmed_animation_at_old_tile() {
    let mut state = GameState::new(24, 18, 1, 42);
    state.popups.clear();
    for tile in &mut state.tiles {
        tile.terrain = TerrainKind::Grassland;
        tile.fog = FogState::Visible;
    }
    let unit = state.selected_unit().expect("starter").clone();
    let target = unit
        .position
        .offset(1, 0, state.width, state.height)
        .expect("target");
    let mut ui = InGameUiModel::default();
    let mut camera = Camera::centered_on_selected(&state);
    let before_camera = camera;
    let mut movement = MovementInputState::default();
    assert_eq!(
        apply_ui_action(
            InGameAction::MoveBy(1, 0),
            &mut state,
            &mut ui,
            &mut camera,
            &mut movement,
            None
        ),
        None,
        "keyboard move stays in game"
    );
    assert_eq!(
        camera, before_camera,
        "do not snap camera to endpoint before animation"
    );
    let moved = state.selected_unit().expect("selected unit retained");
    assert_eq!(
        moved.position, target,
        "model receives the confirmed endpoint"
    );
    movement.actors.begin_frame(1000);
    let pose = movement.actors.pose(moved, &camera, 300);
    assert_eq!(
        pose.position,
        camera.tile_to_screen(unit.position),
        "presenter starts at actual origin"
    );
    assert!(pose.moving, "input dispatcher records accepted travel");
    movement.actors.begin_frame(1300);
    assert_eq!(
        movement.actors.pose(moved, &camera, 300).position,
        camera.tile_to_screen(target),
        "presenter arrives exactly once"
    );
}
