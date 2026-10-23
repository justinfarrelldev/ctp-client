//! Bounded public rejection identity; internal/credential-bearing bodies are never retained.
use serde::Deserialize;
/// Bounded public envelope and isolated HTTP regression tests.
#[cfg(test)]
mod tests;

/// Safe command failure, distinct from transport uncertainty and later replication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rejection {
    /// HTTP admission/validation result.
    pub status:u16,
    /// Stable allowlisted category; unknown future codes become generic.
    pub code:String,
    /// Bounded viewer-safe message, never raw error/body text.
    pub message:String,
    /// Opaque server correlation identity, not a command/session credential.
    pub correlation_id:Option<String>,
}
/// Only additive public fields from the documented gateway envelope.
#[derive(Deserialize)]
struct Wire {
    /// Stable public category.
    code:Option<String>,
    /// Viewer-safe sentence supplied by the gateway.
    message:Option<String>,
    /// Private diagnostics lookup identity.
    correlation_id:Option<String>,
}
impl Rejection {
    /// Decodes current/legacy/middleware failures without ever displaying a raw body.
    #[must_use]
    pub fn decode(status:u16,body:&str)->Self {
        let wire=serde_json::from_str::<Wire>(body).ok();
        let known=wire.as_ref().and_then(|wire|wire.code.as_deref()).filter(|code|matches!(*code,"revision_conflict"|"idempotency_conflict"|"controller_restricted"|"phase_restricted"|"queue_identity"|"queue_shape"|"queue_capacity"|"content_unavailable"|"production_prerequisite"|"command_invalid"|"request_invalid"|"request_too_large"|"authentication_required"|"not_found"|"state_conflict"|"rate_limited"|"internal_error"));
        let message=known.and_then(|_|wire.as_ref()?.message.as_deref()).filter(|message|!message.is_empty()&&message.chars().count()<=512_usize&&!message.chars().any(char::is_control)).unwrap_or("The server rejected this order.").to_owned();
        let correlation_id=wire.as_ref().and_then(|wire|wire.correlation_id.as_ref()).filter(|id|!id.is_empty()&&id.len()<=128_usize&&id.bytes().all(|byte|byte.is_ascii_alphanumeric()||matches!(byte,b'-'|b'_'))).cloned();
        Self{status,code:known.unwrap_or("command_rejected").to_owned(),message,correlation_id}
    }
    /// Persistent UI sentence; refresh success does not invalidate a rejected operation.
    #[must_use]
    pub fn label(&self)->String {
        self.correlation_id.as_ref().map_or_else(||format!("Order rejected ({}): {}",self.code,self.message),|id|format!("Order rejected ({}): {} Reference: {id}",self.code,self.message))
    }
}
