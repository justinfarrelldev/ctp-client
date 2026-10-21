//! Authoritative UI-thread reconciliation. Workers never own model or SDL state.
use serde_json::Value;
use super::{RuntimeAuthority,MovementInputState,replace_state_from_projection};
use super::network::Outcome;
use crate::gameplay::model::{GameState,NetworkStatus};
use crate::gameplay::server::envelope::PreparedCommand;
use crate::gameplay::ui_model::InGameUiModel;

/// Enqueues one immutable user operation; authoritative facts remain untouched.
pub(super) fn submit(authority:&RuntimeAuthority,state:&mut GameState,route:&str,payload:Value) {
    if state.network != NetworkStatus::Ready { return; }
    let result=PreparedCommand::new(&authority.session,route,payload).map_err(|error| error.to_string()).and_then(|command| authority.io.as_ref().ok_or_else(|| "no network worker".to_owned())?.submit(command));
    match result {
        Ok(())=>state.network=NetworkStatus::Pending,
        Err(error)=>{
            state.popups.push(crate::gameplay::model::GameplayPopup {title:"Order not submitted".to_owned(),body:error,kind:crate::gameplay::model::PopupKind::Info});
        }
    }
}

/// Applies at most one complete outcome each frame, retaining explicit recovery.
pub(super) fn poll(authority:&mut RuntimeAuthority,state:&mut GameState,ui:&mut InGameUiModel,movement:&mut MovementInputState) {
    let outcome=authority.io.as_ref().and_then(super::network::Network::poll);
    match outcome {
        Some(Outcome::Snapshot(snapshot))=>{
            if snapshot.revision < authority.session.revision() { return; }
            let pending=state.network == NetworkStatus::Pending;
            authority.session.apply_projection(*snapshot.clone());
            replace_state_from_projection(authority,state,&snapshot);
            if pending { state.network=NetworkStatus::Pending; }
            ui.status_text.clear();
            movement.retain_selectable(state);
        }
        Some(Outcome::Receipt(snapshot))=>{
            if snapshot.revision >= authority.session.revision() {
                authority.session.apply_projection(*snapshot.clone());
                replace_state_from_projection(authority,state,&snapshot);
            }
            state.network=NetworkStatus::Ready;
            ui.status_text.clear();
            movement.retain_selectable(state);
        }
        Some(Outcome::Recovering(error) | Outcome::Failure(error))=>{
            state.network=NetworkStatus::Resynchronizing;
            ui.status_text=format!("Resynchronizing: {error}");
            movement.cancel_drag();
        }
        None=>{}
    }
}
