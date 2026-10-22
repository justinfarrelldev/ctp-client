//! Pointer interaction uses last-drawn source identities, not phantom destinations.
use super::MovementInputState;
use crate::gameplay::{
    model::{GameState, MapPoint, PlayerId, UnitId},
    runtime::Camera,
};
/// Consumes obsolete-frame clicks without rebasing and otherwise picks exact glyph.
pub(super) fn begin_presented(
    state: &mut GameState,
    movement: &mut MovementInputState,
    camera: &Camera,
    x: i32,
    y: i32,
) -> bool {
    if movement.presented.stale(state) {
        movement.cancel_drag();
        return true;
    }
    let Some(id) = movement.presented.at(state, x, y) else {
        return false;
    };
    if !begin_exact(state, movement, id) {
        return false;
    }
    if let Some(drag) = movement.drag.as_mut() {
        drag.gesture_origin = camera.screen_to_tile(x, y).unwrap_or(drag.origin);
    }
    true
}
/// Keeps original resting-cell activation while excluding visually traveling actors.
pub(super) fn begin_map(
    state: &mut GameState,
    movement: &mut MovementInputState,
    point: MapPoint,
) -> bool {
    if movement.presented.stale(state) {
        return false;
    }
    let candidate = movement
        .active_unit()
        .filter(|id| {
            state.units.iter().any(|unit| {
                unit.id == *id
                    && unit.alive
                    && unit.owner == PlayerId::Human
                    && unit.position == point
                    && !movement.presented.moving(state, *id)
            })
        })
        .or_else(|| {
            state
                .units
                .iter()
                .find(|unit| {
                    unit.alive
                        && unit.owner == PlayerId::Human
                        && unit.position == point
                        && !movement.presented.moving(state, unit.id)
                })
                .map(|unit| unit.id)
        });
    candidate.is_some_and(|id| begin_exact(state, movement, id))
}
/// Canonical authoritative origin is fixed even when the drawn glyph is in transit.
fn begin_exact(state: &mut GameState, movement: &mut MovementInputState, id: UnitId) -> bool {
    let Some(origin) = state
        .units
        .iter()
        .find(|unit| unit.id == id && unit.alive && unit.owner == PlayerId::Human)
        .map(|unit| unit.position)
    else {
        return false;
    };
    if !state.select_unit(id) {
        return false;
    }
    movement.begin_drag(state, id, origin);
    true
}
