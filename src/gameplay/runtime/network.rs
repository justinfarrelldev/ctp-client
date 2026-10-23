//! Worker-owned IO. SDL and authoritative render state remain on the UI thread.
use crate::gameplay::server::{
    ServerClient, ServerGameplaySession, ServerProjection, envelope::PreparedCommand,
};
use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};
use recovery::{ReadGate, RefreshRequests};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;
use std::time::Duration;

/// Original queue-clear questions use inert source-shaped command channels only.
#[cfg(test)]
mod confirmation_tests;
/// Historical display results must not become transport recovery/acknowledgements.
#[cfg(test)]
mod notice_tests;
/// Source research modal journeys use inert real-shaped channels, never services.
#[cfg(test)]
mod research_ui_tests;
/// Shared admission and revision-aware recovery, independent of SDL state.
mod recovery;
/// Fake-time recovery/coalescing regressions perform no IO.
#[cfg(test)]
mod recovery_tests;

/// Worker messages contain complete viewer state, never canonical patches.
#[derive(Debug)]
pub(super) enum Outcome {
    /// Validated command receipt or current-resync snapshot.
    Snapshot(Box<ServerProjection>),
    /// Acknowledges the one submitted user operation, not a background read.
    Receipt(Box<ServerProjection>),
    /// A discrepancy requires explicit resynchronizing presentation.
    Recovering(String),
    /// Current read failed; input stays blocked until a later successful read.
    Failure(String),
    /// Safe admission/validation failure persists after a successful refresh.
    Rejected(crate::gameplay::server::rejection::Rejection),
}

/// Injectable service boundary used by pure retry/recovery tests.
pub(super) trait Transport {
    /// Submit precisely the captured operation, without regenerating its key.
    fn submit(&mut self, command: &PreparedCommand) -> Result<ServerProjection, crate::gameplay::GameplayError>;
    /// Obtain the complete current authorized viewer model.
    fn resync(&mut self) -> Result<ServerProjection, String>;
}
/// Production transport; owns no SDL resource.
struct HttpTransport {
    /// Endpoint and request settings.
    client: ServerClient,
    /// Game membership and session IDs.
    session: ServerGameplaySession,
    /// All event endpoint connections share admission, including recovery reads.
    gate: Arc<ReadGate>,
    /// UI departure cancels a budget wait without blocking SDL shutdown.
    stop: Arc<AtomicBool>,
}
impl Transport for HttpTransport {
    fn submit(&mut self, command: &PreparedCommand) -> Result<ServerProjection, crate::gameplay::GameplayError> {
        command.execute(&self.client)?.viewer_projection.ok_or_else(|| crate::gameplay::runtime_error("receipt has no viewer projection".to_owned()))
    }
    fn resync(&mut self) -> Result<ServerProjection, String> {
        if !self.gate.wait(&self.stop) {
            return Err("gameplay session closed".to_owned());
        }
        self.client
            .fetch_resync_projection(self.session.game_id, self.session.session_id)
            .map_err(|error| error.to_string())
    }
}

/// One command's retry/recovery policy, independent of any thread or real service.
pub(super) fn execute(
    transport: &mut impl Transport,
    command: &PreparedCommand,
    emit: &mut impl FnMut(Outcome),
) {
    let mut result = transport.submit(command);
    if result.as_ref().is_err_and(|error| {
        matches!(*error,crate::gameplay::GameplayError::Runtime { .. }) && {
            let message=error.to_string();
            message.contains("failed to read") || message.contains("failed to write") || message.contains("could not connect")
        }
    }) {
        // The first execution may have committed. Reuse the exact body/key.
        result = transport.submit(command);
    }
    match result {
        Ok(snapshot) => emit(Outcome::Receipt(Box::new(snapshot))),
        Err(error) => {
            match error {
                crate::gameplay::GameplayError::Rejected { rejection } => emit(Outcome::Rejected(rejection)),
                other => emit(Outcome::Recovering(other.to_string())),
            }
            refresh(transport, emit);
        }
    }
}

/// A stale current read cannot finish recovery: UI correctly rejects it, so
/// retain retry state. A cached own receipt still acknowledges without regression.
fn complete(outcome: Outcome, known: &mut i64) -> Outcome {
    match outcome {
        Outcome::Snapshot(snapshot) if snapshot.revision < *known => Outcome::Failure(format!(
            "Current snapshot revision {} is older than received revision {}",
            snapshot.revision, *known
        )),
        Outcome::Snapshot(snapshot) => {
            *known = (*known).max(snapshot.revision);
            Outcome::Snapshot(snapshot)
        }
        Outcome::Receipt(snapshot) => {
            *known = (*known).max(snapshot.revision);
            Outcome::Receipt(snapshot)
        }
        Outcome::Recovering(error) => Outcome::Recovering(error),
        Outcome::Failure(error) => Outcome::Failure(error),
        Outcome::Rejected(rejection) => Outcome::Rejected(rejection),
    }
}
/// Full current-model recovery, never a local fallback or partial event patch.
fn refresh(transport: &mut impl Transport, emit: &mut impl FnMut(Outcome)) {
    match transport.resync() {
        Ok(snapshot) => emit(Outcome::Snapshot(Box::new(snapshot))),
        Err(error) => emit(Outcome::Failure(error)),
    }
}

