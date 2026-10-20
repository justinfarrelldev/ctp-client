//! Accessible ranger values and real pointer capture never become world movement.

use super::{apply, consume};
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::model::GameState;
use crate::gameplay::runtime::MovementInputState;
use crate::gameplay::ui_model::InGameUiModel;
use crate::gameplay::ui_model::library::LibraryCategory;
use crate::gameplay::ui_model::management::{ManagementUi, ManagerKind};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use std::path::Path;

#[test]
fn ranger_keyboard_drag_release_and_focus_loss_are_exclusive_and_non_authoritative() {
    let mut state = GameState::new(8_u16, 8_u16, 1_u8, 1_u64);
    state.popups.clear();
    let before = state.clone();
    let mut ui = InGameUiModel {
        management: ManagementUi::new(
            LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"))
                .expect("source layouts"),
        )
        .expect("controls"),
        ..InGameUiModel::default()
    };
    ui.management.open(ManagerKind::Library, &state, None);
    ui.management.library.set_category(LibraryCategory::Units);
    let thumb = ui
        .management
        .widgets(&state)
        .into_iter()
        .find(|widget| widget.id == "IndexSheet/ranger/thumb")
        .expect("visible thumb");
    let maximum = thumb
        .node
        .field("scroll-maximum")
        .expect("current range")
        .parse::<usize>()
        .expect("range number");
    apply(
        thumb.intent.expect("accessible source slider"),
        &mut state,
        &mut ui,
        None,
    );
    let mut movement = MovementInputState::default();
    let end = Event::KeyDown {
        timestamp: 0_u32,
        window_id: 0_u32,
        keycode: Some(Keycode::END),
        scancode: None,
        keymod: Mod::NOMOD,
        repeat: false,
    };
    assert!(
        consume(&end, &mut state, &mut ui, &mut movement, None),
        "focused source slider consumes End"
    );
    assert_eq!(
        ui.management.view.as_ref().expect("view").scroll["IndexSheet"],
        maximum,
        "accessible keyboard value reaches current end"
    );
    assert!(
        ui.management.drag_scroll(i32::MIN, &state),
        "captured pointer above viewport clamps"
    );
    assert_eq!(
        ui.management.view.as_ref().expect("view").scroll["IndexSheet"],
        0_usize,
        "drag reaches beginning without overflow"
    );
    let lost = Event::Window {
        timestamp: 0_u32,
        window_id: 0_u32,
        win_event: sdl2::event::WindowEvent::FocusLost,
    };
    assert!(
        !consume(&lost, &mut state, &mut ui, &mut movement, None),
        "window event still reaches the ordinary lifecycle handler"
    );
    assert!(
        ui.management.scroll_drag.is_none(),
        "focus loss cannot retain invisible capture"
    );
    assert_eq!(state, before, "no game command, turn or world movement");
}
