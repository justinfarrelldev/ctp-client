//! Source-shaped setup middleware/handlers; only ephemeral test sockets, no live services.
use super::retail_aliases;
use crate::gameplay::{GameplayError,server::{ServerClient,ServerConfig,test_support::synthetic_starting_resync_event}};
use serde_json::{Value,json};
use std::{io::{BufRead,BufReader,Read,Write},net::{TcpListener,TcpStream},thread,time::{Duration,Instant}};

/// Only public fake setup facts are recorded, never generated login credentials.
struct Trace {
    /// Accepted human plus AI choices, in the actual request order.
    countries:Vec<String>,
    /// Startup validation is never retried after a semantic rejection.
    starts:u8,
    /// Exactly three known pre-admission429s, not extra committed AI rows.
    limited:u8,
}
/// Reads complete HTTP bodies rather than assuming a single TCP fragment.
fn request(stream:&mut TcpStream)->(String,Value) {
    let mut reader=BufReader::new(stream);let mut first=String::new();reader.read_line(&mut first).expect("mock request line");let path=first.split_whitespace().nth(1_usize).expect("request path").to_owned();let mut length=0_usize;
    loop {let mut line=String::new();reader.read_line(&mut line).expect("mock headers");if line=="\r\n" {break;}
        if let Some((name,value))=line.split_once(':'){if name.eq_ignore_ascii_case("Content-Length"){length=value.trim().parse().expect("bounded client body");}}}
    let mut bytes=vec![0_u8;length];reader.read_exact(&mut bytes).expect("complete isolated request");let body=if bytes.is_empty(){Value::Null}else{serde_json::from_slice(&bytes).expect("actual request JSON")};(path,body)
}
/// JSON/legacy middleware/plain validation/SSE shapes match the current Go endpoints.
fn serve(listener:&TcpListener,reject_start:bool)->Trace {
    listener.set_nonblocking(true).expect("bounded gateway admission");let deadline=Instant::now().checked_add(Duration::from_secs(10_u64)).expect("test deadline");
    let aliases=retail_aliases();let mut trace=Trace {countries:Vec::new(),starts:0_u8,limited:0_u8};let mut limited_body=None;
    for _ in 0_u8..30_u8 {
        let (mut stream,_)=loop {match listener.accept(){Ok(connection)=>break connection,Err(error) if error.kind()==std::io::ErrorKind::WouldBlock=>{assert!(Instant::now()<deadline,"isolated gateway cannot wait forever");thread::sleep(Duration::from_millis(2_u64));},Err(error)=>assert_eq!(error.kind(),std::io::ErrorKind::WouldBlock,"isolated listener failed before bootstrap completed")}};stream.set_read_timeout(Some(Duration::from_secs(3_u64))).expect("bounded test read");let (path,body)=request(&mut stream);let mut done=false;
        let (status,response)=match path.as_str() {
            "/health"=>(200_u16,json!({"status":"OK"}).to_string()),
            "/account/create_account"=>(200_u16,"1".to_owned()),
            "/account/login"=>(200_u16,json!({"account_id":7_i64,"session_id":11_i64}).to_string()),
            "/game/create_game"=>(200_u16,json!({"id":44_i64}).to_string()),
            "/game/select_country"=>{trace.countries.push(body["country"].as_str().expect("explicit human country").to_owned());(200_u16,json!({"id":44_i64}).to_string())},
            "/game/add_ai" if trace.countries.len()==3_usize&&trace.limited<3_u8=>{
                if let Some(previous)=limited_body.as_ref(){assert_eq!(&body,previous,"429 retry must keep the exact AI request, never append a different/duplicate slot");}else{limited_body=Some(body.clone());}trace.limited=trace.limited.saturating_add(1_u8);(429_u16,json!({"status":"Request Failed","body":"The API is at capacity, try again later."}).to_string())
            },
            "/game/add_ai"=>{trace.countries.push(body["country"].as_str().expect("explicit AI country").to_owned());(200_u16,json!({"id":44_i64}).to_string())},
            "/game/start_game"=>{
                trace.starts=trace.starts.saturating_add(1_u8);
                if reject_start||trace.countries.iter().any(|country|!aliases.contains(&country.to_ascii_lowercase())) {done=true;(400_u16,"selected country is not available in the pinned retail civilization catalog".to_owned())}else{(200_u16,json!({"id":44_i64}).to_string())}
            },
            _ if path.starts_with("/game/44/events?")=>{done=true;(200_u16,synthetic_starting_resync_event())},
            _=>{done=true;(400_u16,"unexpected setup operation".to_owned())},
        };
        write!(stream,"HTTP/1.1 {status} Result\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}",response.len()).expect("isolated handler reply");if done {break;}
    }
    trace
}
#[test]
fn launch_recovery_all_eight_empires_use_supported_countries_and_capacity_retry_keeps_same_setup() {
    let listener=TcpListener::bind("127.0.0.1:0").expect("ephemeral gateway");let port=listener.local_addr().expect("port").port();let worker=thread::spawn(move||serve(&listener,false));let client=ServerClient::from_config(&ServerConfig {base_url:format!("http://127.0.0.1:{port}"),required:true}).expect("mock client");let session=client.bootstrap_local_singleplayer("Tester",8_u8).expect("all original-compatible participants start");let trace=worker.join().expect("gateway");
    assert_eq!(trace.countries.len(),8_usize,"never silently reduce requested empires");assert_eq!(trace.starts,1_u8,"exactly one validated game start");assert_eq!(trace.limited,3_u8,"reproduce the three reported incidental admission warnings");assert_eq!(session.game_id,44_i64,"same authoritative game throughout setup/retries");
}
#[test]
fn launch_recovery_terminal_setup_rejection_retains_exact_stage_status_without_retry_or_world_fetch() {
    let listener=TcpListener::bind("127.0.0.1:0").expect("ephemeral gateway");let port=listener.local_addr().expect("port").port();let worker=thread::spawn(move||serve(&listener,true));let client=ServerClient::from_config(&ServerConfig {base_url:format!("http://127.0.0.1:{port}"),required:true}).expect("mock client");let result=client.bootstrap_local_singleplayer("Tester",4_u8);let trace=worker.join().expect("gateway");
    assert!(matches!(result,Err(GameplayError::Setup {stage,status:Some(400_u16),message}) if stage=="/game/start_game"&&message.contains("civilization")),"bare retail validation failure must not become generic order rejection or fatal menu error");assert_eq!(trace.starts,1_u8,"a semantic rejection cannot trigger country substitution/replayed start");assert_eq!(trace.countries.len(),4_usize,"preserve requested count on failure");
}
