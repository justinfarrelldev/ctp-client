//! CTP2-style unit activation and source-aligned path input state.

extern crate alloc;

use alloc::collections::VecDeque;
use alloc::vec::Vec;

use super::super::model::{GameState, MapPoint, PlayerId, UnitId};

/// Neighbor offsets used by original-style eight-direction land pathing.
const NEIGHBOR_DELTAS: [(i16, i16); 8] = [
    (1_i16, 1_i16),
    (1_i16, 0_i16),
    (0_i16, 1_i16),
    (-1_i16, 1_i16),
    (1_i16, -1_i16),
    (-1_i16, 0_i16),
    (0_i16, -1_i16),
    (-1_i16, -1_i16),
];

/// Input state for CTP2-style unit activation and click-drag movement.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct MovementInputState {
    /// Unit currently activated by the player for path planning.
    active_unit: Option<UnitId>,
    /// Drawable CTP2 path preview, matching `SelectedItem::SetDrawablePathDest`.
    preview: Option<MovementPathPreview>,
    /// In-progress click-drag state from `StartMoveClick`/`MoveDrag`.
    drag: Option<MovementDrag>,
}

/// Drawable legal and stop-feedback paths for one activated unit.
#[derive(Debug, Clone, PartialEq, Eq)]
struct MovementPathPreview {
    /// Unit whose path is being previewed.
    unit_id: UnitId,
    /// Tile where the path starts.
    origin: MapPoint,
    /// Legal path segment equivalent to CTP2 `m_good_path`, excluding origin.
    good_path: Vec<MapPoint>,
    /// Invalid continuation equivalent to CTP2 `m_bad_path`, excluding origin.
    bad_path: Vec<MapPoint>,
}

/// In-progress click-drag state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MovementDrag {
    /// Unit being ordered by this drag.
    unit_id: UnitId,
    /// Tile where the drag started.
    origin: MapPoint,
    /// Whether the cursor has left the origin tile, matching `m_isDragging`.
    has_left_origin: bool,
}

/// One pathfinding neighbor candidate with deterministic tie-breaking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NeighborCandidate {
    /// Neighbor tile.
    point: MapPoint,
    /// Chebyshev distance from neighbor to target.
    distance: u16,
    /// Stable source-order ordinal for equal-distance candidates.
    ordinal: u8,
}

impl MovementInputState {
    /// Returns the currently active unit.
    #[must_use]
    pub(super) const fn active_unit(&self) -> Option<UnitId> {
        self.active_unit
    }

    /// Returns the current legal path preview.
    #[must_use]
    pub(super) fn planned_path(&self) -> &[MapPoint] {
        self.preview
            .as_ref()
            .map_or(&[], |preview| preview.good_path.as_slice())
    }

    /// Returns the current invalid stop-feedback path preview.
    #[must_use]
    pub(super) fn planned_bad_path(&self) -> &[MapPoint] {
        self.preview
            .as_ref()
            .map_or(&[], |preview| preview.bad_path.as_slice())
    }

    /// Returns the origin tile for the current path preview.
    #[must_use]
    pub(super) fn planned_path_origin(&self) -> Option<MapPoint> {
        self.preview.as_ref().map(|preview| preview.origin)
    }

    /// Returns the origin tile for the current invalid stop-feedback path.
    #[must_use]
    pub(super) fn planned_bad_path_origin(&self) -> Option<MapPoint> {
        let preview = self.preview.as_ref()?;
        preview.good_path.last().copied().or(Some(preview.origin))
    }

    /// Starts a potential drag for the active unit.
    fn begin_drag(&mut self, unit_id: UnitId, origin: MapPoint) {
        self.active_unit = Some(unit_id);
        self.preview = Some(MovementPathPreview {
            unit_id,
            origin,
            good_path: Vec::new(),
            bad_path: Vec::new(),
        });
        self.drag = Some(MovementDrag {
            unit_id,
            origin,
            has_left_origin: false,
        });
    }

    /// Recomputes the hover path without creating a submittable drag order.
    pub(super) fn preview_path_to(&mut self, state: &GameState, target: MapPoint) {
        self.update_path_to(state, target, false);
    }

    /// Recomputes the current drag path to a visible map point.
    pub(super) fn extend_drag_to(&mut self, state: &GameState, target: MapPoint) {
        self.update_path_to(state, target, true);
    }

    /// Completes a CTP2 pathing click on a hovered destination.
    pub(super) fn finish_preview_order_to(
        &mut self,
        state: &GameState,
        target: MapPoint,
    ) -> Option<(UnitId, Vec<MapPoint>)> {
        let active_unit_id = self.active_unit?;
        self.update_path_to(state, target, false);
        self.drag = None;
        let preview = self.preview.take()?;
        self.active_unit = Some(preview.unit_id);
        if preview.unit_id != active_unit_id || preview.good_path.is_empty() {
            return None;
        }
        Some((preview.unit_id, preview.good_path))
    }

