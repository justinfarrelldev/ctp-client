//! Bounded incremental HTTP/SSE invalidations; canonical event payloads are ignored.
use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::mpsc::SyncSender;
use std::time::Duration;
use super::{ServerClient, ServerGameplaySession, HTTP_TIMEOUT};

/// Incremental transfer/body parser, retaining incomplete UTF-8 lines as bytes.
#[derive(Debug, Default)]
pub(in crate::gameplay) struct Invalidations {
    /// Unconsumed transfer bytes.
    wire: Vec<u8>,
    /// Whether successful response headers were received.
    headers: bool,
    /// Chunked transfer framing decision.
    chunked: bool,
    /// Decoded body bytes waiting for a complete SSE record.
    body: Vec<u8>,
    /// Zero-length terminating transfer chunk.
    ended: bool,
}
impl Invalidations {
    /// Accepts arbitrary network fragments and emits only complete event cursors.
    pub(in crate::gameplay) fn push(&mut self, bytes:&[u8])->Result<Vec<u64>,String> {
        self.wire.extend_from_slice(bytes);
        if self.wire.len().saturating_add(self.body.len()) > 0x0004_0000_usize { return Err("event stream exceeded bounded buffer".to_owned()); }
        if !self.headers {
            let Some(end)=self.wire.windows(4).position(|part| part == b"\r\n\r\n") else { return Ok(Vec::new()); };
            let header=core::str::from_utf8(&self.wire[..end]).map_err(|error| error.to_string())?;
            if header.lines().next().is_none_or(|line| line.split_whitespace().nth(1) != Some("200")) { return Err("event stream was not authorized/successful".to_owned()); }
            self.chunked=header.to_ascii_lowercase().contains("transfer-encoding: chunked");
            self.wire.drain(..end.saturating_add(4));
            self.headers=true;
        }
        if self.chunked {
            while !self.ended {
                let Some(end)=self.wire.windows(2).position(|part| part == b"\r\n") else { break; };
                let size=core::str::from_utf8(&self.wire[..end]).map_err(|error| error.to_string())?.split(';').next().ok_or_else(|| "missing transfer size".to_owned())?;
                let count=usize::from_str_radix(size.trim(),16).map_err(|error| error.to_string())?;
                let start=end.saturating_add(2);
                let finish=start.checked_add(count).ok_or_else(|| "transfer size overflow".to_owned())?;
                if finish.saturating_add(2) > self.wire.len() { break; }
                if self.wire.get(finish..finish.saturating_add(2)) != Some(b"\r\n") { return Err("invalid transfer framing".to_owned()); }
                self.body.extend_from_slice(&self.wire[start..finish]);
                self.wire.drain(..finish.saturating_add(2));
                self.ended=count == 0;
            }
        } else { self.body.append(&mut self.wire); }
        let mut cursors=Vec::new();
        // Normalize CRLF after full UTF-8 records only. Neither payload nor event
        // kind is interpreted as authoritative game state.
        loop {
            let separator=self.body.windows(2).position(|part| part == b"\n\n").map(|position| (position,2_usize))
                .or_else(|| self.body.windows(4).position(|part| part == b"\r\n\r\n").map(|position| (position,4_usize)));
            let Some((end,size))=separator else { break; };
            let event=core::str::from_utf8(&self.body[..end]).map_err(|error| error.to_string())?;
            if let Some(cursor)=event.lines().filter_map(|line| line.strip_prefix("id:")).filter_map(|value| value.trim().parse::<u64>().ok()).next_back() { cursors.push(cursor); }
            self.body.drain(..end.saturating_add(size));
        }
        Ok(cursors)
    }
}

/// Watches authorized invalidations until cancellation or a bounded IO failure.
/// The caller reconnects with the last emitted cursor; no canonical fallback exists.
pub(in crate::gameplay) fn watch(client:&ServerClient, session:&ServerGameplaySession, cursor:&mut u64, stop:&Arc<AtomicBool>, sink:&SyncSender<u64>)->Result<(),String> {
    let address=client.endpoint.socket_address();
    let resolved=address.to_socket_addrs().map_err(|error| error.to_string())?.next().ok_or_else(|| "server address unavailable".to_owned())?;
    let mut stream=TcpStream::connect_timeout(&resolved,HTTP_TIMEOUT).map_err(|error| error.to_string())?;
    stream.set_read_timeout(Some(Duration::from_millis(250))).map_err(|error| error.to_string())?;
    stream.set_write_timeout(Some(HTTP_TIMEOUT)).map_err(|error| error.to_string())?;
    let path=client.endpoint.path_with_prefix(&format!("/game/{}/events?session_id={}",session.game_id,session.session_id));
    let request=format!("GET {path} HTTP/1.1\r\nHost: {}\r\nAccept: text/event-stream\r\nLast-Event-ID: {cursor}\r\nConnection: close\r\n\r\n",client.endpoint.host_header());
    stream.write_all(request.as_bytes()).map_err(|error| error.to_string())?;
    let mut parser=Invalidations::default();
    let mut buffer=[0_u8;8192];
    while !stop.load(Ordering::Relaxed) {
        match stream.read(&mut buffer) {
            Ok(0) => return Ok(()),
            Ok(count) => { for id in parser.push(&buffer[..count])? { *cursor=id; let _delivery=sink.try_send(id); } }
            Err(error) if matches!(error.kind(),std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock) => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(())
}
