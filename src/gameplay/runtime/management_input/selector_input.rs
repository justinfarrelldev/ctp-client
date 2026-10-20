//! Detached source dropdown focus and bounded input navigation.

use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::InGameUiModel;
use crate::gameplay::ui_model::management::{ManagerIntent, ManagerKind};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

/// Consumes popup input before its parent manager, never a world/game command.
pub(super) fn consume(event: &Event, ui: &mut InGameUiModel, state: &GameState) -> bool {
    let Some(path) = ui
        .management
        .view
        .as_ref()
        .and_then(|view| view.selections.get("dropdown"))
        .cloned()
        .filter(|path| !path.is_empty())
    else {
        return false;
    };
    let choices = ui.management.dropdown_choices(state);
    if choices.is_empty() {
        dismiss(ui);
        return matches!(
            *event,
            Event::KeyDown { .. } | Event::MouseButtonDown { .. }
        );
    }
    match *event {
        Event::Window {
            win_event: WindowEvent::FocusLost | WindowEvent::Hidden,
            ..
        } => {
            dismiss(ui);
            false
        }
        Event::KeyDown {
            keycode: Some(code),
            ..
        } => keyboard(ui, state, &path, &choices, code),
        Event::MouseWheel { y, .. } => {
            ui.management
                .scroll(&format!("{path}.window.listbox"), y.saturating_neg(), state);
            true
        }
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } => {
            let owned = popup_intent(ui, state, &path, x, y).is_some();
            if owned {
                false
            } else {
                dismiss(ui);
                true
            }
        }
        Event::MouseMotion { x, y, .. } => {
            if let Some(ManagerIntent::Select(key, id)) = popup_intent(ui, state, &path, x, y) {
                if matches!(key.as_str(), "city" | "government") {
                    ui.management
                        .apply_local(ManagerIntent::Select("dropdown-highlight".to_owned(), id));
                }
            }
            true
        }
        _ => false,
    }
}

/// Only popup-owned controls or its toggle may receive a click while detached.
fn popup_intent(
    ui: &InGameUiModel,
    state: &GameState,
    path: &str,
    x: i32,
    y: i32,
) -> Option<ManagerIntent> {
    let popup = format!("{path}.window");
    let button = format!("{path}.autobutton");
    ui.management
        .widgets(state)
        .into_iter()
        .rev()
        .find(|widget| {
            widget.rect.contains(x, y)
                && widget.intent.is_some()
                && (widget.id.starts_with(&popup) || widget.id == button || widget.id == path)
        })?
        .intent
}

/// Escape/outside/focus loss dismiss only the popup, not its owning manager.
fn dismiss(ui: &mut InGameUiModel) {
    ui.management
        .apply_local(ManagerIntent::Select("dropdown".to_owned(), String::new()));
}

/// Home/End/page/arrow navigation retains identity until explicit acceptance.
fn keyboard(
    ui: &mut InGameUiModel,
    state: &GameState,
    path: &str,
    choices: &[(String, String)],
    code: Keycode,
) -> bool {
    let page = page_size(ui, state, path);
    let Some(view) = ui.management.view.as_ref() else {
        return false;
    };
    let selected = view
        .selections
        .get("dropdown-highlight")
        .map(String::as_str)
        .or_else(|| {
            if view.kind == ManagerKind::Empire {
                view.selections.get("government").map(String::as_str)
            } else {
                view.city_id.as_deref()
            }
        });
    let current_choice = choices
        .iter()
        .position(|choice| Some(choice.0.as_str()) == selected);
    let current = current_choice.unwrap_or(0_usize);
    let next = match code {
        Keycode::ESCAPE => {
            dismiss(ui);
            return true;
        }
        Keycode::RETURN | Keycode::KP_ENTER | Keycode::SPACE => {
            if current_choice.is_none() {
                "Highlighted selection is no longer available; choose a current eligible item"
                    .clone_into(&mut ui.status_text);
                dismiss(ui);
                return true;
            }
            let government = view.kind == ManagerKind::Empire;
            let valid = if government {
                ui.management.select_government(&choices[current].0, state)
            } else {
                ui.management.select_city(&choices[current].0, state)
            };
            if !valid {
                "Selection is no longer available".clone_into(&mut ui.status_text);
            }
            return true;
        }
        Keycode::HOME => 0_usize,
        Keycode::END => choices.len().saturating_sub(1_usize),
        Keycode::UP => current.saturating_sub(1_usize),
        Keycode::DOWN => current
            .saturating_add(1_usize)
            .min(choices.len().saturating_sub(1_usize)),
        Keycode::PAGEUP => current.saturating_sub(page),
        Keycode::PAGEDOWN => current
            .saturating_add(page)
            .min(choices.len().saturating_sub(1_usize)),
        _ => return true,
    };
    ui.management.apply_local(ManagerIntent::Select(
        "dropdown-highlight".to_owned(),
        choices[next].0.clone(),
    ));
    ensure_visible(ui, state, path, next, page);
    true
}

/// Measured source item/pane sizes, not a separate guessed popup page capacity.
fn ensure_visible(
    ui: &mut InGameUiModel,
    state: &GameState,
    path: &str,
    index: usize,
    page: usize,
) {
    let list = format!("{path}.window.listbox");
    let offset = ui
        .management
        .view
        .as_ref()
        .and_then(|view| view.scroll.get(&list))
        .copied()
        .unwrap_or(0_usize);
    let target = if index < offset {
        index
    } else if index >= offset.saturating_add(page) {
        index.saturating_add(1_usize).saturating_sub(page)
    } else {
        offset
    };
    ui.management.scroll_to(&list, target, state);
}

/// Native pane/item metadata drives both page keys and selected-row visibility.
fn page_size(ui: &InGameUiModel, state: &GameState, path: &str) -> usize {
    let list = format!("{path}.window.listbox");
    let height = ui
        .management
        .widgets(state)
        .into_iter()
        .find(|widget| widget.id == list)
        .and_then(|widget| {
            widget
                .node
                .field("list-content-height")
                .and_then(|value| value.parse::<usize>().ok())
        })
        .unwrap_or(0_usize);
    let row = ui
        .management
        .row_templates
        .get(
            if ui
                .management
                .view
                .as_ref()
                .is_some_and(|view| view.kind == ManagerKind::Empire)
            {
                "GovernmentListItem"
            } else {
                "cw_CityItem"
            },
        )
        .and_then(|item| item.integer("heightpix"))
        .and_then(|item_height| usize::try_from(item_height).ok())
        .unwrap_or(22_usize)
        .max(1_usize);
    height.div_euclid(row).max(1_usize)
}
