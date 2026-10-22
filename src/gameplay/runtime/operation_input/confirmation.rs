//! Original MessageBoxDialog controls consume explicit modal input before the map.
use crate::gameplay::{model::GameState, ui_model::{management::{ManagerCommand, ManagerIntent}, operations::OperationDialog}};
use sdl2::keyboard::{Keycode, Mod};

/*
 * Source: ctp2_source/ctp2_code/ui/interface/MessageBoxDialog.cpp::constructor,
 * LeftButtonActionCallback/RightButtonActionCallback make the window strongly
 * modal and send affirmative/cancel once. messagebox.ldl positions the Cancel
 * RightButton left of the OK LeftButton; stable IDs below follow that order.
 * Client safety difference: no implicit affirmative focus or Don'tShow replay.
 * ESC/Cancel never dispatch, and an old captured command never rebases itself.
 */
/// Original controls in left-to-right order, not a first-command selection.
const BUTTONS: [&str; 2] = ["MessageBoxDialog.DialogBackground.RightButton", "MessageBoxDialog.DialogBackground.LeftButton"];

/// Only an explicitly focused original control may accept Return/Space.
pub(super) fn key(dialog: &mut OperationDialog, state: &GameState, code: Keycode, modifiers: Mod) -> (bool, Option<ManagerCommand>) {
    if code == Keycode::ESCAPE { return (true, None); }
    if modifiers.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD | Mod::LALTMOD | Mod::RALTMOD) { return (false, None); }
    match code {
        Keycode::TAB | Keycode::RIGHT | Keycode::DOWN => focus(dialog, modifiers.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD)),
        Keycode::LEFT | Keycode::UP => focus(dialog, true),
        Keycode::RETURN | Keycode::KP_ENTER | Keycode::SPACE => {
            let selected = dialog.confirmation_focus.clone();
            return selected.map_or((false, None), |id| activate(dialog, state, &id));
        }
        _ => {},
    }
    (false, None)
}

/// Focus is presentation-only and never defaults to an arbitrary positive choice.
fn focus(dialog: &mut OperationDialog, reverse: bool) {
    let current = BUTTONS.iter().position(|id| dialog.confirmation_focus.as_deref() == Some(*id));
    let next = if reverse {
        current.unwrap_or(0_usize).checked_sub(1_usize).unwrap_or(1_usize)
    } else {
        current.map_or(0_usize, |index| index.saturating_add(1_usize).checked_rem(BUTTONS.len()).unwrap_or(0_usize))
    };
    dialog.confirmation_focus = BUTTONS.get(next).map(|id| (*id).to_owned());
}

/// Hit rectangles are precisely the cached original SDL/semantic descriptors.
pub(super) fn pointer(dialog: &mut OperationDialog, state: &GameState, x: i32, y: i32) -> (bool, Option<ManagerCommand>) {
    let selected = dialog.confirmation.iter().rev().find(|widget| widget.rect.contains(x, y) && widget.intent.is_some()).map(|widget| widget.id.clone());
    selected.map_or((false, None), |id| activate(dialog, state, &id))
}

/// No stale command or unrelated descriptor can acquire question approval.
fn activate(dialog: &mut OperationDialog, state: &GameState, id: &str) -> (bool, Option<ManagerCommand>) {
    dialog.confirmation_focus = Some(id.to_owned());
    match dialog.confirmation.iter().find(|widget| widget.id == id).and_then(|widget| widget.intent.clone()) {
        Some(ManagerIntent::Submit(command)) if command.valid_for(state) => (false, Some(command)),
        Some(ManagerIntent::Close) => (true, None),
        _ => (false, None),
    }
}
