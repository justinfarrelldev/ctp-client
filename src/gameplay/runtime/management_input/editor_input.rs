//! Native text-event policy and an injectable, explicit clipboard boundary.

use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::InGameUiModel;
use crate::gameplay::ui_model::library::editor::{ClipboardRequest, Motion};
use crate::gameplay::ui_model::management::ManagerKind;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};

/// OS clipboard operations are injected so tests never access the real clipboard.
pub(in crate::gameplay::runtime) trait Clipboard {
    /// Reads explicit paste input.
    /// # Errors
    /// Reports OS permission/format failures without changing the query.
    fn read(&mut self) -> Result<String, String>;
    /// Writes explicit copy/cut input.
    /// # Errors
    /// Reports failures before any selected query text is deleted.
    fn write(&mut self, value: &str) -> Result<(), String>;
}

/// Native adapter; construction does not read or modify the clipboard.
pub(in crate::gameplay::runtime) struct NativeClipboard {
    /// SDL owns and validates the platform clipboard handle.
    clipboard: sdl2::clipboard::ClipboardUtil,
}
impl NativeClipboard {
    /// Keeps native side effects out of the pure editor and mocked tests.
    pub(in crate::gameplay::runtime) fn new(video: &sdl2::VideoSubsystem) -> Self {
        Self {
            clipboard: video.clipboard(),
        }
    }
}
impl Clipboard for NativeClipboard {
    fn read(&mut self) -> Result<String, String> {
        self.clipboard.clipboard_text()
    }
    fn write(&mut self, value: &str) -> Result<(), String> {
        self.clipboard.set_clipboard_text(value)
    }
}

/// Only an actual focused query field may consume editing or clipboard requests.
fn active(ui: &InGameUiModel) -> bool {
    ui.management
        .view
        .as_ref()
        .is_some_and(|view| view.kind == ManagerKind::Library)
        && ui.management.focused.as_deref() == Some("SearchWord")
}

/// Unicode input comes from SDL text events, never guessed from physical keycodes.
pub(super) fn consume(event: &Event, ui: &mut InGameUiModel, state: &GameState) -> bool {
    if !active(ui) {
        return false;
    }
    match event.clone() {
        Event::TextInput { text, .. } => {
            ui.management
                .editor
                .insert(&mut ui.management.library.query, &text);
            true
        }
        Event::TextEditing {
            text,
            start,
            length,
            ..
        } => {
            ui.management.editor.compose(&text, start, length);
            true
        }
        Event::KeyDown {
            keycode: Some(key),
            keymod,
            repeat,
            ..
        } => keyboard(ui, key, keymod, repeat),
        Event::KeyUp { keymod, .. } => {
            ui.management.pointer_extend = keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD);
            true
        }
        Event::MouseMotion { x, mousestate, .. } if ui.management.search_dragging => {
            if mousestate.left() {
                place_caret(ui, state, x, true);
            } else {
                ui.management.search_dragging = false;
            }
            true
        }
        Event::MouseButtonUp {
            mouse_btn: sdl2::mouse::MouseButton::Left,
            ..
        } if ui.management.search_dragging => {
            ui.management.search_dragging = false;
            true
        }
        _ => false,
    }
}

