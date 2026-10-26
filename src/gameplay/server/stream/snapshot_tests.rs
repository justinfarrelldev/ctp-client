//! Negotiated snapshot streams use injected bytes only, never sockets or files.
use super::Invalidations;

/// One complete response using the additive authenticated snapshot contract.
fn wire(data: &str) -> String {
    let event = format!("id: 900\nevent: resync\ndata: {data}\n\n");
    format!("HTTP/1.1 200 OK\r\nX-Gameplay-Stream: snapshots-v1\r\nTransfer-Encoding: chunked\r\n\r\n{:x}\r\n{event}\r\n0\r\n\r\n", event.len())
}

#[test]
fn negotiated_fragmented_full_snapshot_arrives_without_a_second_http_read() {
    let data = format!(r#"{{"current_revision":7,"event_cursor":900,"snapshot":{{"revision":7,"turn":3,"tiles":[],"padding":"{}"}}}}"#, "é".repeat(6_000_usize));
    for size in [1_usize, 97_usize, 8192_usize] {
        let mut parser = Invalidations::default();
        let mut hints = Vec::new();
        for bytes in wire(&data).as_bytes().chunks(size) {
            hints.extend(parser.push(bytes).expect("bounded fragmented snapshot"));
        }
        assert!(hints.is_empty(), "a complete snapshot must not trigger a limited current read");
        let snapshot = parser.take_snapshot().expect("complete authorized model, not a hint");
        assert_eq!(snapshot.cursor, 900_u64, "sequence namespace remains independent");
        assert_eq!(snapshot.projection.revision, 7_i64, "exact canonical revision");
        assert_eq!(snapshot.projection.turn, 3_u32, "turn is received, never advanced locally");
        assert!(parser.take_snapshot().is_none(), "each model is consumed once");
    }
}

#[test]
fn negotiated_snapshots_reject_ambiguous_or_inconsistent_metadata() {
    for data in [
        r#"{"current_revision":7,"event_cursor":901,"snapshot":{"revision":7,"turn":3}}"#,
        r#"{"current_revision":8,"event_cursor":900,"snapshot":{"revision":7,"turn":3}}"#,
        r#"{"current_revision":7,"snapshot":{"revision":7,"turn":3}}"#,
        r#"{"current_revision":7,"event_cursor":900,"snapshot":{"revision":7,"turn":0}}"#,
        r#"{"current_revision":-1,"event_cursor":900,"snapshot":{"revision":-1,"turn":3}}"#,
        r#"{"revision":7,"turn":3}"#,
    ] {
        assert!(Invalidations::default().push(wire(data).as_bytes()).is_err(), "fail closed instead of accepting a wrapper or guessed cursor: {data}");
    }
}

#[test]
fn negotiated_stream_retains_only_latest_complete_snapshot_and_handles_terminal() {
    let mut parser = Invalidations::default();
    parser.push(wire(r#"{"current_revision":7,"event_cursor":900,"snapshot":{"revision":7,"turn":3}}"#).as_bytes()).expect("initial snapshot");
    assert!(parser.take_snapshot().is_some(), "complete current model available immediately");
    let mut terminal = Invalidations::default();
    terminal.push(b"HTTP/1.1 200 OK\r\nX-Gameplay-Stream: snapshots-v1\r\n\r\nid: 901\nevent: terminal\ndata: {\"revision\":8,\"turn\":3,\"terminal\":{\"reason\":\"score\"}}\n\n").expect("projected terminal model");
    assert_eq!(terminal.take_snapshot().expect("terminal snapshot").projection.revision, 8_i64, "terminal is a complete model, not fabricated advancement");
}

#[test]
fn malformed_snapshot_errors_never_echo_viewer_or_credential_values() {
    let error = Invalidations::default().push(wire(r#"{"current_revision":"private-session-value","event_cursor":900,"snapshot":{"revision":7,"turn":3}}"#).as_bytes()).expect_err("invalid revision type");
    assert!(!error.contains("private-session-value"), "stream diagnostics cannot echo response payloads");
}

#[test]
fn negotiated_snapshot_memory_is_bounded_before_decoding() {
    let mut parser = Invalidations::default();
    parser.push(b"HTTP/1.1 200 OK\r\nX-Gameplay-Stream: snapshots-v1\r\n\r\nid: 900\nevent: resync\ndata: ").expect("bounded headers");
    let block = vec![b'x'; 8192_usize];
    let mut rejected = false;
    for _ in 0_u16..2_100_u16 {
        if parser.push(&block).is_err() { rejected = true; break; }
    }
    assert!(rejected, "a malicious incomplete snapshot cannot grow memory indefinitely");
    assert!(parser.take_snapshot().is_none(), "partial bytes cannot become a model");
}
