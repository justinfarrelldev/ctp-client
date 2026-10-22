//! Bounded incremental HTTP/SSE invalidations; event payloads are never game state.
use super::{HTTP_TIMEOUT, ServerClient, ServerGameplaySession};
use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

/// Small revision hints stay separate from large viewer snapshot payloads.
mod lines;
use lines::EventLines;
/// Invalidation hints, never snapshots/patches or command acknowledgements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::gameplay) struct Change {
    /// Durable sequence only; the resync event's revision ID is not a cursor.
    pub cursor: Option<u64>,
    /// Optional observed revision can avoid a read already covered by a receipt.
    pub revision: Option<i64>,
}
/// Explicit transfer phases avoid buffering an entire large resync HTTP chunk.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Transfer {
    /// Bounded response headers have not finished.
    #[default]
    Headers,
    /// Connection-delimited unchunked event body.
    Identity,
    /// Waiting for the next hexadecimal chunk header.
    Size,
    /// Exact number of remaining data bytes, possibly much larger than one read.
    Data(usize),
    /// Two-byte chunk terminator.
    EndChunk,
    /// Zero chunk; trailers cannot become another event.
    Ended,
}

/// HTTP framing plus streaming invalidation metadata, never canonical payloads.
#[derive(Debug, Default)]
pub(in crate::gameplay) struct Invalidations {
    /// Only currently incomplete transfer framing/read data is retained.
    wire: Vec<u8>,
    /// Explicit parser state.
    transfer: Transfer,
    /// Bounded event metadata.
    events: EventLines,
}
impl Invalidations {
    /// Accepts arbitrary network fragments and emits complete event cursors only.
    pub(in crate::gameplay) fn push(&mut self, bytes: &[u8]) -> Result<Vec<Change>, String> {
        if self.transfer == Transfer::Ended {
            return Ok(Vec::new());
        }
        self.wire.extend_from_slice(bytes);
        let mut cursors = Vec::new();
        loop {
            match self.transfer {
                Transfer::Headers => {
                    if !self.headers()? {
                        break;
                    }
                }
                Transfer::Size => {
                    if !self.size()? {
                        break;
                    }
                }
                Transfer::Identity => {
                    self.events.consume(&self.wire, &mut cursors)?;
                    self.wire.clear();
                    break;
                }
                Transfer::Data(remaining) => {
                    let count = remaining.min(self.wire.len());
                    if count == 0 {
                        break;
                    }
                    self.events.consume(&self.wire[..count], &mut cursors)?;
                    self.wire.drain(..count);
                    self.transfer = if count == remaining {
                        Transfer::EndChunk
                    } else {
                        Transfer::Data(remaining.saturating_sub(count))
                    };
                }
                Transfer::EndChunk => {
                    if self.wire.len() < 2 {
                        break;
                    }
                    if !self.wire.starts_with(b"\r\n") {
                        return Err("invalid transfer framing".to_owned());
                    }
                    self.wire.drain(..2);
                    self.transfer = Transfer::Size;
                }
                Transfer::Ended => {
                    self.wire.clear();
                    break;
                }
            }
        }
        Ok(cursors)
    }

    /// Validates success before accepting any event or membership-protected payload.
    fn headers(&mut self) -> Result<bool, String> {
        let end = self.wire.windows(4).position(|part| part == b"\r\n\r\n");
        if end.unwrap_or(self.wire.len()) > 0x4000_usize {
            return Err("event headers exceeded their bound".to_owned());
        }
        let Some(end) = end else {
            return Ok(false);
        };
        let header = core::str::from_utf8(&self.wire[..end]).map_err(|error| error.to_string())?;
        let status = header
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|code| code.parse::<u16>().ok())
            .ok_or_else(|| "event stream status is invalid".to_owned())?;
        if status != 200_u16 {
            return Err(format!("event stream returned HTTP {status}"));
        }
        self.transfer = if header
            .to_ascii_lowercase()
            .contains("transfer-encoding: chunked")
        {
            Transfer::Size
        } else {
            Transfer::Identity
        };
        self.wire.drain(..end.saturating_add(4));
        Ok(true)
    }

    /// Begins a chunk without allocating its declared size.
    fn size(&mut self) -> Result<bool, String> {
        let Some(end) = self.wire.windows(2).position(|part| part == b"\r\n") else {
            if self.wire.len() > 4096 {
                return Err("transfer size line exceeded its bound".to_owned());
            }
            return Ok(false);
        };
        let size = core::str::from_utf8(&self.wire[..end])
            .map_err(|error| error.to_string())?
            .split(';')
            .next()
            .ok_or_else(|| "missing transfer size".to_owned())?;
        let count = usize::from_str_radix(size.trim(), 16).map_err(|error| error.to_string())?;
        self.wire.drain(..end.saturating_add(2));
        self.transfer = if count == 0 {
            Transfer::Ended
        } else {
            Transfer::Data(count)
        };
        Ok(true)
    }
}

/// Watches authorized invalidations until cancellation or a bounded IO failure.
/// The caller reconnects with the last emitted cursor; no canonical fallback exists.
pub(in crate::gameplay) fn watch(
    client: &ServerClient,
    session: &ServerGameplaySession,
    cursor: &mut u64,
    stop: &Arc<AtomicBool>,
    notify: &mut impl FnMut(Option<i64>),
) -> Result<(), String> {
    let address = client.endpoint.socket_address();
    let resolved = address
        .to_socket_addrs()
        .map_err(|error| error.to_string())?
        .next()
        .ok_or_else(|| "server address unavailable".to_owned())?;
    let mut stream =
        TcpStream::connect_timeout(&resolved, HTTP_TIMEOUT).map_err(|error| error.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_millis(250)))
        .map_err(|error| error.to_string())?;
    stream
        .set_write_timeout(Some(HTTP_TIMEOUT))
        .map_err(|error| error.to_string())?;
    let path = client.endpoint.path_with_prefix(&format!(
        "/game/{}/events?session_id={}",
        session.game_id, session.session_id
    ));
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {}\r\nAccept: text/event-stream\r\nLast-Event-ID: {cursor}\r\nConnection: close\r\n\r\n",
        client.endpoint.host_header()
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|error| error.to_string())?;
    let mut parser = Invalidations::default();
    let mut buffer = [0_u8; 8192];
    while !stop.load(Ordering::Relaxed) {
        match stream.read(&mut buffer) {
            Ok(0) => return Ok(()),
            Ok(count) => {
                for change in parser.push(&buffer[..count])? {
                    if let Some(sequence) = change.cursor {
                        *cursor = (*cursor).max(sequence);
                    }
                    notify(change.revision);
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(())
}
