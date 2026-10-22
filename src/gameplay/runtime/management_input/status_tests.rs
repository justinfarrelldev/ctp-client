//! Native keyboard/pointer modality shares descriptor-based source descriptions.
use super::tests::{snapshot, ui};
use crate::gameplay::{
    model::GameState,
    runtime::{MovementInputState, cursor},
    server::ServerProjection,
    ui_model::management::ManagerKind,
};
use sdl2::{
    event::Event,
    keyboard::{Keycode, Mod},
    mouse::MouseState,
};
#[test]
fn actual_tab_and_pointer_events_transfer_footer_ownership_without_clearing_keyboard_focus() {
    let viewer: ServerProjection = serde_json::from_value(snapshot()).expect("fixture");
    let mut state = GameState::from_server_projection(&viewer, 2, 1);
    state.popups.clear();
    let mut model = ui();
    model.management.open(ManagerKind::Unit, &state, None);
    model.pointer = Some((0_i32, 0_i32));
    let mut movement = MovementInputState::default();
    let tab = Event::KeyDown {
        timestamp: 0,
        window_id: 0,
        keycode: Some(Keycode::TAB),
        scancode: None,
        keymod: Mod::NOMOD,
        repeat: false,
    };
    assert!(
        super::consume(&tab, &mut state, &mut model, &mut movement, None),
        "native manager consumes Tab"
    );
    assert!(
        model.keyboard_status,
        "keyboard focus owns description, not old pointer location"
    );
    let focused = model
        .management
        .focused
        .clone()
        .expect("enabled source focus");
    let caption = model
        .source_status_for_id(&focused, &state)
        .expect("first source close/advice description");
    assert_eq!(
        model.footer_status(&state),
        caption,
        "semantic/native focus agreement"
    );
    cursor::track_pointer(
        &Event::MouseMotion {
            timestamp: 0,
            window_id: 0,
            which: 0,
            mousestate: MouseState::from_sdl_state(0),
            x: 0_i32,
            y: 0_i32,
            xrel: 0_i32,
            yrel: 0_i32,
        },
        &mut model,
        &mut movement,
    );
    assert!(
        !model.keyboard_status,
        "real pointer motion releases keyboard description ownership"
    );
    assert_eq!(
        model.management.focused.as_deref(),
        Some(focused.as_str()),
        "moving a pointer must not destroy keyboard focus/caret"
    );
    assert_eq!(
        model.footer_status(&state),
        "Unexplored",
        "source MouseMoveAway leaves no stale tooltip"
    );
}
