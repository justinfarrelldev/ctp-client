//! Verified typed government policy/rate parameters and original source controls.
//!
//! visibility.go advertises sample scalar values, while government.go accepts
//! the documented bounded parameter space. Only these exact registered schemas
//! are parameterized; production/targeted actions retain literal payload checks.

use super::{ManagementUi, ManagerCommand, ManagerWidget};
use crate::gameplay::assets::layout::LayoutNode;
use crate::gameplay::model::GameState;
use crate::gameplay::server::ServerProjection;
use crate::gameplay::ui_model::MenuRect;
use serde_json::{Value, json};

/// Native source descriptor composition without SDL ownership or live mutations.
mod controls;

/// Injects the original atomic policy control's actual child descriptors.
pub(super) fn append(
    ui: &ManagementUi,
    state: &GameState,
    node: &LayoutNode,
    path: &str,
    rect: MenuRect,
    widgets: &mut Vec<ManagerWidget>,
) -> bool {
    controls::append(ui, state, node, path, rect, widgets)
}

/// Original cause/effect control semantics, independent of localized captions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Control {
    /// Source index minus two equals current unitless rations.
    Rations,
    /// Source two minus index equals current unitless workday.
    Workday,
    /// Source index minus two equals current unitless wages.
    Wages,
    /// Whole public-works percentage.
    PublicWorks,
    /// Whole science percentage, bounded by current government when disclosed.
    Science,
}

/// Source range and receipt-owned current value; no game state is simulated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Spec {
    /// Receipt that supplied the current values; stale gestures cannot be rebased.
    pub revision: i64,
    /// Exact supported original source control.
    pub control: Control,
    /// Lowest source index, not necessarily its unitless policy level.
    pub minimum: i32,
    /// Highest source index or actual percent.
    pub maximum: i32,
    /// Current source index; out-of-range server levels remain explicit.
    pub current: i32,
    /// Original spinner step or one quantized slider position.
    pub step: i32,
}

/// Only actual source control paths acquire a mutable policy meaning.
fn control(path: &str) -> Option<Control> {
    match path {
        "TabGroup.Tab1.TabPanel.Food.Modify.Primary.Slider" => Some(Control::Rations),
        "TabGroup.Tab1.TabPanel.Production.Modify.Primary.Slider" => Some(Control::Workday),
        "TabGroup.Tab1.TabPanel.Commerce.Modify.Primary.Slider" => Some(Control::Wages),
        "TabGroup.Tab1.TabPanel.Production.Modify.Secondary.Spinner" => Some(Control::PublicWorks),
        "TabGroup.Tab1.TabPanel.Commerce.Modify.Secondary.Spinner" => Some(Control::Science),
        _ => None,
    }
}

/// Server nil social-policy pointers mean level zero; nil rate pointers use the
/// government.go/lifecycle.go defaults. Malformed present values never default.
fn number(facts: &Value, name: &str, default: i64) -> Option<i64> {
    facts.get(name).map_or(Some(default), Value::as_i64)
}

/// Current range/index from verified source callbacks and public current facts.
pub(super) fn spec(state: &GameState, path: &str) -> Option<Spec> {
    let kind = control(path)?;
    let viewer = state.viewer.as_ref()?;
    let facts = &viewer.diplomacy.as_ref()?.facts;
    let (value, maximum, step) = match kind {
        Control::Rations => (
            number(facts, "rations", 0_i64)?.checked_add(2_i64)?,
            4_i64,
            1_i32,
        ),
        Control::Workday => (
            2_i64.checked_sub(number(facts, "workday", 0_i64)?)?,
            4_i64,
            1_i32,
        ),
        Control::Wages => (
            number(facts, "wages", 0_i64)?.checked_add(2_i64)?,
            4_i64,
            1_i32,
        ),
        Control::PublicWorks => (
            number(facts, "public_works_percent", 0_i64)?,
            100_i64,
            10_i32,
        ),
        Control::Science => (
            number(facts, "science_percent", 100_i64)?,
            science_maximum(viewer, facts)?,
            10_i32,
        ),
    };
    Some(Spec {
        revision: viewer.revision,
        control: kind,
        minimum: 0_i32,
        maximum: i32::try_from(maximum).ok()?,
        current: i32::try_from(value).ok()?,
        step,
    })
}

