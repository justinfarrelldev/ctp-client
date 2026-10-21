//! Explicit context-menu selection; never issue commands merely by opening a menu.

use super::{
    MovementInputState, RuntimeAuthority, begin_or_activate_unit_drag, submit_authoritative_command,
};
use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::context::ContextIntent;
use crate::gameplay::ui_model::{ControlPanelTab, InGameUiModel};

/// Applies one chosen semantic row. Server actions are revalidated against the
/// current viewer revision, so a menu cannot replay a stale legal payload.
pub(super) fn apply_context_intent(
    intent: ContextIntent,
    state: &mut GameState,
    ui: &mut InGameUiModel,
    movement: &mut MovementInputState,
    authority: Option<&mut RuntimeAuthority>,
) {
    match intent {
        ContextIntent::Dialog(dialog) => ui.operation = Some(*dialog),
        ContextIntent::Command(command) => {
            if command.valid_for(state) {
                if let Some(authority) = authority {
                    submit_authoritative_command(authority, state, &command.route, command.payload);
                }
            }
        }
        ContextIntent::City(id) => {
            ui.selected_city = Some(id);
            ui.active_tab = ControlPanelTab::City;
            open_city(
                ui,
                state,
                id,
                crate::gameplay::ui_model::management::ManagerKind::City,
            );
        }
        ContextIntent::Build(id) => {
            ui.selected_city = Some(id);
            state.city_build_menu = Some(id);
            ui.active_tab = ControlPanelTab::City;
            open_city(
                ui,
                state,
                id,
                crate::gameplay::ui_model::management::ManagerKind::Build,
            );
        }
        ContextIntent::Army(id) => {
            if state.select_unit(id) {
                ui.active_tab = ControlPanelTab::Unit;
                ui.management.open(
                    crate::gameplay::ui_model::management::ManagerKind::Army,
                    state,
                    None,
                );
            }
        }
        ContextIntent::Move(id) => {
            if state.select_unit(id) {
                if let Some(point) = state.selected_unit().map(|unit| unit.position) {
                    begin_or_activate_unit_drag(state, movement, point);
                }
            }
        }
        ContextIntent::MapInfo(point) => ui.map_info = Some(point),
        ContextIntent::Submit(action) => {
            let still_legal = action.http_supported()
                && state
                    .viewer
                    .as_ref()
                    .is_some_and(|viewer| viewer.interface.legal_actions.contains(&action));
            if still_legal {
                if let Some(authority) = authority {
                    submit_authoritative_command(authority, state, &action.route, action.payload);
                }
            } else {
                "This order is no longer legal in the current server revision"
                    .clone_into(&mut ui.status_text);
            }
        }
        ContextIntent::Unavailable(reason) => reason.clone_into(&mut ui.status_text),
    }
}

/// Resolves a clicked city's canonical identity instead of opening the first city.
fn open_city(
    ui: &mut InGameUiModel,
    state: &GameState,
    id: crate::gameplay::model::CityId,
    kind: crate::gameplay::ui_model::management::ManagerKind,
) {
    let city = state.viewer.as_ref().and_then(|viewer| {
        viewer
            .cities
            .iter()
            .find(|city| crate::gameplay::model::CityId::from_server_id(&city.id) == id)
    });
    ui.management
        .open(kind, state, city.map(|city| city.id.as_str()));
}

/// Consumes keyboard input while a context/info window owns focus, preventing
/// End Turn/order shortcuts from firing through it. Enter is an explicit choice.
pub(super) fn handle_key(
    key: sdl2::keyboard::Keycode,
    state: &mut GameState,
    ui: &mut InGameUiModel,
    movement: &mut MovementInputState,
    authority: Option<&mut RuntimeAuthority>,
) {
    use sdl2::keyboard::Keycode;
    if ui.map_info.is_some() {
        if matches!(key, Keycode::ESCAPE | Keycode::RETURN) {
            ui.map_info = None;
        }
        return;
    }
    match key {
        Keycode::ESCAPE => ui.context = None,
        Keycode::UP => {
            if let Some(menu) = ui.context.as_mut() {
                menu.move_focus(-1);
            }
        }
        Keycode::DOWN => {
            if let Some(menu) = ui.context.as_mut() {
                menu.move_focus(1);
            }
        }
        Keycode::RETURN => {
            if let Some(intent) = ui.context.take().and_then(|menu| menu.focused_intent()) {
                apply_context_intent(intent, state, ui, movement, authority);
            }
        }
        _ => {}
    }
}

