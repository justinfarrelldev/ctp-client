//! Full models are opt-in, bounded and revision/cursor checked before delivery.
use super::super::ServerProjection;
use serde::Deserialize;

/// Maximum complete negotiated snapshot data, excluding small SSE framing.
pub(super) const MAX_SNAPSHOT_BYTES: usize = 16_usize * 1024_usize * 1024_usize;

/// One received complete model; sequence metadata never becomes a revision.
#[derive(Debug)]
pub(in crate::gameplay) struct Snapshot {
    /// Durable sequence checked against the resync envelope and signed wire bound.
    pub cursor: u64,
    /// Complete authorized model, never an event patch or local prediction.
    pub projection: Box<ServerProjection>,
}

/// Mandatory metadata for the negotiated server contract, not legacy hints.
#[derive(Deserialize)]
struct Envelope {
    /// Snapshot's canonical revision.
    current_revision: i64,
    /// Durable resume cursor from the same committed aggregate row.
    event_cursor: u64,
    /// Entire viewer-safe model.
    snapshot: ServerProjection,
}

/// Reject inconsistent models instead of treating a malformed wrapper as a world.
pub(super) fn decode(kind: &str, id: Option<u64>, data: &[u8]) -> Result<Snapshot, String> {
    let cursor = id.filter(|value| i64::try_from(*value).is_ok()).ok_or_else(|| "snapshot has no valid durable cursor".to_owned())?;
    let projection = if kind == "terminal" {
        serde_json::from_slice::<ServerProjection>(data).map_err(|_error| "terminal snapshot is malformed".to_owned())?
    } else {
        let envelope: Envelope = serde_json::from_slice(data).map_err(|_error| "snapshot envelope is malformed".to_owned())?;
        if envelope.current_revision != envelope.snapshot.revision || envelope.event_cursor != cursor {
            return Err("snapshot revision/cursor metadata is inconsistent".to_owned());
        }
        envelope.snapshot
    };
    if projection.revision < 0_i64 || projection.turn == 0_u32 {
        return Err("snapshot revision/turn is invalid".to_owned());
    }
    Ok(Snapshot { cursor, projection: Box::new(projection) })
}
