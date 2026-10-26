//! Production lanes against an isolated in-memory HTTP gateway, never a live server.
use super::{Network, Outcome};
use crate::gameplay::server::{ServerClient, ServerConfig, ServerGameplaySession, ServerProjection, envelope::PreparedCommand};
use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};
use serde_json::{Value, json};
use std::{io::{BufRead, BufReader, Read, Write}, net::{TcpListener, TcpStream}, thread, time::{Duration, Instant}};

#[test]
fn negotiated_ai_turn_arrives_on_same_stream_without_current_read_or_receipt_confusion() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("isolated gateway");
    listener.set_nonblocking(true).expect("bounded listener");
    let port = listener.local_addr().expect("ephemeral port").port();
    let session = ServerGameplaySession {
        roster:crate::gameplay::server::roster::PlayerRoster::default(), game_id:44_i64, session_id:71_i64, account_id:7_i64,
        projection:Some(serde_json::from_value::<ServerProjection>(json!({"revision":5_i64,"turn":1_u32,"contract":{"version":2_u32,"features":{"snapshot_stream":true}}})).expect("capability from real viewer schema")),
    };
    let command = PreparedCommand::new(&session,"pass",json!({})).expect("one immutable end-turn");
    let expected = command.body.clone();
    let stop = Arc::new(AtomicBool::new(false));
    let fake_stop = Arc::clone(&stop);
    let gateway = thread::spawn(move || serve(&listener,&fake_stop,&expected));
    let client = ServerClient::from_config(&ServerConfig { base_url:format!("http://127.0.0.1:{port}"), required:true }).expect("fake endpoint only");
    let network = Network::start(client,session).expect("production workers");
    network.submit(command).expect("nonblocking submit");
    let started = Instant::now();
    let mut receipt = false;
    let mut advanced = false;
    let mut failures = Vec::new();
    // A bounded hang guard, not a flaky CPU benchmark. Exact zero initial read
    // admission is checked independently with fake time in isolation_tests.
    while !(receipt && advanced) && started.elapsed() < Duration::from_secs(2_u64) {
        if let Some(outcome) = network.poll() {
            match outcome {
                Outcome::Receipt(snapshot) => receipt = snapshot.revision == 6_i64,
                Outcome::Snapshot(snapshot) => advanced = advanced || (snapshot.revision == 9_i64 && snapshot.turn == 2_u32),
                Outcome::Failure(error) | Outcome::Recovering(error) => failures.push(error),
                Outcome::Rejected(rejection) => failures.push(rejection.label()),
            }
        }
        thread::sleep(Duration::from_millis(2_u64));
    }
    drop(network);
    stop.store(true,Ordering::Relaxed);
    let counts = gateway.join().expect("bounded gateway thread").expect("accurate mock contract");
    assert!(receipt && advanced,"own receipt AND actual AI-completed next turn arrive, never one substituting for the other");
    assert!(failures.is_empty(),"healthy stream does not cause recovery: {failures:?}");
    assert_eq!(counts,(1_u32,1_u32),"one persistent subscription and one command; no secondary throttled current read");
}

/// Record exact request headers/body while keeping the fake service fully bounded.
fn request(stream: &mut TcpStream) -> Result<(String, Value), String> {
    let mut reader = BufReader::new(stream);
    let mut headers = String::new();
    let mut length = 0_usize;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).map_err(|error| error.to_string())?;
        if line == "\r\n" || line.is_empty() { break; }
        if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") { length = value.trim().parse::<usize>().map_err(|error| error.to_string())?; }
        headers.push_str(&line);
        if headers.len() > 4096_usize { return Err("mock request exceeded bound".to_owned()); }
    }
    let mut bytes = vec![0_u8;length];
    reader.read_exact(&mut bytes).map_err(|error| error.to_string())?;
    let body = if bytes.is_empty() { json!({}) } else { serde_json::from_slice(&bytes).map_err(|error| error.to_string())? };
    Ok((headers,body))
}

/// Emit complete resyncs with independent canonical revision/durable sequence.
fn snapshot(stream: &mut TcpStream, revision: i64, turn: u32, cursor: u64) -> Result<(),String> {
    write!(stream,"id: {cursor}\nevent: resync\ndata: {{\"current_revision\":{revision},\"event_cursor\":{cursor},\"snapshot\":{{\"revision\":{revision},\"turn\":{turn}}}}}\n\n").map_err(|error| error.to_string())
}

/// The fake accepts either connection order and retains the subscription alive.
fn serve(listener: &TcpListener, stop: &AtomicBool, expected: &Value) -> Result<(u32,u32),String> {
    let started = Instant::now();
    let mut held = None;
    let mut reads = 0_u32;
    let mut posts = 0_u32;
    while !stop.load(Ordering::Relaxed) && started.elapsed() < Duration::from_secs(3_u64) {
        match listener.accept() {
            Ok((mut stream,_peer)) => {
                stream.set_read_timeout(Some(Duration::from_secs(1_u64))).map_err(|error| error.to_string())?;
                stream.set_write_timeout(Some(Duration::from_secs(1_u64))).map_err(|error| error.to_string())?;
                let (headers,body) = request(&mut stream)?;
                if headers.starts_with("GET /game/44/events?session_id=71 ") && headers.contains("X-Gameplay-Stream: snapshots-v1\r\n") && !headers.contains("X-Gameplay-Resync") {
                    reads = reads.saturating_add(1_u32);
                    stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nX-Gameplay-Stream: snapshots-v1\r\n\r\n").map_err(|error| error.to_string())?;
                    snapshot(&mut stream,5_i64,1_u32,50_u64)?;
                    if posts > 0_u32 { snapshot(&mut stream,9_i64,2_u32,900_u64)?; }
                    held = Some(stream);
                } else if headers.starts_with("POST /game/44/pass ") && body == *expected {
                    posts = posts.saturating_add(1_u32);
                    let receipt = json!({"revision":6_i64,"viewer_projection":{"revision":6_i64,"turn":1_u32}}).to_string();
                    write!(stream,"HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{receipt}",receipt.len()).map_err(|error| error.to_string())?;
                    if let Some(subscription) = held.as_mut() { snapshot(subscription,9_i64,2_u32,900_u64)?; }
                } else { return Err("unexpected request; full snapshots must not cause another current read".to_owned()); }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => thread::sleep(Duration::from_millis(2_u64)),
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok((reads,posts))
}
