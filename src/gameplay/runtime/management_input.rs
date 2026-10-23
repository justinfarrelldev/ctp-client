//! Exclusive manager focus and shared accessible/pointer command dispatch.

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use sdl2::mouse::MouseButton;

use super::{MovementInputState, RuntimeAuthority, submit_authoritative_command};
use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::InGameUiModel;

/// Focused source Build-list keys delegate to exact current controls.
mod build_input;
/// Native Build head-selection/quote requests use an isolated gateway only.
#[cfg(test)]
mod build_tests;
/// Source contact-details keyboard and receipt-safe related navigation.
#[cfg(test)]
mod diplomacy_tests;
/// Clipboard access remains injectable; editing itself is pure presentation.
pub(super) mod editor_input;
/// Fileless native-event text editing regressions.
#[cfg(test)]
mod editor_tests;
/// Government chooser navigation, stale identities and exact isolated HTTP receipts.
#[cfg(test)]
mod government_tests;
/// Accessible National manager navigation preserves exact city/return identity.
#[cfg(test)]
mod national_tests;
/// Original item enabling-advance Set Goal through isolated source/HTTP input.
#[cfg(test)]
mod research_goal_tests;
/// Source thumb capture and keyboard range behavior.
mod scroll_input;
/// Source ranger capture and accessible-value regressions.
#[cfg(test)]
mod scroll_tests;
/// Detached dropdown focus, keyboard navigation and cancellation.
mod selector_input;
/// Fileless source city-selection journeys and stale-receipt validation.
#[cfg(test)]
mod selector_tests;
/// Native source status ownership follows keyboard/pointer modality.
#[cfg(test)]
mod status_tests;
#[cfg(test)]
mod tests;
/// Exact new-contract operation envelopes through isolated HTTP stubs.
#[cfg(test)]
mod v2_tests;
/// Policy thumb previews and one-shot, revision-checked value submission.
mod value_input;
/// Fileless policy range and release/cancellation regressions.
#[cfg(test)]
mod value_tests;
use crate::gameplay::ui_model::management::{ManagerIntent, ManagerKind};

/// Consumes all non-window/quit input while a manager owns focus. Returns false
/// when ordinary gameplay/popup dispatch should proceed. A modal click never
/// doubles as an order, end-turn activation, map zoom or edge scroll.
pub(super) fn consume(
    event: &Event,
    state: &mut GameState,
    ui: &mut InGameUiModel,
    movement: &mut MovementInputState,
    authority: Option<&mut RuntimeAuthority>,
) -> bool {
    if ui.management.view.is_none() || !state.popups.is_empty() {
        ui.manager_press = None;
        return false;
    }
    if window_event(event, ui, state) {
        return false;
    }
    movement.cancel_drag();
    match local_input(event, ui, state) {
        value_input::Outcome::Consumed => return true,
        value_input::Outcome::Action(intent) => {
            apply(intent, state, ui, authority);
            return true;
        }
        value_input::Outcome::Ignored => {}
    }
    let intent = match event.clone() {
        Event::KeyDown {
            keycode: Some(Keycode::ESCAPE),
            repeat: false,
            ..
        } => Some(ManagerIntent::Close),
        Event::KeyDown {
            keycode: Some(Keycode::TAB | Keycode::DOWN),
            keymod,
            repeat: false,
            ..
        } => {
            focus(
                ui,
                state,
                if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                    -1_i32
                } else {
                    1_i32
                },
            );
            None
        }
        Event::KeyDown {
            keycode: Some(Keycode::UP),
            repeat: false,
            ..
        } => {
            focus(ui, state, -1);
            None
        }
        Event::KeyDown {
            keycode: Some(Keycode::RETURN | Keycode::SPACE),
            repeat: false,
            ..
        } => ui
            .management
            .focused
            .as_deref()
            .and_then(|id| ui.management.activate_id(id, state)),
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            x,
            y,
            clicks,
            ..
        } => click_intent(ui, state, x, y, clicks),
        Event::MouseWheel { y, .. } => ui.pointer.and_then(|(x, pointer_y)| {
            ui.management
                .widgets(state)
                .into_iter()
                .rev()
                .find(|widget| {
                    widget.rect.contains(x, pointer_y)
                        && widget.node.field("objecttype").is_some_and(|kind| {
                            kind.eq_ignore_ascii_case("ctp2_ListBox")
                                || kind.eq_ignore_ascii_case("ctp2_hypertextbox")
                        })
                })
                .map(|widget| ManagerIntent::Scroll(widget.id, y.saturating_neg()))
        }),
        _ => None,
    };
    if let Some(intent) = intent {
        apply(intent, state, ui, authority);
    }
    true
}

