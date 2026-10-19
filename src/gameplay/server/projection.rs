//! Decode the server's viewer-safe resync envelope, not its wrapper as a world.
//!
//! `events.Store.resync` emits `{current_revision, snapshot: Projection}`.
//! Command receipts instead put that projection directly in `viewer_projection`.

use super::ServerProjection;
use crate::gameplay::{GameplayError, runtime_error};
use core::iter;
use serde::{Deserialize, Deserializer};

/// Required envelope emitted by the server's current-snapshot SSE endpoint.
#[derive(Deserialize)]
struct ResyncEnvelope {
    /// Canonical revision, independently checked against the snapshot.
    current_revision: i64,
    /// Viewer projection; never canonical state or fields guessed from a wrapper.
    snapshot: ServerProjection,
}

impl ServerProjection {
    /// Extracts the current viewer snapshot from a complete, one-shot SSE response.
    /// Ignores unrelated events/comments and supports LF/CRLF, multiline data and
    /// the SSE optional single space after a field's colon.
    ///
    /// # Errors
    /// Returns an error for missing/malformed snapshots or inconsistent revisions,
    /// rather than accepting an arbitrary JSON object as a default empty game.
    pub(in crate::gameplay) fn from_resync_event(body: &str) -> Result<Self, GameplayError> {
        let mut event_name = "";
        let mut data = String::new();
        for line in body.lines().chain(iter::once("")) {
            if line.is_empty() {
                if event_name == "resync" {
                    return decode_snapshot(&data);
                }
                event_name = "";
                data.clear();
            } else if let Some(value) = line.strip_prefix("event:") {
                event_name = value.strip_prefix(' ').unwrap_or(value);
            } else if let Some(value) = line.strip_prefix("data:") {
                data.push_str(value.strip_prefix(' ').unwrap_or(value));
                data.push('\n');
            }
        }
        Err(runtime_error(
            "server resync did not include a projection".to_owned(),
        ))
    }
}

/// Decodes mandatory wire metadata and unwraps exactly the viewer snapshot.
fn decode_snapshot(data: &str) -> Result<ServerProjection, GameplayError> {
    let envelope: ResyncEnvelope = serde_json::from_str(data).map_err(|error| {
        runtime_error(format!(
            "server resync envelope did not match schema: {error}"
        ))
    })?;
    if envelope.current_revision < 0_i64 || envelope.snapshot.revision != envelope.current_revision
    {
        return Err(runtime_error(format!(
            "server resync revision mismatch: envelope {}, snapshot {}",
            envelope.current_revision, envelope.snapshot.revision
        )));
    }
    if envelope.snapshot.turn == 0_u32 {
        return Err(runtime_error(
            "server resync snapshot has invalid turn zero".to_owned(),
        ));
    }
    Ok(envelope.snapshot)
}

/// Normalizes Go nil slices (`null`) to empty vectors, for snapshots and receipts.
///
/// # Errors
/// Returns a deserializer error for anything other than null or a typed array.
pub(super) fn deserialize_nullable_vec<'input, Element, Decoder>(
    decoder: Decoder,
) -> Result<Vec<Element>, Decoder::Error>
where
    Element: Deserialize<'input>,
    Decoder: Deserializer<'input>,
{
    Option::<Vec<Element>>::deserialize(decoder).map(Option::unwrap_or_default)
}
