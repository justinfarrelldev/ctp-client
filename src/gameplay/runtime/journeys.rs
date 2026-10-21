//! Viewer-authorized destination intents and presentation-only path geometry.
use crate::gameplay::model::{GameState, MapPoint, UnitId};
use serde_json::{Value, json};

/// Selects retained/current-turn/adjacent exploration without guessing tile IDs.
pub(super) fn request(
    state: &GameState,
    unit_id: UnitId,
    path: &[MapPoint],
) -> Option<(&'static str, Value)> {
    let viewer = state.viewer.as_ref()?;
    let source = viewer
        .units
        .iter()
        .find(|unit| UnitId::from_server_id(&unit.id) == unit_id)?;
    let target = *path.last()?;
    let coordinate = json!({"unit_id":source.id,"target_x":target.x,"target_y":target.y});
    if viewer.interface.supports("retained_movement_orders")
        && crate::gameplay::server::commands::valid(viewer, "set-movement-order", &coordinate)
    {
        return Some(("set-movement-order", coordinate));
    }
    if path.len() == 1 && crate::gameplay::server::commands::valid(viewer, "move", &coordinate) {
        return Some(("move", coordinate));
    }
    if viewer.interface.contract.is_some()
        && !viewer
            .interface
            .legal_actions
            .iter()
            .any(|action| action.route == "move" && action.entity_id == source.id)
    {
        return None;
    }
    let ids = super::authority::server_tile_ids_for_path(Some(viewer), path)?;
    Some(("move", json!({"unit_id":source.id,"path":ids})))
}

/// Server retained-order routing is diagonal, then axis, not hidden pathfinding.
/// Geometry conveys coordinates only and never creates a tile observation.
pub(super) fn line(origin: MapPoint, target: MapPoint) -> Vec<MapPoint> {
    let mut point = origin;
    let mut path = Vec::new();
    while point != target {
        point = MapPoint::new(step(point.x, target.x), step(point.y, target.y));
        path.push(point);
    }
    path
}

/// Overflow-safe one-coordinate step toward the public destination.
const fn step(current: u16, target: u16) -> u16 {
    if current < target {
        current.saturating_add(1)
    } else if current > target {
        current.saturating_sub(1)
    } else {
        current
    }
}
