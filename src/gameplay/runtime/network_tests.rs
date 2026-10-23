//! Injected network state tests; no threads, sockets, files or live services.
use super::network::{Outcome, Transport, execute};
use crate::gameplay::server::{ServerGameplaySession, ServerProjection, envelope::PreparedCommand};
use serde_json::json;

/// Records exact attempts and simulates a lost first receipt.
struct Fake {
    /// Immutable attempts observed by the fake service.
    attempts: Vec<PreparedCommand>,
    /// Current viewer revision.
    revision: i64,
}
impl Transport for Fake {
    fn submit(&mut self, command:&PreparedCommand)->Result<ServerProjection,crate::gameplay::GameplayError> {
        self.attempts.push(command.clone());
        if self.attempts.len() == 1 { return Err(crate::gameplay::runtime_error("failed to read server response: lost".to_owned())); }
        Ok(ServerProjection {revision:self.revision,turn:1,..ServerProjection::default()})
    }
    fn resync(&mut self)->Result<ServerProjection,String> { Ok(ServerProjection {revision:self.revision,turn:1,..ServerProjection::default()}) }
}

#[test]
fn async_retry_keeps_exact_envelope_and_reconciles_without_local_mutation() {
    let session=ServerGameplaySession {game_id:1,session_id:2,account_id:3,projection:Some(ServerProjection {revision:4,turn:1,..ServerProjection::default()})};
    let request=PreparedCommand::new(&session,"pass",json!({})).expect("capture envelope");
    let mut fake=Fake {attempts:Vec::new(),revision:5};
    let mut outcomes=Vec::new();
    execute(&mut fake,&request,&mut |outcome| outcomes.push(outcome));
    assert_eq!(fake.attempts,vec![request.clone(),request],"lost-response retry cannot create another order/key");
    assert!(matches!(outcomes.pop(),Some(Outcome::Receipt(snapshot)) if snapshot.revision == 5),"only an authoritative snapshot is published");
}

/// Rejects once then supplies a complete unchanged snapshot; no retry can execute it.
struct RejectedTransport { attempts:u32 }
impl Transport for RejectedTransport {
    fn submit(&mut self,_command:&PreparedCommand)->Result<ServerProjection,crate::gameplay::GameplayError> {
        self.attempts=self.attempts.saturating_add(1_u32);
        Err(crate::gameplay::GameplayError::Rejected { rejection:crate::gameplay::server::rejection::Rejection::decode(422_u16,r#"{"code":"production_prerequisite","message":"The proposed production queue is not currently buildable.","correlation_id":"reference-1"}"#) })
    }
    fn resync(&mut self)->Result<ServerProjection,String>{Ok(ServerProjection {revision:4_i64,turn:1_u32,..ServerProjection::default()})}
}
#[test]
fn repaired_rejection_is_emitted_separately_from_refresh_and_never_retried_as_transport_loss() {
    let session=ServerGameplaySession {game_id:1_i64,session_id:2_i64,account_id:3_i64,projection:Some(ServerProjection {revision:4_i64,turn:1_u32,..ServerProjection::default()})};
    let command=PreparedCommand::new(&session,"edit-production",json!({"city_id":"city","entries":[{"item_id":"UNIT_WARRIOR"}]})).expect("immutable request");
    let mut transport=RejectedTransport{attempts:0_u32};let mut outcomes=Vec::new();execute(&mut transport,&command,&mut |outcome|outcomes.push(outcome));
    assert_eq!(transport.attempts,1_u32,"authoritative rejection cannot become another order or automatic rebase");
    assert!(matches!(*outcomes.as_slice(),[Outcome::Rejected(_),Outcome::Snapshot(_)]),"rejection persists independently of successful replication");
}

#[test]
fn asynchronous_reconciliation_never_regresses_the_session_revision() {
    let mut session=ServerGameplaySession {game_id:1,session_id:2,account_id:3,projection:Some(ServerProjection {revision:9,turn:3,..ServerProjection::default()})};
    session.apply_projection(ServerProjection {revision:8,turn:2,..ServerProjection::default()});
    assert_eq!(session.revision(),9,"older idempotent receipts cannot replace newer state");
}
