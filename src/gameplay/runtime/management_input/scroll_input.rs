//! Pointer and keyboard capture for source ranger value controls, never map input.

use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::InGameUiModel;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

/// Handles only captured thumbs or a focused ranger's actual keyboard value API.
pub(super) fn consume(event: &Event, ui: &mut InGameUiModel, state: &GameState) -> bool {
    if ui.management.scroll_drag.is_some() {
        match *event {
            Event::MouseMotion { y, mousestate, .. } => {
                if mousestate.left() {
                    ui.management.drag_scroll(y, state);
                } else {
                    ui.management.scroll_drag = None;
                }
                return true;
            }
            Event::MouseButtonUp {
                mouse_btn: MouseButton::Left,
                ..
            } => {
                ui.management.scroll_drag = None;
                return true;
            }
            _ => {}
        }
    }
    let Event::KeyDown {
        keycode: Some(code),
        ..
    } = *event
    else {
        return false;
    };
    let Some(id) = ui.management.focused.as_ref() else {
        return false;
    };
    let Some(thumb) = ui
        .management
        .widgets(state)
        .into_iter()
        .find(|widget| widget.id == *id && widget.node.field("scroll-owner").is_some())
    else {
        return false;
    };
    let Some(owner) = thumb.node.field("scroll-owner").map(str::to_owned) else {
        return false;
    };
    let page = thumb
        .node
        .field("scroll-page")
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(1_i32);
    let maximum = thumb
        .node
        .field("scroll-maximum")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0_usize);
    match code {
        Keycode::UP => {
            ui.management.scroll(&owner, -1_i32, state);
        }
        Keycode::DOWN => {
            ui.management.scroll(&owner, 1_i32, state);
        }
        Keycode::PAGEUP => {
            ui.management.scroll(&owner, page.saturating_neg(), state);
        }
        Keycode::PAGEDOWN => {
            ui.management.scroll(&owner, page, state);
        }
        Keycode::HOME => {
            ui.management.scroll_to(&owner, 0_usize, state);
        }
        Keycode::END => {
            ui.management.scroll_to(&owner, maximum, state);
        }
        _ => return false,
    }
    true
}
