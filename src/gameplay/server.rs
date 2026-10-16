//! Blocking HTTP client for the authoritative Open CTP server.
//!
//! The in-game client must not silently become a separate local rules engine.
//! This module performs the local-server health check and the documented
//! single-human, server-managed-AI bootstrap flow described by the server README.

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write as _;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::{GameplayError, asset_error, runtime_error};

/// Default local server endpoint.
pub const DEFAULT_SERVER_URL: &str = "http://localhost:9000";
/// Environment variable overriding the authoritative server endpoint.
pub const SERVER_URL_ENV: &str = "OPEN_CTP_SERVER_URL";
/// HTTP timeout used during launch bootstrap.
const HTTP_TIMEOUT: Duration = Duration::from_secs(5);
/// Backoff delays for transient local setup capacity responses.
const SETUP_RETRY_DELAYS: [Duration; 5] = [
    Duration::from_millis(100),
    Duration::from_millis(250),
    Duration::from_millis(500),
    Duration::from_secs(1),
    Duration::from_secs(2),
];
/// AI country slots used for local single-human games.
const AI_COUNTRIES: [&str; 7] = [
    "Romans",
    "Aztecs",
    "Egyptians",
    "Japanese",
    "Vikings",
    "Persians",
    "Zulu",
];

/// Server use policy for gameplay launch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    /// Base URL for the authoritative server.
    pub base_url: String,
    /// Whether gameplay launch must fail when the server is unavailable.
    pub required: bool,
}

impl ServerConfig {
    /// Builds default interactive server configuration.
    #[must_use]
    pub fn interactive_default() -> Self {
        let base_url =
            std::env::var(SERVER_URL_ENV).unwrap_or_else(|_| DEFAULT_SERVER_URL.to_owned());
        Self {
            base_url,
            required: true,
        }
    }

    /// Builds disabled configuration for rendering/model tests that must not perform network I/O.
    #[cfg(test)]
    #[must_use]
    pub fn disabled_for_test() -> Self {
        Self {
            base_url: DEFAULT_SERVER_URL.to_owned(),
            required: false,
        }
    }
}

/// Server-side gameplay session metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerGameplaySession {
    /// Created game id.
    pub game_id: i64,
    /// Authenticated session id.
    pub session_id: i64,
    /// Account id for the human player.
    pub account_id: i64,
    /// Latest viewer-safe projection returned by the server.
    pub projection: Option<ServerProjection>,
}

/// Minimal viewer-safe server projection needed by the renderer.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct ServerProjection {
    /// Canonical revision.
    #[serde(default)]
    pub revision: i64,
    /// Current turn.
    #[serde(default)]
    pub turn: u32,
    /// Visible/remembered tiles.
    #[serde(default)]
    pub tiles: Vec<ServerTileProjection>,
    /// Visible units.
    #[serde(default)]
    pub units: Vec<ServerUnitProjection>,
    /// Visible/remembered cities.
    #[serde(default)]
    pub cities: Vec<ServerCityProjection>,
    /// Diplomacy and resource summary.
    #[serde(default)]
    pub diplomacy: Option<ServerDiplomacyProjection>,
}

/// Minimal server tile projection.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct ServerTileProjection {
    /// Server tile id.
    #[serde(default)]
    pub id: String,
    /// X coordinate.
    #[serde(default)]
    pub x: u16,
    /// Y coordinate.
    #[serde(default)]
    pub y: u16,
    /// Server terrain id.
    #[serde(default)]
    pub terrain: String,
    /// River metadata when supplied by a viewer-safe server projection.
    /// Older servers omit this field; omission must not fabricate rivers.
    #[serde(default)]
    pub river: bool,
    /// Canonical resource ID, when exposed by the viewer-safe server contract.
    #[serde(default)]
    pub good_id: String,
    /// Whether this tile is remembered but currently not visible.
    #[serde(default)]
    pub stale: bool,
}

