//! UI-thread handoff tests inject outcomes, never sockets or filesystem effects.
use super::{
    MovementInputState, RuntimeAuthority, network::Outcome, state_from_authority_projection, sync,
};
use crate::gameplay::{
    model::NetworkStatus,
    server::{ServerClient, ServerConfig, ServerGameplaySession, ServerProjection},
    ui_model::InGameUiModel,
};
use serde_json::json;

/// In-memory session; its unused transport is never contacted.
fn fixture() -> RuntimeAuthority {
    let projection:ServerProjection=serde_json::from_value(json!({"revision":9_i64,"turn":1_u32,"world":{"min_x":0_u16,"min_y":0_u16,"width":20_u16,"height":12_u16},"tiles":[]})).expect("viewer fixture");
    RuntimeAuthority {
        client: ServerClient::from_config(&ServerConfig {
            base_url: "http://127.0.0.1:1".to_owned(),
            required: true,
        })
        .expect("parse only; no network"),
        session: ServerGameplaySession {
            game_id: 1,
            session_id: 2,
            account_id: 3,
            projection: Some(projection),
        },
        empire_count: 1,
        seed: 1,
        io: None,
    }
}
#[test]
fn background_reads_cannot_release_pending_and_old_receipts_cannot_regress() {
    let mut authority = fixture();
    let mut state = state_from_authority_projection(
        &authority,
        authority
            .session
            .projection
            .as_ref()
            .expect("initial viewer"),
    );
    state.network = NetworkStatus::Pending;
    let mut ui = InGameUiModel::default();
    let mut movement = MovementInputState::default();
    let mut next = authority.session.projection.clone().expect("viewer");
    next.revision = 10;
    sync::apply(
        &mut authority,
        &mut state,
        &mut ui,
        &mut movement,
        Outcome::Snapshot(Box::new(next)),
    );
    assert_eq!(
        state.network,
        NetworkStatus::Pending,
        "background reads do not acknowledge our operation"
    );
    assert_eq!(
        state.viewer.as_ref().expect("viewer").revision,
        10,
        "newer complete data replaces authority"
    );
    sync::apply(
        &mut authority,
        &mut state,
        &mut ui,
        &mut movement,
        Outcome::Receipt(Box::new(ServerProjection {
            revision: 9,
            turn: 1,
            ..ServerProjection::default()
        })),
    );
    assert_eq!(
        state.network,
        NetworkStatus::Ready,
        "own receipt closes pending state even if superseded"
    );
    assert_eq!(
        state.viewer.as_ref().expect("viewer").revision,
        10,
        "old cached acknowledgement cannot undo newer state"
    );
}
#[test]
fn recovery_replaces_same_revision_completely_and_queued_is_not_completed_movement() {
    let mut authority = fixture();
    let mut state = state_from_authority_projection(
        &authority,
        authority.session.projection.as_ref().expect("viewer"),
    );
    let mut ui = InGameUiModel::default();
    let mut movement = MovementInputState::default();
    sync::apply(
        &mut authority,
        &mut state,
        &mut ui,
        &mut movement,
        Outcome::Recovering("lost response".to_owned()),
    );
    assert_eq!(
        state.network,
        NetworkStatus::Resynchronizing,
        "explicit rollback/recovery state"
    );
    state.width = 1;
    let current = authority
        .session
        .projection
        .clone()
        .expect("complete authoritative viewer");
    sync::apply(
        &mut authority,
        &mut state,
        &mut ui,
        &mut movement,
        Outcome::Snapshot(Box::new(current.clone())),
    );
    assert_eq!(
        state.width, 20,
        "even same-revision recovery must replace the complete model"
    );
    assert_eq!(
        state.network,
        NetworkStatus::Ready,
        "safe current read releases recovery"
    );
    let mut queued = current;
    queued.revision = 10;
    queued.interface.planning_status = "queued".to_owned();
    sync::apply(
        &mut authority,
        &mut state,
        &mut ui,
        &mut movement,
        Outcome::Receipt(Box::new(queued)),
    );
    assert!(
        ui.status_text.contains("queued"),
        "acknowledged intent is visibly distinct from execution"
    );
    assert!(
        state.units.is_empty(),
        "queued receipt never invents unit movement"
    );
}
