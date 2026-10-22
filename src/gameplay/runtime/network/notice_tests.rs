//! Historical display notices cannot become current transport recovery/receipts.
use super::{Network, Outcome, PreparedCommand};
use crate::gameplay::{
    model::NetworkStatus,
    runtime::{
        MovementInputState, RuntimeAuthority, authority::state_from_authority_projection, sync,
    },
    server::{
        ServerClient, ServerConfig, ServerGameplaySession, ServerProjection, v2::Notification,
    },
    ui_model::InGameUiModel,
};
use alloc::sync::Arc;
use core::sync::atomic::AtomicBool;
use serde_json::json;
use std::sync::mpsc::{self, Receiver, SyncSender};
/// Injected production-shaped channels only; no worker/socket/file is created.
struct Harness {
    /// Authority with inert, observable channels instead of live transport.
    authority: RuntimeAuthority,
    /// Exact submitted user requests; notice display must never add one.
    jobs: Receiver<PreparedCommand>,
    /// Inject complete UI-thread outcomes without contacting a service.
    results: SyncSender<Outcome>,
}
/// Stable old rejected-command identity, not the current pending command.
fn rejected() -> Notification {
    Notification {
        id: "historical-rejection".to_owned(),
        kind: "order.rejected".to_owned(),
        command_id: "old-command".to_owned(),
        turn: 1_u32,
        ..Notification::default()
    }
}
/// Read-only schema fixture and in-memory channels, with no thread start.
fn fixture() -> Harness {
    let projection:ServerProjection=serde_json::from_value(json!({"revision":9_i64,"turn":2_u32,"world":{"min_x":0_u16,"min_y":0_u16,"width":20_u16,"height":12_u16},"tiles":[{"id":"b","x":1_u16,"y":1_u16,"terrain":"TERRAIN_PLAINS"}],"units":[{"id":"own-current","owner_id":"account-3","tile_id":"b","type_id":"UNIT_SETTLER","movement":2_i16,"max_movement":2_i16}]})).expect("authorized viewer");
    let (job_sink, jobs) = mpsc::sync_channel(1_usize);
    let (results, result_receiver) = mpsc::sync_channel(8_usize);
    let io = Network {
        jobs: job_sink,
        results: result_receiver,
        stop: Arc::new(AtomicBool::new(false)),
    };
    let authority = RuntimeAuthority {
        client: ServerClient::from_config(&ServerConfig {
            base_url: "http://127.0.0.1:1".to_owned(),
            required: true,
        })
        .expect("parse unused endpoint only"),
        session: ServerGameplaySession {
            game_id: 1_i64,
            session_id: 2_i64,
            account_id: 3_i64,
            projection: Some(projection),
        },
        empire_count: 1_u8,
        seed: 1_u64,
        io: Some(io),
    };
    Harness {
        authority,
        jobs,
        results,
    }
}
#[test]
fn historical_rejection_display_preserves_ready_pending_and_recovery_without_rolling_back_unrelated_presentation_or_requesting_another_read()
 {
    for status in [
        NetworkStatus::Ready,
        NetworkStatus::Pending,
        NetworkStatus::Resynchronizing,
    ] {
        let mut harness = fixture();
        let mut projection = harness
            .authority
            .session
            .projection
            .clone()
            .expect("viewer");
        projection.interface.notifications = vec![rejected()];
        let mut state = state_from_authority_projection(&harness.authority, &projection);
        state.network = status;
        let mut ui = InGameUiModel::default();
        let mut movement = MovementInputState::default();
        let unit = state.selected_unit().expect("current owned actor");
        let target = crate::gameplay::model::MapPoint::new(2_u16, 1_u16);
        movement.actors.preview(unit, target);
        movement.actors.remember(unit.id, &[target]);
        movement.actors.begin_frame(10_u64);
        let before = movement.clone();
        sync::frame(
            Some(&mut harness.authority),
            &mut state,
            &mut ui,
            &mut movement,
        );
        assert_eq!(
            state.network, status,
            "historical display cannot own live synchronization status"
        );
        assert_eq!(
            movement, before,
            "old rejection cannot cancel/rollback newer unrelated presentation or input"
        );
        assert!(
            harness.jobs.try_recv().is_err(),
            "display cannot enqueue an operation"
        );
        assert_eq!(
            ui.messages.items.len(),
            1_usize,
            "received fact is still displayed"
        );
        sync::frame(
            Some(&mut harness.authority),
            &mut state,
            &mut ui,
            &mut movement,
        );
        assert_eq!(
            ui.messages.items.len(),
            1_usize,
            "same revision/stable ID idempotent"
        );
        assert_eq!(state.network, status, "repeat display unchanged");
    }
}
#[test]
fn actual_transport_disagreement_still_rolls_back_and_verified_current_snapshot_finishes_recovery_even_with_a_rejection_notice()
 {
    let mut harness = fixture();
    let mut state = state_from_authority_projection(
        &harness.authority,
        harness
            .authority
            .session
            .projection
            .as_ref()
            .expect("viewer"),
    );
    let mut ui = InGameUiModel::default();
    let mut movement = MovementInputState::default();
    movement.actors.preview(
        state.selected_unit().expect("own actor"),
        crate::gameplay::model::MapPoint::new(2_u16, 1_u16),
    );
    let unit = state.selected_unit().expect("current");
    let camera = crate::gameplay::runtime::Camera::for_test_identity();
    assert!(
        movement.actors.pose(unit, &camera, 500_u64).moving,
        "in-flight presentation to roll back"
    );
    harness
        .results
        .send(Outcome::Recovering(
            "explicit current-command disagreement".to_owned(),
        ))
        .expect("injected failure");
    sync::frame(
        Some(&mut harness.authority),
        &mut state,
        &mut ui,
        &mut movement,
    );
    assert_eq!(
        state.network,
        NetworkStatus::Resynchronizing,
        "real transport recovery still owns state"
    );
    assert!(
        !movement
            .actors
            .pose(state.selected_unit().expect("current"), &camera, 500_u64)
            .moving,
        "real mismatch still rolls back"
    );
    let mut current = harness
        .authority
        .session
        .projection
        .clone()
        .expect("viewer");
    current.revision = 10_i64;
    current.interface.notifications = vec![rejected()];
    harness
        .results
        .send(Outcome::Snapshot(Box::new(current)))
        .expect("verified full recovery");
    sync::frame(
        Some(&mut harness.authority),
        &mut state,
        &mut ui,
        &mut movement,
    );
    assert_eq!(
        state.network,
        NetworkStatus::Ready,
        "current snapshot completes actual recovery; historical message cannot reopen it"
    );
    assert_eq!(ui.messages.items.len(), 1_usize, "failure still displayed");
    assert_eq!(
        ui.status_text, "State resynchronized.",
        "live status belongs to completed transport recovery, not old notice"
    );
}
#[test]
fn complete_receipt_with_old_rejection_stays_ready_and_background_snapshot_cannot_acknowledge_newer_pending_command()
 {
    for receipt in [false, true] {
        let mut harness = fixture();
        let mut state = state_from_authority_projection(
            &harness.authority,
            harness
                .authority
                .session
                .projection
                .as_ref()
                .expect("viewer"),
        );
        state.network = NetworkStatus::Pending;
        let mut next = harness
            .authority
            .session
            .projection
            .clone()
            .expect("viewer");
        next.revision = 10_i64;
        next.interface.notifications = vec![rejected()];
        let outcome = if receipt {
            Outcome::Receipt(Box::new(next))
        } else {
            Outcome::Snapshot(Box::new(next))
        };
        harness.results.send(outcome).expect("injected result");
        let mut ui = InGameUiModel::default();
        let mut movement = MovementInputState::default();
        sync::frame(
            Some(&mut harness.authority),
            &mut state,
            &mut ui,
            &mut movement,
        );
        assert_eq!(
            state.network,
            if receipt {
                NetworkStatus::Ready
            } else {
                NetworkStatus::Pending
            },
            "only exact transport receipt acknowledges; notices do neither recovery nor acknowledgement"
        );
        assert_eq!(
            state.viewer.as_ref().expect("viewer").revision,
            10_i64,
            "complete authoritative facts preserved"
        );
        assert!(
            harness.jobs.try_recv().is_err(),
            "no notice-generated order"
        );
    }
}
