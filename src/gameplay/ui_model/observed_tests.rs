//! Cached display facts use no terrain lookup, local economy, files or service.
use super::observed;
use crate::gameplay::server::ServerProjection;
use serde_json::json;

#[test]
fn inspection_quotes_and_forecasts_use_actual_observed_values_not_base_rules() {
    let projection:ServerProjection=serde_json::from_value(json!({"revision":9_i64,"turn":3_u32,"tiles":[{"id":"a","terrain":"plains","stale":true,"river":true,"yields":{"food":0_i64,"production":17_i64,"gold":4_i64},"movement_cost":5_i64}],"rules":{"movement":{"terrains":[{"id":"plains","food":999_i64,"cost":999_i64}]}},"cities":[{"id":"c","rush_buy_quote":{"revision":9_i64,"status":"insufficient-gold","cost":45_i64},"production_forecast":{"revision":9_i64,"status":"estimated","rate":4_i64,"turns":2_u32,"assumption":"constant-current-economy"}}]})).expect("observed fixture");
    let tile = &projection.tiles[0];
    assert_eq!(
        observed::tile_value(Some(tile), "food"),
        "0",
        "real zero, not the base-rule 999"
    );
    assert_eq!(
        observed::tile_value(Some(tile), "production"),
        "17",
        "remembered effective value"
    );
    assert_eq!(
        observed::tile_value(Some(tile), "cost"),
        "5",
        "general cost not planning access"
    );
    assert_eq!(
        observed::tile_value(None, "food"),
        "—",
        "unavailable never becomes zero"
    );
    assert_eq!(
        observed::quote(&projection.cities[0], 9),
        "45",
        "quote available even when purchase unaffordable"
    );
    assert_eq!(
        observed::quote(&projection.cities[0], 10),
        "—",
        "stale price cannot be shown as current"
    );
    assert_eq!(
        observed::production(&projection.cities[0], 9),
        "2",
        "real effective forecast"
    );
}
