//! Safe command rejection survives successful replication and cannot become execution.
use super::{fixture, Outcome};
use crate::gameplay::{model::NetworkStatus,runtime::{sync,MovementInputState},server::rejection::Rejection};

#[test]
fn repaired_rejection_persists_code_and_correlation_after_successful_equal_revision_refresh() {
    let mut harness=fixture();harness.state.network=NetworkStatus::Pending;
    let before=harness.state.viewer.clone();
    let rejection=Rejection::decode(422_u16,r#"{"code":"production_prerequisite","message":"The proposed production queue is not currently buildable.","correlation_id":"safe-reference-1"}"#);
    sync::apply(&mut harness.authority,&mut harness.state,&mut harness.ui,&mut MovementInputState::default(),Outcome::Rejected(rejection));
    assert_eq!(harness.state.network,NetworkStatus::Resynchronizing,"refresh still reconciles tentative presentation");
    let snapshot=harness.authority.session.projection.clone().expect("unchanged current viewer");
    sync::apply(&mut harness.authority,&mut harness.state,&mut harness.ui,&mut MovementInputState::default(),Outcome::Snapshot(Box::new(snapshot)));
    assert_eq!(harness.state.network,NetworkStatus::Ready,"successful replication restores input");
    assert!(harness.ui.status_text.contains("production_prerequisite")&&harness.ui.status_text.contains("safe-reference-1"),"successful refresh cannot erase the rejected action or disguise it as a transport fault");
    assert_eq!(harness.state.viewer,before,"rejection changes no production/funds/revision");
}

#[test]
fn repaired_rejection_unknown_or_malformed_response_never_displays_raw_body_or_credentials() {
    for body in [r#"{"code":"future_private_code","message":"session_id=private","correlation_id":"safe-reference"}"#,r#"{"error":"session_id=private"}"#,"not JSON session_id=private"] {
        let rejection=Rejection::decode(422_u16,body);
        assert!(!rejection.label().contains("private"),"only documented safe envelope fields are surfaced");
        assert_eq!(rejection.code,"command_rejected","unknown codes are generic persistent rejections, not retries");
    }
}
