//! CTP2-style unit activation and source-aligned path input state.

extern crate alloc;

use alloc::vec::Vec;

use super::super::model::{GameState, MapPoint, PlayerId, UnitId};

/// Input state for CTP2-style unit activation and click-drag movement.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct MovementInputState {
    /// Unit currently activated by the player for path planning.
    active_unit: Option<UnitId>,
    /// In-progress drag path from the active unit.
    drag: Option<MovementDrag>,
}

/// In-progress click-drag path state.
#[derive(Debug, Clone, PartialEq, Eq)]
struct MovementDrag {
    /// Unit being ordered by this drag.
    unit_id: UnitId,
    /// Tile where the drag started.
    origin: MapPoint,
    /// Planned computed path excluding the origin tile.
    path: Vec<MapPoint>,
}

impl MovementInputState {
    /// Returns the currently active unit.
    #[must_use]
    pub(super) const fn active_unit(&self) -> Option<UnitId> {
        self.active_unit
    }

    /// Returns the currently planned drag path.
    #[must_use]
    pub(super) fn planned_path(&self) -> &[MapPoint] {
        self.drag.as_ref().map_or(&[], |drag| drag.path.as_slice())
    }

    /// Returns the origin tile for the currently planned path.
    #[must_use]
    pub(super) fn planned_path_origin(&self) -> Option<MapPoint> {
        self.drag.as_ref().map(|drag| drag.origin)
    }

    /// Starts a drag for the active unit.
    fn begin_drag(&mut self, unit_id: UnitId, origin: MapPoint) {
        self.active_unit = Some(unit_id);
        self.drag = Some(MovementDrag {
            unit_id,
            origin,
            path: Vec::new(),
        });
    }

    /// Recomputes the current drag path to a visible, passable map point.
    pub(super) fn extend_drag_to(&mut self, state: &GameState, target: MapPoint) {
        let Some(drag) = self.drag.as_mut() else {
            return;
        };
        drag.path.clear();
        if target == drag.origin
            || !state
                .tile(target)
                .is_some_and(|tile| tile.terrain.passable_by_land())
        {
            return;
        }
        for point in computed_path_between(drag.origin, target, state.width, state.height) {
            if !state
                .tile(point)
                .is_some_and(|tile| tile.terrain.passable_by_land())
            {
                drag.path.clear();
                return;
            }
            drag.path.push(point);
        }
    }

    /// Completes the drag, leaving the unit active for subsequent orders.
    pub(super) fn finish_drag(&mut self) -> Option<(UnitId, Vec<MapPoint>)> {
        let drag = self.drag.take()?;
        self.active_unit = Some(drag.unit_id);
        if drag.path.is_empty() {
            None
        } else {
            Some((drag.unit_id, drag.path))
        }
    }

    /// Cancels any in-progress drag without clearing activation.
    pub(super) fn cancel_drag(&mut self) {
        self.drag = None;
    }

    /// Clears activation if the unit no longer exists as a selectable human unit.
    pub(super) fn retain_selectable(&mut self, state: &GameState) {
        if self.active_unit.is_some_and(|unit_id| {
            !state
                .units
                .iter()
                .any(|unit| unit.id == unit_id && unit.alive && unit.owner == PlayerId::Human)
        }) {
            self.active_unit = None;
            self.drag = None;
        }
    }
}

/// Activates an owned unit and starts the same potential drag, matching CTP2 `SelectArmyStartMoveClick`.
pub(super) fn begin_or_activate_unit_drag(
    state: &mut GameState,
    movement: &mut MovementInputState,
    point: MapPoint,
) -> bool {
    if let Some(active_id) = movement.active_unit() {
        if state.units.iter().any(|unit| {
            unit.id == active_id
                && unit.alive
                && unit.owner == PlayerId::Human
                && unit.position == point
        }) {
            movement.begin_drag(active_id, point);
            return true;
        }
    }
    if let Some(unit_id) = human_unit_at(state, point) {
        if state.select_unit(unit_id) {
            movement.begin_drag(unit_id, point);
            return true;
        }
    }
    false
}

/// Returns the topmost human unit on a map tile.
fn human_unit_at(state: &GameState, point: MapPoint) -> Option<UnitId> {
    state
        .units
        .iter()
        .find(|unit| unit.alive && unit.owner == PlayerId::Human && unit.position == point)
        .map(|unit| unit.id)
}

/// Builds an eight-direction computed path between two tiles, excluding the start tile.
fn computed_path_between(
    start: MapPoint,
    target: MapPoint,
    width: u16,
    height: u16,
) -> Vec<MapPoint> {
    let mut path = Vec::new();
    let mut current = start;
    while current != target {
        let dx = coordinate_step(current.x, target.x);
        let dy = coordinate_step(current.y, target.y);
        let Some(next) = current.offset(dx, dy, width, height) else {
            break;
        };
        if next == current {
            break;
        }
        path.push(next);
        current = next;
    }
    path
}

/// Returns one signed step from one coordinate toward another.
const fn coordinate_step(current: u16, target: u16) -> i16 {
    if current < target {
        1_i16
    } else if current > target {
        -1_i16
    } else {
        0_i16
    }
}