/// Key modifiers alter selection/movement, not the Unicode input stream.
fn keyboard(ui: &mut InGameUiModel, key: Keycode, mods: Mod, repeat: bool) -> bool {
    // Windows/Linux AltGr may be reported as Ctrl+Alt; those combinations
    // produce Unicode text and must not trigger select-all/cut/copy/paste.
    let control = !mods.intersects(Mod::LALTMOD | Mod::RALTMOD)
        && (mods.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD)
            || (cfg!(target_os = "macos") && mods.intersects(Mod::LGUIMOD | Mod::RGUIMOD)));
    let shift = mods.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD);
    ui.management.pointer_extend = shift;
    let editor = &mut ui.management.editor;
    let query = &mut ui.management.library.query;
    match key {
        Keycode::A if control => editor.select_all(query),
        Keycode::C | Keycode::X | Keycode::INSERT | Keycode::DELETE
            if !repeat
                && ((control && matches!(key, Keycode::C | Keycode::X | Keycode::INSERT))
                    || (shift && key == Keycode::DELETE)) =>
        {
            if !editor.selected(query).is_empty() {
                ui.management.clipboard_request =
                    Some(if matches!(key, Keycode::C | Keycode::INSERT) {
                        ClipboardRequest::Copy(editor.selected(query).to_owned())
                    } else {
                        ClipboardRequest::Cut {
                            value: query.clone(),
                            range: editor.range(query),
                        }
                    });
            }
        }
        Keycode::V | Keycode::INSERT
            if !repeat && ((key == Keycode::V && control) || (key == Keycode::INSERT && shift)) =>
        {
            ui.management.clipboard_request = Some(ClipboardRequest::Paste);
        }
        Keycode::Z if control && shift => {
            editor.redo(query);
        }
        Keycode::Z if control => {
            editor.undo(query);
        }
        Keycode::Y if control => {
            editor.redo(query);
        }
        Keycode::LEFT => editor.move_caret(
            query,
            if control {
                Motion::WordLeft
            } else {
                Motion::Left
            },
            shift,
        ),
        Keycode::RIGHT => editor.move_caret(
            query,
            if control {
                Motion::WordRight
            } else {
                Motion::Right
            },
            shift,
        ),
        Keycode::HOME | Keycode::UP => editor.move_caret(query, Motion::Home, shift),
        Keycode::END | Keycode::DOWN => editor.move_caret(query, Motion::End, shift),
        Keycode::BACKSPACE => {
            editor.erase(query, true, control);
        }
        Keycode::DELETE => {
            editor.erase(query, false, control);
        }
        Keycode::RETURN | Keycode::KP_ENTER if !editor.composition.is_empty() => {}
        Keycode::RETURN | Keycode::KP_ENTER if !repeat => ui.management.search_library(),
        Keycode::ESCAPE if !editor.composition.is_empty() => editor.clear_composition(),
        Keycode::SPACE => {}
        _ => return false,
    }
    true
}

/// Places a pointer caret using the same actual prefix metrics as painting.
pub(super) fn caret_offset(ui: &InGameUiModel, state: &GameState, x: i32) -> Option<usize> {
    let field = ui
        .management
        .widgets(state)
        .into_iter()
        .find(|widget| widget.id == "SearchWord")?;
    Some(
        ui.management.edit_metrics.hit(
            &ui.management.library.query,
            ui.management
                .editor
                .caret_offset(&ui.management.library.query),
            field.rect.width,
            x.saturating_sub(field.rect.x),
        ),
    )
}

/// Dragging extends the selection that started at the field, never a map move.
fn place_caret(ui: &mut InGameUiModel, state: &GameState, x: i32, extend: bool) {
    if let Some(offset) = caret_offset(ui, state, x) {
        ui.management
            .editor
            .set_caret(&ui.management.library.query, offset, extend);
    }
}

/// Runs only explicit focused requests. Stale/failed cuts cannot erase new input.
pub(in crate::gameplay::runtime) fn flush_clipboard(
    ui: &mut InGameUiModel,
    clipboard: &mut impl Clipboard,
) {
    let Some(request) = ui.management.clipboard_request.take() else {
        return;
    };
    if !active(ui) {
        return;
    }
    let result = match request {
        ClipboardRequest::Copy(text) => clipboard.write(&text),
        ClipboardRequest::Paste => clipboard.read().map(|text| {
            ui.management
                .editor
                .insert(&mut ui.management.library.query, &text);
        }),
        ClipboardRequest::Cut { value, range } => {
            if value != ui.management.library.query || range != ui.management.editor.range(&value) {
                "Search selection changed; clipboard cut cancelled".clone_into(&mut ui.status_text);
                return;
            }
            clipboard.write(value.get(range).unwrap_or("")).map(|()| {
                ui.management
                    .editor
                    .insert(&mut ui.management.library.query, "");
            })
        }
    };
    if let Err(error) = result {
        ui.status_text = format!("Clipboard: {error}");
    }
}
