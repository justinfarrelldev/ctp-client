//! Presented glyph picking and canonical destination/gesture separation.
use super::{MovementInputState, begin_map_unit_drag, begin_or_activate_unit_drag};
use crate::gameplay::{
    model::{GameState, MapPoint, UnitId},
    runtime::{Camera, actor_hits::ActorHit, handle_event},
    ui_model::{InGameUiModel, MenuRect},
};
use sdl2::{event::Event, mouse::MouseButton};
/// An injected last-presented own glyph far from its received canonical destination.
fn scene() -> (GameState, MovementInputState, UnitId) {
    let mut state = GameState::new(24_u16, 18_u16, 1_u8, 42_u64);
    let mut unit = state.selected_unit().expect("starter").clone();
    unit.position = MapPoint::new(5_u16, 5_u16);
    let id = unit.id;
    state.units = vec![unit];
    state.selected_unit = None;
    state.popups.clear();
    state.authoritative = true;
    let mut movement = MovementInputState::default();
    movement.record_presented(
        &state,
        vec![ActorHit {
            id,
            canonical: MapPoint::new(5_u16, 5_u16),
            bounds: MenuRect {
                x: 100_i32,
                y: 100_i32,
                width: 12_u32,
                height: 18_u32,
            },
            mask: None,
            visible: true,
            flipped: false,
            moving: true,
        }],
    );
    (state, movement, id)
}
#[test]
fn native_glyph_click_selects_exact_moving_identity_not_grid_cell_and_jitter_never_orders_backwards()
 {
    let (mut state, mut movement, id) = scene();
    let mut ui = InGameUiModel::default();
    let mut camera = Camera::for_test_identity();
    let point = camera.screen_to_tile(101_i32, 101_i32);
    assert_eq!(
        point,
        camera.screen_to_tile(102_i32, 101_i32),
        "jitter stays in gesture tile"
    );
    assert_ne!(
        point,
        Some(MapPoint::new(5_u16, 5_u16)),
        "sprite is not at canonical destination"
    );
    let result = handle_event(
        &Event::MouseButtonDown {
            timestamp: 0_u32,
            window_id: 0_u32,
            which: 0_u32,
            mouse_btn: MouseButton::Left,
            clicks: 1_u8,
            x: 101_i32,
            y: 101_i32,
        },
        &mut state,
        &mut ui,
        &mut camera,
        &mut movement,
        None,
    );
    assert!(result.is_none(), "in-game click");
    assert_eq!(
        state.selected_unit,
        Some(id),
        "actual visible actor ID, not canonical/grid target"
    );
    assert_eq!(
        movement.active_unit(),
        Some(id),
        "shared semantic activation"
    );
    movement.extend_drag_to(&state, point.expect("map gesture"));
    assert!(
        movement.finish_drag().is_none(),
        "same visual gesture tile cannot accidentally issue a return path from canonical destination"
    );
    assert_eq!(
        state.units[0].position,
        MapPoint::new(5_u16, 5_u16),
        "selection/gesture never mutates authority"
    );
    assert!(
        state.popups.is_empty(),
        "no accidental transport/order error"
    );
}
#[test]
fn changed_authority_revision_consumes_map_click_without_rebasing_old_drawn_identity() {
    let (mut state, mut movement, _id) = scene();
    state.viewer = Some(
        serde_json::from_value(serde_json::json!({"revision":2_i64,"turn":1_u32}))
            .expect("changed authority"),
    );
    let mut ui = InGameUiModel::default();
    let mut camera = Camera::for_test_identity();
    let result = handle_event(
        &Event::MouseButtonDown {
            timestamp: 0_u32,
            window_id: 0_u32,
            which: 0_u32,
            mouse_btn: MouseButton::Left,
            clicks: 1_u8,
            x: 101_i32,
            y: 101_i32,
        },
        &mut state,
        &mut ui,
        &mut camera,
        &mut movement,
        None,
    );
    assert!(result.is_none(), "click consumed");
    assert_eq!(
        state.selected_unit, None,
        "old descriptor never rebases to current facts"
    );
    assert_eq!(
        movement.active_unit(),
        None,
        "no implicit neighboring unit or move"
    );
}
#[test]
fn a_revision_change_during_a_held_drag_cancels_the_gesture_without_rebasing_its_order() {
    let (mut state, mut movement, id) = scene();
    state.authoritative = false;
    for tile in &mut state.tiles {
        tile.terrain = crate::gameplay::model::TerrainKind::Grassland;
        tile.fog = crate::gameplay::model::FogState::Visible;
    }
    state.viewer = Some(
        serde_json::from_value(serde_json::json!({"revision":1_i64,"turn":1_u32})).expect("viewer"),
    );
    assert!(
        begin_or_activate_unit_drag(&mut state, &mut movement, MapPoint::new(5_u16, 5_u16)),
        "explicit current gesture"
    );
    movement.extend_drag_to(&state, MapPoint::new(6_u16, 5_u16));
    assert!(
        !movement.planned_path().is_empty(),
        "valid injected land path establishes order before revision change"
    );
    state.viewer.as_mut().expect("viewer").revision = 2_i64;
    movement.retain_selectable(&state);
    assert!(
        movement.finish_drag().is_none(),
        "receipt cannot rebase held order into a new revision"
    );
    assert_eq!(
        movement.active_unit(),
        Some(id),
        "current owned selection may survive, order may not"
    );
}
#[test]
fn explicit_context_and_semantic_activate_keep_second_member_identity_even_with_a_different_active_member_in_the_same_cell()
 {
    let (mut state, mut movement, first) = scene();
    let mut second = state.units[0].clone();
    second.id = UnitId(99_999_u32);
    let second_id = second.id;
    state.units.push(second);
    let mut ui = InGameUiModel::default();
    let mut camera = Camera::for_test_identity();
    assert!(
        begin_or_activate_unit_drag(&mut state, &mut movement, MapPoint::new(5_u16, 5_u16)),
        "first active"
    );
    crate::gameplay::runtime::context_input::apply_context_intent(
        crate::gameplay::ui_model::context::ContextIntent::Move(second_id),
        &mut state,
        &mut ui,
        &mut movement,
        None,
    );
    assert_eq!(
        (state.selected_unit, movement.active_unit()),
        (Some(second_id), Some(second_id)),
        "explicit second member cannot be silently replaced by prior active first member"
    );
    assert!(state.select_unit(first), "explicit first again");
    let result = crate::gameplay::runtime::controls::apply_intent(
        crate::gameplay::runtime::controls::ControlIntent::ActivateUnit,
        &mut state,
        &mut ui,
        &mut camera,
        &mut movement,
        None,
    );
    assert!(result.is_none(), "activate stays in-game");
    assert_eq!(
        movement.active_unit(),
        Some(first),
        "accessible Activate Unit honors exact selected ID rather than active neighbor"
    );
}
#[test]
fn unselected_moving_body_right_click_selects_exact_actor_and_opens_original_context_popup() {
    let (mut state, mut movement, id) = scene();
    let mut ui = InGameUiModel {
        active_tab: crate::gameplay::ui_model::ControlPanelTab::Domestic,
        ..InGameUiModel::default()
    };
    let mut camera = Camera::for_test_identity();
    let result = handle_event(
        &Event::MouseButtonDown {
            timestamp: 0_u32,
            window_id: 0_u32,
            which: 0_u32,
            mouse_btn: MouseButton::Right,
            clicks: 1_u8,
            x: 101_i32,
            y: 101_i32,
        },
        &mut state,
        &mut ui,
        &mut camera,
        &mut movement,
        None,
    );
    assert!(result.is_none(), "in-game source context activation");
    assert_eq!(
        state.selected_unit,
        Some(id),
        "source ArmyContextClick selects actual body ID, not underlying grid cell"
    );
    assert_eq!(
        movement.active_unit(),
        Some(id),
        "same exact selection/order identity"
    );
    assert_eq!(
        ui.active_tab,
        crate::gameplay::ui_model::ControlPanelTab::Unit,
        "installed default AutoSwitchTabs selects the Unit tab"
    );
    let popup = ui
        .context
        .as_ref()
        .expect("original ActivateSelectedInfo calls BuildList then contextMenu.Open");
    assert_eq!(
        popup.activate("Move"),
        Some(crate::gameplay::ui_model::context::ContextIntent::Move(id)),
        "popup remains tied to the exact displayed actor, never another same-cell unit"
    );
    assert!(
        ui.map_info.is_none(),
        "do not inspect unrelated terrain behind actor"
    );
    assert!(
        movement.finish_drag().is_none(),
        "right click activates information, not an order gesture"
    );
    assert_eq!(
        state.units[0].position,
        MapPoint::new(5_u16, 5_u16),
        "no local canonical movement"
    );
}
#[test]
fn hovering_painted_owned_body_uses_hand_not_terrain_movement_feedback_behind_the_traveler() {
    let (mut state, mut movement, _id) = scene();
    state.authoritative = false;
    for tile in &mut state.tiles {
        tile.terrain = crate::gameplay::model::TerrainKind::Grassland;
        tile.fog = crate::gameplay::model::FogState::Visible;
    }
    assert!(
        begin_or_activate_unit_drag(&mut state, &mut movement, MapPoint::new(5_u16, 5_u16)),
        "current activation"
    );
    movement.cancel_drag();
    let point = Camera::for_test_identity()
        .screen_to_tile(101_i32, 101_i32)
        .expect("pointer tile");
    movement.preview_path_to(&state, point);
    assert!(
        !movement.planned_path().is_empty(),
        "legal terrain behind body gives misleading feedback without actor picking"
    );
    let ui = InGameUiModel {
        pointer: Some((101_i32, 101_i32)),
        hovered_tile: Some(point),
        ..InGameUiModel::default()
    };
    assert_eq!(
        crate::gameplay::runtime::cursor::cursor_for_frame(&state, &ui, &movement),
        crate::gameplay::runtime::cursor::CursorKind::Hand,
        "owned presented body is selection/information, not an implied destination behind its sprite"
    );
}
#[test]
fn an_existing_drag_over_a_painted_body_keeps_legal_movement_feedback_instead_of_becoming_a_selection_click()
 {
    let (mut state, mut movement, _id) = scene();
    state.authoritative = false;
    for tile in &mut state.tiles {
        tile.terrain = crate::gameplay::model::TerrainKind::Grassland;
        tile.fog = crate::gameplay::model::FogState::Visible;
    }
    assert!(
        begin_or_activate_unit_drag(&mut state, &mut movement, MapPoint::new(5_u16, 5_u16)),
        "held gesture"
    );
    let point = Camera::for_test_identity()
        .screen_to_tile(101_i32, 101_i32)
        .expect("pointer tile");
    movement.extend_drag_to(&state, point);
    assert!(
        !movement.planned_path().is_empty(),
        "legal held path established"
    );
    let ui = InGameUiModel {
        pointer: Some((101_i32, 101_i32)),
        hovered_tile: Some(point),
        ..InGameUiModel::default()
    };
    assert_eq!(
        crate::gameplay::runtime::cursor::cursor_for_frame(&state, &ui, &movement),
        crate::gameplay::runtime::cursor::CursorKind::Move,
        "body selection override applies to hover, not a held movement gesture"
    );
}
#[test]
fn pointer_does_not_select_phantom_destination_but_explicit_semantic_selection_and_stationary_member_still_work()
 {
    let (mut state, mut movement, id) = scene();
    let destination = MapPoint::new(5_u16, 5_u16);
    assert!(
        !begin_map_unit_drag(&mut state, &mut movement, destination),
        "traveling actor is visibly elsewhere"
    );
    assert_eq!(state.selected_unit, None, "no phantom destination glyph");
    assert!(
        begin_or_activate_unit_drag(&mut state, &mut movement, destination),
        "explicit semantic selection remains legal"
    );
    assert_eq!(state.selected_unit, Some(id), "exact existing identity");
    movement.deactivate();
    state.selected_unit = None;
    let mut stationary = state.units[0].clone();
    stationary.id = UnitId(99_999_u32);
    let stationary_id = stationary.id;
    state.units.push(stationary);
    assert!(
        begin_map_unit_drag(&mut state, &mut movement, destination),
        "stationary destination actor remains selectable"
    );
    assert_eq!(
        state.selected_unit,
        Some(stationary_id),
        "never silently choose traveler merely because canonical cell matches"
    );
}
