//! Policy previews never mutate the server-owned snapshot or call real services.

use super::{apply, consume, value_input};
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::model::GameState;
use crate::gameplay::runtime::MovementInputState;
use crate::gameplay::server::ServerProjection;
use crate::gameplay::ui_model::{
    InGameUiModel,
    management::{ManagementUi, ManagerIntent, ManagerKind},
};
use sdl2::event::{Event, WindowEvent};
use sdl2::mouse::MouseState;
use serde_json::json;
use std::path::Path;

/// Explicit current policy levels and current scalar field capabilities.
fn fixture() -> (InGameUiModel, GameState) {
    let projection:ServerProjection=serde_json::from_value(json!({"revision":12_i64,"turn":4_u32,"diplomacy":{"science_percent":50_i64,"public_works_percent":10_i64,"workday":1_i64,"wages":-1_i64,"rations":0_i64},"legal_actions":[{"route":"set-social-policy","entity_id":"human","fields":["workday","wages","rations"],"payload":{"workday":0_i64,"wages":0_i64,"rations":0_i64}},{"route":"set-science-rate","entity_id":"human","fields":["science_percent"],"payload":{"science_percent":100_i64}}]})).expect("viewer fields");
    let mut state = GameState::from_server_projection(&projection, 1_u8, 1_u64);
    state.popups.clear();
    let mut ui = InGameUiModel {
        management: ManagementUi::new(
            LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2")).expect("source"),
        )
        .expect("layouts"),
        ..InGameUiModel::default()
    };
    ui.management.open(ManagerKind::Empire, &state, None);
    (ui, state)
}

#[test]
fn source_policy_keyboard_has_exact_quantized_bounds_and_current_receipt_identity() {
    let (mut ui, state) = fixture();
    let before = state.clone();
    let path = "TabGroup.Tab1.TabPanel.Production.Modify.Primary.Slider";
    ui.management.focused = Some(format!("{path}/thumb"));
    for (key, value) in [
        (sdl2::keyboard::Keycode::HOME, 0_i32),
        (sdl2::keyboard::Keycode::END, 4_i32),
        (sdl2::keyboard::Keycode::RIGHT, 2_i32),
        (sdl2::keyboard::Keycode::LEFT, 0_i32),
    ] {
        let event = Event::KeyDown {
            timestamp: 0_u32,
            window_id: 0_u32,
            keycode: Some(key),
            scancode: None,
            keymod: sdl2::keyboard::Mod::NOMOD,
            repeat: false,
        };
        let outcome = value_input::consume(&event, &mut ui, &state);
        assert!(
            matches!(outcome,value_input::Outcome::Action(ManagerIntent::SetValue(owner,actual,12_i64)) if owner==path&&actual==value),
            "source keyboard emits only the explicit quantized value at the displayed revision"
        );
    }
    assert_eq!(
        state, before,
        "forming keyboard intent cannot modify authoritative state"
    );
}

#[test]
fn stale_parameter_button_intents_cannot_rebase_onto_a_new_receipt() {
    let (mut ui, mut state) = fixture();
    let intent = ui
        .management
        .activate_id(
            "TabGroup.Tab1.TabPanel.Production.Modify.Primary.Slider/value/3",
            &state,
        )
        .expect("explicit displayed policy");
    state.viewer.as_mut().expect("viewer").revision = 13_i64;
    let before = state.clone();
    apply(intent, &mut state, &mut ui, None);
    assert!(
        ui.status_text.contains("newer server revision"),
        "reject stale value before trying any authoritative submission, rather than recomputing companions against another receipt"
    );
    assert_eq!(state, before, "stale value cannot mutate received state");
}

#[test]
fn newer_receipts_cancel_policy_capture_before_release_can_rebase_onto_new_companions() {
    let (mut ui, mut state) = fixture();
    let path = "TabGroup.Tab1.TabPanel.Production.Modify.Primary.Slider";
    let thumb = ui
        .management
        .widgets(&state)
        .into_iter()
        .find(|widget| widget.id == format!("{path}/thumb"))
        .expect("thumb");
    apply(thumb.intent.expect("capture"), &mut state, &mut ui, None);
    state.viewer.as_mut().expect("viewer").revision = 13_i64;
    state
        .viewer
        .as_mut()
        .expect("viewer")
        .diplomacy
        .as_mut()
        .expect("policy")
        .facts["wages"] = json!(2_i64);
    let before = state.clone();
    let release = Event::MouseButtonUp {
        timestamp: 0_u32,
        window_id: 0_u32,
        which: 0_u32,
        mouse_btn: sdl2::mouse::MouseButton::Left,
        clicks: 1_u8,
        x: 0_i32,
        y: 0_i32,
    };
    assert!(
        consume(
            &release,
            &mut state,
            &mut ui,
            &mut MovementInputState::default(),
            None
        ),
        "stale release consumed"
    );
    assert!(
        ui.management.value_drag.is_none() && ui.status_text.contains("newer server revision"),
        "explicit stale capture rejection"
    );
    assert_eq!(
        state, before,
        "never replay a gesture using newly received companion policy values"
    );
}

#[test]
fn original_policy_thumb_previews_quantized_values_without_mutating_game_and_focus_loss_cancels() {
    let (mut ui, mut state) = fixture();
    let before = state.clone();
    let path = "TabGroup.Tab1.TabPanel.Production.Modify.Primary.Slider";
    let widgets = ui.management.widgets(&state);
    let thumb = widgets
        .iter()
        .find(|widget| widget.id == format!("{path}/thumb"))
        .expect("source thumb");
    ui.pointer = Some((
        thumb.rect.x.saturating_add(2_i32),
        thumb.rect.y.saturating_add(4_i32),
    ));
    apply(
        thumb.intent.clone().expect("begin value"),
        &mut state,
        &mut ui,
        None,
    );
    assert!(
        ui.management.value_drag.is_some(),
        "actual source thumb owns a presentation capture"
    );
    let control = widgets
        .iter()
        .find(|widget| widget.id == path)
        .expect("source track");
    let motion = Event::MouseMotion {
        timestamp: 0_u32,
        window_id: 0_u32,
        which: 0_u32,
        mousestate: MouseState::from_sdl_state(1_u32),
        x: control
            .rect
            .x
            .saturating_add(i32::try_from(control.rect.width).expect("source width")),
        y: control.rect.y,
        xrel: 0_i32,
        yrel: 0_i32,
    };
    assert!(
        consume(
            &motion,
            &mut state,
            &mut ui,
            &mut MovementInputState::default(),
            None
        ),
        "policy capture owns motion"
    );
    assert_eq!(
        ui.management.value_drag.as_ref().expect("capture").value,
        4_i32,
        "source five-level quantized endpoint"
    );
    assert_eq!(
        state, before,
        "dragging cannot perform live policy probes or mutate local authoritative facts"
    );
    let lost = Event::Window {
        timestamp: 0_u32,
        window_id: 0_u32,
        win_event: WindowEvent::FocusLost,
    };
    consume(
        &lost,
        &mut state,
        &mut ui,
        &mut MovementInputState::default(),
        None,
    );
    assert!(
        ui.management.value_drag.is_none(),
        "focus loss cancels pending policy change"
    );
    assert_eq!(state, before, "cancellation never submits a game command");
}
