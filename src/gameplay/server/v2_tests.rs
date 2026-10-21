//! In-memory contracts matching server v2; no filesystem or service access.

use super::ServerProjection;
use crate::gameplay::model::{FogState, GameState, MapPoint, TileFeature, UnitOrder};
use serde_json::json;

/// A bounded viewer fixture deliberately omits all never-observed cells.
fn snapshot() -> ServerProjection {
    serde_json::from_value(json!({
        "revision":12,"turn":3,
        "contract":{"version":2,"features":{"retained_movement_orders":true,"governor":true}},
        "world":{"min_x":0,"min_y":0,"width":40,"height":30,"wrap_x":false,"wrap_y":false},
        "calendar":{"version":"ctp2-timescale-v1","turn":3,"year":-3800},
        "tiles":[{"id":"a","x":1,"y":1,"terrain":"TERRAIN_PLAINS","river":false,"good_id":"","ruin":true,"owner_id":"","yields":{"food":0,"production":4,"gold":2},"yield_context":"terrain","movement_cost":5},
                 {"id":"b","x":2,"y":1,"terrain":"TERRAIN_PLAINS","stale":true,"observed_turn":1}],
        "units":[{"id":"u","owner_id":"human","tile_id":"a","asleep":true}],
        "cities":[{"id":"c","owner_id":"human","tile_id":"a","art":{"age_id":"AGE_ANCIENT","capital":true},"governor":{"enabled":true,"priority":"growth"},"rush_buy_quote":{"revision":12,"status":"insufficient-gold","cost":45},"production_forecast":{"revision":12,"status":"stalled","rate":0,"assumption":"constant-current-economy"}}],
        "research":{"goal_id":"ADVANCE_FUTURE","forecast":{"revision":12,"status":"estimated","rate":2,"turns":3,"assumption":"constant-current-economy"}},
        "movement_orders":[{"unit_id":"u","player_id":"human","unit_ids":["u"],"tile_id":"a","target_x":20,"target_y":10,"command_id":"journey"}],
        "action_intents":[{"route":"move","entity_id":"u","fields":["unit_id","target_x","target_y"],"payload":{"unit_id":"u"}}],
        "notifications":[{"id":"n","kind":"ruin.explored","turn":3,"tile_id":"a","reward_kind":"gold","amount":150}],
        "statistics":[{"turn":2,"score":10,"city_count":1,"unit_count":1,"population":2}]
    })).expect("valid viewer fixture")
}

#[test]
fn v2_retains_public_private_and_optional_contracts() {
    let projection = snapshot();
    assert!(projection.interface.supports("governor"), "explicit feature retained");
    assert!(!projection.interface.supports("policy_optimizer"), "missing is not authorization");
    assert_eq!(projection.interface.research.as_ref().map(|research| research.goal_id.as_str()), Some("ADVANCE_FUTURE"), "goal is not immediate research");
    assert_eq!(projection.interface.notifications[0].amount, 150, "actual reward retained");
    assert_eq!(projection.interface.statistics[0].score, 10, "actual own history retained");
    assert_eq!(projection.interface.movement_orders[0].target_x, 20, "server destination retained");
    assert_eq!(projection.interface.action_intents[0].route, "move", "shape permission retained");
    assert_eq!(projection.interface.calendar.as_ref().map(|date| date.year), Some(-3800), "date is authoritative");
    assert_eq!(projection.interface.calendar.as_ref().map(super::v2::Calendar::label).as_deref(), Some("3800BC"), "original date notation");
    assert_eq!(projection.interface.research.as_ref().and_then(|research| research.forecast.as_ref()).map(|forecast| forecast.label(12)).as_deref(), Some("3"), "finite authoritative estimate");
    assert_eq!(projection.cities[0].production_forecast.as_ref().map(|forecast| forecast.label(12)).as_deref(), Some("—"), "stalled remains unavailable");
    assert_eq!(projection.cities[0].rush_buy_quote.as_ref().and_then(|quote| quote.cost), Some(45), "quote is separate from commands");
    assert_eq!(projection.cities[0].production_forecast.as_ref().and_then(|forecast| forecast.turns), None, "stalled is not zero turns");
    assert!(projection.cities[0].art.as_ref().is_some_and(|art| art.capital), "observed capital retained");
    assert!(projection.cities[0].governor.as_ref().is_some_and(|governor| governor.enabled), "own governor retained");
}

#[test]
fn v2_unknown_tile_metadata_is_distinct_from_explicit_negatives() {
    let projection = snapshot();
    assert_eq!(projection.tiles[0].river, Some(false), "known negative");
    assert_eq!(projection.tiles[1].river, None, "legacy unknown");
    assert_eq!(projection.tiles[0].good_id.as_deref(), Some(""), "known no deposit");
    assert_eq!(projection.tiles[1].good_id, None, "unknown deposit");
    assert_eq!(projection.tiles[0].ruin, Some(true), "observed ruin");
    assert_eq!(projection.tiles[0].yields.as_ref().map(|yields| yields.food), Some(0), "known zero");
    assert_eq!(projection.tiles[1].yields, None, "unknown yield");
}

#[test]
fn v2_bounds_do_not_grow_with_exploration_or_expose_hidden_cells() {
    let projection = snapshot();
    let state = GameState::from_server_projection(&projection, 1, 1);
    assert_eq!((state.width, state.height), (40, 30), "public world bounds independent of sight");
    assert!(state.tile(MapPoint::new(39,29)).is_some_and(|tile| tile.fog == FogState::Hidden), "public bounds reveal no terrain");
    assert!(state.tile(MapPoint::new(1,1)).is_some_and(|tile| tile.feature == Some(TileFeature::Ruin)), "ruin comes only from observation");
    assert!(matches!(state.units[0].order, UnitOrder::Sentinel), "sleep survives replacement");
}