    /// Completes the drag, leaving the unit active for subsequent orders.
    pub(super) fn finish_drag(&mut self) -> Option<(UnitId, Vec<MapPoint>)> {
        let drag = self.drag.take()?;
        self.active_unit = Some(drag.unit_id);
        if !drag.has_left_origin {
            return None;
        }
        let preview = self.preview.take()?;
        if preview.unit_id != drag.unit_id
            || preview.origin != drag.origin
            || preview.good_path.is_empty()
        {
            return None;
        }
        Some((drag.unit_id, preview.good_path))
    }

    /// Cancels in-progress pathing without clearing unit activation.
    pub(super) fn cancel_drag(&mut self) {
        self.drag = None;
        self.preview = None;
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
            self.preview = None;
        }
    }

    /// Recomputes drawable pathing from the active unit to a cursor tile.
    fn update_path_to(&mut self, state: &GameState, target: MapPoint, mark_dragging: bool) {
        let Some(active_unit_id) = self.active_unit else {
            return;
        };
        let Some(unit) = state
            .units
            .iter()
            .find(|unit| unit.id == active_unit_id && unit.alive && unit.owner == PlayerId::Human)
        else {
            self.active_unit = None;
            self.drag = None;
            self.preview = None;
            return;
        };
        let origin = self
            .drag
            .filter(|drag| drag.unit_id == active_unit_id)
            .map_or(unit.position, |drag| drag.origin);
        if mark_dragging {
            if let Some(drag) = self.drag.as_mut() {
                if drag.unit_id == active_unit_id && target != drag.origin {
                    drag.has_left_origin = true;
                }
            }
        }
        self.preview = Some(path_preview_between(state, active_unit_id, origin, target));
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

/// Builds a legal/invalid path preview between two map points.
fn path_preview_between(
    state: &GameState,
    unit_id: UnitId,
    origin: MapPoint,
    target: MapPoint,
) -> MovementPathPreview {
    if target == origin {
        return MovementPathPreview {
            unit_id,
            origin,
            good_path: Vec::new(),
            bad_path: Vec::new(),
        };
    }
    if is_land_passable(state, target) {
        if let Some(good_path) = computed_path_between(state, origin, target) {
            return MovementPathPreview {
                unit_id,
                origin,
                good_path,
                bad_path: Vec::new(),
            };
        }
    }
    MovementPathPreview {
        unit_id,
        origin,
        good_path: Vec::new(),
        bad_path: direct_path_between(origin, target, state.width, state.height),
    }
}

/// Builds an eight-direction passability-aware path between two tiles, excluding the start tile.
fn computed_path_between(
    state: &GameState,
    start: MapPoint,
    target: MapPoint,
) -> Option<Vec<MapPoint>> {
    if start == target {
        return Some(Vec::new());
    }
    if !state.in_bounds(start) || !is_land_passable(state, target) {
        return None;
    }
    let tile_count = usize::from(state.width).checked_mul(usize::from(state.height))?;
    let mut visited = vec![false; tile_count];
    let mut parents = vec![None; tile_count];
    let start_index = tile_index(start, state.width, state.height)?;
    visited[start_index] = true;
    let mut queue = VecDeque::new();
    queue.push_back(start);
    while let Some(current) = queue.pop_front() {
        for neighbor in ordered_neighbors(current, target, state.width, state.height) {
            if !is_land_passable(state, neighbor) {
                continue;
            }
            let Some(neighbor_index) = tile_index(neighbor, state.width, state.height) else {
                continue;
            };
            if visited[neighbor_index] {
                continue;
            }
            visited[neighbor_index] = true;
            parents[neighbor_index] = Some(current);
            if neighbor == target {
                return reconstruct_path(start, target, &parents, state.width, state.height);
            }
            queue.push_back(neighbor);
        }
    }
    None
}

/// Reconstructs a path from parent links, excluding the start tile.
fn reconstruct_path(
    start: MapPoint,
    target: MapPoint,
    parents: &[Option<MapPoint>],
    width: u16,
    height: u16,
) -> Option<Vec<MapPoint>> {
    let mut reversed = Vec::new();
    let mut current = target;
    while current != start {
        reversed.push(current);
        let index = tile_index(current, width, height)?;
        current = parents.get(index).copied().flatten()?;
    }
    reversed.reverse();
    Some(reversed)
}

/// Returns passable eight-direction neighbors in deterministic CTP2-like target-biased order.
fn ordered_neighbors(
    current: MapPoint,
    target: MapPoint,
    width: u16,
    height: u16,
) -> Vec<MapPoint> {
    let mut candidates = Vec::with_capacity(NEIGHBOR_DELTAS.len());
    for (ordinal, (dx, dy)) in NEIGHBOR_DELTAS.iter().copied().enumerate() {
        let Some(point) = current.offset(dx, dy, width, height) else {
            continue;
        };
        candidates.push(NeighborCandidate {
            point,
            distance: point.tile_distance(target),
            ordinal: u8::try_from(ordinal).map_or(u8::MAX, |value| value),
        });
    }
    candidates.sort_by(|left, right| {
        left.distance
            .cmp(&right.distance)
            .then(left.ordinal.cmp(&right.ordinal))
    });
    candidates
        .into_iter()
        .map(|candidate| candidate.point)
        .collect()
}

/// Builds a direct invalid-feedback line between two tiles, excluding the start tile.
fn direct_path_between(
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

/// Returns whether a land unit may enter a point.
fn is_land_passable(state: &GameState, point: MapPoint) -> bool {
    state
        .tile(point)
        .is_some_and(|tile| tile.terrain.passable_by_land())
}

/// Returns a row-major tile index for a map point.
fn tile_index(point: MapPoint, width: u16, height: u16) -> Option<usize> {
    if point.x >= width || point.y >= height {
        return None;
    }
    usize::from(point.y)
        .checked_mul(usize::from(width))
        .and_then(|row| row.checked_add(usize::from(point.x)))
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

#[cfg(test)]
mod tests {
    //! Movement pathing tests derived from the original CTP2 selected-item flow.

    use super::{MovementInputState, begin_or_activate_unit_drag};
    use crate::gameplay::model::{FogState, GameState, MapPoint, TerrainKind};

    #[test]
    fn hover_preview_recomputes_path_without_creating_submittable_drag() {
        let mut state = GameState::new(24, 18, 1, 93);
        state.popups.clear();
        make_world_passable(&mut state);
        let selected = state
            .selected_unit()
            .expect("generated state should select a human unit")
            .clone();
        let target = selected
            .position
            .offset(1, 1, state.width, state.height)
            .expect("target should be in bounds");
        let mut movement = MovementInputState::default();

        assert!(
            begin_or_activate_unit_drag(&mut state, &mut movement, selected.position),
            "clicking the selected unit should activate it"
        );
        assert_eq!(
            movement.finish_drag(),
            None,
            "activation click released on the origin should not submit movement"
        );

        movement.preview_path_to(&state, target);

        assert_eq!(
            movement.planned_path(),
            &[target],
            "hovering after activation should draw the same source-style legal path as CTP2"
        );
        assert_eq!(
            movement.finish_drag(),
            None,
            "hover previews must not become drag submissions"
        );
    }

    #[test]
    fn computed_path_routes_around_impassable_direct_tile() {
        let mut state = GameState::new(24, 18, 1, 94);
        state.popups.clear();
        make_world_passable(&mut state);
        let selected = state
            .selected_unit()
            .expect("generated state should select a human unit")
            .clone();
        let blocker = selected
            .position
            .offset(1, 0, state.width, state.height)
            .expect("blocker should be in bounds");
        let target = selected
            .position
            .offset(2, 0, state.width, state.height)
            .expect("target should be in bounds");
        set_terrain(&mut state, blocker, TerrainKind::Water);
        let mut movement = MovementInputState::default();

        assert!(
            begin_or_activate_unit_drag(&mut state, &mut movement, selected.position),
            "clicking the selected unit should activate it"
        );
        movement.extend_drag_to(&state, target);

        assert!(
            !movement.planned_path().is_empty(),
            "source-aligned pathing should find a legal route instead of clearing the path"
        );
        assert_eq!(
            movement.planned_path().last().copied(),
            Some(target),
            "the computed path should still reach the cursor destination"
        );
        assert!(
            !movement.planned_path().contains(&blocker),
            "the computed path must route around impassable water"
        );
    }

    #[test]
    fn invalid_destination_keeps_bad_path_for_stop_feedback_without_submission() {
        let mut state = GameState::new(24, 18, 1, 95);
        state.popups.clear();
        make_world_passable(&mut state);
        let selected = state
            .selected_unit()
            .expect("generated state should select a human unit")
            .clone();
        let blocked_target = selected
            .position
            .offset(1, 0, state.width, state.height)
            .expect("blocked target should be in bounds");
        set_terrain(&mut state, blocked_target, TerrainKind::Water);
        let mut movement = MovementInputState::default();

        assert!(
            begin_or_activate_unit_drag(&mut state, &mut movement, selected.position),
            "clicking the selected unit should activate it"
        );
        movement.extend_drag_to(&state, blocked_target);

        assert!(
            movement.planned_path().is_empty(),
            "invalid destinations should not produce a legal move path"
        );
        assert_eq!(
            movement.planned_bad_path(),
            &[blocked_target],
            "invalid destinations should retain a red stop-feedback leg like CTP2 DrawLegalMove"
        );
        assert_eq!(
            movement.finish_drag(),
            None,
            "invalid stop-feedback paths must never submit movement"
        );
    }

    /// Makes every tile visible grassland for deterministic pathing tests.
    fn make_world_passable(state: &mut GameState) {
        for tile in &mut state.tiles {
            tile.terrain = TerrainKind::Grassland;
            tile.fog = FogState::Visible;
        }
    }

    /// Sets terrain at a generated map point.
    fn set_terrain(state: &mut GameState, point: MapPoint, terrain: TerrainKind) {
        let index = usize::from(point.y)
            .checked_mul(usize::from(state.width))
            .and_then(|row| row.checked_add(usize::from(point.x)))
            .expect("test map point should fit in usize");
        let tile = state
            .tiles
            .get_mut(index)
            .expect("test map point should address a generated tile");
        tile.terrain = terrain;
    }
}
