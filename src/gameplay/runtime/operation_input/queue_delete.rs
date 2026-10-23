//! Strongly modal original Delete question, no implicit affirmative or file I/O.
use crate::gameplay::ui_model::{InGameUiModel, management::saved::save};
use sdl2::{
    event::Event,
    keyboard::{Keycode, Mod},
    mouse::MouseButton,
};
/* Released ui/interface/EditQueue.cpp::DeleteCallback -> MessageBoxDialog.cpp::
 * Query/LeftButtonActionCallback/RightButtonActionCallback -> DeleteQueryCallback.
 * messagebox.ldl source MB_OK/MB_CANCEL rectangles/fonts/hit order are retained.
 * Explicit original OK only, no default Enter or destructive Don'tShow replay.
 * File approval queues one captured local effect, not a server/game command.
 */
/// Owns modal input before all map/order/name editing, letting Quit propagate.
pub(super) fn consume(event: &Event, ui: &mut InGameUiModel) -> bool {
    match event.clone() {
        Event::Quit { .. } => return false,
        Event::KeyDown {
            keycode: Some(Keycode::ESCAPE),
            repeat: false,
            ..
        } => {
            activate(ui, save::CANCEL);
        }
        Event::KeyDown {
            keycode: Some(code),
            keymod,
            repeat: false,
            ..
        } => {
            if !keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD | Mod::LALTMOD | Mod::RALTMOD) {
                match code {
                    Keycode::TAB | Keycode::LEFT | Keycode::RIGHT | Keycode::UP | Keycode::DOWN => {
                        focus(
                            ui,
                            keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD)
                                || matches!(code, Keycode::LEFT | Keycode::UP),
                        );
                    }
                    Keycode::RETURN | Keycode::KP_ENTER | Keycode::SPACE => {
                        if let Some(id) = ui
                            .operation
                            .as_ref()
                            .and_then(|dialog| dialog.confirmation_focus.clone())
                        {
                            activate(ui, &id);
                        }
                    }
                    _ => {}
                }
            }
        }
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } => {
            let id = ui.operation.as_ref().and_then(|dialog| {
                dialog
                    .confirmation
                    .iter()
                    .rev()
                    .find(|widget| widget.rect.contains(x, y) && widget.intent.is_some())
                    .map(|widget| widget.id.clone())
            });
            if let Some(id) = id {
                activate(ui, &id);
            }
        }
        _ => {}
    }
    true
}
/// Original source IDs are the same accessible and native approval/cancel boundary.
pub(in crate::gameplay::runtime) fn activate(ui: &mut InGameUiModel, id: &str) -> bool {
    let Some(dialog) = ui
        .operation
        .as_ref()
        .filter(|dialog| dialog.queue_delete.is_some())
    else {
        return false;
    };
    if !dialog
        .confirmation
        .iter()
        .any(|widget| widget.id == id && widget.intent.is_some())
    {
        return false;
    }
    if id == save::CANCEL {
        ui.operation = None;
        return true;
    }
    let Some(capture) = dialog.queue_delete.as_ref() else {
        return false;
    };
    if id != save::OK || capture.approved {
        return false;
    }
    if !capture.current(&ui.management) {
        "The selected profile file/preview has changed; no deletion was requested"
            .clone_into(&mut ui.status_text);
        return false;
    }
    if let Some(approved) = ui
        .operation
        .as_mut()
        .and_then(|form| form.queue_delete.as_mut())
    {
        approved.approved = true;
    }
    true
}
/// Original visual left-to-right Cancel then OK, with initially no selected button.
fn focus(ui: &mut InGameUiModel, reverse: bool) {
    let Some(dialog) = ui.operation.as_mut() else {
        return;
    };
    let selected = dialog.confirmation_focus.as_deref();
    let next = if selected == Some(save::CANCEL) {
        save::OK
    } else if selected == Some(save::OK) || !reverse {
        save::CANCEL
    } else {
        save::OK
    };
    dialog.confirmation_focus = Some(next.to_owned());
}
