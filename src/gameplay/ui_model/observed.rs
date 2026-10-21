//! Cached viewer facts for display, never a local economy or inspection request.
use crate::gameplay::model::GameState;
use crate::gameplay::server::{ServerCityProjection, ServerTileProjection};

/// Effective tile values retain legacy unavailable versus explicit zero.
#[must_use]
pub fn tile_value(tile: Option<&ServerTileProjection>, field: &str) -> String {
    let value = tile.and_then(|tile| match field {
        "food" => tile.yields.as_ref().map(|yields| yields.food),
        "production" => tile.yields.as_ref().map(|yields| yields.production),
        "gold" => tile.yields.as_ref().map(|yields| yields.gold),
        "cost" => tile.movement_cost,
        _ => None,
    });
    value.map_or_else(|| "—".to_owned(), |value| value.to_string())
}
/// A known, current owned price is distinct from permission to buy.
#[must_use]
pub fn quote(city: &ServerCityProjection, revision: i64) -> String {
    city.rush_buy_quote
        .as_ref()
        .filter(|quote| {
            quote.revision == revision
                && matches!(
                    quote.status.as_str(),
                    "available" | "insufficient-gold" | "completed"
                )
        })
        .and_then(|quote| quote.cost)
        .map_or_else(|| "—".to_owned(), |cost| cost.to_string())
}
/// Effective current-economy head estimate; stalled/continuous remains unavailable.
#[must_use]
pub fn production(city: &ServerCityProjection, revision: i64) -> String {
    city.production_forecast
        .as_ref()
        .map_or_else(|| "—".to_owned(), |forecast| forecast.label(revision))
}
/// Do not invent an initial date for old/synthetic pins without a calendar.
#[must_use]
pub fn date(state: &GameState) -> String {
    state
        .viewer
        .as_ref()
        .and_then(|viewer| viewer.interface.calendar.as_ref())
        .filter(|date| date.turn == state.turn)
        .map_or_else(
            || format!("Turn {}", state.turn),
            crate::gameplay::server::v2::Calendar::label,
        )
}
