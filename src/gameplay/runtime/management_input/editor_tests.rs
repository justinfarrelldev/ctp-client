//! Production SDL text-event dispatch with an entirely in-memory clipboard double.

use super::editor_input::{Clipboard, flush_clipboard};
use super::{ManagerKind, consume};
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::model::GameState;
use crate::gameplay::runtime::MovementInputState;
use crate::gameplay::ui_model::InGameUiModel;
use crate::gameplay::ui_model::management::ManagementUi;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use std::path::Path;

/// Neither method calls SDL or touches the real operating-system clipboard.
#[derive(Default)]
struct MemoryClipboard {
    /// Fake clipboard payload.
    value: String,
    /// Simulate an OS write refusal without deleting the selected query.
    fail_write: bool,
    /// Counts explicit clipboard requests.
    calls: u32,
}
impl Clipboard for MemoryClipboard {
    fn read(&mut self) -> Result<String, String> {
        self.calls = self.calls.saturating_add(1_u32);
        Ok(self.value.clone())
    }
    fn write(&mut self, value: &str) -> Result<(), String> {
        self.calls = self.calls.saturating_add(1_u32);
        if self.fail_write {
            Err("denied".to_owned())
        } else {
            value.clone_into(&mut self.value);
            Ok(())
        }
    }
}

/// Original layouts are read-only; all editor/game values stay in memory.
fn fixture() -> (InGameUiModel, GameState) {
    let mut state = GameState::new(8_u16, 8_u16, 1_u8, 1_u64);
    state.popups.clear();
    let mut ui = InGameUiModel {
        management: ManagementUi::new(
            LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"))
                .expect("installed layout"),
        )
        .expect("manager roots"),
        ..InGameUiModel::default()
    };
    ui.management.open(ManagerKind::Library, &state, None);
    (ui, state)
}
/// Explicit key event with modifiers, never a coordinate-only interaction.
fn key(code: Keycode, mods: Mod) -> Event {
    Event::KeyDown {
        timestamp: 0_u32,
        window_id: 0_u32,
        keycode: Some(code),
        scancode: None,
        keymod: mods,
        repeat: false,
    }
}
/// Source event ownership is checked for every editor interaction.
fn send(event: &Event, ui: &mut InGameUiModel, state: &mut GameState) {
    assert!(
        consume(event, state, ui, &mut MovementInputState::default(), None),
        "search owns the event"
    );
}
/// Commits actual SDL Unicode input, not platform-specific key-to-character guesses.
fn text(value: &str, ui: &mut InGameUiModel, state: &mut GameState) {
    send(
        &Event::TextInput {
            timestamp: 0_u32,
            window_id: 0_u32,
            text: value.to_owned(),
        },
        ui,
        state,
    );
}

#[test]
fn native_keyboard_selection_clipboard_undo_and_redo_never_change_the_game() {
    let (mut ui, mut state) = fixture();
    let before = state.clone();
    let mut clipboard = MemoryClipboard::default();
    text("é🙂", &mut ui, &mut state);
    send(&key(Keycode::A, Mod::LCTRLMOD), &mut ui, &mut state);
    send(&key(Keycode::C, Mod::LCTRLMOD), &mut ui, &mut state);
    flush_clipboard(&mut ui, &mut clipboard);
    assert_eq!(clipboard.value, "é🙂", "exact selected Unicode copied");
    send(&key(Keycode::X, Mod::LCTRLMOD), &mut ui, &mut state);
    flush_clipboard(&mut ui, &mut clipboard);
    assert!(
        ui.management.library.query.is_empty(),
        "cut succeeds only after clipboard write"
    );
    send(&key(Keycode::V, Mod::LCTRLMOD), &mut ui, &mut state);
    flush_clipboard(&mut ui, &mut clipboard);
    assert_eq!(
        ui.management.library.query, "é🙂",
        "paste restores actual clipboard data"
    );
    send(&key(Keycode::HOME, Mod::NOMOD), &mut ui, &mut state);
    send(&key(Keycode::DELETE, Mod::NOMOD), &mut ui, &mut state);
    assert_eq!(
        ui.management.library.query, "🙂",
        "Delete removes a whole UTF-8 scalar"
    );
    send(&key(Keycode::Z, Mod::LCTRLMOD), &mut ui, &mut state);
    assert_eq!(
        ui.management.library.query, "é🙂",
        "Ctrl+Z restores the query"
    );
    send(&key(Keycode::Y, Mod::LCTRLMOD), &mut ui, &mut state);
    assert_eq!(ui.management.library.query, "🙂", "Ctrl+Y redoes the edit");
    assert_eq!(
        state, before,
        "no order, research, end turn or network dispatch"
    );
    assert_eq!(
        clipboard.calls, 3_u32,
        "only explicit clipboard requests run"
    );
}

