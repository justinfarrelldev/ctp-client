//! Tests for authoritative server HTTP client.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use serde_json::json;

use super::{
    HttpEndpoint, ServerClient, ServerConfig, ServerGameplaySession, ServerProjection,
    decode_chunked_body, parse_resync_projection,
};

#[test]
fn endpoint_parses_localhost_with_default_port() {
    let endpoint = HttpEndpoint::parse("http://localhost").expect("endpoint should parse");

    assert_eq!(endpoint.socket_address(), "localhost:9000");
    assert_eq!(endpoint.path_with_prefix("/health"), "/health");
}

#[test]
fn resync_projection_parses_sse_payload() {
    let projection = parse_resync_projection(
            "id: 1\nevent: resync\ndata: {\"revision\":2,\"turn\":1,\"tiles\":[{\"id\":\"t\",\"x\":1,\"y\":2,\"terrain\":\"plains\"}]}\n\n",
        )
        .expect("projection should parse");

    assert_eq!(projection.revision, 2);
    assert_eq!(projection.tiles[0].terrain, "plains");
}

#[test]
fn chunked_body_decoder_concatenates_chunks() {
    let body = decode_chunked_body(b"5\r\nhello\r\n6\r\n world\r\n0\r\n\r\n")
        .expect("chunked body should decode");

    assert_eq!(body, "hello world");
}

#[test]
fn health_check_hits_local_server() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("test listener should bind");
    let port = listener
        .local_addr()
        .expect("listener should expose address")
        .port();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("client should connect");
        let mut request = [0_u8; 512];
        let count = stream.read(&mut request).expect("request should read");
        let request_text = String::from_utf8_lossy(&request[..count]);
        assert!(
            request_text.starts_with("GET /health HTTP/1.1"),
            "health check should request /health, got {request_text}"
        );
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 15\r\n\r\n{\"status\":\"OK\"}")
            .expect("response should write");
    });
    let client = ServerClient::from_config(&ServerConfig {
        base_url: format!("http://127.0.0.1:{port}"),
        required: true,
    })
    .expect("client should parse config");

    client.check_health().expect("health check should pass");
    handle.join().expect("server thread should finish");
}

#[test]
fn bootstrap_posts_authoritative_setup_sequence() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("test listener should bind");
    let port = listener
        .local_addr()
        .expect("listener should expose address")
        .port();
    let handle = thread::spawn(move || {
        let responses = vec![
            json!({"status":"OK"}).to_string(),
            "1".to_owned(),
            json!({"account_id":7_i32,"session_id":11_i32}).to_string(),
            json!({"id":44_i32}).to_string(),
            json!({"id":44_i32}).to_string(),
            json!({"id":44_i32}).to_string(),
            json!({"id":44_i32}).to_string(),
            "event: resync\ndata: {\"revision\":1,\"turn\":1}\n\n".to_owned(),
        ];
        for response in responses {
            let (mut stream, _) = listener.accept().expect("client should connect");
            let mut request = [0_u8; 4096];
            let count = stream.read(&mut request).expect("request should read");
            let request_text = String::from_utf8_lossy(&request[..count]);
            assert!(
                request_text.contains("/health")
                    || request_text.contains("/account/create_account")
                    || request_text.contains("/account/login")
                    || request_text.contains("/game/create_game")
                    || request_text.contains("/game/select_country")
                    || request_text.contains("/game/add_ai")
                    || request_text.contains("/game/start_game")
                    || request_text.contains("/game/44/events?session_id=11"),
                "unexpected request: {request_text}"
            );
            let message = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                response.len(),
                response
            );
            stream
                .write_all(message.as_bytes())
                .expect("response should write");
        }
    });
    let client = ServerClient::from_config(&ServerConfig {
        base_url: format!("http://127.0.0.1:{port}"),
        required: true,
    })
    .expect("client should parse config");

    let session = client
        .bootstrap_local_singleplayer("Tester", 2)
        .expect("bootstrap should complete against fake server");

    assert_eq!(session.game_id, 44);
    assert_eq!(session.session_id, 11);
    assert_eq!(
        session
            .projection
            .expect("projection should exist")
            .revision,
        1
    );
    handle.join().expect("server thread should finish");
}

#[test]
fn submit_command_posts_authoritative_command_envelope() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("test listener should bind");
    let port = listener
        .local_addr()
        .expect("listener should expose address")
        .port();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("client should connect");
        let mut request = [0_u8; 4096];
        let count = stream.read(&mut request).expect("request should read");
        let request_text = String::from_utf8_lossy(&request[..count]);
        assert!(
            request_text.starts_with("POST /game/44/round-complete HTTP/1.1"),
            "command route should be authoritative, got {request_text}"
        );
        assert!(
            request_text.contains("\"session_id\":11"),
            "command envelope should include session id: {request_text}"
        );
        assert!(
            request_text.contains("\"expected_revision\":1"),
            "command envelope should include expected revision: {request_text}"
        );
        let response = json!({
            "revision": 2_i32,
            "receipt_id": "receipt-1",
            "event_cursor": 9_i32,
            "viewer_projection": {"revision": 2_i32, "turn": 1_i32}
        })
        .to_string();
        let message = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            response.len(),
            response
        );
        stream
            .write_all(message.as_bytes())
            .expect("response should write");
    });
    let client = ServerClient::from_config(&ServerConfig {
        base_url: format!("http://127.0.0.1:{port}"),
        required: true,
    })
    .expect("client should parse config");
    let session = ServerGameplaySession {
        game_id: 44,
        session_id: 11,
        account_id: 7,
        projection: Some(ServerProjection {
            revision: 1,
            turn: 1,
            ..ServerProjection::default()
        }),
    };

    let response = client
        .submit_command(&session, "round-complete", json!({}))
        .expect("command should post to fake server");

    assert_eq!(response.revision, 2);
    assert_eq!(
        response
            .viewer_projection
            .expect("projection should be returned")
            .revision,
        2
    );
    handle.join().expect("server thread should finish");
}
