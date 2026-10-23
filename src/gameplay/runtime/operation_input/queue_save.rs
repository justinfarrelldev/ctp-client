//! Source Save-name native/accessibility input; approval only queues local I/O.
use crate::gameplay::{
    model::GameState,
    ui_model::{InGameUiModel, management::saved::save},
};
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::{Keycode, Mod},
    mouse::MouseButton,
};

/* Source: ctp2_source/ctp2_code/ui/interface/MessageBoxDialog.cpp::{TextQuery,
 * TextFieldActionCallback,LeftButtonActionCallback,RightButtonActionCallback}
 * and EditQueue.cpp::SaveNameResponse; messagebox.ldl::TextField/LeftButton/
 * RightButton geometry and source draw order own both native and semantic hits.
 * Enter in field explicitly approves (source EXECUTE); ESC/Cancel keeps local
 * Custom work. Client Unicode selection/IME/clipboard is shared with other forms.
 * Source truncation/static buffers and error-dismissal are not ported: invalid,
 * stale or failed saves retain the editable name and immutable draft. Deferred
 * approval is not filesystem success and never changes gameplay Pending/state.
 */
/// Owns modal input before manager/map; no files or server services are called.
pub(super) fn consume(event: &Event, state: &GameState, ui: &mut InGameUiModel) -> bool {
    if matches!(*event, Event::Quit { .. }) {
        return false;
    }
    let pending = ui
        .operation
        .as_ref()
        .and_then(|dialog| dialog.queue_save.as_ref())
        .is_some_and(|capture| capture.name.is_some());
    match event.clone() {
        Event::KeyDown {
            keycode: Some(Keycode::ESCAPE),
            repeat: false,
            ..
        } => {
            if let Some(dialog) = ui
                .operation
                .as_mut()
                .filter(|dialog| !dialog.editor.composition.is_empty())
            {
                dialog.editor.clear_composition();
            } else {
                activate(ui, save::CANCEL);
            }
        }
        Event::KeyDown {
            keycode: Some(code),
            keymod,
            repeat: false,
            ..
        } if !pending => key(ui,state,code,keymod),
        Event::TextInput { text, .. } if !pending => {
            if let Some(dialog) = ui
                .operation
                .as_mut()
                .filter(|dialog| dialog.confirmation_focus.as_deref() == Some(save::FIELD))
            {
                dialog
                    .editor
                    .insert_with_limit(&mut dialog.text, &text, save::MAX_NAME);
            }
        }
        Event::TextEditing {
            text,
            start,
            length,
            ..
        } if !pending => {
            if let Some(dialog) = ui
                .operation
                .as_mut()
                .filter(|dialog| dialog.confirmation_focus.as_deref() == Some(save::FIELD))
            {
                dialog.editor.compose(&text, start, length);
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
                if id == save::FIELD && !pending {
                    if let Some(dialog) = ui.operation.as_mut() {
                        let field = dialog.field_rect();
                        let offset = dialog.metrics.hit(
                            &dialog.text,
                            dialog.editor.caret_offset(&dialog.text),
                            field.width,
                            x.saturating_sub(field.x),
                        );
                        dialog.editor.set_caret(&dialog.text, offset, false);
                    }
                }
                activate(ui, &id);
            }
        }
        Event::Window {
            win_event: WindowEvent::FocusLost | WindowEvent::Hidden,
            ..
        } => {
            if let Some(dialog) = ui.operation.as_mut() {
                dialog.editor.clear_composition();
                dialog.clipboard = None;
            }
        }
        _ => {}
    }
    true
}
/// Source field editing and explicit keyboard approval stay distinct from buttons.
fn key(ui:&mut InGameUiModel,state:&GameState,code:Keycode,keymod:Mod){
    let focus=ui.operation.as_ref().and_then(|dialog|dialog.confirmation_focus.clone());
    if code==Keycode::TAB {focus_next(ui,keymod.intersects(Mod::LSHIFTMOD|Mod::RSHIFTMOD));}
    else if code==Keycode::RETURN || code==Keycode::KP_ENTER || (code==Keycode::SPACE && focus.as_deref()!=Some(save::FIELD)) {
        let target=match focus.as_deref(){Some(save::FIELD|save::OK)=>Some(save::OK),Some(save::CANCEL)=>Some(save::CANCEL),_=>None};
        if !keymod.intersects(Mod::LCTRLMOD|Mod::RCTRLMOD|Mod::LALTMOD|Mod::RALTMOD) && ui.operation.as_ref().is_some_and(|dialog|dialog.editor.composition.is_empty()){if let Some(target)=target{activate(ui,target);}}
    }else if focus.as_deref()==Some(save::FIELD){if let Some(dialog)=ui.operation.as_mut(){super::key(dialog,state,code,keymod);}}
}

/// Same exact original source IDs for actual native and accessible activation.
pub(in crate::gameplay::runtime) fn activate(ui: &mut InGameUiModel, id: &str) -> bool {
    let Some(dialog) = ui
        .operation
        .as_ref()
        .filter(|dialog| dialog.queue_save.is_some())
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
    let Some(capture) = dialog.queue_save.as_ref() else {
        return false;
    };
    if capture.name.is_some() {
        return false;
    }
    if id == save::FIELD {
        if let Some(focused) = ui.operation.as_mut() {
            focused.confirmation_focus = Some(id.to_owned());
        }
        return true;
    }
    if id != save::OK || !dialog.editor.composition.is_empty() {
        return false;
    }
    if !capture.current(&ui.management) {
        "This Save belongs to an old local template; no file was changed"
            .clone_into(&mut ui.status_text);
        return false;
    }
    if !save::name_valid(&dialog.text) {
        "Enter a safe flat queue name (1–255 UTF-8 bytes); paths, controls, reserved names and trailing dots/spaces are invalid".clone_into(&mut ui.status_text);
        return false;
    }
    let name = dialog.text.clone();
    if let Some(approved) = ui.operation.as_mut().and_then(|form|form.queue_save.as_mut()) {
        approved.name = Some(name);
    }
    true
}
/// Field then visual left-to-right Cancel/OK; no implicit destructive approval.
fn focus_next(ui: &mut InGameUiModel, reverse: bool) {
    let controls = [save::FIELD, save::CANCEL, save::OK];
    let Some(dialog) = ui.operation.as_mut() else {
        return;
    };
    let current = controls
        .iter()
        .position(|id| dialog.confirmation_focus.as_deref() == Some(*id))
        .unwrap_or(0_usize);
    let next = if reverse {
        current.checked_sub(1_usize).unwrap_or(2_usize)
    } else {
        current
            .saturating_add(1_usize)
            .checked_rem(controls.len())
            .unwrap_or(0_usize)
    };
    dialog.confirmation_focus = controls.get(next).map(|id| (*id).to_owned());
}
