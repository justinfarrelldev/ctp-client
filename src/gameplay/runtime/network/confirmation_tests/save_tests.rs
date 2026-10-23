//! Original Save-name field and fake persistence; never real filesystem effects.
use super::{Harness, begin_row_press, buildable_fixture, key, manager_event};
use crate::gameplay::{
    runtime::{
        management_input::editor_input::Clipboard,
        operation_input,
        saved_queues::writes::{QueueWriter, service_with},
    },
    ui_model::management::saved::save,
};
use sdl2::keyboard::Keycode;
use sdl2::{event::Event, keyboard::Mod};
use std::sync::mpsc::TryRecvError;

/// Exact fake persistence result and calls; never opens an actual path.
#[derive(Default)]
struct Store {
    /// Exact effect arguments; in-memory only.
    calls: Vec<(String, Vec<u8>)>,
    /// Injected permission failure.
    failure: bool,
}
impl QueueWriter for Store {
    fn create(&mut self, name: &str, bytes: &[u8]) -> Result<Vec<u8>, String> {
        self.calls.push((name.to_owned(), bytes.to_vec()));
        if self.failure {
            Err("fake permission denied".to_owned())
        } else {
            Ok(bytes.to_vec())
        }
    }
}
/// In-memory clipboard, not SDL/OS file or network access.
struct Clip {
    /// Mock clipboard bytes, not a file or actual OS clipboard.
    value: String,
}
impl Clipboard for Clip {
    fn read(&mut self) -> Result<String, String> {
        Ok(self.value.clone())
    }
    fn write(&mut self, value: &str) -> Result<(), String> {
        self.value = value.to_owned();
        Ok(())
    }
}
/// Real SDL text event, routed only through the source modal dispatcher.
fn text(harness: &mut Harness, value: &str) {
    super::input(
        harness,
        &Event::TextInput {
            timestamp: 0_u32,
            window_id: 0_u32,
            text: value.to_owned(),
        },
    );
}
/// Native control-modified editing event without OS input injection.
fn modified(code: Keycode, modifier: Mod) -> Event {
    Event::KeyDown {
        timestamp: 0_u32,
        window_id: 0_u32,
        keycode: Some(code),
        scancode: None,
        keymod: modifier,
        repeat: false,
    }
}

/// Same original semantic button activation as native focused Return.
fn activate(harness: &mut Harness, id: &str) {
    harness.ui.management.focused = Some(id.to_owned());
    manager_event(harness, &key(Keycode::RETURN));
}
/// Original separate local draft, with one explicitly selected known unit.
fn draft() -> Harness {
    let mut harness = buildable_fixture();
    activate(&mut harness, "NormalModeButtons.CustomButton");
    begin_row_press(&mut harness, "ItemsBox.UnitsList/UNIT_SETTLER");
    activate(&mut harness, "ItemsBox.AddButton");
    harness
}
/* Released ui/interface/EditQueue.cpp::SaveButton -> MessageBoxDialog.cpp::
 * TextQuery/constructor/TextFieldActionCallback -> EditQueue::SaveNameResponse.
 * messagebox.ldl::MessageBoxDialog centered320x174, TextField(30,92,260x24),
 * str_code_SaveQueueAsColon, original str_ldl_MB_OK/str_ldl_MB_CANCEL. No fake
 * authoritative route or generic400-wide Rename/queue input substitution.
 */
