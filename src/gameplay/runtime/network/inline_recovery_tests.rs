//! Inline command recovery races use inert transports/channels, never services/files.
use super::workers::{Delivery, command_loop};
use super::{Outcome, Transport};
use crate::gameplay::GameplayError;
use crate::gameplay::server::{
    ServerGameplaySession, ServerProjection, envelope::PreparedCommand, rejection::Rejection,
};
use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};
use serde_json::{Value, json};
use std::{sync::mpsc, thread, time::Duration};

/// Exact ordering chosen by the test, not a wall-clock performance assertion.
#[derive(Clone, Copy)]
struct Case {
    /// Preserve either an actual typed rejection or an uncertain command result.
    rejected: bool,
    /// The inline current read fails instead of returning revision7.
    failed_read: bool,
    /// A complete model delivered while that current read is explicitly blocked.
    during_read: Option<i64>,
    /// Earlier progress must not suppress a genuine later recovery failure.
    before_error: bool,
}

/// Fake command transport exposes the current-read boundary to the test thread.
struct RecoveryTransport {
    /// Captured command body, including immutable request identity.
    expected: Value,
    /// Which ordinary command error path to exercise.
    rejected: bool,
    /// Whether to fail the current read.
    failed_read: bool,
    /// Optional complete progress strictly before the command error.
    before_error: Option<Arc<Delivery>>,
    /// Signals that the recovery read has begun, after the ordered error marker.
    entered: mpsc::SyncSender<()>,
    /// No current response is released until the test explicitly permits it.
    release: mpsc::Receiver<()>,
}
impl Transport for RecoveryTransport {
    fn submit(&mut self, command: &PreparedCommand) -> Result<ServerProjection, GameplayError> {
        if command.body != self.expected {
            return Err(crate::gameplay::runtime_error(
                "inert command identity changed".to_owned(),
            ));
        }
        if let Some(delivery) = self.before_error.as_ref() {
            delivery.stream(Box::new(model(9_i64)));
        }
        if self.rejected {
            Err(GameplayError::Rejected {
                rejection: Rejection {
                    status: 400_u16,
                    code: "command_invalid".to_owned(),
                    message: "The server rejected this order.".to_owned(),
                    correlation_id: Some("inert-rejection".to_owned()),
                },
            })
        } else {
            Err(crate::gameplay::runtime_error(
                "inert command uncertainty".to_owned(),
            ))
        }
    }
    fn resync(&mut self) -> Result<ServerProjection, String> {
        self.entered.send(()).map_err(|error| error.to_string())?;
        self.release.recv().map_err(|error| error.to_string())?;
        if self.failed_read {
            Err("inert current read failed".to_owned())
        } else {
            Ok(model(7_i64))
        }
    }
}

/// Complete fake model; turn advances only in explicitly received revision9.
fn model(revision: i64) -> ServerProjection {
    ServerProjection {
        revision,
        turn: if revision == 7_i64 { 1_u32 } else { 2_u32 },
        ..ServerProjection::default()
    }
}

/// Observed ordered marker, optional model and post-read outcomes.
#[derive(Debug)]
struct Observation {
    /// Original command rejection or transport uncertainty, never suppressed.
    marker: Option<Outcome>,
    /// Full model delivered independently from the command's acknowledgement.
    stream: Option<Outcome>,
    /// Outcomes published after releasing the inline current read.
    after_read: Vec<Outcome>,
}

