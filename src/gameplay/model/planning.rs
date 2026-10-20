//! Viewer-safe movement accounting, never local mutation of authoritative units.

use super::{GameState, MapPoint, UnitId};

/// Exact planning cost if known; unknown/explored cells do not invent turn labels.
#[must_use]
pub fn cost(state: &GameState, point: MapPoint) -> Option<u32> {
    if !state.authoritative {
        return Some(100);
    }
    let source = state
        .viewer
        .as_ref()?
        .tiles
        .iter()
        .find(|tile| tile.x == point.x && tile.y == point.y)?;
    if source.stale {
        return None;
    }
    u32::try_from(source.planning.as_ref()?.cost)
        .ok()
        .filter(|value| *value > 0)
}

/// One numbered endpoint of a turn's traversable path segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurnBoundary {
    /// Tile reached at the end of this travel segment.
    pub point: MapPoint,
    /// One-based turn relative to the current turn.
    pub turn: u32,
}

/// Accounts for current remaining movement, full future budgets and known costs.
/// Returns no misleading continuation past unknown costs or unaffordable tiles.
#[must_use]
pub fn turn_boundaries(state: &GameState, id: UnitId, path: &[MapPoint]) -> Vec<TurnBoundary> {
    let Some(unit) = state.units.iter().find(|unit| unit.id == id && unit.alive) else {
        return Vec::new();
    };
    let maximum = u32::try_from(unit.max_movement_points).unwrap_or(0);
    if maximum == 0 {
        return Vec::new();
    }
    let mut remaining = u32::try_from(unit.movement_points).unwrap_or(0);
    let mut turn = 1_u32;
    let mut boundaries = Vec::new();
    let mut previous = None;
    for point in path {
        let Some(cost) = cost(state, *point).filter(|cost| *cost <= maximum) else {
            break;
        };
        if cost > remaining {
            if let Some(endpoint) = previous {
                boundaries.push(TurnBoundary {
                    point: endpoint,
                    turn,
                });
            }
            turn = turn.saturating_add(1);
            remaining = maximum;
        }
        remaining = remaining.saturating_sub(cost);
        previous = Some(*point);
    }
    if let Some(point) = previous {
        if boundaries.last().is_none_or(|last| last.point != point) {
            boundaries.push(TurnBoundary { point, turn });
        }
    }
    boundaries
}

#[cfg(test)]
mod tests {
    use super::{TurnBoundary, turn_boundaries};
    use crate::gameplay::model::{GameState, MapPoint};
    use crate::gameplay::server::ServerProjection;
    #[test]
    fn turn_numbers_use_authoritative_costs_and_remaining_budget_not_step_count() {
        let projection:ServerProjection=serde_json::from_str(r#"{"revision":1,"turn":1,"tiles":[{"id":"a","x":0,"y":0,"terrain":"plains"},{"id":"b","x":1,"y":0,"terrain":"plains","planning":{"cost":2}},{"id":"c","x":2,"y":0,"terrain":"mountain","planning":{"cost":3}},{"id":"d","x":3,"y":0,"terrain":"plains","planning":{"cost":1}}],"units":[{"id":"u","owner_id":"human","tile_id":"a","movement":2,"max_movement":4}]}"#).expect("planning fixture");
        let state = GameState::from_server_projection(&projection, 1, 42);
        let id = state.selected_unit.expect("unit");
        assert_eq!(
            turn_boundaries(
                &state,
                id,
                &[
                    MapPoint::new(1, 0),
                    MapPoint::new(2, 0),
                    MapPoint::new(3, 0)
                ]
            ),
            vec![
                TurnBoundary {
                    point: MapPoint::new(1, 0),
                    turn: 1
                },
                TurnBoundary {
                    point: MapPoint::new(3, 0),
                    turn: 2
                }
            ],
            "terrain and remaining budget determine boundaries"
        );
        assert!(
            turn_boundaries(&state, id, &[MapPoint::new(10, 10)]).is_empty(),
            "unknown cells do not invent turn numbers"
        );
    }
}
