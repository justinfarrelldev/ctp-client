//! Worker progress uses inert transports/channels, never files or network IO.
use super::{Outcome, Transport, recovery::{ReadGate, RefreshRequests}, workers::{Delivery, background_loop, command_loop}};
use crate::gameplay::{GameplayError, server::{ServerGameplaySession, ServerProjection, envelope::PreparedCommand}};
use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};
use std::{sync::mpsc, thread, time::Duration};
use serde_json::json;

/// Frozen background read, released explicitly after the command has completed.
struct BlockedRead {
    /// Signals that background transport is waiting.
    entered: mpsc::SyncSender<()>,
    /// Test controls completion without a wall-clock sleep.
    release: mpsc::Receiver<()>,
    /// Also cover a superseded failed optional read.
    fail: bool,
}
impl Transport for BlockedRead {
    fn submit(&mut self, _command: &PreparedCommand) -> Result<ServerProjection, GameplayError> {
        Err(crate::gameplay::runtime_error("background worker cannot submit".to_owned()))
    }
    fn resync(&mut self) -> Result<ServerProjection, String> {
        self.entered.send(()).map_err(|error| error.to_string())?;
        self.release.recv().map_err(|error| error.to_string())?;
        if self.fail { return Err("late optional read failure".to_owned()); }
        Ok(ServerProjection { revision:7_i64, turn:1_u32, ..ServerProjection::default() })
    }
}
/// Independent command transport supplies the real-shaped committed receipt.
struct CommandTransport;
impl Transport for CommandTransport {
    fn submit(&mut self, _command: &PreparedCommand) -> Result<ServerProjection, GameplayError> {
        Ok(ServerProjection { revision:8_i64, turn:1_u32, ..ServerProjection::default() })
    }
    fn resync(&mut self) -> Result<ServerProjection, String> { Err("not needed".to_owned()) }
}

#[test]
fn blocked_background_refresh_cannot_delay_command_or_regress_its_receipt() {
    for fail in [false, true] {
    let (sink, results) = mpsc::sync_channel(8_usize);
    let delivery = Arc::new(Delivery::new(7_i64, sink));
    let stop = Arc::new(AtomicBool::new(false));
    let refresh = Arc::new(RefreshRequests::default());
    refresh.observe(None);
    let (invalid_sink, invalidations) = mpsc::sync_channel(1_usize);
    let (entered_sink, entered) = mpsc::sync_channel(1_usize);
    let (release, released) = mpsc::sync_channel(1_usize);
    let read_delivery = Arc::clone(&delivery);
    let read_stop = Arc::clone(&stop);
    let read = thread::spawn(move || background_loop(BlockedRead { entered:entered_sink, release:released, fail }, &invalidations, &refresh, &read_delivery, &read_stop));
    invalid_sink.send(()).expect("one pending invalidation");
    entered.recv_timeout(Duration::from_secs(2_u64)).expect("background transport entered");
    let (jobs, requests) = mpsc::sync_channel(1_usize);
    let command_delivery = Arc::clone(&delivery);
    let command_stop = Arc::clone(&stop);
    let commands = thread::spawn(move || command_loop(CommandTransport, &requests, &command_delivery, &command_stop));
    let session = ServerGameplaySession { roster:crate::gameplay::server::roster::PlayerRoster::default(), game_id:1_i64, session_id:2_i64, account_id:3_i64, projection:Some(ServerProjection { revision:7_i64, turn:1_u32, ..ServerProjection::default() }) };
    jobs.send(PreparedCommand::new(&session,"pass",json!({})).expect("immutable command")).expect("bounded enqueue");
    let outcome = results.recv_timeout(Duration::from_secs(2_u64));
    stop.store(true, Ordering::Relaxed);
    release.send(()).expect("release blocked read");
    drop(jobs); drop(invalid_sink);
    commands.join().expect("bounded command worker");
    read.join().expect("bounded read worker");
    assert!(matches!(outcome,Ok(Outcome::Receipt(snapshot)) if snapshot.revision==8_i64), "command receipt arrives BEFORE releasing optional read");
    assert!(results.try_recv().is_err(), "superseded current read neither regresses state nor creates false recovery");
    }
}

#[test]
fn snapshot_delivery_coalesces_models_without_acknowledging_a_user_command() {
    let (sink, results) = mpsc::sync_channel(8_usize);
    let delivery = Delivery::new(7_i64, sink);
    for revision in [8_i64, 10_i64, 9_i64] { delivery.stream(Box::new(ServerProjection { revision, turn:2_u32, ..ServerProjection::default() })); }
    assert!(results.try_recv().is_err(), "stream snapshots never masquerade as command receipts");
    assert!(matches!(delivery.take_stream(),Some(Outcome::Snapshot(snapshot)) if snapshot.revision==10_i64), "full states coalesce monotonically to newest complete model");
    assert!(delivery.take_stream().is_none(), "mailbox stays bounded to one model");
}

#[test]
fn negotiated_subscription_has_immediate_independent_admission_but_legacy_budget_is_unchanged() {
    let stream = ReadGate::stream();
    assert_eq!(stream.reserve_at(0_u64),0_u64,"bootstrap current read does not consume snapshot subscription budget");
    assert!(stream.reserve_at(0_u64)>=3_000_u64,"reconnects still obey subscription limit");
    assert!(ReadGate::new().reserve_at(0_u64)>=3_000_u64,"legacy shared20/minute budget is preserved");
}
