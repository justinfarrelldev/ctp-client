//! Isolated gateway: real HTTP framing, no live service or filesystem effects.
use super::Rejection;
use crate::gameplay::{GameplayError,server::{ServerClient,ServerConfig}};
use serde_json::json;
use std::{io::{BufRead,BufReader,Read,Write},net::TcpListener,thread,time::Duration};

#[test]
fn repaired_rejection_http_retains_safe_structured_identity_instead_of_flattening_body() {
    let listener=TcpListener::bind("127.0.0.1:0").expect("isolated gateway");let port=listener.local_addr().expect("port").port();
    let server=thread::spawn(move || {
        let (mut socket,_)=listener.accept().expect("mock request");socket.set_read_timeout(Some(Duration::from_secs(5_u64))).expect("bounded read");
        let mut reader=BufReader::new(&mut socket);let mut first=String::new();reader.read_line(&mut first).expect("request line");let mut length=0_usize;
        loop {let mut line=String::new();reader.read_line(&mut line).expect("header");if line=="\r\n" {break;}
            if let Some(value)=line.strip_prefix("Content-Length:"){length=value.trim().parse().expect("length");}}
        let mut body=vec![0_u8;length];reader.read_exact(&mut body).expect("complete body");
        let result=r#"{"error":"gameplay request rejected","code":"production_prerequisite","message":"The proposed production queue is not currently buildable.","correlation_id":"reference-1"}"#;
        socket.write_all(format!("HTTP/1.1 422 Unprocessable Entity\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{result}",result.len()).as_bytes()).expect("mock response");first
    });
    let client=ServerClient::from_config(&ServerConfig {base_url:format!("http://127.0.0.1:{port}"),required:true}).expect("fake endpoint");
    let error=client.post_json("/game/44/edit-production",&json!({"city_id":"c2","entries":[]})).expect_err("rejected");
    assert!(matches!(error,GameplayError::Rejected {rejection} if rejection.status==422_u16&&rejection.code=="production_prerequisite"&&rejection.correlation_id.as_deref()==Some("reference-1")),"typed public identity survives HTTP boundary");
    assert!(server.join().expect("fake server").starts_with("POST /game/44/edit-production "),"one exact route");
}

#[test]
fn repaired_rejection_bounds_messages_and_correlation_without_losing_http_status() {
    for body in [json!({"code":"queue_shape","message":"bad\nmessage","correlation_id":"credential=value"}),json!({"code":"queue_shape","message":"x".repeat(513_usize),"correlation_id":"x".repeat(129_usize)})] {
        let rejection=Rejection::decode(400_u16,&body.to_string());
        assert_eq!(rejection.status,400_u16,"status retained");assert_eq!(rejection.message,"The server rejected this order.","unbounded/control-bearing message rejected");assert!(rejection.correlation_id.is_none(),"unsafe reference omitted");
    }
}