#[test]
fn altgr_unicode_typing_never_selects_all_or_cuts_the_os_clipboard() {
    let (mut ui, mut state) = fixture();
    let before = state.clone();
    let mut clipboard = MemoryClipboard::default();
    text("seed", &mut ui, &mut state);
    send(&key(Keycode::HOME, Mod::NOMOD), &mut ui, &mut state);
    send(
        &key(Keycode::A, Mod::LCTRLMOD | Mod::RALTMOD),
        &mut ui,
        &mut state,
    );
    text("ą", &mut ui, &mut state);
    assert_eq!(
        ui.management.library.query, "ąseed",
        "AltGr+A is text, not select-all"
    );
    send(&key(Keycode::A, Mod::LCTRLMOD), &mut ui, &mut state);
    send(
        &key(Keycode::X, Mod::LCTRLMOD | Mod::RALTMOD),
        &mut ui,
        &mut state,
    );
    flush_clipboard(&mut ui, &mut clipboard);
    assert_eq!(
        clipboard.calls, 0_u32,
        "AltGr+X must never be misinterpreted as an explicit cut"
    );
    text("ź", &mut ui, &mut state);
    assert_eq!(
        ui.management.library.query, "ź",
        "real committed text replaces the actual selected range"
    );
    assert_eq!(state, before, "typing cannot dispatch game commands");
}

#[test]
fn legacy_windows_insert_delete_clipboard_shortcuts_match_modern_shortcuts() {
    let (mut ui, mut state) = fixture();
    let mut clipboard = MemoryClipboard::default();
    text("draft", &mut ui, &mut state);
    send(&key(Keycode::A, Mod::LCTRLMOD), &mut ui, &mut state);
    send(&key(Keycode::INSERT, Mod::LCTRLMOD), &mut ui, &mut state);
    flush_clipboard(&mut ui, &mut clipboard);
    assert_eq!(clipboard.value, "draft", "Ctrl+Insert copies selected text");
    send(&key(Keycode::DELETE, Mod::LSHIFTMOD), &mut ui, &mut state);
    flush_clipboard(&mut ui, &mut clipboard);
    assert!(
        ui.management.library.query.is_empty(),
        "Shift+Delete cuts after successful write"
    );
    send(&key(Keycode::INSERT, Mod::LSHIFTMOD), &mut ui, &mut state);
    flush_clipboard(&mut ui, &mut clipboard);
    assert_eq!(
        ui.management.library.query, "draft",
        "Shift+Insert pastes through the bounded editor"
    );
}

#[test]
fn failed_or_stale_cut_is_non_destructive_and_ime_escape_does_not_close_browser() {
    let (mut ui, mut state) = fixture();
    let mut clipboard = MemoryClipboard {
        fail_write: true,
        ..MemoryClipboard::default()
    };
    text("draft", &mut ui, &mut state);
    send(&key(Keycode::A, Mod::LCTRLMOD), &mut ui, &mut state);
    send(&key(Keycode::X, Mod::LCTRLMOD), &mut ui, &mut state);
    flush_clipboard(&mut ui, &mut clipboard);
    assert_eq!(
        ui.management.library.query, "draft",
        "failed cut keeps the query"
    );
    assert!(
        ui.status_text.contains("denied"),
        "error is visible, not ignored"
    );
    clipboard.fail_write = false;
    send(&key(Keycode::X, Mod::LCTRLMOD), &mut ui, &mut state);
    text("changed", &mut ui, &mut state);
    flush_clipboard(&mut ui, &mut clipboard);
    assert_eq!(
        ui.management.library.query, "changed",
        "stale cut does not erase new input"
    );
    assert_eq!(
        clipboard.calls, 1_u32,
        "stale request does not overwrite the clipboard"
    );
    send(
        &Event::TextEditing {
            timestamp: 0_u32,
            window_id: 0_u32,
            text: "未確定".to_owned(),
            start: 1_i32,
            length: 1_i32,
        },
        &mut ui,
        &mut state,
    );
    assert_eq!(
        ui.management.library.query, "changed",
        "IME composition is not committed query text"
    );
    send(&key(Keycode::ESCAPE, Mod::NOMOD), &mut ui, &mut state);
    assert!(
        ui.management.view.is_some(),
        "first Escape cancels IME only"
    );
    assert!(
        ui.management.editor.composition.is_empty(),
        "composition cleared"
    );
    send(&key(Keycode::ESCAPE, Mod::NOMOD), &mut ui, &mut state);
    assert!(
        ui.management.view.is_none(),
        "next Escape closes browser normally"
    );
}