/// Exercise production command-loop ordering; all fake worker threads are joined.
fn run(case: Case) -> Result<Observation, String> {
    let (sink, results) = mpsc::sync_channel(8_usize);
    let delivery = Arc::new(Delivery::new(7_i64, sink));
    let stop = Arc::new(AtomicBool::new(false));
    let (entered_sink, entered) = mpsc::sync_channel(1_usize);
    let (release, released) = mpsc::sync_channel(1_usize);
    let (jobs, requests) = mpsc::sync_channel(1_usize);
    let session = ServerGameplaySession {
        roster: crate::gameplay::server::roster::PlayerRoster::default(),
        game_id: 1_i64,
        session_id: 2_i64,
        account_id: 3_i64,
        projection: Some(model(7_i64)),
    };
    let command =
        PreparedCommand::new(&session, "pass", json!({})).map_err(|error| error.to_string())?;
    let transport = RecoveryTransport {
        expected: command.body.clone(),
        rejected: case.rejected,
        failed_read: case.failed_read,
        before_error: case.before_error.then(|| Arc::clone(&delivery)),
        entered: entered_sink,
        release: released,
    };
    let command_delivery = Arc::clone(&delivery);
    let command_stop = Arc::clone(&stop);
    let worker = thread::spawn(move || {
        command_loop(transport, &requests, &command_delivery, &command_stop);
    });
    jobs.send(command).map_err(|error| error.to_string())?;
    let started = entered.recv_timeout(Duration::from_secs(2_u64));
    let marker = results.recv_timeout(Duration::from_secs(2_u64)).ok();
    if let Some(revision) = case.during_read {
        delivery.stream(Box::new(model(revision)));
    }
    let stream = delivery.take_stream();
    stop.store(true, Ordering::Relaxed);
    let release_result = release.send(());
    drop(jobs);
    let joined = worker.join();
    started.map_err(|error| error.to_string())?;
    release_result.map_err(|error| error.to_string())?;
    joined.map_err(|_panic| "inert command worker failed".to_owned())?;
    Ok(Observation {
        marker,
        stream,
        after_read: results.try_iter().collect(),
    })
}

/// Check the preserved error identity without confusing it with a receipt.
fn marker_matches(outcome: Option<Outcome>, rejected: bool) -> bool {
    match outcome {
        Some(Outcome::Rejected(rejection)) => {
            rejected
                && rejection.code == "command_invalid"
                && rejection.correlation_id.as_deref() == Some("inert-rejection")
        }
        Some(Outcome::Recovering(error)) => {
            !rejected && error.contains("inert command uncertainty")
        }
        None | Some(Outcome::Snapshot(_) | Outcome::Receipt(_) | Outcome::Failure(_)) => false,
    }
}

#[test]
fn inline_recovery_cannot_reopen_failure_after_a_complete_stream_delivery() {
    for rejected in [false, true] {
        for (failed_read, revision) in [(false, 9_i64), (true, 9_i64), (true, 7_i64)] {
            let observed = run(Case {
                rejected,
                failed_read,
                during_read: Some(revision),
                before_error: false,
            });
            assert!(observed.is_ok(), "bounded inert recovery: {observed:?}");
            if let Ok(observed) = observed {
                assert!(
                    marker_matches(observed.marker, rejected),
                    "preserve actual ordered rejection/uncertainty"
                );
                assert!(
                    matches!(observed.stream,Some(Outcome::Snapshot(snapshot)) if snapshot.revision == revision),
                    "received progress is a full snapshot, never an acknowledgement"
                );
                assert!(
                    observed.after_read.is_empty(),
                    "superseded inline current result cannot regress state or reopen recovery"
                );
            }
        }
    }
}

#[test]
fn inline_recovery_still_reports_genuine_failures_and_returns_current_data() {
    for rejected in [false, true] {
        for failed_read in [false, true] {
            for before_error in [false, true] {
                let observed = run(Case {
                    rejected,
                    failed_read,
                    during_read: None,
                    before_error,
                });
                assert!(observed.is_ok(), "bounded inert recovery: {observed:?}");
                if let Ok(observed) = observed {
                    assert!(
                        marker_matches(observed.marker, rejected),
                        "never suppress the command error"
                    );
                    let mut remaining = observed.after_read;
                    assert_eq!(
                        remaining.len(),
                        1_usize,
                        "without progress DURING the read its outcome is still required"
                    );
                    if failed_read || before_error {
                        assert!(
                            matches!(remaining.pop(), Some(Outcome::Failure(_))),
                            "genuine read failure/stale current data still requires recovery"
                        );
                    } else {
                        assert!(
                            matches!(remaining.pop(),Some(Outcome::Snapshot(snapshot)) if snapshot.revision == 7_i64),
                            "valid current data remains a snapshot, not a receipt"
                        );
                    }
                }
            }
        }
    }
}
