//! Viewer-safe management contracts from server `visibility.go`.
//!
//! Unknown rule/report fields are preserved as JSON rather than reinterpreted as
//! local game rules. Commands use the server's legal-action payloads verbatim.

use serde::Deserialize;
use serde_json::Value;

use super::projection::deserialize_nullable_vec;

/// Viewer-specific movement facts; hidden cells do not acquire invented costs.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct TilePlanning {
    /// Cost after visible terrain, city, improvement and river effects.
    pub cost: i64,
    /// Allowed movement domains.
    #[serde(default, deserialize_with = "deserialize_nullable_vec")]
    pub domains: Vec<String>,
    /// Whether the viewer may enter under ownership/treaty rules.
    #[serde(default)]
    pub accessible: bool,
    /// Whether enemy control stops continued travel.
    #[serde(default)]
    pub enemy_zoc: bool,
}

/// A command explicitly authorized for this viewer and revision.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct LegalAction {
    /// Canonical engine command identifier.
    #[serde(default)]
    pub command: String,
    /// HTTP command route relative to the gameplay endpoint.
    #[serde(default)]
    pub route: String,
    /// Exact server entity identity; never resolved from tile position.
    #[serde(default)]
    pub entity_id: String,
    /// Exact destination/entity target, if applicable.
    #[serde(default)]
    pub target_id: String,
    /// Payload field names accepted by this action.
    #[serde(default, deserialize_with = "deserialize_nullable_vec")]
    pub fields: Vec<String>,
    /// Viewer-safe server-selected payload, including production metadata.
    #[serde(default)]
    pub payload: Value,
}

impl LegalAction {
    /// The server currently advertises some engine-only actions that its HTTP
    /// gateway does not register. Do not present those as executable orders.
    #[must_use]
    pub fn http_supported(&self) -> bool {
        matches!(
            self.route.as_str(),
            "pass"
                | "set-war"
                | "establish-contact"
                | "propose-agreement"
                | "respond-agreement"
                | "set-government"
                | "set-science-rate"
                | "set-public-works-rate"
                | "set-social-policy"
                | "set-readiness"
                | "round-complete"
                | "set-research"
                | "move"
                | "disband"
                | "embark"
                | "disembark"
                | "gift-unit"
                | "pirate-trade"
                | "space-launch"
                | "fortify"
                | "found-city"
                | "army-create"
                | "army-join"
                | "army-leave"
                | "combat"
                | "nuclear-attack"
                | "establish-embassy"
                | "soothsaying"
                | "cause-unhappiness"
                | "advertise"
                | "incite-revolution"
                | "indulgence-sales"
                | "slave-uprising"
                | "throw-party"
                | "steal-technology"
                | "terraform"
                | "build-improvement"
                | "cancel-terraform"
                | "queue-production"
                | "cancel-production"
                | "rush-buy"
                | "set-specialists"
                | "create-trade-route"
                | "leave"
                | "end-early"
        )
    }
}

/// Research facts belonging to the authenticated player's nation.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct Research {
    /// Already discovered canonical advances.
    #[serde(default, deserialize_with = "deserialize_nullable_vec")]
    pub advances: Vec<String>,
    /// Current canonical research goal, empty if none.
    #[serde(default)]
    pub target_id: String,
    /// Authoritative accumulated research.
    #[serde(default)]
    pub progress: i64,
    /// Currently available canonical goals.
    #[serde(default, deserialize_with = "deserialize_nullable_vec")]
    pub available: Vec<String>,
}

/// Management data flattened into the viewer snapshot, not another envelope.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct ServerInterface {
    /// Actions authorized by the engine for this exact revision/viewer.
    #[serde(default, deserialize_with = "deserialize_nullable_vec")]
    pub legal_actions: Vec<LegalAction>,
    /// Current research and available goals.
    #[serde(default)]
    pub research: Option<Research>,
    /// Runtime content definitions; optional for older servers.
    #[serde(default)]
    pub rules: Option<Value>,
    /// Authoritative city accounting reports; never locally simulated.
    #[serde(default, deserialize_with = "deserialize_nullable_vec")]
    pub city_reports: Vec<Value>,
    /// Viewer-exposed route accounting.
    #[serde(default, deserialize_with = "deserialize_nullable_vec")]
    pub trade_routes: Vec<Value>,
    /// Players whose embassy information is available.
    #[serde(default, deserialize_with = "deserialize_nullable_vec")]
    pub embassy_player_ids: Vec<String>,
    /// Authoritative open/queued/closed planning state.
    #[serde(default)]
    pub planning_status: String,
}

impl ServerInterface {
    /// Resolves a legal action without substituting another entity or target.
    #[must_use]
    pub fn action(&self, route: &str, entity: &str, target: Option<&str>) -> Option<&LegalAction> {
        self.legal_actions.iter().find(|action| {
            action.route == route
                && action.entity_id == entity
                && target.is_none_or(|id| action.target_id == id)
        })
    }
}
