//! Fragmented HTTP/SSE decoding never requires a real socket or writes files.
use super::stream::Invalidations;

#[test]
fn fragmented_chunked_sse_preserves_cursor_and_ignores_keepalives() {
    let event="id: 19\nevent: gameplay\ndata: {\"revision\":12}\n\n";
    let wire=format!("HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n{:x}\r\n{}\r\n0\r\n\r\n",event.len(),event);
    for size in [1_usize,2,7,4096] {
        let mut decoder=Invalidations::default();
        let mut received=Vec::new();
        for bytes in wire.as_bytes().chunks(size) { received.extend(decoder.push(bytes).expect("valid fragmented stream")); }
        assert_eq!(received,vec![19_u64],"complete records only, chunk size {size}");
    }
}

#[test]
fn sse_errors_and_unbounded_data_fail_closed() {
    let mut decoder=Invalidations::default();
    assert!(decoder.push(b"HTTP/1.1 403 Forbidden\r\n\r\n").is_err(),"membership denial is not a stream");
    let mut oversized=Invalidations::default();
    assert!(oversized.push(&vec![b'x'; 300_000]).is_err(),"bounded buffering");
}