#[test]
fn build_save_source_text_query_opens_actual_field_and_cancel_leaves_exact_local_draft_without_any_effect()
 {
    let mut harness = draft();
    harness.ui.management.focused = Some("CustomModeButtons.SaveButton".to_owned());
    let before = harness.ui.management.clone();
    let facts = harness.state.viewer.clone();
    activate(&mut harness, "CustomModeButtons.SaveButton");
    let dialog = harness
        .ui
        .operation
        .as_ref()
        .expect("original Save-name TextQuery must open");
    let field = dialog
        .confirmation
        .iter()
        .find(|widget| widget.node.name == "TextField")
        .expect("source visible text field");
    assert_eq!(
        (
            field.rect.x,
            field.rect.y,
            field.rect.width,
            field.rect.height
        ),
        (270_i32, 305_i32, 260_u32, 24_u32),
        "original centered source field, no ad-hoc fitting"
    );
    assert!(
        !dialog
            .confirmation
            .iter()
            .any(|widget| widget.node.name == "DontShowButton"),
        "TextQuery explicitly hides Don't Show"
    );
    assert!(
        dialog.choices.is_empty(),
        "local file save is not an advertised city command"
    );
    super::input(&mut harness, &key(Keycode::ESCAPE));
    assert!(
        harness.ui.operation.is_none(),
        "ESC closes only source name question"
    );
    assert_eq!(
        harness.ui.management, before,
        "local work and original parent unchanged"
    );
    assert_eq!(harness.state.viewer, facts, "no game changes");
    assert_eq!(
        harness.jobs.as_ref().expect("jobs").try_recv(),
        Err(TryRecvError::Empty),
        "no city/turn jobs"
    );
}

/// Native separate IME preview, deliberately not committed filename text.
fn compose(harness:&mut Harness,value:&str,length:i32) {
    super::input(harness,&Event::TextEditing {timestamp:0_u32,window_id:0_u32,text:value.to_owned(),start:0_i32,length});
}

/// Native source browser selects the exact just-verified cached profile file.
fn saved_preview(harness:&mut Harness) {
    activate(harness,"ItemsBox.LoadSaveButton");
    harness.ui.management.view.as_mut().expect("view").selections.remove(crate::gameplay::ui_model::management::saved::REFRESH);
    begin_row_press(harness,"LoadBox.QueuesList/profile:京都 100% queue");
}

#[test]
fn build_save_native_unicode_ime_clipboard_approval_captures_once_and_only_verified_file_success_returns_parent()
 {
    let mut harness = draft();
    let canonical = harness.state.viewer.clone();
    activate(&mut harness, "CustomModeButtons.SaveButton");
    text(&mut harness, "old");
    super::input(&mut harness, &modified(Keycode::A, Mod::LCTRLMOD));
    compose(&mut harness,"京都",2_i32);
    super::input(&mut harness, &key(Keycode::RETURN));
    assert!(
        harness
            .ui
            .operation
            .as_ref()
            .and_then(|dialog| dialog.queue_save.as_ref())
            .is_some_and(|capture| capture.name.is_none()),
        "IME preview Return never writes an uncommitted name"
    );
    text(&mut harness, "京都 100% ");
    let mut clip = Clip {
        value: "queue".to_owned(),
    };
    super::input(&mut harness, &modified(Keycode::V, Mod::LCTRLMOD));
    operation_input::flush(&mut harness.ui, &mut clip);
    super::input(&mut harness, &modified(Keycode::Z, Mod::LCTRLMOD));
    super::input(&mut harness, &modified(Keycode::Y, Mod::LCTRLMOD));
    assert_eq!(
        harness
            .ui
            .operation
            .as_ref()
            .map(|dialog| dialog.text.as_str()),
        Some("京都 100% queue"),
        "Unicode selection/commit/paste/undo/redo share the actual field"
    );
    super::input(&mut harness, &key(Keycode::RETURN));
    super::input(&mut harness, &key(Keycode::RETURN));
    text(&mut harness, "should not alter approved name");
    let mut store = Store::default();
    assert!(
        harness.ui.operation.is_some(),
        "approval isn't file success"
    );
    assert_eq!(
        harness.state.viewer, canonical,
        "local pending is not gameplay Pending/execution"
    );
    assert!(
        service_with(&mut harness.ui, &mut store),
        "one injected file operation"
    );
    assert_eq!(
        store.calls,
        vec![("京都 100% queue".to_owned(), b"U UNIT_SETTLER\n".to_vec())],
        "literal exact captured name/ordered original bytes"
    );
    assert!(
        harness.ui.operation.is_none() && harness.ui.management.saved_queues.custom.draft.is_none(),
        "actual verified write restores source old mode"
    );
    assert_eq!(
        harness
            .ui
            .management
            .view
            .as_ref()
            .and_then(|view| view.city_id.as_deref()),
        Some("c2"),
        "not first city"
    );
    saved_preview(&mut harness);
    assert!(
        harness
            .ui
            .management
            .widgets(&harness.state)
            .iter()
            .any(|widget| widget.id.starts_with("LoadBox.Contents/preview:0")
                && widget.label == "Settler"),
        "saved catalog/source browser preview roundtrip"
    );
    assert!(
        !service_with(&mut harness.ui, &mut store),
        "unchanged frame no second write"
    );
    assert_eq!(
        harness.state.viewer, canonical,
        "save/browser never edits canonical queue"
    );
    assert_eq!(
        harness.jobs.as_ref().expect("jobs").try_recv(),
        Err(TryRecvError::Empty),
        "no city/game jobs"
    );
}