/// Minimal server unit projection.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct ServerUnitProjection {
    /// Server unit id.
    #[serde(default)]
    pub id: String,
    /// Runtime unit type id.
    #[serde(default)]
    pub type_id: String,
    /// Owning player id.
    #[serde(default)]
    pub owner_id: String,
    /// Current tile id.
    #[serde(default)]
    pub tile_id: String,
    /// Remaining movement points.
    #[serde(default)]
    pub movement: i16,
    /// Maximum movement points.
    #[serde(default)]
    pub max_movement: i16,
}

/// Minimal server city projection.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct ServerCityProjection {
    /// Server city id.
    #[serde(default)]
    pub id: String,
    /// Owning player id.
    #[serde(default)]
    pub owner_id: String,
    /// Current tile id.
    #[serde(default)]
    pub tile_id: String,
    /// Population.
    #[serde(default)]
    pub population: u32,
    /// Whether the city is remembered but currently not visible.
    #[serde(default)]
    pub stale: bool,
}

/// Minimal server diplomacy/resource projection.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct ServerDiplomacyProjection {
    /// Visible gold reserve.
    #[serde(default)]
    pub gold: i32,
    /// Visible public works reserve.
    #[serde(default)]
    pub public_works: i32,
}

/// Blocking authoritative server client.
#[derive(Debug, Clone)]
pub struct ServerClient {
    /// Parsed endpoint.
    endpoint: HttpEndpoint,
}

impl ServerGameplaySession {
    /// Returns the latest known authoritative revision.
    #[must_use]
    pub fn revision(&self) -> i64 {
        self.projection
            .as_ref()
            .map_or(0, |projection| projection.revision)
    }

    /// Applies a fresh projection returned by a command receipt.
    pub fn apply_projection(&mut self, projection: ServerProjection) {
        self.projection = Some(projection);
    }
}

/// Response from one authoritative gameplay command.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ServerCommandResponse {
    /// New authoritative revision.
    #[serde(default)]
    pub revision: i64,
    /// Server receipt id.
    #[serde(default)]
    pub receipt_id: String,
    /// Viewer-safe projection after the command.
    #[serde(default)]
    pub viewer_projection: Option<ServerProjection>,
}

impl ServerClient {
    /// Creates a client from server configuration.
    ///
    /// # Errors
    ///
    /// Returns [`GameplayError`] when the URL is unsupported or malformed.
    pub fn from_config(config: &ServerConfig) -> Result<Self, GameplayError> {
        Ok(Self {
            endpoint: HttpEndpoint::parse(&config.base_url)?,
        })
    }

    /// Verifies `/health` returns a successful response.
    ///
    /// # Errors
    ///
    /// Returns [`GameplayError`] when the local server is not reachable.
    pub fn check_health(&self) -> Result<(), GameplayError> {
        let response = self.request("GET", "/health", None, &[])?;
        if response.status == 200 {
            Ok(())
        } else {
            Err(runtime_error(format!(
                "Open CTP server health check failed with HTTP {}. Start the local server in $CTPSERVER and keep it running.",
                response.status
            )))
        }
    }

    /// Creates a local server-managed single-human game and fetches the first projection.
    ///
    /// # Errors
    ///
    /// Returns [`GameplayError`] when any authoritative setup endpoint fails.
    pub fn bootstrap_local_singleplayer(
        &self,
        leader_name: &str,
        empire_count: u8,
    ) -> Result<ServerGameplaySession, GameplayError> {
        self.check_health()?;
        let login = self.create_or_login_ephemeral_account(leader_name)?;
        let game = self.post_json(
            "/game/create_game",
            &json!({
                "session_id": login.session_id,
                "owner_account_id": login.account_id,
                "name": format!("{}'s Local Game", leader_name),
                "style": "sequential",
                "starting_settlers": 1_i32,
                "starting_gold": 0_i32,
                "is_public": false,
                "is_locked": true,
                "password_protected": false
            }),
        )?;
        let game_id = json_i64(&game, "id")?;
        let _selected = self.post_json(
            "/game/select_country",
            &json!({"session_id": login.session_id, "game_id": game_id, "country": "Greeks"}),
        )?;
        for (index, country) in AI_COUNTRIES
            .iter()
            .take(usize::from(empire_count.saturating_sub(1)))
            .enumerate()
        {
            let _ai = self.post_json(
                "/game/add_ai",
                &json!({
                    "session_id": login.session_id,
                    "game_id": game_id,
                    "country": country,
                    "name": format!("AI {}", index.saturating_add(1))
                }),
            )?;
        }
        let _started = self.post_json(
            "/game/start_game",
            &json!({"session_id": login.session_id, "game_id": game_id}),
        )?;
        let projection = self.fetch_resync_projection(game_id, login.session_id)?;
        Ok(ServerGameplaySession {
            game_id,
            session_id: login.session_id,
            account_id: login.account_id,
            projection: Some(projection),
        })
    }

