//! Semantic messages are idempotent presentation, never local rewards or history.
use super::messages::MessageLog;
use crate::gameplay::server::v2::Notification;

#[test]
fn historical_rejection_text_never_claims_ongoing_recovery_or_success_after_the_snapshot_was_received()
 {
    let mut state = crate::gameplay::model::GameState::new(8_u16, 8_u16, 1_u8, 1_u64);
    let book = crate::gameplay::assets::layout::LayoutBook::default();
    let notice = Notification {
        kind: "order.rejected".to_owned(),
        ..Notification::default()
    };
    for status in [
        crate::gameplay::model::NetworkStatus::Ready,
        crate::gameplay::model::NetworkStatus::Pending,
        crate::gameplay::model::NetworkStatus::Resynchronizing,
    ] {
        state.network = status;
        assert_eq!(
            super::messages::text(&notice, &state, &book),
            "An order could not be completed. Review the current game state before trying again.",
            "message is a historical failure, not the live network-status label"
        );
    }
}
#[test]
fn unknown_ruin_rewards_are_unavailable_not_fabricated_empty_results() {
    let state = crate::gameplay::model::GameState::new(8, 8, 1, 1);
    let book = crate::gameplay::assets::layout::LayoutBook::default();
    let mut notice = Notification {
        kind: "ruin.explored".to_owned(),
        reward_kind: "future-profile".to_owned(),
        ..Notification::default()
    };
    assert_eq!(
        super::messages::text(&notice, &state, &book),
        "The ruins have been explored; reward details are unavailable.",
        "unknown outcome never means nothing was found"
    );
    notice.reward_kind = "empty".to_owned();
    assert_eq!(
        super::messages::text(&notice, &state, &book),
        "The ruins have been explored; nothing was found.",
        "only explicit empty is an empty reward"
    );
}

#[test]
fn notices_deduplicate_receipts_resyncs_and_retry_without_replaying_rewards() {
    let mut log = MessageLog::default();
    let notice = Notification {
        id: "n1".to_owned(),
        kind: "ruin.explored".to_owned(),
        reward_kind: "gold".to_owned(),
        amount: 268,
        turn: 3,
        ..Notification::default()
    };
    assert_eq!(
        log.ingest(core::slice::from_ref(&notice)).len(),
        1,
        "first semantic result"
    );
    assert!(
        log.ingest(core::slice::from_ref(&notice)).is_empty(),
        "same snapshot does not generate another popup/reward"
    );
    assert_eq!(log.unread(), 1, "read state is local");
    log.open("n1");
    assert_eq!(log.unread(), 0, "explicit accessible opening marks read");
    assert_eq!(log.selected.as_deref(), Some("n1"), "exact stable identity");
    assert!(
        log.ingest(core::slice::from_ref(&notice)).is_empty(),
        "read notice remains deduplicated"
    );
    assert_eq!(
        log.items[0].amount, 268,
        "amount remains authoritative metadata only"
    );
    log.ingest(&[]);
    assert!(
        log.items.is_empty(),
        "missing retention window does not invent older results"
    );
}
