//! Exclusive manager focus and shared accessible/pointer command dispatch.

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

use super::{MovementInputState, RuntimeAuthority, submit_authoritative_command};
use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::InGameUiModel;

#[cfg(test)]
mod tests;
use crate::gameplay::ui_model::management::ManagerIntent;

/// Consumes all non-window/quit input while a manager owns focus. Returns false
/// when ordinary gameplay/popup dispatch should proceed. A modal click never
/// doubles as an order, end-turn activation, map zoom or edge scroll.
pub(super) fn consume(
    event: &Event,
    state: &mut GameState,
    ui: &mut InGameUiModel,
    movement: &mut MovementInputState,
    authority: Option<&mut RuntimeAuthority>,
) -> bool {
    if ui.management.view.is_none() || !state.popups.is_empty() {
        return false;
    }
    if matches!(*event, Event::Quit { .. } | Event::Window { .. }) {
        return false;
    }
    movement.cancel_drag();
    let intent = match *event {
        Event::KeyDown {
            keycode: Some(Keycode::ESCAPE),
            repeat: false,
            ..
        } => Some(ManagerIntent::Close),
        Event::KeyDown {
            keycode: Some(Keycode::TAB | Keycode::DOWN),
            repeat: false,
            ..
        } => {
            focus(ui, state, 1_i32);
            None
        }
        Event::KeyDown {
            keycode: Some(Keycode::UP),
            repeat: false,
            ..
        } => {
            focus(ui, state, -1);
            None
        }
        Event::KeyDown {
            keycode: Some(Keycode::RETURN | Keycode::SPACE),
            repeat: false,
            ..
        } => ui
            .management
            .focused
            .as_deref()
            .and_then(|id| ui.management.activate_id(id, state)),
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } => {
            let intent = ui.management.intent_at(x, y, state);
            if intent.is_none() {
                if let Some(reason) = ui
                    .management
                    .widgets(state)
                    .into_iter()
                    .rev()
                    .find(|widget| widget.rect.contains(x, y))
                    .and_then(|widget| widget.unavailable)
                {
                    ui.status_text = reason;
                }
            }
            intent
        }
        Event::MouseWheel { y, .. } => ui.pointer.and_then(|(x, pointer_y)| {
            ui.management
                .widgets(state)
                .into_iter()
                .rev()
                .find(|widget| {
                    widget.rect.contains(x, pointer_y)
                        && widget
                            .node
                            .field("objecttype")
                            .is_some_and(|kind| kind.eq_ignore_ascii_case("ctp2_ListBox"))
                })
                .map(|widget| ManagerIntent::Scroll(widget.id, y.saturating_neg()))
        }),
        _ => None,
    };
    if let Some(intent) = intent {
        apply(intent, state, ui, authority);
    }
    true
}

/// Accessible activation and real SDL events execute this exact same dispatcher.
pub(super) fn apply(
    intent: ManagerIntent,
    state: &mut GameState,
    ui: &mut InGameUiModel,
    authority: Option<&mut RuntimeAuthority>,
) {
    match intent {
        ManagerIntent::Submit(command) => {
            if !command.valid_for(state) {
                "This choice belongs to an outdated server revision"
                    .clone_into(&mut ui.status_text);
                return;
            }
            if let Some(authority) =
                authority.filter(|authority| authority.session.revision() == command.revision)
            {
                submit_authoritative_command(authority, state, &command.route, command.payload);
            } else {
                "No matching authoritative session; no game state was changed"
                    .clone_into(&mut ui.status_text);
            }
        }
        ManagerIntent::Open(kind) => {
            let city = ui
                .management
                .view
                .as_ref()
                .and_then(|view| view.city_id.clone());
            ui.management.open(kind, state, city.as_deref());
        }
        local => {
            if ui.management.apply_local(local) && ui.management.view.is_none() {
                ui.great_library = None;
            }
        }
    }
}

/// Opens a newly founded city's Build Manager exactly once after its receipt.
pub(super) fn open_pending_build(state: &mut GameState, ui: &mut InGameUiModel) {
    let Some(id) = state.city_build_menu.take() else {
        return;
    };
    let city = state.viewer.as_ref().and_then(|viewer| {
        viewer
            .cities
            .iter()
            .find(|city| crate::gameplay::model::CityId::from_server_id(&city.id) == id)
    });
    let canonical = city.map(|city| city.id.clone());
    ui.selected_city = Some(id);
    ui.active_tab = crate::gameplay::ui_model::ControlPanelTab::City;
    ui.management.open(
        crate::gameplay::ui_model::management::ManagerKind::Build,
        state,
        canonical.as_deref(),
    );
}

/// Cycles only currently enabled controls; Enter cannot trigger a hidden choice.
fn focus(ui: &mut InGameUiModel, state: &GameState, delta: i32) {
    let widgets: Vec<_> = ui
        .management
        .widgets(state)
        .into_iter()
        .filter(|widget| widget.intent.is_some())
        .collect();
    if widgets.is_empty() {
        ui.management.focused = None;
        return;
    }
    let current = ui
        .management
        .focused
        .as_ref()
        .and_then(|id| widgets.iter().position(|widget| widget.id == *id));
    let next = if delta < 0_i32 {
        current
            .unwrap_or(widgets.len())
            .checked_sub(1)
            .unwrap_or_else(|| widgets.len().saturating_sub(1))
    } else {
        current.map_or(0_usize, |index| {
            index
                .saturating_add(1)
                .checked_rem(widgets.len())
                .unwrap_or(0)
        })
    };
    ui.management.focused = Some(widgets[next].id.clone());
}
