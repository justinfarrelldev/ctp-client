//! Source message selection, Close and permitted eye navigation.
use super::{Camera, MovementInputState};
use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::{ControlPanelTab, InGameUiModel, MenuRect, messages::MessageLog};
use sdl2::{event::Event, keyboard::Keycode, mouse::MouseButton};

/// Displays received snapshot facts; immutable state and no transport access
/// prevent historical results from acknowledging or reopening live recovery.
pub(super) fn refresh(
    state: &GameState,
    ui: &mut InGameUiModel,
    movement: &mut MovementInputState,
) {
    let Some(viewer) = state.viewer.as_ref() else {
        return;
    };
    if ui.messages.revision == Some(viewer.revision) {
        return;
    }
    ui.messages.revision = Some(viewer.revision);
    if let Some(date) = viewer
        .interface
        .calendar
        .as_ref()
        .filter(|date| date.turn == viewer.turn)
    {
        ui.messages.dates.insert(date.turn, date.label());
        while ui.messages.dates.len() > 256 {
            ui.messages.dates.pop_first();
        }
    }
    let fresh = ui.messages.ingest(&viewer.interface.notifications);
    if let Some(notice) = fresh.iter().find(|notice| notice.kind == "ruin.explored") {
        ui.messages.open(&notice.id);
        movement.cancel_drag();
    }
}
/// Exact source Close/eye rectangles shared with accessible activation and painting.
pub(super) fn close_rect(log: &MessageLog) -> MenuRect {
    let frame = log.window();
    MenuRect {
        x: frame
            .x
            .saturating_add(i32::try_from(frame.width).unwrap_or(0))
            .saturating_sub(64),
        y: frame
            .y
            .saturating_add(i32::try_from(frame.height).unwrap_or(0))
            .saturating_sub(34),
        width: 48,
        height: 20,
    }
}
/// Original `StandardEyeButton` offset, 33x33, bottom-left padding20.
pub(super) fn eye_rect(log: &MessageLog) -> MenuRect {
    let frame = log.window();
    MenuRect {
        x: frame.x.saturating_add(20),
        y: frame
            .y
            .saturating_add(i32::try_from(frame.height).unwrap_or(0))
            .saturating_sub(53),
        width: 33,
        height: 33,
    }
}
/// Semantic IDs do not depend on screen coordinates and are safe to use in E2E tests.
pub(super) fn activate(
    id: &str,
    state: &GameState,
    ui: &mut InGameUiModel,
    camera: &mut Camera,
) -> bool {
    match id {
        "message-close" => {
            ui.messages.selected = None;
            true
        }
        "message-scroll-up" => {
            ui.messages.scroll(-1);
            true
        }
        "message-scroll-down" => {
            ui.messages.scroll(1);
            true
        }
        "message-eye" => {
            if let Some(point) = ui.messages.location(state) {
                camera.center_on(point);
                ui.messages.selected = None;
                true
            } else {
                false
            }
        }
        _ => {
            if let Some(notice) = id.strip_prefix("message/") {
                if ui.messages.items.iter().any(|item| item.id == notice) {
                    ui.messages.open(notice);
                    return true;
                }
            }
            false
        }
    }
}
/// Original ranger descriptors drive arrows, pages and thumb capture exactly.
fn ranger(event: &Event, state: &GameState, ui: &mut InGameUiModel) -> bool {
    match *event {
        Event::MouseButtonUp {
            mouse_btn: MouseButton::Left,
            ..
        }
        | Event::Window {
            win_event: sdl2::event::WindowEvent::FocusLost,
            ..
        } => ui.messages.drag.take().is_some(),
        Event::MouseMotion { y, .. } if ui.messages.drag.is_some() => {
            if let Some(drag) = ui.messages.drag {
                let displacement =
                    u32::try_from(y.saturating_sub(drag.grab).saturating_sub(drag.top))
                        .unwrap_or(0)
                        .min(drag.travel);
                let maximum = ui.messages.items.len().saturating_sub(5);
                ui.messages.start = usize::try_from(
                    u128::from(displacement)
                        .saturating_mul(u128::try_from(maximum).unwrap_or(0))
                        .div_euclid(u128::from(drag.travel.max(1))),
                )
                .unwrap_or(maximum);
            }
            true
        }
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } => {
            let widgets = ui.management.panel_widgets(state, ui);
            let Some(widget) = widgets.iter().rev().find(|widget| {
                widget.id.starts_with("MessageListBox/")
                    && widget.rect.contains(x, y)
                    && widget.intent.is_some()
            }) else {
                return false;
            };
            match widget.intent.clone() {
                Some(crate::gameplay::ui_model::management::ManagerIntent::Scroll(
                    _path,
                    delta,
                )) => ui.messages.scroll(delta),
                Some(crate::gameplay::ui_model::management::ManagerIntent::BeginScroll(_path)) => {
                    if let Some(track) = widgets
                        .iter()
                        .find(|candidate| candidate.id == "MessageListBox/ranger/track")
                    {
                        ui.messages.drag = Some(crate::gameplay::ui_model::messages::MessageDrag {
                            grab: y.saturating_sub(widget.rect.y),
                            top: track.rect.y,
                            travel: track.rect.height.saturating_sub(widget.rect.height),
                        });
                    }
                }
                _ => return false,
            }
            true
        }
        _ => false,
    }
}

/// A message popup consumes orders/turn input; the list consumes only its own area.
pub(super) fn consume(
    event: &Event,
    state: &GameState,
    ui: &mut InGameUiModel,
    camera: &mut Camera,
) -> bool {
    if ui.messages.current().is_some() {
        match *event {
            Event::Quit { .. } => return false,
            Event::KeyDown {
                keycode: Some(Keycode::ESCAPE | Keycode::RETURN | Keycode::KP_ENTER),
                repeat: false,
                ..
            } => {
                activate("message-close", state, ui, camera);
            }
            Event::MouseButtonDown {
                mouse_btn: MouseButton::Left,
                x,
                y,
                ..
            } => {
                if close_rect(&ui.messages).contains(x, y) {
                    activate("message-close", state, ui, camera);
                } else if eye_rect(&ui.messages).contains(x, y) {
                    activate("message-eye", state, ui, camera);
                }
            }
            _ => {}
        }
        return true;
    }
    if ui.active_tab != ControlPanelTab::Messages {
        ui.messages.drag = None;
        return false;
    }
    if ranger(event, state, ui) {
        return true;
    }
    match *event {
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } if MessageLog::list().contains(x, y) => {
            if let Some(id) = ui
                .messages
                .items
                .iter()
                .enumerate()
                .find(|entry| {
                    ui.messages
                        .row(entry.0)
                        .is_some_and(|row| row.contains(x, y))
                })
                .map(|entry| entry.1.id.clone())
            {
                ui.messages.open(&id);
            }
            true
        }
        Event::MouseWheel { y, .. }
            if ui.pointer.is_some_and(|(pointer_x, pointer_y)| {
                MessageLog::list().contains(pointer_x, pointer_y)
            }) =>
        {
            ui.messages.scroll(if y < 0 { 1 } else { -1 });
            true
        }
        _ => false,
    }
}