#[test]
fn build_save_native_invalid_stale_failed_or_canceled_request_retains_work_and_cannot_write_or_blindly_retry()
 {
    for mode in 0_u8..4_u8 {
        let mut harness = draft();
        activate(&mut harness, "CustomModeButtons.SaveButton");
        let work = harness.ui.management.saved_queues.custom.draft.clone();
        let mut store = Store {
            failure: mode == 2_u8,
            ..Store::default()
        };
        text(
            &mut harness,
            if mode == 0_u8 { "../escape" } else { "literal" },
        );
        if mode == 1_u8 {
            harness
                .ui
                .management
                .saved_queues
                .custom
                .draft
                .as_mut()
                .expect("draft")
                .version = 4_u64;
        }
        super::input(&mut harness, &key(Keycode::RETURN));
        if mode == 3_u8 {
            assert!(
                operation_input::queue_save::activate(&mut harness.ui, save::CANCEL),
                "accessible Cancel withdraws deferred approval before I/O"
            );
        }
        service_with(&mut harness.ui, &mut store);
        assert_eq!(
            store.calls.len(),
            usize::from(mode == 2_u8),
            "only actually approved current valid name reaches fake backend"
        );
        if mode != 3_u8 {
            assert!(
                harness.ui.operation.is_some(),
                "validation/stale/I/O failure retains original source form"
            );
        }
        if mode != 1_u8 {
            assert_eq!(
                harness.ui.management.saved_queues.custom.draft, work,
                "no lost work"
            );
        }
        assert!(
            !service_with(&mut harness.ui, &mut store),
            "no blind retry on another frame"
        );
        assert_eq!(
            harness.jobs.as_ref().expect("jobs").try_recv(),
            Err(TryRecvError::Empty),
            "no game jobs"
        );
    }
}

#[test]
fn build_save_source_overlap_hit_focus_loss_and_accessible_ok_share_actual_field_bounds() {
    let mut harness = draft();
    activate(&mut harness, "CustomModeButtons.SaveButton");
    text(&mut harness, "name");
    super::input(
        &mut harness,
        &Event::MouseButtonDown {
            timestamp: 0_u32,
            window_id: 0_u32,
            which: 0_u32,
            mouse_btn: sdl2::mouse::MouseButton::Left,
            clicks: 1_u8,
            x: 300_i32,
            y: 324_i32,
        },
    );
    assert_eq!(
        harness
            .ui
            .operation
            .as_ref()
            .and_then(|dialog| dialog.confirmation_focus.as_deref()),
        Some(save::FIELD),
        "source field paints/hits over seven-pixel button overlap, never spurious cancel/approve"
    );
    super::input(
        &mut harness,
        &Event::TextEditing {
            timestamp: 0_u32,
            window_id: 0_u32,
            text: "未".to_owned(),
            start: 0_i32,
            length: 1_i32,
        },
    );
    super::input(
        &mut harness,
        &Event::Window {
            timestamp: 0_u32,
            window_id: 0_u32,
            win_event: sdl2::event::WindowEvent::FocusLost,
        },
    );
    assert!(
        harness
            .ui
            .operation
            .as_ref()
            .is_some_and(|dialog| dialog.editor.composition.is_empty()),
        "window interruption clears only IME preview"
    );
    assert!(
        operation_input::queue_save::activate(&mut harness.ui, save::OK),
        "accessible original OK shares native effect capture"
    );
    let mut store = Store::default();
    assert!(
        service_with(&mut harness.ui, &mut store),
        "one original source control approval"
    );
    assert_eq!(store.calls.len(), 1_usize, "one exact file operation");
}
