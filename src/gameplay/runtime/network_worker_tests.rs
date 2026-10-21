//! Actual worker integration against an isolated, bounded fake HTTP gateway.
use super::network::{Network, Outcome};
use crate::gameplay::server::{
    ServerClient, ServerConfig, ServerGameplaySession, ServerProjection, envelope::PreparedCommand,
};
use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::{Duration, Instant},
};

#[test]
fn worker_retries_a_committed_but_lost_http_receipt_with_identical_ids() {
    check_lost_receipt("");
}

#[test]
fn worker_retries_truncated_json_receipts_without_new_command_identity() {
    check_lost_receipt(
        "HTTP/1.1 200 OK\r\nContent-Length: 200\r\nConnection: close\r\n\r\n{\"revision\":6,",
    );
}

/// Drives real worker retry/shutdown with either absent headers or incomplete JSON.
fn check_lost_receipt(prefix: &'static str) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("isolated mock gateway");
    let port = listener.local_addr().expect("ephemeral port").port();
    listener.set_nonblocking(true).expect("bounded listener");
    let stop = Arc::new(AtomicBool::new(false));
    let fake_stop = Arc::clone(&stop);
    let fake = thread::spawn(move || serve(&listener, &fake_stop, prefix));
    let client = ServerClient::from_config(&ServerConfig {
        base_url: format!("http://127.0.0.1:{port}"),
        required: true,
    })
    .expect("mock URL");
    let session = ServerGameplaySession {
        game_id: 44,
        session_id: 71,
        account_id: 7,
        projection: Some(ServerProjection {
            revision: 5,
            turn: 1,
            ..ServerProjection::default()
        }),
    };
    let request = PreparedCommand::new(&session, "pass", json!({})).expect("capture one operation");
    let network = Network::start(client, session).expect("worker creation");
    network
        .submit(request.clone())
        .expect("one bounded operation");
    let started = Instant::now();
    let mut receipt = false;
    while !receipt && started.elapsed() < Duration::from_secs(4) {
        if let Some(Outcome::Receipt(snapshot)) = network.poll() {
            receipt = snapshot.revision == 6;
        }
        thread::sleep(Duration::from_millis(2));
    }
    drop(network);
    stop.store(true, Ordering::Relaxed);
    let attempts = fake
        .join()
        .expect("fake thread")
        .expect("bounded mock gateway");
    assert!(
        receipt,
        "lost acknowledgement is retried, not mistaken for a new operation"
    );
    assert_eq!(
        attempts,
        vec![request.body.clone(), request.body],
        "service sees exactly the same idempotency key, command ID, revision and payload twice"
    );
}

/// Persists one fake command result and replays it without a second state change.
fn serve(
    listener: &TcpListener,
    stop: &Arc<AtomicBool>,
    prefix: &str,
) -> Result<Vec<Value>, String> {
    let started = Instant::now();
    let mut attempts = Vec::new();
    while !stop.load(Ordering::Relaxed) && started.elapsed() < Duration::from_secs(6) {
        match listener.accept() {
            Ok((mut stream, _)) => {
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .map_err(|error| error.to_string())?;
                let (line, body) = request(&mut stream)?;
                let revision = if attempts.is_empty() { 5_i64 } else { 6_i64 };
                if line.starts_with("POST /game/44/pass ") {
                    attempts.push(body);
                    if attempts.len() == 1 {
                        stream
                            .write_all(prefix.as_bytes())
                            .map_err(|error| error.to_string())?;
                        continue;
                    } // committed; TCP closes before a complete receipt
                    respond(&mut stream,&json!({"revision":6_i64,"receipt_id":"receipt-6","viewer_projection":{"revision":6_i64,"turn":1_u32}}).to_string(),"application/json")?;
                } else if line.starts_with("GET /game/44/events?session_id=71 ") {
                    let event = format!(
                        "id: {revision}\nevent: resync\ndata: {{\"current_revision\":{revision},\"snapshot\":{{\"revision\":{revision},\"turn\":1}}}}\n\n"
                    );
                    respond(&mut stream, &event, "text/event-stream")?;
                } else {
                    return Err(format!("unexpected mock route: {line}"));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(1));
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(attempts)
}
/// Reads the actual content-length envelope, not a timing-dependent single read.
fn request(stream: &mut TcpStream) -> Result<(String, Value), String> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|error| error.to_string())?;
    let mut length = 0_usize;
    loop {
        let mut header = String::new();
        reader
            .read_line(&mut header)
            .map_err(|error| error.to_string())?;
        if header == "\r\n" || header.is_empty() {
            break;
        }
        if let Some(value) = header.to_ascii_lowercase().strip_prefix("content-length:") {
            length = value
                .trim()
                .parse()
                .map_err(|error: core::num::ParseIntError| error.to_string())?;
        }
    }
    let mut bytes = vec![0_u8; length];
    reader
        .read_exact(&mut bytes)
        .map_err(|error| error.to_string())?;
    let body = if bytes.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?
    };
    Ok((line, body))
}
/// Writes finite SSE or JSON; no real service or canonical data exists here.
fn respond(stream: &mut TcpStream, body: &str, content_type: &str) -> Result<(), String> {
    let message = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(message.as_bytes())
        .map_err(|error| error.to_string())
}
