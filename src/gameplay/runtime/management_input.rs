//! Exclusive manager focus and shared accessible/pointer command dispatch.

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use sdl2::mouse::MouseButton;

use super::{MovementInputState, RuntimeAuthority, submit_authoritative_command};
use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::InGameUiModel;

/// Clipboard access remains injectable; editing itself is pure presentation.
pub(super) mod editor_input;
/// Fileless native-event text editing regressions.
#[cfg(test)]
mod editor_tests;
/// Source thumb capture and keyboard range behavior.
mod scroll_input;
/// Source ranger capture and accessible-value regressions.
#[cfg(test)]
mod scroll_tests;
/// Detached dropdown focus, keyboard navigation and cancellation.
mod selector_input;
/// Fileless source city-selection journeys and stale-receipt validation.
#[cfg(test)]
mod selector_tests;
#[cfg(test)]
mod tests;
use crate::gameplay::ui_model::management::{ManagerIntent, ManagerKind};

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
    if matches!(
        *event,
        Event::Window {
            win_event: sdl2::event::WindowEvent::FocusLost | sdl2::event::WindowEvent::Hidden,
            ..
        }
    ) {
        ui.management.search_dragging = false;
        ui.management.scroll_drag = None;
        ui.management.pointer_extend = false;
        ui.management.clipboard_request = None;
        ui.management.editor.clear_composition();
    }
    if matches!(*event, Event::Quit { .. } | Event::Window { .. }) {
        selector_input::consume(event, ui, state);
        return false;
    }
    movement.cancel_drag();
    if scroll_input::consume(event, ui, state) {
        return true;
    }
    if selector_input::consume(event, ui, state) {
        return true;
    }
    if editor_input::consume(event, ui, state) {
        return true;
    }
    let intent = match event.clone() {
        Event::KeyDown {
            keycode: Some(Keycode::ESCAPE),
            repeat: false,
            ..
        } => Some(ManagerIntent::Close),
        Event::KeyDown {
            keycode: Some(Keycode::TAB | Keycode::DOWN),
            keymod,
            repeat: false,
            ..
        } => {
            focus(
                ui,
                state,
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    -1_i32
                } else {
                    1_i32
                },
            );
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
        } => pointer_intent(ui, state, x, y),
        Event::MouseWheel { y, .. } => ui.pointer.and_then(|(x, pointer_y)| {
            ui.management
                .widgets(state)
                .into_iter()
                .rev()
                .find(|widget| {
                    widget.rect.contains(x, pointer_y)
                        && widget.node.field("objecttype").is_some_and(|kind| {
                            kind.eq_ignore_ascii_case("ctp2_ListBox")
                                || kind.eq_ignore_ascii_case("ctp2_hypertextbox")
                        })
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

/// Shared pointer descriptors also expose explicit disabled reasons; empty
/// decorative children cannot hide the enclosing disabled control's explanation.
fn pointer_intent(
    ui: &mut InGameUiModel,
    state: &GameState,
    x: i32,
    y: i32,
) -> Option<ManagerIntent> {
    let intent = ui.management.intent_at(x, y, state);
    ui.management.focused = None;
    ui.management.search_dragging = false;
    ui.management.scroll_drag = None;
    ui.management.editor.clear_composition();
    if intent == Some(ManagerIntent::Focus("SearchWord".to_owned())) {
        if let Some(offset) = editor_input::caret_offset(ui, state, x) {
            ui.management.search_dragging = true;
            return Some(ManagerIntent::EditCaret(
                offset,
                ui.management.pointer_extend,
            ));
        }
    }
    if intent.is_none() {
        if let Some(reason) = ui
            .management
            .widgets(state)
            .into_iter()
            .rev()
            .find(|widget| widget.rect.contains(x, y) && widget.unavailable.is_some())
            .and_then(|widget| widget.unavailable)
        {
            ui.status_text = reason;
        }
    }
    intent
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
        ManagerIntent::Open(kind) => ui.management.open_related(kind, state),
        ManagerIntent::ScrollTo(path, value) => {
            ui.management.scroll_to(&path, value, state);
        }
        ManagerIntent::BeginScroll(path) => {
            ui.management
                .begin_scroll(&path, ui.pointer.map(|point| point.1), state);
            ui.management.focused = Some(format!("{path}/ranger/thumb"));
        }
        ManagerIntent::Scroll(path, delta) => {
            ui.management.scroll(&path, delta, state);
        }
        ManagerIntent::Select(key, id) if key == "city" => {
            if !ui.management.select_city(&id, state) {
                "The selected city is no longer owned or visible".clone_into(&mut ui.status_text);
            }
        }
        ManagerIntent::Article(id) => {
            if ui.management.library.navigate(&id) {
                ui.management.open_related(ManagerKind::Library, state);
                ui.management.apply_local(ManagerIntent::Article(id));
            } else {
                ui.status_text = format!("Article {id} is not installed");
            }
        }
        local => {
            if ui.management.apply_local(local)
                && ui
                    .management
                    .view
                    .as_ref()
                    .is_none_or(|view| view.kind != ManagerKind::Library)
            {
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
