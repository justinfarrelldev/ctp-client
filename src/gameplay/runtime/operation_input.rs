//! Modal input/target confirmation sharing semantic commands with native controls.
use super::management_input::editor_input::Clipboard;
use super::{RuntimeAuthority, submit_authoritative_command};
use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::InGameUiModel;
use crate::gameplay::ui_model::library::editor::{ClipboardRequest, Motion};
use crate::gameplay::ui_model::management::ManagerCommand;
use crate::gameplay::ui_model::operations::{InputKind, OperationDialog};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{Keycode, Mod};
use sdl2::mouse::MouseButton;

/// Consumes form events before map shortcuts; Quit still propagates to the session.
pub(super) fn consume(
    event: &Event,
    state: &mut GameState,
    ui: &mut InGameUiModel,
    authority: Option<&mut RuntimeAuthority>,
) -> bool {
    let Some(dialog) = ui.operation.as_mut() else {
        return false;
    };
    let (close, selected) = match event.clone() {
        Event::Quit { .. } => return false,
        Event::TextInput { text, .. } if dialog.input.is_some() => {
            let maximum = limit(dialog);
            dialog
                .editor
                .insert_with_limit(&mut dialog.text, &text, maximum);
            (false, None)
        }
        Event::TextEditing {
            text,
            start,
            length,
            ..
        } if dialog.input.is_some() => {
            dialog.editor.compose(&text, start, length);
            (false, None)
        }
        Event::KeyDown {
            keycode: Some(code),
            keymod,
            repeat: false,
            ..
        } => key(dialog, state, code, keymod),
        Event::MouseWheel { y, .. } => {
            if y < 0_i32 {
                dialog.start = dialog.start.saturating_add(8).min(
                    dialog
                        .choices
                        .len()
                        .saturating_sub(1)
                        .div_euclid(8)
                        .saturating_mul(8),
                );
            } else {
                dialog.start = dialog.start.saturating_sub(8);
            }
            dialog.focused = dialog.start;
            (false, None)
        }
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } => pointer(dialog, state, x, y),
        Event::Window {
            win_event: WindowEvent::FocusLost,
            ..
        } => {
            dialog.clipboard = None;
            dialog.editor.clear_composition();
            (false, None)
        }
        _ => (false, None),
    };
    let submitted = selected
        .filter(|command| command.valid_for(state))
        .is_some_and(|command| {
            if matches!(
                command.route.as_str(),
                "propose-agreement" | "respond-agreement"
            ) {
                ui.management.open_diplomatic_review(state, command)
            } else {
                authority.is_some_and(|active| {
                    submit_authoritative_command(active, state, &command.route, command.payload);
                    true
                })
            }
        });
    if close || submitted {
        ui.operation = None;
    }
    true
}

