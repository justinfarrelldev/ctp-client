//! Immutable command envelope kept intact across transport retries.
use serde_json::{Value, json};
use super::{ServerClient, ServerCommandResponse, ServerGameplaySession, unique_client_key};
use crate::gameplay::{GameplayError, runtime_error};

/// One exact user operation. A retry never changes revision, command or key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedCommand {
    /// Exact gameplay endpoint path.
    pub path: String,
    /// Complete immutable authenticated body.
    pub body: Value,
}
impl PreparedCommand {
    /// Captures the operation and shared envelope exactly once.
    ///
    /// # Errors
    /// Rejects non-object payloads, invalid routes and oversized requests.
    pub fn new(session:&ServerGameplaySession, route:&str, payload:Value)->Result<Self,GameplayError> {
        if !(super::interface::LegalAction { route:route.to_owned(), ..super::interface::LegalAction::default() }).http_supported() { return Err(runtime_error("unsupported command route".to_owned())); }
        let Value::Object(mut body)=payload else { return Err(runtime_error("command payload must be an object".to_owned())); };
        let key=unique_client_key(route)?;
        body.insert("session_id".to_owned(),json!(session.session_id));
        body.insert("expected_revision".to_owned(),json!(session.revision()));
        body.insert("idempotency_key".to_owned(),json!(key));
        body.insert("command_id".to_owned(),json!(format!("ctp-client-{key}")));
        let body=Value::Object(body);
        if body.to_string().len() > 0x0004_0000_usize { return Err(runtime_error("command exceeds server's 256 KiB limit".to_owned())); }
        Ok(Self { path:format!("/game/{}/{route}",session.game_id),body })
    }

    /// Executes this exact envelope; caller retains it if the response is lost.
    ///
    /// # Errors
    /// Returns transport, server rejection or inconsistent receipt errors.
    pub fn execute(&self, client:&ServerClient)->Result<ServerCommandResponse,GameplayError> {
        let value=client.post_json(&self.path,&self.body)?;
        let response:ServerCommandResponse=serde_json::from_value(value).map_err(|error| runtime_error(format!("invalid command receipt: {error}")))?;
        if response.revision < 0 || !response.viewer_projection.as_ref().is_some_and(|projection| projection.revision == response.revision && projection.turn > 0) { return Err(runtime_error("command receipt is missing a consistent viewer snapshot".to_owned())); }
        Ok(response)
    }
}