/// Preserve existing thumb/dropdown/editor ownership before focused list keys.
fn local_input(event: &Event, ui: &mut InGameUiModel, state: &GameState) -> value_input::Outcome {
    let outcome = value_input::consume(event, ui, state);
    if !matches!(outcome, value_input::Outcome::Ignored) { return outcome; }
    if scroll_input::consume(event, ui, state) || selector_input::consume(event, ui, state) || editor_input::consume(event, ui, state) {
        return value_input::Outcome::Consumed;
    }
    build_input::key(event, state, ui)
}

/*
 * Source: ctp2_source/ctp2_code/ui/aui_common/aui_listbox.cpp::
 * aui_ListBox::MouseLDoubleClickInside sends DOUBLECLICKSELECT for the pointed row.
 * Original ui/interface/EditQueue.cpp::{ListCallback,Add,InsertInQueue} appends
 * the selected visible item; {QueueListCallback,Remove,UpdateButtons} removes
 * only the pointed selected queue index, protecting AlreadyBoughtFront index0.
 * editqueue.ldl::BuildEditorWindow.ItemsBox.{UnitsList,BuildingList,WonderList}
 * and QueueGroup.QueueList establish existing hit/selection rectangles/styles.
 * sci_advancescreen.cpp::sci_advancescreen_listAction uses the same Research OK.
 * Client matches first-down/up and count2 by exact role/ID/city/revision before
 * applying the clicked selection and existing Add/Remove authorization/funding/
 * session guards; a receipt cannot silently rebase the gesture or shift it onto
 * another row. Key/wheel/focus/hidden-window and modal interruptions cancel it.
 * No local execution. Higher counts only select, avoiding cascading removals;
 * Research retains its existing explicit-row confirmation behavior.
 * Special Capitol replacement and full source selection/scroll acceptance remain
 * separate; no new style/placement/scaling or ordinary Remove Query is invented.
 */
/// One original list callback shares exact native/accessibility command bindings.
fn click_intent(ui: &mut InGameUiModel, state: &GameState, x: i32, y: i32, clicks: u8) -> Option<ManagerIntent> {
    let Some(intent) = pointer_intent(ui, state, x, y) else { ui.manager_press = None; return None; };
    let ManagerIntent::Select(key, id) = intent else { ui.manager_press = None; return Some(intent); };
    ui.management.focus_build_row(state, &key, &id);
    if let Some(view) = ui.management.view.as_ref().filter(|view| view.kind == ManagerKind::Build && matches!(key.as_str(), "queue" | "buildable")) {
        let Some(viewer) = state.viewer.as_ref() else { ui.manager_press = None; return Some(ManagerIntent::Select(key, id)); };
        let press = crate::gameplay::ui_model::ManagerRowPress {
            revision: viewer.revision, kind: view.kind,
            city_id: view.selections.get("city").or(view.city_id.as_ref()).cloned(),
            key: key.clone(), id: id.clone(),
        };
        let matched = clicks == 2_u8 && ui.manager_press.as_ref() == Some(&press);
        ui.manager_press = (clicks == 1_u8).then_some(press);
        if !matched { return Some(ManagerIntent::Select(key, id)); }
    } else { ui.manager_press = None; }
    let action = match (ui.management.view.as_ref().map(|view| view.kind), key.as_str()) {
        (Some(ManagerKind::Research), "advance") if clicks > 1_u8 => Some("Background.BackButton"),
        (Some(ManagerKind::Build), "buildable") if clicks == 2_u8 => Some("ItemsBox.AddButton"),
        (Some(ManagerKind::Build), "queue") if clicks == 2_u8 => Some("QueueGroup.RemoveButton"),
        _ => None,
    };
    let Some(action) = action else { return Some(ManagerIntent::Select(key, id)); };
    if !ui.management.apply_local(ManagerIntent::Select(key, id)) { return None; }
    ui.management.activate_id(action, state)
}

