//! Source Trade columns and local filters; orders retain complete current payloads.
use super::{
    ManagementUi, ManagerCommand, ManagerIntent, ManagerKind, ManagerView, ManagerWidget, data,
};
use crate::gameplay::{
    assets::layout::LayoutNode, model::GameState, server::interface::LegalAction,
    ui_model::MenuRect,
};
use alloc::collections::BTreeMap;
use serde_json::{Value, json};

/// Exact source lists keep candidate payloads separate from existing route IDs.
pub(super) const MARKET: &str = "TradeTabs.Market.TabPanel.List";
/// Existing outbound routes, not offers or inbound sales.
pub(super) const SUMMARY: &str = "TradeTabs.Summary.TabPanel.List";
/// Independent own-destination view; its last column is resource bonus, not cost.
pub(super) const IMPORT: &str = "TradeTabs.Import.TabPanel.List";
/// Source slider has five ranked destinations per good; quotes are not projected.
const SLIDER: &str = "TradeTabs.Market.TabPanel.Filters.CitiesSlider";
/// Do not rank offers by a duplicated economy or an unrelated route's revenue.
const QUOTES_UNAVAILABLE: &str = "Ranked destinations require current per-good price/transport quotes; this server snapshot does not expose them";

/// Payload identity is collision-free even for multiple sources sharing a target.
pub(super) fn identity(action: &LegalAction) -> String {
    json!([action.entity_id, action.target_id, action.payload]).to_string()
}
/// Only current own-source authorizations are offers. Filters never authorize orders.
fn market<'state>(state: &'state GameState, view: &ManagerView) -> Vec<&'state LegalAction> {
    let own = data::owned_city_ids(state);
    state.viewer.as_ref().map_or_else(Vec::new, |viewer| {
        viewer
            .interface
            .legal_actions
            .iter()
            .filter(|action| {
                action.route == "create-trade-route"
                    && action.http_supported()
                    && own.contains(&action.entity_id)
                    && eligible(state, view, &own, &action.target_id)
            })
            .collect()
    })
}
/// Own/friendly/all are presentation categories over already projected identities.
fn eligible(state: &GameState, view: &ManagerView, own: &[String], destination: &str) -> bool {
    let filter = view
        .selections
        .get("trade-filter")
        .map_or("own", String::as_str);
    if filter == "all" || own.iter().any(|city| city == destination) {
        return true;
    }
    if filter != "friendly" {
        return false;
    }
    let Some(viewer) = state.viewer.as_ref() else {
        return false;
    };
    let owner = viewer
        .cities
        .iter()
        .find(|city| city.id == destination && !city.stale)
        .map(|city| city.owner_id.as_str());
    viewer
        .diplomacy
        .as_ref()
        .and_then(|diplomacy| diplomacy.facts["relations"].as_array())
        .is_some_and(|relations| {
            relations.iter().any(|relation| {
                owner.is_some_and(|known| Some(known) == relation["target_player_id"].as_str())
                    && matches!(relation["treaty"].as_str(), Some("peace" | "alliance"))
            })
        })
}
/// Only a fresh owned source enters the original outbound summary.
fn flows<'state>(state: &'state GameState, endpoint: &str) -> Vec<&'state Value> {
    let own = data::owned_city_ids(state);
    state.viewer.as_ref().map_or_else(Vec::new, |viewer| {
        viewer
            .interface
            .trade_routes
            .iter()
            .filter(|route| {
                route[endpoint]
                    .as_str()
                    .is_some_and(|id| own.iter().any(|city| city == id))
            })
            .collect()
    })
}
/// Returns exact IDs; duplicate identical authorizations are one logical offer.
pub(super) fn rows(state: &GameState, view: &ManagerView, path: &str) -> Vec<(String, String)> {
    if path == MARKET {
        return market(state, view)
            .into_iter()
            .map(|action| (identity(action), String::new()))
            .collect::<BTreeMap<_, _>>()
            .into_iter()
            .collect();
    }
    if matches!(path, SUMMARY | IMPORT) {
        return flows(
            state,
            if path == SUMMARY {
                "source_city_id"
            } else {
                "destination_city_id"
            },
        )
        .into_iter()
        .filter_map(|route| Some((route["id"].as_str()?.to_owned(), String::new())))
        .collect();
    }
    Vec::new()
}
/// Columns use received names, not IDs reformatted into invented geography.
fn city_name(state: &GameState, id: Option<&str>) -> String {
    state
        .viewer
        .as_ref()
        .and_then(|viewer| {
            viewer
                .cities
                .iter()
                .find(|city| Some(city.id.as_str()) == id)
        })
        .map_or_else(|| "—".to_owned(), data::city_name)
}
/// Finds one selected offer by its full current payload, never the first destination.
fn offer<'state>(
    state: &'state GameState,
    view: &ManagerView,
    id: &str,
) -> Option<&'state LegalAction> {
    let encoded: Value = serde_json::from_str(id).ok()?;
    let parts = encoded.as_array().filter(|parts| parts.len() == 3_usize)?;
    let source = parts.first()?.as_str()?;
    let target = parts.get(1)?.as_str()?;
    let payload = parts.get(2)?;
    let own = data::owned_city_ids(state);
    if !own.iter().any(|city| city == source) || !eligible(state, view, &own, target) {
        return None;
    }
    state
        .viewer
        .as_ref()?
        .interface
        .legal_actions
        .iter()
        .find(|action| {
            action.route == "create-trade-route"
                && action.entity_id == source
                && action.target_id == target
                && &action.payload == payload
        })
        .filter(|action| identity(action) == id)
}
/// Existing route facts remain distinct from market requests and local filter state.
fn route<'state>(state: &'state GameState, id: &str, path: &str) -> Option<&'state Value> {
    let record = state
        .viewer
        .as_ref()?
        .interface
        .trade_routes
        .iter()
        .find(|record| record["id"].as_str() == Some(id))?;
    let endpoint = if path == IMPORT {
        "destination_city_id"
    } else {
        "source_city_id"
    };
    record[endpoint]
        .as_str()
        .filter(|source| {
            data::owned_city_ids(state)
                .iter()
                .any(|city| city == source)
        })
        .map(|_source| record)
}
/// No market price/good/cost is inferred from an existing route or base rules.
pub(super) fn value(
    ui: &ManagementUi,
    state: &GameState,
    path: &str,
    id: &str,
    name: &str,
) -> String {
    if path == MARKET {
        // Row IDs were generated from this snapshot above. Display only projected
        // names; authorization separately matches the full fresh payload in offer.
        return match name {
            "City" | "ToCity" => serde_json::from_str::<Value>(id).ok().map_or_else(
                || "—".to_owned(),
                |parts| {
                    city_name(
                        state,
                        parts
                            .get(usize::from(name == "ToCity"))
                            .and_then(Value::as_str),
                    )
                },
            ),
            "GoodIcon" => String::new(),
            _ => "—".to_owned(),
        };
    }
    let Some(record) = route(state, id, path) else {
        return "—".to_owned();
    };
    match name {
        "City" => city_name(state, record["source_city_id"].as_str()),
        "ToCity" => city_name(state, record["destination_city_id"].as_str()),
        "Good" => record["good_id"]
            .as_str()
            .filter(|good| !good.is_empty())
            .map_or_else(|| "—".to_owned(), |good| data::label(ui, good)),
        "Price" => record["gold"]
            .as_i64()
            .map_or_else(|| "—".to_owned(), |gold| gold.to_string()),
        // Original Import shows resource Food/Production/Gold, not route cost.
        // Current RuntimeRules does not project resource definitions/bonuses.
        "Caravans" if path == IMPORT => "—".to_owned(),
        "Caravans" => record
            .get("transport_cost")
            .map_or_else(
                || v2(state).then_some(0_i64),
                |cost| cost.as_i64().filter(|cost| *cost >= 0),
            )
            .map_or_else(|| "—".to_owned(), |cost| cost.to_string()),
        "GoodIcon" | "Piracy" => String::new(),
        _ => "—".to_owned(),
    }
}
/// Only explicitly received goods/piracy select source images, never hidden tiles.
pub(super) fn image(
    ui: &ManagementUi,
    state: &GameState,
    path: &str,
    id: &str,
    name: &str,
) -> Option<String> {
    if !matches!(path, SUMMARY | IMPORT) {
        return None;
    }
    let record = route(state, id, path)?;
    match name {
        "GoodIcon" => ui
            .library
            .catalog
            .art
            .get(record["good_id"].as_str()?)
            .and_then(|art| art.icon.clone()),
        "Piracy" => record["pirating_army_id"]
            .as_str()
            .filter(|army| !army.is_empty())
            .map(|_army| "upic02.tga".to_owned()),
        _ => None,
    }
}
/// The observed v2 Go route integer's omission is zero, not a legacy default.
fn v2(state: &GameState) -> bool {
    state
        .viewer
        .as_ref()
        .and_then(|viewer| viewer.interface.contract.as_ref())
        .is_some_and(|contract| contract.version == 2)
}
/// Original control value is local SetNumCities(1); quote absence is not zero.
pub(super) fn text(view: &ManagerView, path: &str) -> Option<String> {
    (view.kind == ManagerKind::Trade && path == "TradeTabs.Market.TabPanel.Filters.NumCities")
        .then(|| "1".to_owned())
}
/// Source radio selection is independent of offer/route identity.
pub(super) fn selected(view: &ManagerView, path: &str) -> bool {
    view.kind == ManagerKind::Trade
        && filter(path).is_some_and(|filter| {
            view.selections
                .get("trade-filter")
                .map_or("own", String::as_str)
                == filter
        })
}
/// Recognizes only original checkbox paths, not forged selection-like strings.
fn filter(path: &str) -> Option<&'static str> {
    match path {
        "TradeTabs.Market.TabPanel.Filters.OwnButton" => Some("own"),
        "TradeTabs.Market.TabPanel.Filters.FriendlyButton" => Some("friendly"),
        "TradeTabs.Market.TabPanel.Filters.AllButton" => Some("all"),
        _ => None,
    }
}
/// Exact command and pure local filters share descriptors with native interactions.
pub(super) fn binding(
    state: &GameState,
    view: &ManagerView,
    path: &str,
) -> Option<(Option<ManagerIntent>, Option<String>)> {
    if view.kind != ManagerKind::Trade {
        return None;
    }
    if let Some(filter) = filter(path) {
        return Some((
            Some(ManagerIntent::Select(
                "trade-filter".to_owned(),
                filter.to_owned(),
            )),
            None,
        ));
    }
    match path {
        "TradeTabs.Market.TabPanel.CreateRouteButton"=>Some(view.selections.get("trade-market").and_then(|id|offer(state,view,id)).and_then(|action|ManagerCommand::advertised(state,action)).map_or_else(||(None,Some("Select one exact currently authorized source/destination offer".to_owned())),|command|(Some(ManagerIntent::Submit(command)),None))),
        "TradeTabs.Summary.TabPanel.BreakRouteButton"|"TradeTabs.Import.TabPanel.BreakRouteButton"=>Some((None,Some("The server has no registered trade-route cancellation operation; no different command is substituted".to_owned()))),
        SLIDER=>Some((None,Some(QUOTES_UNAVAILABLE.to_owned()))),
        _=>None,
    }
}
/// Preserve the actual source thumb/track even while ranked quotes are unavailable.
pub(super) fn slider(
    view: &ManagerView,
    node: &LayoutNode,
    path: &str,
    rect: MenuRect,
    widgets: &mut Vec<ManagerWidget>,
) -> bool {
    if view.kind != ManagerKind::Trade || path != SLIDER {
        return false;
    }
    let mut track = LayoutNode::default();
    track
        .fields
        .insert("c3-slider-track".to_owned(), "true".to_owned());
    track.fields.insert(
        "pattern".to_owned(),
        node.field("pattern").unwrap_or("uptg20e.tga").to_owned(),
    );
    for (suffix, mut style, part) in [
        ("track", track, rect),
        (
            "thumb",
            node.child("thumb").cloned().unwrap_or_default(),
            MenuRect {
                width: 9,
                height: 20,
                ..rect
            },
        ),
    ] {
        style.children.clear();
        widgets.push(ManagerWidget {
            id: format!("{path}/{suffix}"),
            label: String::new(),
            rect: part,
            node: style,
            intent: None,
            unavailable: Some(QUOTES_UNAVAILABLE.to_owned()),
            selected: false,
        });
    }
    true
}
/// Receipt reconciliation never reuses another row with the same destination.
pub(super) fn reconcile(view: &mut ManagerView, state: &GameState) {
    if view
        .selections
        .get("trade-market")
        .is_some_and(|id| offer(state, view, id).is_none())
    {
        view.selections.remove("trade-market");
    }
    if view
        .selections
        .get("trade-route")
        .is_some_and(|id| route(state, id, SUMMARY).is_none())
    {
        view.selections.remove("trade-route");
    }
    if view
        .selections
        .get("trade-import")
        .is_some_and(|id| route(state, id, IMPORT).is_none())
    {
        view.selections.remove("trade-import");
    }
}
