//! Original Build queue-head purchase controls, backed only by received quotes.
use super::{ManagerCommand, ManagerIntent, ManagerKind, ManagerView, data};
use crate::gameplay::{model::GameState, server::{ServerCityProjection, v2::Quote}};
use serde_json::json;

/*
 * Original released-source provenance for this control group:
 * - ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::{UpdateButtons,RushBuy}
 *   requires a selected queue index0, a valid GetOvertimeCost, adequate treasury
 *   and not AlreadyBoughtFront. Future/no selection uses ---; an unaffordable
 *   selected head still displays its real price. RushBuy repeats head checks.
 * - The same file's constructor wires RushBuyCallback, while ListCallback and
 *   QueueListCallback call UpdateButtons after exact choice/entry selection.
 * - ctp2/ctp2_data/english/uidata/layouts/editqueue.ldl::BuildEditorWindow:
 *   RushBuyButton (385,118,width104), RushBuyCost (495,118,102x22) and
 *   QueueGroup.QueueList supply styles/relative bounds/selection controls.
 * Client integration replaces private CityData pricing/mutation with a current
 * owner-only quote and exact advertised city command. An entry ID, not its
 * possibly repeated content ID, proves explicit head selection. Missing price
 * is unavailable (—), not a free purchase; source not-applicable uses ---.
 * Enqueue/receipt cannot spend or complete production locally. No scaling edits.
 */

/// Exact retained queue-head identity; a repeated content row cannot become head.
fn selected_head<'state>(state: &'state GameState, view: &ManagerView) -> Option<(&'state ServerCityProjection, &'state str)> {
    let selected = view.selections.get("queue")?;
    let city = data::city(state, view)?;
    let items = city.facts["queue"].as_array()?;
    let head = items.first()?;
    let key = |entry: &'state serde_json::Value| entry["entry_id"].as_str().or_else(|| entry["id"].as_str());
    if key(head) != Some(selected.as_str()) || items.iter().filter(|entry| key(entry) == Some(selected.as_str())).count() != 1_usize {
        return None;
    }
    Some((city, head["id"].as_str().filter(|id| !id.is_empty())?))
}

/// Revision binds price to the same complete city/head facts, never a recalculation.
fn quote<'city>(state: &GameState, city: &'city ServerCityProjection) -> Option<&'city Quote> {
    let revision = state.viewer.as_ref()?.revision;
    city.rush_buy_quote.as_ref().filter(|quote| quote.revision == revision)
}

/// Source price box follows selection even when the city cannot afford the head.
pub(super) fn text(state: &GameState, view: &ManagerView, path: &str) -> Option<String> {
    if view.kind != ManagerKind::Build || path != "RushBuyCost" {
        return None;
    }
    let Some((city, _content)) = selected_head(state, view) else {
        return Some("---".to_owned());
    };
    let Some(quote) = quote(state, city) else {
        return Some("—".to_owned());
    };
    Some(match quote.status.as_str() {
        "available" | "insufficient-gold" => quote.cost.filter(|cost| *cost >= 0_i64)
            .map_or_else(|| "—".to_owned(), |cost| cost.to_string()),
        "completed" | "empty-queue" | "not-rushable" => "---".to_owned(),
        _ => "—".to_owned(),
    })
}

/// A selected head/quote does not grant authority to choose a different city/order.
pub(super) fn binding(state: &GameState, view: &ManagerView, path: &str) -> Option<(Option<ManagerIntent>, Option<String>)> {
    if view.kind != ManagerKind::Build || path != "RushBuyButton" {
        return None;
    }
    let Some((city, content)) = selected_head(state, view) else {
        return Some(disabled("Select the exact current owned-city queue-head entry to rush"));
    };
    let Some(viewer) = state.viewer.as_ref() else {
        return Some(disabled("No current authoritative city snapshot"));
    };
    let affordable = quote(state, city).is_some_and(|quote| {
        quote.status == "available" && quote.cost.zip(viewer.diplomacy.as_ref())
            .is_some_and(|(cost, nation)| cost >= 0_i64 && cost <= i64::from(nation.gold))
    });
    if !affordable {
        return Some(disabled("A current available head quote and sufficient known treasury are required"));
    }
    let payload = json!({"city_id":city.id});
    let mut actions = viewer.interface.legal_actions.iter().filter(|action| {
        action.route == "rush-buy" && action.entity_id == city.id && action.payload == payload
            && (action.target_id.is_empty() || action.target_id == content)
    });
    let action = actions.next();
    let command = actions.next().is_none().then_some(action).flatten()
        .and_then(|action| ManagerCommand::advertised(state, action));
    Some(command.map_or_else(
        || disabled("No single exact current authorization for this selected owned-city head"),
        |command| (Some(ManagerIntent::Submit(command)), None),
    ))
}

/// Unavailable controls explain the missing proof instead of becoming silent no-ops.
fn disabled(reason: &str) -> (Option<ManagerIntent>, Option<String>) {
    (None, Some(reason.to_owned()))
}
