//! Independent command/background IO and bounded complete-model delivery.
use super::{Outcome, Transport, complete, execute, recovery::RefreshRequests};
use crate::gameplay::server::{ServerProjection, envelope::PreparedCommand};
use core::sync::atomic::{AtomicBool, Ordering};
use std::{sync::{Mutex, MutexGuard, TryLockError, mpsc::{self, Receiver, SyncSender}}, time::Duration};

/// Small transport ledger, not SDL or mutable gameplay authority.
#[derive(Debug)]
struct Delivered {
    /// Highest complete model, never an invalidation hint.
    known: i64,
    /// Complete-model deliveries distinguish superseded same-revision reads too.
    generation: u64,
    /// Failed current reads retry until actual complete data arrives.
    recovering: bool,
    /// Only the newest complete stream model needs a frame slot.
    latest: Option<Box<ServerProjection>>,
}

/// Ordered acknowledgements/faults plus a single coalesced stream model.
#[derive(Debug)]
pub(super) struct Delivery {
    /// Receipts/recovery cannot be coalesced away or confused with stream data.
    results: SyncSender<Outcome>,
    /// One tiny ledger serializes competing completion decisions. No decoding,
    /// admission wait, HTTP request or gameplay work occurs under this lock.
    state: Mutex<Delivered>,
}
impl Delivery {
    /// One bounded result channel and mailbox per gameplay lifetime.
    pub(super) const fn new(known: i64, results: SyncSender<Outcome>) -> Self {
        Self { results, state:Mutex::new(Delivered { known, generation:0_u64, recovering:false, latest:None }) }
    }
    /// A panicked transport cannot poison the authoritative UI model: only this
    /// owned, already-decoded ledger is recovered, never partially executed data.
    fn ledger(&self) -> MutexGuard<'_, Delivered> {
        match self.state.lock() { Ok(state) => state, Err(poisoned) => poisoned.into_inner() }
    }
    /// Complete progress and failed-read status captured atomically before IO.
    fn progress(&self) -> (i64, u64, bool) {
        let state = self.ledger();
        (state.known, state.generation, state.recovering)
    }
    /// Serialize completion ordering; the bounded channel may backpressure only
    /// IO lanes. SDL drains this channel before taking the stream ledger, so a
    /// full outcome queue never makes SDL wait behind a blocked producer here.
    fn publish(&self, state: &mut Delivered, outcome: Outcome) {
        let outcome = complete(outcome, &mut state.known);
        if matches!(outcome, Outcome::Receipt(_) | Outcome::Snapshot(_)) {
            state.generation = state.generation.saturating_add(1_u64);
        }
        // Rejection/Recovering already trigger one inline command recovery read;
        // only its failure starts background retries, avoiding duplicate reads.
        state.recovering = matches!(outcome, Outcome::Failure(_));
        let _delivery = self.results.send(outcome);
    }
    /// Deliver command results without allowing a stream to acknowledge them.
    fn emit(&self, outcome: Outcome) { self.publish(&mut self.ledger(), outcome); }
    /// A concurrent receipt/stream can supersede an optional read in flight.
    /// Check and publish under the same lock: a stale failure cannot race behind
    /// an already-presented fresh model and reopen false recovery.
    fn current(&self, generation: u64, result: Result<ServerProjection, String>) {
        self.current_model(generation, result.map(Box::new));
    }
    /// Inline command recovery uses the same completion guard as background reads;
    /// its projection is already boxed, so do not allocate/copy that model again.
    fn current_model(&self, generation: u64, result: Result<Box<ServerProjection>, String>) {
        let candidate = result.as_ref().ok().map(|snapshot| snapshot.revision);
        let outcome = match result {
            Ok(snapshot) => Outcome::Snapshot(snapshot),
            Err(error) => Outcome::Failure(error),
        };
        let mut state = self.ledger();
        if state.generation > generation && candidate.is_none_or(|revision| revision < state.known) { return; }
        self.publish(&mut state, outcome);
        drop(state);
    }
    /// Coalesce complete models, never patches, errors or receipt identity.
    pub(super) fn stream(&self, snapshot: Box<ServerProjection>) {
        let mut state = self.ledger();
        if snapshot.revision < state.known { return; }
        state.known = snapshot.revision;
        state.generation = state.generation.saturating_add(1_u64);
        state.latest = Some(snapshot);
        state.recovering = false;
    }
    /// SDL checks ordered outcomes first, retaining explicit recovery frames.
    /// Never block here: the outcome queue can fill AFTER SDL sampled it empty,
    /// leaving a producer holding the ledger under channel backpressure. Defer
    /// the optional model to another frame, when SDL can drain outcomes first.
    pub(super) fn take_stream(&self) -> Option<Outcome> {
        let mut state = match self.state.try_lock() {
            Ok(state) => state,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => return None,
        };
        state.latest.take().map(Outcome::Snapshot)
    }
}

