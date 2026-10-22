//! Pure request coalescing and shared SSE admission use fake times, no IO.
use super::recovery::{ReadGate, RefreshRequests};
use super::{Outcome, complete};
use crate::gameplay::server::ServerProjection;
#[test]
fn older_current_snapshot_retries_instead_of_silently_leaving_ui_stuck_but_older_receipt_still_acknowledges()
 {
    let mut known = 9_i64;
    let older = ServerProjection {
        revision: 8_i64,
        turn: 1_u32,
        ..ServerProjection::default()
    };
    assert!(
        matches!(
            complete(Outcome::Snapshot(Box::new(older.clone())), &mut known),
            Outcome::Failure(_)
        ),
        "UI rejects old snapshots, so worker must retain recovery rather than stop retrying"
    );
    assert_eq!(known, 9_i64, "never regress known complete revision");
    assert!(
        matches!(
            complete(Outcome::Receipt(Box::new(older)), &mut known),
            Outcome::Receipt(_)
        ),
        "cached receipt may acknowledge own operation without regressing newer snapshot"
    );
    let next = ServerProjection {
        revision: 10_i64,
        turn: 1_u32,
        ..ServerProjection::default()
    };
    assert!(
        matches!(
            complete(Outcome::Snapshot(Box::new(next)), &mut known),
            Outcome::Snapshot(_)
        ),
        "new full current state recovers normally"
    );
    assert_eq!(
        known, 10_i64,
        "retain high-water complete snapshot revision"
    );
}
#[test]
fn known_receipts_and_repeated_event_revisions_do_not_cause_background_resync() {
    let requests = RefreshRequests::default();
    requests.observe(Some(9_i64));
    requests.observe(Some(8_i64));
    assert!(
        !requests.take_needed(9_i64),
        "received full revision9 already covers own event/replay, regardless of event sequence"
    );
    requests.observe(Some(12_i64));
    requests.observe(Some(10_i64));
    assert!(
        requests.take_needed(9_i64),
        "coalesced high revision must survive lower events"
    );
    assert!(
        !requests.take_needed(12_i64),
        "current complete revision covers all pending observations"
    );
    requests.observe(None);
    requests.observe(Some(8_i64));
    assert!(
        requests.take_needed(12_i64),
        "explicit semantic recovery cannot be lost behind an old event"
    );
    assert!(
        !requests.take_needed(12_i64),
        "a mandatory request is consumed once, not every frame"
    );
}
#[test]
fn watcher_and_current_reads_share_the_sse_connection_budget_without_sleeping_in_tests() {
    let gate = ReadGate::new();
    let first = gate.reserve_at(0_u64);
    let second = gate.reserve_at(0_u64);
    let later = gate.reserve_at(20_000_u64);
    assert!(
        first >= 3_000_u64,
        "initial watcher must not race the bootstrap current read into HTTP429"
    );
    assert!(
        second >= first.saturating_add(3_000_u64),
        "independent worker requests share the20/minute route budget"
    );
    assert_eq!(
        later, 20_000_u64,
        "idle time does not accumulate a long backlog or busy retry"
    );
}