/// Original centered terrain-information geometry in logical coordinates.
pub(super) const fn map_info_close_rect() -> crate::gameplay::ui_model::MenuRect {
    crate::gameplay::ui_model::MenuRect {
        x: 587,
        y: 347,
        width: 64,
        height: 20,
    }
}

/// Retains the camera argument type in event handlers without changing map state.
#[cfg(test)]
mod tests {
    use super::super::Camera;
    use super::super::{handle_left_click, handle_right_click};
    use super::{GameState, InGameUiModel, MovementInputState};

    #[test]
    fn context_keyboard_navigation_is_explicit_and_blocks_underlying_turn_shortcuts() {
        let mut state = GameState::new(24, 18, 1, 42);
        state.popups.clear();
        let point = state.selected_unit().expect("starter").position;
        let mut ui = InGameUiModel {
            context: crate::gameplay::ui_model::context::ContextView::new(&state, point, 400, 200),
            ..InGameUiModel::default()
        };
        let mut movement = MovementInputState::default();
        let before = state.clone();
        super::handle_key(
            sdl2::keyboard::Keycode::A,
            &mut state,
            &mut ui,
            &mut movement,
            None,
        );
        assert_eq!(
            state, before,
            "End Turn shortcut cannot pass through an open context"
        );
        super::handle_key(
            sdl2::keyboard::Keycode::DOWN,
            &mut state,
            &mut ui,
            &mut movement,
            None,
        );
        assert_eq!(
            ui.context.as_ref().expect("menu").focused,
            1_usize,
            "focus Move, not disabled rows"
        );
        super::handle_key(
            sdl2::keyboard::Keycode::RETURN,
            &mut state,
            &mut ui,
            &mut movement,
            None,
        );
        assert!(ui.context.is_none(), "explicit selection closes menu");
        assert_eq!(
            movement.active_unit(),
            state.selected_unit,
            "Enter activates Move without moving"
        );
        assert_eq!(
            state.units, before.units,
            "navigation and Move activation do not move units"
        );
    }

    #[test]
    fn opening_unit_context_never_executes_first_row_or_moves_units() {
        let mut state = GameState::new(24, 18, 1, 42);
        state.popups.clear();
        let mut ui = InGameUiModel::default();
        let mut camera = Camera::centered_on_selected(&state);
        let position = state.selected_unit().expect("starter").position;
        let (x, y) = camera.tile_click_center_to_screen(position);
        let before = state.clone();
        let mut movement = MovementInputState::default();
        assert_eq!(
            handle_right_click(x, y, &mut state, &mut ui, &camera, &mut movement, None),
            None,
            "opening context is not terminal"
        );
        assert_eq!(
            state, before,
            "opening context never dispatches or mutates model"
        );
        let menu = ui.context.as_ref().expect("visible unit menu");
        let intent = menu.activate("Move").expect("semantic Move");
        let rect = menu.row_rect(1).expect("Move row");
        assert_eq!(
            menu.intent_at(rect.x.saturating_add(2), rect.y.saturating_add(2)),
            Some(intent),
            "semantic and pointer selection agree"
        );
        assert_eq!(
            handle_left_click(
                700,
                100,
                &mut state,
                &mut ui,
                &mut camera,
                &mut movement,
                None
            ),
            None,
            "outside click cancels menu"
        );
        assert!(ui.context.is_none(), "menu dismissed");
        assert_eq!(
            state, before,
            "cancel click cannot click through into world"
        );
    }
}
