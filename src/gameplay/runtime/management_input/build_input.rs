//! Source focused Build-list keyboard input, before generic manager focus cycling.
use super::value_input::Outcome;
use crate::gameplay::{model::GameState, ui_model::{BuildListKey, InGameUiModel}};
use sdl2::{event::Event, keyboard::{Keycode, Mod}};

/*
 * Source: ctp2_source/ctp2_code/ui/aui_common/aui_listbox.cpp::HandleKey and
 * SendKeyboardAction; ui/interface/EditQueue.cpp constructor wires Add for only
 * the three left lists. editqueue.ldl::BuildEditorWindow.ItemsBox/QueueGroup
 * retains original focus/list/ranger controls, not global Up/Down Tab cycling.
 * Existing parameter/scroll/dropdown/editor handlers keep priority; Source Space
 * uses the same exact current Add binding and runtime session/queue checks.
 */
/// Parses plain source keys; no OS injection, coordinate picking or game mutation.
pub(super) fn key(event: &Event, state: &GameState, ui: &mut InGameUiModel) -> Outcome {
    let Event::KeyDown {keycode: Some(code), keymod, repeat: false, ..} = *event else { return Outcome::Ignored; };
    if keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD | Mod::LALTMOD | Mod::RALTMOD | Mod::LGUIMOD | Mod::RGUIMOD) { return Outcome::Ignored; }
    let key = match code {Keycode::UP => BuildListKey::Up, Keycode::DOWN => BuildListKey::Down, Keycode::SPACE => BuildListKey::Activate, _ => return Outcome::Ignored};
    let (handled, intent) = ui.management.build_list_key(key, state);
    if !handled { return Outcome::Ignored; }
    ui.keyboard_status = true;
    intent.map_or(Outcome::Consumed, Outcome::Action)
}