    /// Submits a gameplay command through the authoritative command gateway.
    ///
    /// # Errors
    ///
    /// Returns [`GameplayError`] when the server rejects the command or the response cannot be decoded.
    pub fn submit_command(
        &self,
        session: &ServerGameplaySession,
        command: &str,
        payload: Value,
    ) -> Result<ServerCommandResponse, GameplayError> {
        let mut body = match payload {
            Value::Object(map) => map,
            _ => Map::new(),
        };
        let idempotency_key = unique_client_key(command)?;
        body.insert("session_id".to_owned(), json!(session.session_id));
        body.insert("expected_revision".to_owned(), json!(session.revision()));
        body.insert("idempotency_key".to_owned(), json!(idempotency_key));
        body.insert(
            "command_id".to_owned(),
            json!(format!("ctp-client-{idempotency_key}")),
        );
        let path = format!("/game/{}/{}", session.game_id, command);
        let value = self.post_json(&path, &Value::Object(body))?;
        serde_json::from_value(value).map_err(|error| {
            asset_error(format!(
                "server command response did not match schema: {error}"
            ))
        })
    }

    /// Creates and logs into a unique local account for the launch session.
    fn create_or_login_ephemeral_account(
        &self,
        leader_name: &str,
    ) -> Result<LoginResponse, GameplayError> {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| runtime_error(format!("system clock is before UNIX epoch: {error}")))?
            .as_nanos();
        let email = format!("ctp-client-{suffix}@local.invalid");
        let password = format!("ctp-client-{suffix}");
        let _created = self.post_json(
            "/account/create_account",
            &json!({
                "account": {
                    "email": email,
                    "name": leader_name,
                    "experience_level": 2_i32,
                    "location": "Local",
                    "info": "Temporary local client account"
                },
                "password": password
            }),
        )?;
        let login = self.post_json(
            "/account/login",
            &json!({"email": email, "password": password}),
        )?;
        serde_json::from_value(login).map_err(|error| {
            asset_error(format!(
                "server login response did not match schema: {error}"
            ))
        })
    }

    /// Fetches a one-shot current projection through the SSE resync endpoint.
    fn fetch_resync_projection(
        &self,
        game_id: i64,
        session_id: i64,
    ) -> Result<ServerProjection, GameplayError> {
        let path = format!("/game/{game_id}/events?session_id={session_id}");
        let response = self.request("GET", &path, None, &[("X-Gameplay-Resync", "current")])?;
        if response.status != 200 {
            return Err(runtime_error(format!(
                "server projection resync failed with HTTP {}",
                response.status
            )));
        }
        parse_resync_projection(&response.body)
    }

    /// Sends JSON and parses JSON response.
    fn post_json(&self, path: &str, body: &Value) -> Result<Value, GameplayError> {
        let body_text = serde_json::to_string(body)
            .map_err(|error| asset_error(format!("failed to encode server request: {error}")))?;
        let response = self.request_json_with_capacity_retry(path, &body_text)?;
        if !(200..300).contains(&response.status) {
            return Err(runtime_error(format!(
                "server request {path} failed with HTTP {}: {}",
                response.status, response.body
            )));
        }
        serde_json::from_str(&response.body).map_err(|error| {
            asset_error(format!("server response for {path} was not JSON: {error}"))
        })
    }

    /// Sends a JSON request, retrying transient local capacity responses.
    fn request_json_with_capacity_retry(
        &self,
        path: &str,
        body_text: &str,
    ) -> Result<HttpResponse, GameplayError> {
        let mut response =
            self.request("POST", path, Some((body_text, "application/json")), &[])?;
        for delay in SETUP_RETRY_DELAYS {
            if response.status != 429_u16 {
                return Ok(response);
            }
            log::warn!(
                "Open CTP server capacity-limited {path}; retrying in {} ms",
                delay.as_millis()
            );
            thread::sleep(delay);
            response = self.request("POST", path, Some((body_text, "application/json")), &[])?;
        }
        Ok(response)
    }

    /// Sends one HTTP request.
    fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<(&str, &str)>,
        extra_headers: &[(&str, &str)],
    ) -> Result<HttpResponse, GameplayError> {
        let address = self.endpoint.socket_address();
        let resolved = address
            .to_socket_addrs()
            .map_err(|error| runtime_error(format!("failed to resolve {address}: {error}")))?
            .next()
            .ok_or_else(|| runtime_error(format!("server address {address} did not resolve")))?;
        let mut stream = TcpStream::connect_timeout(&resolved, HTTP_TIMEOUT).map_err(|error| {
            runtime_error(format!(
                "could not connect to Open CTP server at {}. Start it with `cd $CTPSERVER && task run` (and Supabase if required): {error}",
                self.endpoint.base_url()
            ))
        })?;
        stream
            .set_read_timeout(Some(HTTP_TIMEOUT))
            .map_err(|error| runtime_error(format!("failed to set read timeout: {error}")))?;
        stream
            .set_write_timeout(Some(HTTP_TIMEOUT))
            .map_err(|error| runtime_error(format!("failed to set write timeout: {error}")))?;
        let (body_text, content_type) = body.unwrap_or(("", "text/plain"));
        let full_path = self.endpoint.path_with_prefix(path);
        let mut request = format!(
            "{method} {full_path} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nAccept: application/json\r\nContent-Length: {}\r\n",
            self.endpoint.host_header(),
            body_text.len()
        );
        if !body_text.is_empty() {
            write!(&mut request, "Content-Type: {content_type}\r\n").map_err(|error| {
                runtime_error(format!("failed to write request header: {error}"))
            })?;
        }
        for &(name, value) in extra_headers {
            write!(&mut request, "{name}: {value}\r\n").map_err(|error| {
                runtime_error(format!("failed to write request header: {error}"))
            })?;
        }
        request.push_str("\r\n");
        request.push_str(body_text);
        stream
            .write_all(request.as_bytes())
            .map_err(|error| runtime_error(format!("failed to write server request: {error}")))?;
        let mut bytes = Vec::new();
        stream
            .read_to_end(&mut bytes)
            .map_err(|error| runtime_error(format!("failed to read server response: {error}")))?;
        parse_http_response(&bytes)
    }
}