/// Background admission/HTTP cannot monopolize the command receiving thread.
pub(super) fn command_loop(
    mut transport: impl Transport,
    requests: &Receiver<PreparedCommand>,
    delivery: &Delivery,
    stop: &AtomicBool,
) {
    while !stop.load(Ordering::Relaxed) {
        match requests.recv_timeout(Duration::from_millis(100_u64)) {
            Ok(command) => {
                // execute always emits a command error marker before starting
                // its inline current read. Capture the generation at that point,
                // not before command HTTP (earlier progress cannot supersede a
                // later genuine failure). Initial submission needs no ledger lock.
                let mut generation = 0_u64;
                execute(&mut transport, &command, &mut |outcome| match outcome {
                    Outcome::Receipt(snapshot) => delivery.emit(Outcome::Receipt(snapshot)),
                    marker @ (Outcome::Recovering(_) | Outcome::Rejected(_)) => {
                        delivery.emit(marker);
                        generation = delivery.progress().1;
                    }
                    Outcome::Snapshot(snapshot) => delivery.current_model(generation, Ok(snapshot)),
                    Outcome::Failure(error) => delivery.current_model(generation, Err(error)),
                });
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
    }
}

/// Legacy invalidations and mandatory recovery still obtain complete current
/// models. Their waits never hold the completion ledger or command receiving lane.
pub(super) fn background_loop(
    mut transport: impl Transport,
    invalidations: &Receiver<()>,
    refresh: &RefreshRequests,
    delivery: &Delivery,
    stop: &AtomicBool,
) {
    while !stop.load(Ordering::Relaxed) {
        match invalidations.recv_timeout(Duration::from_millis(100_u64)) {
            Ok(()) | Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        while invalidations.try_recv().is_ok() {}
        let (known, generation, recovering) = delivery.progress();
        if refresh.take_needed(known) || recovering {
            delivery.current(generation, transport.resync());
        }
    }
}

#[cfg(test)]
mod tests {
    //! Completion ordering and genuine recovery use no threads or external IO.
    use super::{Delivery, Outcome, ServerProjection};
    use std::sync::mpsc;

    #[test]
    fn fresh_same_revision_model_supersedes_failed_optional_read_but_not_receipt_identity() {
        let (sink, results) = mpsc::sync_channel(8_usize);
        let delivery = Delivery::new(9_i64, sink);
        let (_, generation, _) = delivery.progress();
        delivery.stream(Box::new(ServerProjection { revision:9_i64, turn:2_u32, ..ServerProjection::default() }));
        delivery.current(generation, Err("superseded".to_owned()));
        assert!(results.try_recv().is_err(), "fresh complete same-revision data prevents false recovery");
        assert!(matches!(delivery.take_stream(),Some(Outcome::Snapshot(_))), "never acknowledges own command");
        delivery.emit(Outcome::Receipt(Box::new(ServerProjection { revision:8_i64, turn:1_u32, ..ServerProjection::default() })));
        assert!(matches!(results.try_recv(),Ok(Outcome::Receipt(_))), "old cached own receipt still acknowledges");
        assert_eq!(delivery.progress().0,9_i64,"receipt cannot regress known state");
    }

    #[test]
    fn busy_completion_ledger_never_blocks_ui_or_discards_latest_model() {
        let (sink, _results) = mpsc::sync_channel(1_usize);
        let delivery = alloc::sync::Arc::new(Delivery::new(9_i64,sink));
        delivery.stream(Box::new(ServerProjection { revision:9_i64, turn:2_u32, ..ServerProjection::default() }));
        // A worker may hold this ledger while outcome delivery backpressures.
        // SDL can have sampled an empty queue just before that worker filled it.
        let held = delivery.ledger();
        let ui_delivery = alloc::sync::Arc::clone(&delivery);
        let (done,result) = mpsc::sync_channel(1_usize);
        let ui = std::thread::spawn(move || { let deferred = ui_delivery.take_stream().is_none(); done.send(deferred).expect("inert UI result"); });
        let nonblocking = result.recv_timeout(std::time::Duration::from_secs(1_u64));
        drop(held);
        ui.join().expect("bounded fake UI");
        assert!(matches!(nonblocking,Ok(true)),"busy optional mailbox defers instead of waiting on an IO producer");
        assert!(matches!(delivery.take_stream(),Some(Outcome::Snapshot(snapshot)) if snapshot.revision==9_i64),"deferring one frame must retain the complete model");
    }

    #[test]
    fn genuinely_stale_current_read_retries_until_complete_current_data_arrives() {
        let (sink, results) = mpsc::sync_channel(8_usize);
        let delivery = Delivery::new(9_i64, sink);
        delivery.current(0_u64, Ok(ServerProjection { revision:8_i64, turn:1_u32, ..ServerProjection::default() }));
        assert!(matches!(results.try_recv(),Ok(Outcome::Failure(_))), "uncovered stale reads stay explicit failures");
        assert!(delivery.progress().2,"retain retry status");
        delivery.current(0_u64, Ok(ServerProjection { revision:9_i64, turn:2_u32, ..ServerProjection::default() }));
        assert!(matches!(results.try_recv(),Ok(Outcome::Snapshot(_))), "complete recovery is data, not an acknowledgement");
        assert!(!delivery.progress().2,"covered current data ends retrying");
    }
}
