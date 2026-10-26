//! Source per-item durations from received costs and effective production rates.
use super::super::{ManagerView, data};
use crate::gameplay::{model::GameState, server::ServerCityProjection};
use serde_json::Value;

/// Fileless regression cases; never contact the game server or write files.
#[cfg(test)]
#[path = "estimate_tests.rs"]
mod tests;

/* Released ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::UpdateChoiceLists
 * passes full ShieldCost/ProductionCost to CityData::HowMuchLonger(production).
 * EditQueue::UpdateQueueList -> citywindow.cpp::CityWindow::PopulateQueueList
 * uses HowMuchLonger() for the head (remaining shields), but full bn->m_cost
 * for EACH future item, not a cumulative completion timeline. CityData.cpp::
 * HowMuchLonger overloads round up division by net projected production.
 * ctp2/ctp2_data/english/uidata/layouts/editqueue.ldl::{eq_UnitItem.box.Time,
 * eq_BuildingWonderItem.box.Time,eq_QueueListItem.StaticThing.Turns} supply the
 * unchanged fonts, alignment and row positions. Source unknown is ASCII ---.
 *
 * Server integration: engine/forecasts.go::Inspector.Production supplies the
 * owner's effective rate AFTER economy/injunction/public-works calculations.
 * We only format full received cost / that current constant-current-economy
 * rate, never recalculate yields, support, policy, growth or future execution.
 * Head uses the exact received finite turns (including completed0, unlike the
 * original minimum1); future rows are independent full-cost durations, not
 * promises about eligibility/completion/overflow. Missing/foreign/stale/rate0,
 * continuous conversions and unknown assumptions cannot invent an estimate.
 * Custom's explicit blank Time/Turns is handled before calling this helper.
 */
/// Supplies only the two source production-duration columns.
pub(in crate::gameplay::ui_model::management) fn text(state: &GameState, view: &ManagerView, path: &str, id: &str, name: &str) -> Option<String> {
    if !matches!(name, "Time" | "Turns") { return None; }
    Some(turns(state, view, path, id).map_or_else(|| "---".to_owned(), |turns| turns.to_string()))
}

/// Exact head forecast or full-cost duration, never a content-ID head heuristic.
fn turns(state: &GameState, view: &ManagerView, path: &str, id: &str) -> Option<i64> {
    let viewer = state.viewer.as_ref()?;
    let city = data::city(state, view)?;
    let forecast = city.production_forecast.as_ref().filter(|forecast| forecast.revision == viewer.revision)?;
    if path == "QueueGroup.QueueList" {
        let entries = city.facts["queue"].as_array()?;
        let mut matching = entries.iter().enumerate().filter(|&(_index, item)| item["entry_id"].as_str().or_else(|| item["id"].as_str()) == Some(id));
        let (index, item) = matching.next()?;
        if matching.next().is_some() || !finite(item) { return None; }
        if index == 0_usize {
            return matches!(forecast.status.as_str(), "estimated" | "completed").then_some(forecast.turns).flatten().map(i64::from);
        }
        duration(item["cost"].as_i64()?, rate(forecast)?)
    } else {
        let item = choice(viewer, city, id)?;
        if !finite(item) { return None; }
        duration(item["cost"].as_i64()?, rate(forecast)?)
    }
}

/// A positive current effective rate is usable only under the documented assumption.
fn rate(forecast: &crate::gameplay::server::v2::Forecast) -> Option<i64> {
    (matches!(forecast.status.as_str(), "estimated" | "completed") && forecast.assumption == "constant-current-economy").then_some(forecast.rate).flatten().filter(|rate| *rate > 0_i64)
}

/// Discrete items only; conversions are never finite-duration production.
fn finite(item: &Value) -> bool {
    matches!(item["kind"].as_str(), Some("unit" | "building" | "wonder"))
}

/// Checked integer ceiling avoids floating-point error and addition overflow.
fn duration(cost: i64, rate: i64) -> Option<i64> {
    if cost <= 0_i64 { return None; }
    cost.checked_div(rate)?.checked_add(i64::from(cost.checked_rem(rate)? != 0_i64))
}

/// Received append choices or exact legacy payloads, never installed catalog prices.
fn choice<'view>(viewer: &'view crate::gameplay::server::ServerProjection, city: &'view ServerCityProjection, id: &str) -> Option<&'view Value> {
    if viewer.interface.supports("production_options") {
        return crate::gameplay::server::production::items(viewer, city)?.iter().find(|item| item["id"].as_str() == Some(id));
    }
    let mut items = viewer.interface.legal_actions.iter().filter(|action| action.route == "queue-production" && action.entity_id == city.id && action.target_id == id && action.payload["city_id"].as_str() == Some(city.id.as_str()) && action.payload["item"]["id"].as_str() == Some(id)).map(|action| &action.payload["item"]);
    let item = items.next()?;
    items.all(|other| other == item).then_some(item)
}