/// Shared pointer descriptors also expose explicit disabled reasons; empty
/// decorative children cannot hide the enclosing disabled control's explanation.
fn pointer_intent(
    ui: &mut InGameUiModel,
    state: &GameState,
    x: i32,
    y: i32,
) -> Option<ManagerIntent> {
    let intent = ui.management.intent_at(x, y, state);
    ui.management.focused = None;
    ui.management.search_dragging = false;
    ui.management.scroll_drag = None;
    ui.management.editor.clear_composition();
    if intent == Some(ManagerIntent::Focus("SearchWord".to_owned())) {
        if let Some(offset) = editor_input::caret_offset(ui, state, x) {
            ui.management.search_dragging = true;
            return Some(ManagerIntent::EditCaret(
                offset,
                ui.management.pointer_extend,
            ));
        }
    }
    if intent.is_none() {
        if let Some(reason) = ui
            .management
            .widgets(state)
            .into_iter()
            .rev()
            .find(|widget| widget.rect.contains(x, y) && widget.unavailable.is_some())
            .and_then(|widget| widget.unavailable)
        {
            ui.status_text = reason;
        }
    }
    intent
}

/// Window lifecycle remains owned by gameplay while cancelling local captures.
fn window_event(event: &Event, ui: &mut InGameUiModel, state: &GameState) -> bool {
    if matches!(*event, Event::KeyDown { .. } | Event::MouseWheel { .. }) { ui.manager_press = None; }
    if !matches!(*event, Event::Quit { .. } | Event::Window { .. }) {
        return false;
    }
    if matches!(
        *event,
        Event::Window {
            win_event: sdl2::event::WindowEvent::FocusLost | sdl2::event::WindowEvent::Hidden,
            ..
        }
    ) {
        ui.manager_press = None;
        ui.management.search_dragging = false;
        ui.management.scroll_drag = None;
        ui.management.pointer_extend = false;
        ui.management.clipboard_request = None;
        ui.management.editor.clear_composition();
    }
    selector_input::consume(event, ui, state);
    value_input::consume(event, ui, state);
    true
}

