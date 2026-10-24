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
use serde_json::Value;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::{GameplayError, asset_error, runtime_error};

/// Complete management metadata and source-authorized actions.
pub mod interface;
/// Source-compatible startup and safe endpoint/status failure context.
mod setup;
#[cfg(test)]
mod interface_tests;
/// Viewer snapshot envelope and Go nil-slice decoding.
mod projection;
/// Complete revision/city/pin-bound append eligibility, never private client rules.
pub(in crate::gameplay) mod production;
/// Public bounded command rejections, distinct from transport uncertainty.
pub mod rejection;
/// Observed art geometry, committed actor context and exact pinned provenance.
pub mod observations;
/// Frozen public setup membership for client-owned original display-slot assignment.
pub mod roster;
/// Bounded public Server-Timing/read-mode metrics, never raw response headers.
mod telemetry;
/// Repaired metadata/cursor compatibility and privacy regressions.
#[cfg(test)]
mod repairs_tests;
use projection::deserialize_nullable_vec;

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
/* Released ctp2/ctp2_data/default/gamedata/civilisation.txt and
 * english/gamedata/civ_str.txt define these original country aliases.
 * Aztecs are absent in this released42-record catalog (an installation extension,
 * not a base-game default). Server internal/game/start_gameplay.go resolves
 * pinned civilization aliases and rejects the whole start for an unknown one.
 * Use supported defaults, never silently retry a rejected setup with fewer AIs,
 * a different country or injected rules. Custom catalogs may still reject safely.
 */
/// Original-compatible default AI slots, independent of encounter/render ordering.
const AI_COUNTRIES: [&str; 7] = [
    "Romans",
    "English",
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
    /// Persisted server participant slots; frozen setup order only for legacy servers.
    pub roster:roster::PlayerRoster,
}

/// Minimal viewer-safe server projection needed by the renderer.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct ServerProjection {
    /// Required canonical revision; a wrapper must not become a default world.
    pub revision: i64,
    /// Required authoritative turn.
    pub turn: u32,
    /// Visible/remembered tiles; Go nil slices encode as null.
    #[serde(default, deserialize_with = "deserialize_nullable_vec")]
    pub tiles: Vec<ServerTileProjection>,
    /// Visible units; Go nil slices encode as null.
    #[serde(default, deserialize_with = "deserialize_nullable_vec")]
    pub units: Vec<ServerUnitProjection>,
    /// Visible/remembered cities; nil before the first settlement.
    #[serde(default, deserialize_with = "deserialize_nullable_vec")]
    pub cities: Vec<ServerCityProjection>,
    /// Diplomacy and resource summary.
    #[serde(default)]
    pub diplomacy: Option<ServerDiplomacyProjection>,
    /// Viewer-safe management data, preserved instead of silently discarded.
    #[serde(flatten)]
    pub interface: interface::ServerInterface,
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
    pub river: Option<bool>,
    /// Canonical resource ID, when exposed by the viewer-safe server contract.
    #[serde(default)]
    pub good_id: Option<String>,
    /// Observed ruin; missing legacy data is unknown.
    #[serde(default)]
    pub ruin: Option<bool>,
    /// Observed territorial owner; empty means known neutral.
    #[serde(default)]
    pub owner_id: Option<String>,
    /// Exact observed city-influence assignment; empty is known none, null/missing unknown.
    #[serde(default)]
    pub influence_city_id:Option<String>,
    /// Effective observed tile yields, not base rules.
    #[serde(default)]
    pub yields: Option<v2::TileYield>,
    /// Terrain versus city-center yield semantics.
    #[serde(default)]
    pub yield_context: String,
    /// General observed movement cost, not unit authorization.
    #[serde(default)]
    pub movement_cost: Option<i64>,
    /// Whether this tile is remembered but currently not visible.
    #[serde(default)]
    pub stale: bool,
    /// Viewer-specific planning facts, absent on remembered/unreachable cells.
    #[serde(default)]
    pub planning: Option<interface::TilePlanning>,
    /// Remaining viewer-safe overlay/pollution metadata retained losslessly.
    #[serde(flatten)]
    pub facts: Value,
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
    /// Viewer-exposed canonical capabilities, including synthetic starter settlement.
    #[serde(default, deserialize_with = "deserialize_nullable_vec")]
    pub effects: Vec<ServerUnitEffectProjection>,
    /// Combat/fortification/cargo metadata already exposed by the server.
    #[serde(flatten)]
    pub facts: Value,
}

