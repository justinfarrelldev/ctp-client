//! Bounded SSE metadata hints; data never becomes authoritative gameplay state.
use super::Change;
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
    /// Server event kind; a resync ID is explicitly a snapshot revision.
    kind: String,
    /// Bounded normal invalidation JSON, not retained world data.
    data: Vec<u8>,
    /// Missing/truncated/large hints conservatively require a current read.
    lost_data: bool,
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
                } else if self.line.starts_with(b"data:")
                    && (self.kind == "resync" || self.line.len() > MAX_LINE)
                {
                    self.line.clear();
                    self.dropping = true;
                    self.lost_data = true;
                } else if self.line.len() > MAX_LINE {
                    return Err("event metadata line exceeded its bound".to_owned());
                }
            }
        }
        Ok(())
    }
    /// Publishes a hint only at a complete event terminator, never a partial line.
    fn finish_line(&mut self, output: &mut Vec<Change>) -> Result<(), String> {
        let text = core::str::from_utf8(&self.line)
            .map_err(|error| error.to_string())?
            .trim_end_matches('\r');
        if text.is_empty() {
            if self.id.is_some() || self.kind == "resync" {
                let resync = self.kind == "resync";
                let revision = if resync {
                    self.id.and_then(|id| i64::try_from(id).ok())
                } else if self.lost_data {
                    None
                } else {
                    serde_json::from_slice::<serde_json::Value>(&self.data)
                        .ok()
                        .and_then(|value| value["revision"].as_i64())
                        .filter(|revision| *revision >= 0_i64)
                };
                output.push(Change {
                    cursor: if resync { None } else { self.id },
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
                <= MAX_LINE
            {
                self.data
                    .extend_from_slice(data.strip_prefix(' ').unwrap_or(data).as_bytes());
                self.data.push(b'\n');
            } else {
                self.data.clear();
                self.lost_data = true;
            }
        }
        Ok(())
    }
}