/// Original Unicode editing and explicit keyboard choice acceptance.
fn key(
    dialog: &mut OperationDialog,
    state: &GameState,
    key: Keycode,
    modifiers: Mod,
) -> (bool, Option<ManagerCommand>) {
    let ctrl =
        modifiers.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD) && !modifiers.contains(Mod::RALTMOD);
    let shift = modifiers.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD);
    match key {
        Keycode::ESCAPE if !dialog.editor.composition.is_empty() => {
            dialog.editor.clear_composition();
        }
        Keycode::ESCAPE => return (true, None),
        Keycode::RETURN | Keycode::KP_ENTER if dialog.editor.composition.is_empty() => {
            return (
                false,
                if dialog.input.is_some() {
                    dialog.command(state)
                } else {
                    dialog
                        .choices
                        .get(dialog.focused)
                        .filter(|choice| choice.command.valid_for(state))
                        .map(|choice| choice.command.clone())
                },
            );
        }
        Keycode::A if ctrl => dialog.editor.select_all(&dialog.text),
        Keycode::C | Keycode::X if ctrl => {
            dialog.clipboard = Some(if key == Keycode::C {
                ClipboardRequest::Copy(dialog.editor.selected(&dialog.text).to_owned())
            } else {
                ClipboardRequest::Cut {
                    value: dialog.text.clone(),
                    range: dialog.editor.range(&dialog.text),
                }
            });
        }
        Keycode::V if ctrl => dialog.clipboard = Some(ClipboardRequest::Paste),
        Keycode::Z if ctrl => {
            if shift {
                dialog.editor.redo(&mut dialog.text);
            } else {
                dialog.editor.undo(&mut dialog.text);
            }
        }
        Keycode::Y if ctrl => {
            dialog.editor.redo(&mut dialog.text);
        }
        Keycode::BACKSPACE | Keycode::DELETE => {
            dialog
                .editor
                .erase(&mut dialog.text, key == Keycode::BACKSPACE, ctrl);
        }
        Keycode::LEFT | Keycode::RIGHT | Keycode::HOME | Keycode::END => {
            let motion = match (key, ctrl) {
                (Keycode::LEFT, true) => Motion::WordLeft,
                (Keycode::LEFT, false) => Motion::Left,
                (Keycode::RIGHT, true) => Motion::WordRight,
                (Keycode::RIGHT, false) => Motion::Right,
                (Keycode::HOME, _) => Motion::Home,
                _ => Motion::End,
            };
            dialog.editor.move_caret(&dialog.text, motion, shift);
        }
        Keycode::UP | Keycode::DOWN if !dialog.choices.is_empty() => {
            if key == Keycode::UP {
                dialog.focused = dialog.focused.saturating_sub(1);
            } else {
                dialog.focused = dialog
                    .focused
                    .saturating_add(1)
                    .min(dialog.choices.len().saturating_sub(1));
            }
            dialog.start = dialog.focused.div_euclid(8).saturating_mul(8);
        }
        _ => {}
    }
    (false, None)
}

/// Uses the same row and field rectangles as native painting.
fn pointer(
    dialog: &mut OperationDialog,
    state: &GameState,
    x: i32,
    y: i32,
) -> (bool, Option<ManagerCommand>) {
    if OperationDialog::cancel().contains(x, y) {
        return (true, None);
    }
    if OperationDialog::accept().contains(x, y) {
        return (false, dialog.command(state));
    }
    if dialog.input.is_some() && OperationDialog::field().contains(x, y) {
        let offset = dialog.metrics.hit(
            &dialog.text,
            dialog.editor.caret_offset(&dialog.text),
            OperationDialog::field().width,
            x.saturating_sub(OperationDialog::field().x),
        );
        dialog.editor.set_caret(&dialog.text, offset, false);
        return (false, None);
    }
    let selected = dialog
        .choices
        .iter()
        .enumerate()
        .find(|entry| {
            dialog.row(entry.0).is_some_and(|rect| rect.contains(x, y))
                && entry.1.command.valid_for(state)
        })
        .map(|entry| entry.1.command.clone());
    (false, selected)
}

/// Parameter field bounds; queue text remains below the server body cap.
const fn limit(dialog: &OperationDialog) -> usize {
    if matches!(dialog.input, Some(InputKind::Rename)) {
        128
    } else {
        0x4000_usize
    }
}

/// Performs explicit clipboard effects through an injected boundary; tests use fakes.
pub(super) fn flush(ui: &mut InGameUiModel, clipboard: &mut impl Clipboard) {
    let Some(dialog) = ui.operation.as_mut() else {
        return;
    };
    let Some(request) = dialog.clipboard.take() else {
        return;
    };
    let result = match request {
        ClipboardRequest::Copy(text) => clipboard.write(&text),
        ClipboardRequest::Paste => clipboard.read().map(|text| {
            let maximum = limit(dialog);
            dialog
                .editor
                .insert_with_limit(&mut dialog.text, &text, maximum);
        }),
        ClipboardRequest::Cut { value, range } => clipboard
            .write(value.get(range.clone()).unwrap_or(""))
            .map(|()| {
                if value == dialog.text && range == dialog.editor.range(&dialog.text) {
                    dialog.editor.insert(&mut dialog.text, "");
                }
            }),
    };
    if let Err(error) = result {
        ui.status_text = format!("Clipboard: {error}");
    }
}
