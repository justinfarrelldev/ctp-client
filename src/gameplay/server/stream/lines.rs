//! Bounded legacy hints and explicitly negotiated complete viewer models.
use super::{Change, snapshots::{self, Snapshot, MAX_SNAPSHOT_BYTES}};
/// Only small invalidation JSON may supply revision hints; resync snapshots skip.
const MAX_LINE: usize = 4096;
/// Per-event metadata keeps durable sequence and snapshot revision namespaces apart.
#[derive(Debug, Default)]
pub(super) struct EventLines {
    /// Current bounded metadata/data line.
    line: Vec<u8>,
    /// Remainder of a comment/large data line is discarded incrementally.
    dropping: bool,
    /// Event ID's interpretation depends on the event name.
    id: Option<u64>,
    /// Event kind; repaired resync IDs require matching durable cursor metadata.
    kind: String,
    /// Bounded normal invalidation JSON, not retained world data.
    data: Vec<u8>,
    /// Missing/truncated/large hints conservatively require a current read.
    lost_data: bool,
    /// Enabled only by the server's explicit snapshot-stream response header.
    pub(super) snapshots: bool,
    /// Coalesced full model; historical intermediate snapshots need not queue.
    pub(super) latest: Option<Snapshot>,
}
impl EventLines {
    /// Processes arbitrary fragments without retaining huge fog/rules snapshots.
    pub(super) fn consume(&mut self, bytes: &[u8], output: &mut Vec<Change>) -> Result<(), String> {
        for byte in bytes {
            if *byte == b'\n' {
                if self.dropping {
                    self.dropping = false;
                } else {
                    self.finish_line(output)?;
                }
                self.line.clear();
            } else if !self.dropping {
                self.line.push(*byte);
                if self.line.starts_with(b":") {
                    self.line.clear();
                    self.dropping = true;
                } else if self.full_model() && self.line.len() > MAX_SNAPSHOT_BYTES {
                    return Err("snapshot data exceeded its bound".to_owned());
                } else if self.line.starts_with(b"data:") && !self.full_model() && self.line.len() > MAX_LINE {
                    // Retain only a bounded resync prefix: current_revision and
                    // event_cursor precede snapshot in events.Store.resync. Never
                    // buffer world/rules data or treat the SSE ID as a revision.
                    if self.kind=="resync" && self.data.is_empty() { self.data.extend_from_slice(&self.line[5_usize..]); }
                    self.line.clear();
                    self.dropping = true;
                    self.lost_data = true;
                } else if !self.full_model() && self.line.len() > MAX_LINE {
                    return Err("event metadata line exceeded its bound".to_owned());
                }
            }
        }
        Ok(())
    }
    /// Only negotiated complete-model event kinds may retain large data.
    fn full_model(&self) -> bool {
        self.snapshots && (self.kind == "resync" || self.kind == "terminal") && self.line.starts_with(b"data:")
    }

    /// Publishes only at a complete event terminator, never a partial line.
    fn finish_line(&mut self, output: &mut Vec<Change>) -> Result<(), String> {
        let text = core::str::from_utf8(&self.line)
            .map_err(|error| error.to_string())?
            .trim_end_matches('\r');
        if text.is_empty() {
            if self.snapshots && (self.kind == "resync" || self.kind == "terminal") {
                let next = snapshots::decode(&self.kind, self.id, &self.data)?;
                if self.latest.as_ref().is_none_or(|known| next.projection.revision >= known.projection.revision) {
                    self.latest = Some(next);
                }
            } else if self.id.is_some() || self.kind == "resync" {
                let resync = self.kind == "resync";
                let metadata=if resync { resync_metadata(&self.data) } else {None};
                let cursor=metadata.as_ref().and_then(|value|value["event_cursor"].as_u64()).filter(|cursor|Some(*cursor)==self.id&&i64::try_from(*cursor).is_ok());
                let revision = if resync {
                    metadata.as_ref().and_then(|value|value["current_revision"].as_i64()).filter(|revision|*revision>=0_i64)
                } else if self.lost_data {
                    None
                } else {
                    serde_json::from_slice::<serde_json::Value>(&self.data)
                        .ok()
                        .and_then(|value| value["revision"].as_i64())
                        .filter(|revision| *revision >= 0_i64)
                };
                output.push(Change {
                    cursor: if resync { cursor } else { self.id },
                    revision,
                });
            }
            self.id = None;
            self.kind.clear();
            self.data.clear();
            self.lost_data = false;
        } else if let Some(id) = text.strip_prefix("id:") {
            self.id = id
                .trim()
                .parse::<u64>()
                .ok()
                .filter(|id| i64::try_from(*id).is_ok());
        } else if let Some(kind) = text.strip_prefix("event:") {
            kind.trim().clone_into(&mut self.kind);
        } else if let Some(data) = text.strip_prefix("data:") {
            if self
                .data
                .len()
                .saturating_add(data.len())
                .saturating_add(1_usize)
                <= if self.snapshots && (self.kind == "resync" || self.kind == "terminal") { MAX_SNAPSHOT_BYTES } else { MAX_LINE }
            {
                self.data
                    .extend_from_slice(data.strip_prefix(' ').unwrap_or(data).as_bytes());
                self.data.push(b'\n');
            } else if self.snapshots && (self.kind == "resync" || self.kind == "terminal") {
                return Err("snapshot data exceeded its bound".to_owned());
            } else {
                self.data.clear();
                self.lost_data = true;
            }
        }
        Ok(())
    }
}

/// Decode metadata from either a small complete envelope or the bounded prefix.
/// Unknown/legacy malformed data cannot promote an ambiguous old revision ID.
fn resync_metadata(data:&[u8])->Option<serde_json::Value> {
    if let Ok(value)=serde_json::from_slice(data){return Some(value);}
    let snapshot=data.windows(10_usize).position(|part|part==b"\"snapshot\"")?;
    let prefix=core::str::from_utf8(&data[..snapshot]).ok()?.trim_end().trim_end_matches(',');
    serde_json::from_str(&format!("{prefix}}}")).ok()
}