/// Known government caps constrain input but never overwrite a received policy.
/// With no matching disclosed government, the registered rate domain is 0..100.
fn science_maximum(viewer: &ServerProjection, facts: &Value) -> Option<i64> {
    let Some(id) = facts["government_id"].as_str().filter(|id| !id.is_empty()) else {
        return Some(100_i64);
    };
    let government = viewer
        .interface
        .rules
        .as_ref()
        .and_then(|rules| rules["governments"].as_array())
        .and_then(|governments| {
            governments
                .iter()
                .find(|government| government["id"].as_str() == Some(id))
        });
    government
        .and_then(|record| record.get("max_science_percent"))
        .map_or(Some(100_i64), |maximum| {
            maximum
                .as_i64()
                .map(|maximum| maximum.clamp(0_i64, 100_i64))
        })
}

/// Forms one explicit user policy request, never an optimizer/counterfactual probe.
pub(super) fn command(state: &GameState, path: &str, value: i32) -> Option<ManagerCommand> {
    let spec = spec(state, path)?;
    if !(spec.minimum..=spec.maximum).contains(&value) || value == spec.current {
        return None;
    }
    let viewer = state.viewer.as_ref()?;
    let facts = &viewer.diplomacy.as_ref()?.facts;
    let (route, payload) = match spec.control {
        Control::Science => ("set-science-rate", json!({"science_percent":value})),
        Control::PublicWorks => (
            "set-public-works-rate",
            json!({"public_works_percent":value}),
        ),
        Control::Rations | Control::Workday | Control::Wages => {
            let mut workday = number(facts, "workday", 0_i64)?;
            let mut wages = number(facts, "wages", 0_i64)?;
            let mut rations = number(facts, "rations", 0_i64)?;
            match spec.control {
                Control::Rations => rations = i64::from(value).checked_sub(2_i64)?,
                Control::Workday => workday = 2_i64.checked_sub(i64::from(value))?,
                Control::Wages => wages = i64::from(value).checked_sub(2_i64)?,
                Control::Science | Control::PublicWorks => return None,
            }
            (
                "set-social-policy",
                json!({"workday":workday,"wages":wages,"rations":rations}),
            )
        }
    };
    let request = ManagerCommand {
        revision: viewer.revision,
        route: route.to_owned(),
        payload,
    };
    request.valid_for(state).then_some(request)
}

/// Returns None for routes whose exact legal payload must remain unchanged.
pub(super) fn validate(viewer: &ServerProjection, route: &str, payload: &Value) -> Option<bool> {
    let (fields, minimum, maximum): (&[&str], i64, i64) = match route {
        "set-science-rate" => (&["science_percent"], 0_i64, 100_i64),
        "set-public-works-rate" => (&["public_works_percent"], 0_i64, 100_i64),
        "set-social-policy" => (&["workday", "wages", "rations"], -10_i64, 10_i64),
        _ => return None,
    };
    let shape = |candidate: &Value| {
        candidate.as_object().is_some_and(|values| {
            values.len() == fields.len()
                && fields.iter().all(|field| {
                    values
                        .get(*field)
                        .and_then(Value::as_i64)
                        .is_some_and(|value| (minimum..=maximum).contains(&value))
                })
        })
    };
    let authorized = viewer.interface.legal_actions.iter().any(|action| {
        action.http_supported()
            && action.route == route
            && action.fields.len() == fields.len()
            && shape(&action.payload)
            && fields
                .iter()
                .all(|field| action.fields.iter().any(|advertised| advertised == field))
    });
    Some(shape(payload) && authorized)
}