/// Login response from the server.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct LoginResponse {
    /// Account id.
    account_id: i64,
    /// Session id.
    session_id: i64,
}

/// Parsed HTTP endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
struct HttpEndpoint {
    /// Host without port.
    host: String,
    /// Port.
    port: u16,
    /// Optional path prefix.
    prefix: String,
}

impl HttpEndpoint {
    /// Parses an HTTP URL.
    fn parse(url: &str) -> Result<Self, GameplayError> {
        let without_scheme = url.strip_prefix("http://").ok_or_else(|| {
            runtime_error(
                "only http:// Open CTP server URLs are supported for local launch".to_owned(),
            )
        })?;
        let (authority, prefix) = without_scheme
            .split_once('/')
            .map_or((without_scheme, ""), |(left, right)| (left, right));
        let (host, port) = authority.split_once(':').map_or_else(
            || (authority.to_owned(), 9000_u16),
            |(host, port)| {
                (
                    host.to_owned(),
                    port.parse::<u16>().map_or(9000_u16, |value| value),
                )
            },
        );
        if host.trim().is_empty() {
            return Err(runtime_error("server URL host is empty".to_owned()));
        }
        Ok(Self {
            host,
            port,
            prefix: prefix.trim_matches('/').to_owned(),
        })
    }

