//! Authoritative UI-thread reconciliation. Workers never own model or SDL state.
use super::network::Outcome;
use super::{MovementInputState, RuntimeAuthority, replace_state_from_projection};
use crate::gameplay::model::{GameState, NetworkStatus};
use crate::gameplay::server::envelope::PreparedCommand;
use crate::gameplay::ui_model::InGameUiModel;
use serde_json::Value;

/// Enqueues one immutable user operation; authoritative facts remain untouched.
pub(super) fn submit(
    authority: &RuntimeAuthority,
    state: &mut GameState,
    route: &str,
    payload: Value,
) {
    if state.network != NetworkStatus::Ready {
        return;
    }
    let result = PreparedCommand::new(&authority.session, route, payload)
        .map_err(|error| error.to_string())
        .and_then(|command| {
            authority
                .io
                .as_ref()
                .ok_or_else(|| "no network worker".to_owned())?
                .submit(command)
        });
    match result {
        Ok(()) => state.network = NetworkStatus::Pending,
        Err(error) => {
            state.popups.push(crate::gameplay::model::GameplayPopup {
                title: "Order not submitted".to_owned(),
                body: error,
                kind: crate::gameplay::model::PopupKind::Info,
            });
        }
    }
}

/// Updates network results and semantic notices without ever applying local rewards.
pub(super) fn frame(
    mut authority: Option<&mut RuntimeAuthority>,
    state: &mut GameState,
    ui: &mut InGameUiModel,
    movement: &mut MovementInputState,
) {
    poll(authority.as_deref_mut(), state, ui, movement);
    super::message_input::refresh(authority.as_deref(), state, ui, movement);
}

/// Applies at most one complete outcome each frame, retaining explicit recovery.
fn poll(
    authority: Option<&mut RuntimeAuthority>,
    state: &mut GameState,
    ui: &mut InGameUiModel,
    movement: &mut MovementInputState,
) {
    let Some(authority) = authority else {
        return;
    };
    if let Some(outcome) = authority
        .io
        .as_ref()
        .and_then(super::network::Network::poll)
    {
        apply(authority, state, ui, movement, outcome);
    }
}

/// Pure UI-thread boundary, shared by production polling and injected outcomes.
pub(super) fn apply(
    authority: &mut RuntimeAuthority,
    state: &mut GameState,
    ui: &mut InGameUiModel,
    movement: &mut MovementInputState,
    outcome: Outcome,
) {
    match outcome {
        Outcome::Snapshot(snapshot) => {
            if snapshot.revision < authority.session.revision() {
                return;
            }
            let pending = state.network == NetworkStatus::Pending;
            if snapshot.revision == authority.session.revision()
                && state.network != NetworkStatus::Resynchronizing
            {
                return;
            }
            let recovering = state.network == NetworkStatus::Resynchronizing;
            let before = state.clone();
            replace_state_from_projection(authority, state, &snapshot);
            authority.session.apply_projection(*snapshot);
            movement.actors.reconcile(&before, state);
            ui.management.reconcile(state);
            if pending {
                state.network = NetworkStatus::Pending;
            }
            ui.context = None;
            if recovering {
                "State resynchronized.".clone_into(&mut ui.status_text);
            } else if ui.status_text != "State resynchronized." {
                ui.status_text.clear();
            }
            movement.retain_selectable(state);
        }
        Outcome::Receipt(snapshot) => {
            if snapshot.revision >= authority.session.revision() {
                let before = state.clone();
                replace_state_from_projection(authority, state, &snapshot);
                authority.session.apply_projection(*snapshot);
                movement.actors.reconcile(&before, state);
                ui.management.reconcile(state);
            }
            state.network = NetworkStatus::Ready;
            ui.context = None;
            ui.status_text = if state
                .viewer
                .as_ref()
                .is_some_and(|viewer| viewer.interface.planning_status == "queued")
            {
                "Order queued. End Turn to close planning.".to_owned()
            } else {
                String::new()
            };
            movement.retain_selectable(state);
        }
        Outcome::Recovering(error) | Outcome::Failure(error) => {
            state.network = NetworkStatus::Resynchronizing;
            ui.status_text = format!("Resynchronizing: {error}");
            movement.cancel_drag();
            movement.actors.rollback();
        }
    }
}