/// Minimal capability projection; opaque IDs/values are not needed for art roles.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct ServerUnitEffectProjection {
    /// Exact canonical effect kind, such as `settler`.
    pub kind: String,
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
    /// Observed semantic age/capital facts.
    #[serde(default)]
    pub art: Option<v2::CityArt>,
    /// Owner-private production governor.
    #[serde(default)]
    pub governor: Option<v2::Governor>,
    /// Revision-bound price display, separate from command payload.
    #[serde(default)]
    pub rush_buy_quote: Option<v2::Quote>,
    /// Effective current-economy production estimate.
    #[serde(default)]
    pub production_forecast: Option<v2::Forecast>,
    /// Queue, buildings, specialists, stored resources and accounting facts.
    #[serde(flatten)]
    pub facts: Value,
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
    /// Government, rates, policies, readiness, contacts and proposals.
    #[serde(flatten)]
    pub facts: Value,
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
        if projection.revision >= self.revision() {
            if let Some(roster)=roster::PlayerRoster::from_projection(&projection.interface){self.roster=roster.for_account(self.account_id);}
            self.projection = Some(projection);
        }
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
            Err(GameplayError::Rejected {rejection:rejection::Rejection::decode(response.status,&response.body)})
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
        setup::bootstrap(self,leader_name,empire_count)
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
        envelope::PreparedCommand::new(session, command, payload)?.execute(self)
    }

    /// Fetches a one-shot current projection through the SSE resync endpoint.
    ///
    /// # Errors
    /// Returns an IO, membership or inconsistent snapshot error; never falls back to canonical maps.
    pub(in crate::gameplay) fn fetch_resync_projection(
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
        let projection = ServerProjection::from_resync_event(&response.body)?;
        log::info!(
            "Open CTP game #{game_id} snapshot refresh: revision {}, turn {}, {} tiles, {} units, {} cities",
            projection.revision,
            projection.turn,
            projection.tiles.len(),
            projection.units.len(),
            projection.cities.len()
        );
        Ok(projection)
    }

    /// Sends JSON and parses JSON response.
    fn post_json(&self, path: &str, body: &Value) -> Result<Value, GameplayError> {
        let body_text = serde_json::to_string(body)
            .map_err(|error| asset_error(format!("failed to encode server request: {error}")))?;
        let response = self.request_json_with_capacity_retry(path, &body_text)?;
        if !(200..300).contains(&response.status) {
            return Err(GameplayError::Rejected { rejection:rejection::Rejection::decode(response.status,&response.body) });
        }
        serde_json::from_str(&response.body).map_err(|error| {
            if error.is_eof() {
                runtime_error(format!(
                    "failed to read server response: incomplete JSON: {error}"
                ))
            } else {
                asset_error(format!("server response for {path} was not JSON: {error}"))
            }
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
        let started=std::time::Instant::now();
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
        let response=parse_http_response(&bytes).map_err(|error| runtime_error(format!("failed to read server response: {error}")))?;
        let route=path.split('?').next().and_then(|path|path.rsplit('/').next()).filter(|route|route.len()<=64_usize&&route.bytes().all(|byte|byte.is_ascii_alphanumeric()||byte==b'-')).unwrap_or("unknown");
        log::debug!("Gameplay HTTP route={route} status={} client_http_ms={} server_timing={} read_mode={}",response.status,started.elapsed().as_millis(),response.telemetry.server_timing.as_deref().unwrap_or("unavailable"),response.telemetry.read_mode.as_deref().unwrap_or("unavailable"));
        Ok(response)
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
    /// Explicit bounded gateway metrics, not canonical state or private headers.
    telemetry:telemetry::Metadata,
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
    Ok(HttpResponse { status, body, telemetry:telemetry::Metadata::parse(header_text) })
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

/// Actual startup catalog/legacy rejection regressions, no file or live effects.
#[cfg(test)]
mod launch_tests;
#[cfg(test)]
mod command_tests;
/// Exact command input shapes and current viewer authorization.
pub mod commands;
/// Immutable retryable command envelopes.
pub mod envelope;
#[cfg(test)]
mod projection_tests;
/// Viewer-safe streaming invalidations.
pub(in crate::gameplay) mod stream;
#[cfg(test)]
mod stream_tests;
#[cfg(test)]
pub(super) mod test_support;
#[cfg(test)]
mod tests;
/// Additive viewer contracts and known/unavailable display facts.
pub mod v2;
#[cfg(test)]
mod v2_tests;
