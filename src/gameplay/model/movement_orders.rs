//! Selected-unit movement order helpers.

extern crate alloc;

use alloc::vec::Vec;

use super::generation::straight_line_path;
use super::{FogState, GameState, GameplayEvent, MapPoint, UnitOrder};

/// Orders or immediately moves the selected unit to a target tile.
pub(super) fn order_selected_to(state: &mut GameState, target: MapPoint) -> Vec<GameplayEvent> {
    if !state.in_bounds(target) || !is_land_passable_for_order(state, target) {
        return Vec::new();
    }
    let Some(start) = state.selected_unit().map(|unit| unit.position) else {
        return Vec::new();
    };
    let path = straight_line_path(start, target, state.width, state.height);
    order_selected_along_path(state, path)
}

/// Orders or immediately moves the selected unit along an already-computed path.
pub(super) fn order_selected_along_path(
    state: &mut GameState,
    path: Vec<MapPoint>,
) -> Vec<GameplayEvent> {
    let mut events = Vec::new();
    let Some(unit_id) = state.selected_unit else {
        return events;
    };
    let Some(start) = state.selected_unit().map(|unit| unit.position) else {
        return events;
    };
    if path.is_empty() || !path_is_valid_from(state, start, &path) {
        return events;
    }
    let Some(unit) = state
        .units
        .iter_mut()
        .find(|unit| unit.id == unit_id && unit.alive)
    else {
        return events;
    };
    unit.order = UnitOrder::MoveTo { path };
    events.extend(state.process_unit_path(unit_id));
    state.refresh_fog();
    events
}

/// Returns whether a precomputed path is adjacent, in-bounds, and land-passable.
fn path_is_valid_from(state: &GameState, start: MapPoint, path: &[MapPoint]) -> bool {
    let mut previous = start;
    for point in path {
        if previous.tile_distance(*point) != 1_u16
            || !state.in_bounds(*point)
            || !is_land_passable_for_order(state, *point)
        {
            return false;
        }
        previous = *point;
    }
    true
}

/// Returns whether a land movement order may enter a known point.
fn is_land_passable_for_order(state: &GameState, point: MapPoint) -> bool {
    state
        .tile(point)
        .is_some_and(|tile| tile.fog != FogState::Hidden && tile.terrain.passable_by_land())
}
