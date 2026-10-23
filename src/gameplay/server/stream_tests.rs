//! Fragmented HTTP/SSE decoding never requires a real socket or writes files.
use super::stream::{Change, Invalidations};

#[test]
fn fragmented_chunked_sse_preserves_cursor_and_ignores_keepalives() {
    let event = "id: 19\nevent: gameplay\ndata: {\"revision\":12}\n\n";
    let wire = format!(
        "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n{:x}\r\n{}\r\n0\r\n\r\n",
        event.len(),
        event
    );
    for size in [1_usize, 2, 7, 4096] {
        let mut decoder = Invalidations::default();
        let mut received = Vec::new();
        for bytes in wire.as_bytes().chunks(size) {
            received.extend(decoder.push(bytes).expect("valid fragmented stream"));
        }
        assert_eq!(
            received,
            vec![Change {
                cursor: Some(19_u64),
                revision: Some(12_i64)
            }],
            "complete records only, chunk size {size}"
        );
    }
}

#[test]
fn large_resync_payload_is_skipped_without_promoting_snapshot_revision_to_event_sequence() {
    let event = format!("id: 201\nevent: resync\ndata: {}\n\n", "x".repeat(600_000));
    let wire = format!(
        "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n{:x}\r\n{event}\r\n0\r\n\r\n",
        event.len()
    );
    let mut decoder = Invalidations::default();
    let mut received = Vec::new();
    for part in wire.as_bytes().chunks(113) {
        received.extend(
            decoder
                .push(part)
                .expect("large data payload does not require buffering a snapshot"),
        );
    }
    assert_eq!(
        received,
        vec![Change {
            cursor: None,
            revision: None
        }],
        "malformed legacy resync data has neither a trusted durable cursor nor a revision hint"
    );
}

#[test]
fn event_revisions_are_hints_only_and_missing_malformed_or_large_metadata_requires_safe_current_read()
 {
    for data in [
        "{\"revision\":9,\"sequence\":900}",
        "{\"revision\":-1}",
        "{\"revision\":\"9\"}",
        "{\"revision\":null}",
        "{}",
        "not-json",
    ] {
        let wire =
            format!("HTTP/1.1 200 OK\r\n\r\nid: 900\nevent: command.committed\ndata: {data}\n\n");
        let mut parser = Invalidations::default();
        let expected = (data == "{\"revision\":9,\"sequence\":900}").then_some(9_i64);
        assert_eq!(
            parser.push(wire.as_bytes()).expect("bounded hints"),
            vec![Change {
                cursor: Some(900_u64),
                revision: expected
            }],
            "data is never interpreted as canonical state; hint {data}"
        );
    }
    let wire = format!(
        "HTTP/1.1 200 OK\r\n\r\nid: 901\nevent: command.committed\ndata: {}\n\n",
        "x".repeat(600_000_usize)
    );
    assert_eq!(
        Invalidations::default()
            .push(wire.as_bytes())
            .expect("skip large noncanonical data"),
        vec![Change {
            cursor: Some(901_u64),
            revision: None
        }],
        "oversized data cannot grow retained memory or claim a known revision"
    );
}
#[test]
fn sse_errors_and_unbounded_data_fail_closed() {
    let mut decoder = Invalidations::default();
    assert!(
        decoder.push(b"HTTP/1.1 403 Forbidden\r\n\r\n").is_err(),
        "membership denial is not a stream"
    );
    let mut limited = Invalidations::default();
    let error = limited
        .push(b"HTTP/1.1 429 Too Many Requests\r\n\r\n")
        .expect_err("capacity refusal is not an idle authorized stream");
    assert!(
        error.contains("429"),
        "surface actual HTTP status for actionable recovery diagnostics: {error}"
    );
    let mut oversized = Invalidations::default();
    assert!(
        oversized.push(&vec![b'x'; 300_000]).is_err(),
        "bounded buffering"
    );
}
