//! AUI source descriptions share native/semantic IDs, without mutating receipts.
use super::{InGameUiModel, management::ManagerWidget};
use crate::gameplay::model::{GameState, NetworkStatus};
use alloc::borrow::Cow;

/// Same visible descriptor metadata for semantic ID inspection and native hover.
pub(super) fn source_id(ui: &InGameUiModel, id: &str, state: &GameState) -> Option<String> {
    ui.management
        .widgets(state)
        .iter()
        .find(|widget| widget.id == id)
        .and_then(|widget| caption(ui, widget))
}
/// Original MouseMoveOver/MouseMoveAway ownership is ephemeral. Keyboard focus
/// uses the same metadata; neither overwrites queued/recovery/result messages.
pub(super) fn footer<'ui>(ui: &'ui InGameUiModel, state: &GameState) -> Cow<'ui, str> {
    let fallback = if ui.status_text.is_empty() {
        "Unexplored"
    } else {
        ui.status_text.as_str()
    };
    if state.network != NetworkStatus::Ready
        || !state.popups.is_empty()
        || ui.operation.is_some()
        || ui.messages.selected.is_some()
        || ui.open_menu.is_some()
        || ui.context.is_some()
    {
        return Cow::Borrowed(fallback);
    }
    if !ui.keyboard_status && ui.pointer.is_none() {
        return Cow::Borrowed(fallback);
    }
    let widgets = ui.management.widgets(state);
    let owner = if ui.keyboard_status {
        ui.management
            .focused
            .as_ref()
            .and_then(|id| widgets.iter().find(|widget| widget.id == *id))
    } else {
        ui.pointer.and_then(|(x, y)| {
            widgets.iter().rev().find(|widget| {
                widget.rect.contains(x, y)
                    && (widget.intent.is_some()
                        || widget.unavailable.is_some()
                        || widget.node.field("statustext").is_some())
            })
        })
    };
    owner
        .and_then(|widget| caption(ui, widget))
        .map_or(Cow::Borrowed(fallback), Cow::Owned)
}
/// Disabled reasons take precedence; missing raw localization keys never leak
/// into the native footer. Decorative children do not steal control ownership.
fn caption(ui: &InGameUiModel, widget: &ManagerWidget) -> Option<String> {
    if let Some(reason) = widget.unavailable.as_ref() {
        return Some(reason.clone());
    }
    let key = widget
        .node
        .field("statustext")
        .filter(|key| !key.trim().is_empty())?;
    let localized = ui.management.book.label(key);
    if localized == key && key.starts_with("STATUSBAR_") {
        None
    } else {
        Some(localized.to_owned())
    }
}