    /// Returns host:port for sockets.
    fn socket_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Returns Host header.
    fn host_header(&self) -> String {
        self.socket_address()
    }

    /// Returns display base URL.
    fn base_url(&self) -> String {
        if self.prefix.is_empty() {
            format!("http://{}", self.socket_address())
        } else {
            format!("http://{}/{}", self.socket_address(), self.prefix)
        }
    }

    /// Adds URL prefix to a path.
    fn path_with_prefix(&self, path: &str) -> String {
        let normalized = if path.starts_with('/') {
            path.to_owned()
        } else {
            format!("/{path}")
        };
        if self.prefix.is_empty() {
            normalized
        } else {
            format!("/{}{}", self.prefix, normalized)
        }
    }
}

/// Parsed HTTP response.
#[derive(Debug, Clone, PartialEq, Eq)]
struct HttpResponse {
    /// Status code.
    status: u16,
    /// Response body text.
    body: String,
}

/// Parses a raw HTTP response.
fn parse_http_response(bytes: &[u8]) -> Result<HttpResponse, GameplayError> {
    let text = String::from_utf8_lossy(bytes);
    let (header_text, body_text) = text
        .split_once("\r\n\r\n")
        .ok_or_else(|| runtime_error("server response did not contain HTTP headers".to_owned()))?;
    let status = header_text
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| runtime_error("server response did not contain a status code".to_owned()))?;
    let body = if header_text
        .lines()
        .any(|line| line.eq_ignore_ascii_case("Transfer-Encoding: chunked"))
    {
        decode_chunked_body(body_text.as_bytes())?
    } else {
        body_text.to_owned()
    };
    Ok(HttpResponse { status, body })
}

/// Decodes a small chunked response body.
fn decode_chunked_body(bytes: &[u8]) -> Result<String, GameplayError> {
    let text = String::from_utf8_lossy(bytes);
    let mut body = String::new();
    let mut remaining = text.as_ref();
    loop {
        let Some((size_line, rest)) = remaining.split_once("\r\n") else {
            return Err(runtime_error("chunked response is truncated".to_owned()));
        };
        let size = usize::from_str_radix(size_line.trim(), 16)
            .map_err(|error| runtime_error(format!("invalid chunk size {size_line}: {error}")))?;
        if size == 0 {
            break;
        }
        if rest.len() < size.saturating_add(2) {
            return Err(runtime_error(
                "chunked response payload is truncated".to_owned(),
            ));
        }
        let (chunk, after_chunk) = rest.split_at(size);
        body.push_str(chunk);
        remaining = after_chunk
            .strip_prefix("\r\n")
            .ok_or_else(|| runtime_error("chunked response missing CRLF".to_owned()))?;
    }
    Ok(body)
}

/// Parses the SSE current projection payload.
fn parse_resync_projection(body: &str) -> Result<ServerProjection, GameplayError> {
    for event_block in body.split("\n\n") {
        let mut event_name = "";
        let mut data = String::new();
        for line in event_block.lines() {
            if let Some(value) = line.strip_prefix("event: ") {
                event_name = value.trim();
            } else if let Some(value) = line.strip_prefix("data: ") {
                data.push_str(value.trim());
            }
        }
        if event_name == "resync" {
            return serde_json::from_str(&data).map_err(|error| {
                asset_error(format!("server resync projection was not JSON: {error}"))
            });
        }
    }
    Err(runtime_error(
        "server resync did not include a projection".to_owned(),
    ))
}

/// Builds a unique idempotency token for one client command.
fn unique_client_key(command: &str) -> Result<String, GameplayError> {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| runtime_error(format!("system clock is before UNIX epoch: {error}")))?
        .as_nanos();
    Ok(format!("{command}-{suffix}"))
}

/// Extracts an integer field from JSON.
fn json_i64(value: &Value, field: &str) -> Result<i64, GameplayError> {
    value.get(field).and_then(Value::as_i64).ok_or_else(|| {
        asset_error(format!(
            "server response did not include integer field {field}"
        ))
    })
}

#[cfg(test)]
mod tests;