/// Bounded worker channels; at most one user operation is outstanding.
#[derive(Debug)]
pub(super) struct Network {
    /// Exactly captured jobs; bounded to avoid input flooding.
    jobs: SyncSender<PreparedCommand>,
    /// Complete results/recovery states consumed by the UI thread.
    results: Receiver<Outcome>,
    /// Cooperative cancellation; read timeouts bound stream shutdown latency.
    stop: Arc<AtomicBool>,
}
impl Network {
    /// Starts independent command/current-read and invalidation workers.
    ///
    /// # Errors
    /// Returns thread creation failures without panicking or leaking a watcher.
    pub(super) fn start(
        client: ServerClient,
        session: ServerGameplaySession,
    ) -> Result<Self, String> {
        let (jobs, requests) = mpsc::sync_channel::<PreparedCommand>(1);
        let (result_sink, results) = mpsc::sync_channel::<Outcome>(8);
        let (invalid_sink, invalidations) = mpsc::sync_channel::<()>(1);
        let stop = Arc::new(AtomicBool::new(false));
        let gate = Arc::new(ReadGate::new());
        let refresh_requests = Arc::new(RefreshRequests::default());
        let watcher_gate = Arc::clone(&gate);
        let watcher_refresh = Arc::clone(&refresh_requests);
        let command_refresh = Arc::clone(&refresh_requests);
        let watcher_stop = Arc::clone(&stop);
        let watcher_client = client.clone();
        let watcher_session = session.clone();
        let _watcher = thread::Builder::new()
            .name("ctp-events".to_owned())
            .spawn(move || {
                let mut cursor = 0_u64;
                while watcher_gate.wait(&watcher_stop) {
                    let mut notify = |revision| {
                        watcher_refresh.observe(revision);
                        let _delivery = invalid_sink.try_send(());
                    };
                    if let Err(error) = crate::gameplay::server::stream::watch(
                        &watcher_client,
                        &watcher_session,
                        &mut cursor,
                        &watcher_stop,
                        &mut notify,
                    ) {
                        log::warn!("Gameplay stream reconnect: {error}");
                    }
                    // A disconnect requires one current read, but never a400ms
                    // reconnect/current-read storm on the shared limited route.
                    if !watcher_stop.load(Ordering::Relaxed) {
                        notify(None);
                    }
                }
            })
            .map_err(|error| error.to_string())?;
        let command_stop = Arc::clone(&stop);
        let known_initial = session.revision();
        let commands = thread::Builder::new()
            .name("ctp-commands".to_owned())
            .spawn(move || {
                let mut transport = HttpTransport {
                    client,
                    session,
                    gate,
                    stop: Arc::clone(&command_stop),
                };
                let mut needs_recovery = false;
                let mut known_revision = known_initial;
                while !command_stop.load(Ordering::Relaxed) {
                    let recovering = needs_recovery;
                    let known_before = known_revision;
                    let mut emit = |outcome| {
                        let outcome = complete(outcome, &mut known_revision);
                        needs_recovery = matches!(outcome, Outcome::Failure(_));
                        // UI departure disconnects the receiver, so a worker cannot
                        // retain/mutate a later SDL session or block shutdown forever.
                        let _delivery = result_sink.send(outcome);
                    };
                    match requests.recv_timeout(Duration::from_millis(100)) {
                        Ok(command) => execute(&mut transport, &command, &mut emit),
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            // Wakeups are only hints; already-covered own events
                            // must not force another limited current read.
                            while invalidations.try_recv().is_ok() {}
                            if command_refresh.take_needed(known_before) || recovering {
                                refresh(&mut transport, &mut emit);
                            }
                        }
                    }
                }
            });
        if let Err(error) = commands {
            stop.store(true, Ordering::Relaxed);
            return Err(error.to_string());
        }
        Ok(Self {
            jobs,
            results,
            stop,
        })
    }

    /// Nonblocking enqueue; caller disables further authoritative input until receipt.
    pub(super) fn submit(&self, command: PreparedCommand) -> Result<(), String> {
        self.jobs
            .try_send(command)
            .map_err(|error| error.to_string())
    }

    /// One result per frame leaves recovery presentation visible before replacement.
    pub(super) fn poll(&self) -> Option<Outcome> {
        self.results.try_recv().ok()
    }
}
impl Drop for Network {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        // Workers hold only owned transport data; bounded socket timeouts and
        // receiver disconnection release them without joining from the SDL thread.
    }
}