/// Accessible activation and real SDL events execute this exact same dispatcher.
pub(super) fn apply(
    intent: ManagerIntent,
    state: &mut GameState,
    ui: &mut InGameUiModel,
    authority: Option<&mut RuntimeAuthority>,
) {
    match intent {
        ManagerIntent::Dialog(dialog) => ui.operation = Some(*dialog),
        ManagerIntent::Submit(command) => {
            if !command.valid_for(state) {
                "This choice belongs to an outdated server revision"
                    .clone_into(&mut ui.status_text);
                return;
            }
            if let Some(authority) =
                authority.filter(|authority| authority.session.revision() == command.revision)
            {
                let close = ui.management.view.as_ref().is_some_and(|view| {
                    view.kind == ManagerKind::DiplomacyReview
                        || (view.kind == ManagerKind::Research && command.route == "set-research")
                });
                if submit_authoritative_command(authority, state, &command.route, command.payload) && close {
                    ui.management.apply_local(ManagerIntent::Close);
                }
            } else {
                "No matching authoritative session; no game state was changed"
                    .clone_into(&mut ui.status_text);
            }
        }
        ManagerIntent::Open(kind) => ui.management.open_related(kind, state),
        ManagerIntent::ContactDetails(id) => {
            if !ui.management.open_contact_details(state, &id) {
                "The selected contact is no longer known".clone_into(&mut ui.status_text);
            }
        }
        ManagerIntent::ForCity(kind, id) => {
            if state.viewer.as_ref().is_some_and(|viewer| {
                viewer
                    .cities
                    .iter()
                    .any(|city| city.id == id && !city.stale)
            }) {
                ui.management.open(kind, state, Some(&id));
            }
        }
        ManagerIntent::BeginValue(path, revision) => value_input::begin(ui, state, &path, revision),
        ManagerIntent::SetValue(path, value, revision) => {
            if let Some(command) = value_input::request(ui, state, &path, value, revision) {
                apply(ManagerIntent::Submit(command), state, ui, authority);
            }
        }
        ManagerIntent::ScrollTo(path, value) => {
            ui.management.scroll_to(&path, value, state);
        }
        ManagerIntent::BeginScroll(path) => {
            ui.management
                .begin_scroll(&path, ui.pointer.map(|point| point.1), state);
            ui.management.focused = Some(format!("{path}/ranger/thumb"));
        }
        ManagerIntent::Scroll(path, delta) => {
            ui.management.scroll(&path, delta, state);
        }
        ManagerIntent::Select(key, id) if matches!(key.as_str(), "city" | "government") => {
            select_identity(ui, state, &key, &id);
        }
        ManagerIntent::Article(id) => {
            if ui.management.library.navigate(&id) {
                ui.management.open_related(ManagerKind::Library, state);
                ui.management.apply_local(ManagerIntent::Article(id));
            } else {
                ui.status_text = format!("Article {id} is not installed");
            }
        }
        local => {
            if ui.management.apply_local(local)
                && ui
                    .management
                    .view
                    .as_ref()
                    .is_none_or(|view| view.kind != ManagerKind::Library)
            {
                ui.great_library = None;
            }
        }
    }
    ui.management.refresh_build_description(state);
}

/// Presentation identities are rechecked against the actual current receipt.
fn select_identity(ui: &mut InGameUiModel, state: &GameState, key: &str, id: &str) {
    let (valid, reason) = if key == "city" {
        (
            ui.management.select_city(id, state),
            "The selected city is no longer owned or visible",
        )
    } else {
        (
            ui.management.select_government(id, state),
            "The selected government is no longer eligible",
        )
    };
    if !valid {
        reason.clone_into(&mut ui.status_text);
    }
}

/// Opens a newly founded city's Build Manager exactly once after its receipt.
pub(super) fn open_pending_build(state: &mut GameState, ui: &mut InGameUiModel) {
    let Some(id) = state.city_build_menu.take() else {
        return;
    };
    let city = state.viewer.as_ref().and_then(|viewer| {
        viewer
            .cities
            .iter()
            .find(|city| crate::gameplay::model::CityId::from_server_id(&city.id) == id)
    });
    let canonical = city.map(|city| city.id.clone());
    ui.selected_city = Some(id);
    ui.active_tab = crate::gameplay::ui_model::ControlPanelTab::City;
    ui.management.open(
        crate::gameplay::ui_model::management::ManagerKind::Build,
        state,
        canonical.as_deref(),
    );
}

/// Cycles only currently enabled controls; Enter cannot trigger a hidden choice.
fn focus(ui: &mut InGameUiModel, state: &GameState, delta: i32) {
    ui.keyboard_status = true;
    let widgets: Vec<_> = ui
        .management
        .widgets(state)
        .into_iter()
        .filter(|widget| widget.intent.is_some())
        .collect();
    if widgets.is_empty() {
        ui.management.focused = None;
        return;
    }
    let current = ui
        .management
        .focused
        .as_ref()
        .and_then(|id| widgets.iter().position(|widget| widget.id == *id));
    let next = if delta < 0_i32 {
        current
            .unwrap_or(widgets.len())
            .checked_sub(1)
            .unwrap_or_else(|| widgets.len().saturating_sub(1))
    } else {
        current.map_or(0_usize, |index| {
            index
                .saturating_add(1)
                .checked_rem(widgets.len())
                .unwrap_or(0)
        })
    };
    ui.management.focused = Some(widgets[next].id.clone());
}
