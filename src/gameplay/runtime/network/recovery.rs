//! Coalesced revision observations and shared admission for the SSE route.
use core::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::{
    thread,
    time::{Duration, Instant},
};
/// Current reads and stream connections share the server's20/minute limiter.
/// Leave margin; the already-completed bootstrap consumed the first admission.
const READ_INTERVAL_MS: u64 = 3_100;
/// Metadata may request a read, but never changes canonical model/acknowledgement.
#[derive(Debug, Default)]
pub(super) struct RefreshRequests {
    /// Highest observed revision; sequences never enter this namespace.
    revision: AtomicI64,
    /// Disconnect/explicit recovery survives coalescing behind an old event.
    forced: AtomicBool,
}
impl RefreshRequests {
    /// Records only a hint or mandatory read. No snapshot is fabricated here.
    pub(super) fn observe(&self, revision: Option<i64>) {
        if let Some(current) = revision.filter(|revision| *revision >= 0_i64) {
            self.revision.fetch_max(current, Ordering::Relaxed);
        } else {
            self.forced.store(true, Ordering::Relaxed);
        }
    }
    /// Own command receipts/full snapshots already cover matching event revisions.
    pub(super) fn take_needed(&self, known: i64) -> bool {
        self.forced.swap(false, Ordering::Relaxed) || self.revision.load(Ordering::Relaxed) > known
    }
}
/// Two worker threads reserve a shared monotonic connection budget, without a
/// mutex or any UI-thread sleep. Reservations remain bounded/cooperatively stopped.
#[derive(Debug)]
pub(super) struct ReadGate {
    /// Monotonic epoch; wall-clock corrections cannot create request bursts.
    epoch: Instant,
    /// Next lease in epoch milliseconds, shared only by network workers.
    next: AtomicU64,
}
impl ReadGate {
    /// Bootstrap just fetched the initial viewer on this same limited route.
    pub(super) fn new() -> Self {
        Self {
            epoch: Instant::now(),
            next: AtomicU64::new(READ_INTERVAL_MS),
        }
    }
    /// Pure fake-time admission; returns the slot rather than sleeping in tests.
    pub(super) fn reserve_at(&self, now: u64) -> u64 {
        let mut previous = self.next.load(Ordering::Relaxed);
        loop {
            let slot = previous.max(now);
            match self.next.compare_exchange_weak(
                previous,
                slot.saturating_add(READ_INTERVAL_MS),
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return slot,
                Err(current) => previous = current,
            }
        }
    }
    /// Waits only on the owning worker; cancellation is checked at most50ms apart.
    pub(super) fn wait(&self, stop: &AtomicBool) -> bool {
        let started=Instant::now();
        let slot =
            self.reserve_at(u64::try_from(self.epoch.elapsed().as_millis()).unwrap_or(u64::MAX));
        while !stop.load(Ordering::Relaxed) {
            let now = u64::try_from(self.epoch.elapsed().as_millis()).unwrap_or(u64::MAX);
            if now >= slot {
                log::debug!("Gameplay read client_admission_wait_ms={}",started.elapsed().as_millis());
                return true;
            }
            thread::sleep(Duration::from_millis(slot.saturating_sub(now).min(50_u64)));
        }
        false
    }
}
