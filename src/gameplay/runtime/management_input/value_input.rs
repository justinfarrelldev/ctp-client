//! Presentation-only policy thumb capture and explicit one-shot release.

use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::InGameUiModel;
use crate::gameplay::ui_model::management::{
    ManagerCommand, ManagerIntent, ManagerValueDrag, ManagerWidget,
};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

/// Explicit routing avoids confusing an ignored event with a consumed no-op.
pub(super) enum Outcome {
    /// Not a captured/focused policy event; normal manager handling continues.
    Ignored,
    /// Presentation changed or capture cancelled without submitting anything.
    Consumed,
    /// One explicit requested value, dispatched through current authority checks.
    Action(ManagerIntent),
}

/// Both direct values and captures belong to the receipt that displayed them.
fn current(ui: &mut InGameUiModel, state: &GameState, revision: i64) -> bool {
    if state
        .viewer
        .as_ref()
        .is_some_and(|viewer| viewer.revision == revision)
    {
        return true;
    }
    ui.management.value_drag = None;
    "Policy changed in a newer server revision; input cancelled".clone_into(&mut ui.status_text);
    false
}

/// Resolves a displayed value only against its original current receipt.
pub(super) fn request(
    ui: &mut InGameUiModel,
    state: &GameState,
    path: &str,
    value: i32,
    revision: i64,
) -> Option<ManagerCommand> {
    if !current(ui, state, revision) {
        return None;
    }
    ui.management.value_command(path, value, state)
}

/// Captures only a currently enabled original thumb and preserves its grab offset.
pub(super) fn begin(ui: &mut InGameUiModel, state: &GameState, path: &str, revision: i64) {
    if !current(ui, state, revision) {
        return;
    }
    let Some(thumb) = thumb(ui, state, path).filter(|thumb| thumb.intent.is_some()) else {
        return;
    };
    let Some(viewer) = state.viewer.as_ref() else {
        return;
    };
    let Some(value) = thumb.node.integer("value-current") else {
        return;
    };
    let grab = ui.pointer.map_or_else(
        || i32::try_from(thumb.rect.width.div_euclid(2_u32)).unwrap_or(0_i32),
        |point| point.0.saturating_sub(thumb.rect.x),
    );
    ui.management.value_drag = Some(ManagerValueDrag {
        path: path.to_owned(),
        revision: viewer.revision,
        value,
        grab_offset: grab,
    });
    ui.management.focused = Some(format!("{path}/thumb"));
}

/// Network requests happen only after explicit keyboard/value or pointer release.
pub(super) fn consume(event: &Event, ui: &mut InGameUiModel, state: &GameState) -> Outcome {
    if matches!(
        *event,
        Event::Window {
            win_event: WindowEvent::FocusLost | WindowEvent::Hidden,
            ..
        }
    ) {
        ui.management.value_drag = None;
        return Outcome::Ignored;
    }
    if let Some(capture) = ui.management.value_drag.as_ref() {
        if state
            .viewer
            .as_ref()
            .is_none_or(|viewer| viewer.revision != capture.revision)
        {
            ui.management.value_drag = None;
            "Policy changed in a newer server revision; drag cancelled"
                .clone_into(&mut ui.status_text);
            return Outcome::Consumed;
        }
        match *event {
            Event::MouseMotion { x, mousestate, .. } => {
                if mousestate.left() {
                    update(ui, state, x);
                } else {
                    ui.management.value_drag = None;
                }
                return Outcome::Consumed;
            }
            Event::MouseButtonUp {
                mouse_btn: MouseButton::Left,
                ..
            } => {
                let Some(released) = ui.management.value_drag.take() else {
                    return Outcome::Consumed;
                };
                return Outcome::Action(ManagerIntent::SetValue(
                    released.path,
                    released.value,
                    released.revision,
                ));
            }
            Event::KeyDown {
                keycode: Some(Keycode::ESCAPE),
                ..
            } => {
                ui.management.value_drag = None;
                return Outcome::Consumed;
            }
            _ => {}
        }
    }
    keyboard(event, ui, state)
}

/// Fresh shared descriptor metadata is the single keyboard/pointer value geometry.
fn thumb(ui: &InGameUiModel, state: &GameState, path: &str) -> Option<ManagerWidget> {
    let id = format!("{path}/thumb");
    ui.management
        .widgets(state)
        .into_iter()
        .find(|widget| widget.id == id && widget.node.field("value-owner") == Some(path))
}

/// Converts actual pointer travel to the same bounded source value as rendering.
fn update(ui: &mut InGameUiModel, state: &GameState, x: i32) {
    let Some(capture) = ui.management.value_drag.as_ref() else {
        return;
    };
    let path = capture.path.clone();
    let grab = capture.grab_offset;
    let Some(thumb) = thumb(ui, state, &path) else {
        ui.management.value_drag = None;
        return;
    };
    let Some(control) = ui
        .management
        .widgets(state)
        .into_iter()
        .find(|widget| widget.id == path)
    else {
        ui.management.value_drag = None;
        return;
    };
    let minimum = thumb.node.integer("value-minimum").unwrap_or(0_i32);
    let maximum = thumb.node.integer("value-maximum").unwrap_or(minimum);
    let travel = control.rect.width.saturating_sub(thumb.rect.width);
    let position = u32::try_from(x.saturating_sub(control.rect.x).saturating_sub(grab))
        .unwrap_or(0_u32)
        .min(travel);
    let span = u64::try_from(maximum.saturating_sub(minimum)).unwrap_or(0_u64);
    let offset = if travel == 0_u32 {
        0_u64
    } else {
        u64::from(position)
            .saturating_mul(span)
            .saturating_add(u64::from(travel.div_euclid(2_u32)))
            .div_euclid(u64::from(travel))
    };
    if let Some(active_drag) = ui.management.value_drag.as_mut() {
        active_drag.value = minimum
            .saturating_add(i32::try_from(offset).unwrap_or(0_i32))
            .clamp(minimum, maximum);
    }
}

/// Keyboard changes remain explicit individual commands, not speculative UI facts.
fn keyboard(event: &Event, ui: &InGameUiModel, state: &GameState) -> Outcome {
    let Event::KeyDown {
        keycode: Some(code),
        ..
    } = *event
    else {
        return Outcome::Ignored;
    };
    let Some(focused) = ui.management.focused.as_ref() else {
        return Outcome::Ignored;
    };
    let Some(path) = focused.strip_suffix("/thumb") else {
        return Outcome::Ignored;
    };
    let Some(thumb) = thumb(ui, state, path) else {
        return Outcome::Ignored;
    };
    let minimum = thumb.node.integer("value-minimum").unwrap_or(0_i32);
    let maximum = thumb.node.integer("value-maximum").unwrap_or(minimum);
    let current = thumb.node.integer("value-current").unwrap_or(minimum);
    let step = thumb.node.integer("value-step").unwrap_or(1_i32).max(1_i32);
    let value = match code {
        Keycode::LEFT | Keycode::DOWN => current.saturating_sub(step).max(minimum),
        Keycode::RIGHT | Keycode::UP => current.saturating_add(step).min(maximum),
        Keycode::HOME => minimum,
        Keycode::END => maximum,
        _ => return Outcome::Ignored,
    };
    let Some(viewer) = state.viewer.as_ref() else {
        return Outcome::Ignored;
    };
    Outcome::Action(ManagerIntent::SetValue(
        path.to_owned(),
        value,
        viewer.revision,
    ))
}
