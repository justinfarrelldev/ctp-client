//! Visible presented stacks: stationary cells differ from traveling army actors.
use crate::gameplay::{
    GameplayError,
    model::{FogState, GameState, MapPoint, PlayerId, Unit, UnitId},
    runtime::{Camera, MovementInputState, animation::ActorPose, textures::GameplayTextures},
};
use alloc::collections::{BTreeMap, btree_map::Entry};
/// One permitted representative and already-measured frame pose, not game state.
pub(super) struct Presented<'state> {
    /// Exact currently permitted model identity.
    pub unit: &'state Unit,
    /// Exact cached sprite ID, without cloning unused portrait/icon/name strings.
    pub sprite: Option<u16>,
    /// Shared animation cadence used by both position and sprite sampling.
    pub duration: u64,
    /// Actual current presentation anchor/facing, not canonical destination alone.
    pub pose: ActorPose,
    /// Number of permitted members in this actual presented group.
    pub count: usize,
}
/// A moving army is identified only by received owner/army ID and matching
/// pose/destination; coincident independent units cannot invent membership.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Group<'state> {
    /// Resting actors share the current observed cell.
    Cell(MapPoint),
    /// Exact received traveling army currently presented at this anchor.
    Army(&'state str, &'state str, MapPoint, (i32, i32)),
    /// No authoritative army identity: one moving unit remains independent.
    MovingUnit(UnitId),
}
/// Filters before representative/count/art lookup, including when fog is disabled.
fn permitted(state: &GameState, unit: &Unit) -> bool {
    unit.alive
        && (unit.owner == PlayerId::Human
            || state
                .tile(unit.position)
                .is_some_and(|tile| tile.fog == FogState::Visible))
}
/// Resolves each visible pose exactly once and never loads hidden unit artwork.
pub(super) fn prepare<'state>(
    textures: &mut GameplayTextures<'_>,
    state: &'state GameState,
    camera: &Camera,
    input: &MovementInputState,
) -> Result<Vec<Presented<'state>>, GameplayError> {
    let sources = state.viewer.as_ref().map_or_else(BTreeMap::new, |viewer| {
        viewer
            .units
            .iter()
            .map(|unit| (UnitId::from_server_id(&unit.id), unit))
            .collect::<BTreeMap<_, _>>()
    });
    let mut candidates = Vec::new();
    for unit in state.units.iter().filter(|unit| permitted(state, unit)) {
        let sprite = textures.unit_sprite(unit, sources.get(&unit.id).copied())?;
        let duration = textures.unit_duration(
            sprite.unwrap_or_else(|| unit.kind.sprite_id()),
            input.actors.speed,
            input.actors.enabled,
        );
        let pose = input.actors.pose(unit, camera, duration);
        candidates.push(Presented {
            unit,
            sprite,
            duration,
            pose,
            count: 1_usize,
        });
    }
    Ok(group(state, candidates))
}
/// O(n log n) grouping replaces repeated scans of all canonical co-located units.
/// Selection wins only within a presented group. Hidden/dead members never count.
pub(super) fn group<'state>(
    state: &'state GameState,
    candidates: Vec<Presented<'state>>,
) -> Vec<Presented<'state>> {
    let memberships = state.viewer.as_ref().map_or_else(BTreeMap::new, |viewer| {
        viewer
            .units
            .iter()
            .filter_map(|unit| {
                unit.facts["army_id"]
                    .as_str()
                    .filter(|army| !army.is_empty())
                    .map(|army| {
                        (
                            UnitId::from_server_id(&unit.id),
                            (unit.owner_id.as_str(), army),
                        )
                    })
            })
            .collect::<BTreeMap<_, _>>()
    });
    let mut groups = BTreeMap::<Group<'state>, Presented<'state>>::new();
    for actor in candidates
        .into_iter()
        .filter(|actor| permitted(state, actor.unit))
    {
        let key = if actor.pose.moving {
            memberships
                .get(&actor.unit.id)
                .copied()
                .map_or(Group::MovingUnit(actor.unit.id), |(owner, army)| {
                    Group::Army(owner, army, actor.unit.position, actor.pose.position)
                })
        } else {
            Group::Cell(actor.unit.position)
        };
        match groups.entry(key) {
            Entry::Vacant(slot) => {
                slot.insert(actor);
            }
            Entry::Occupied(mut slot) => {
                let current = slot.get_mut();
                let count = current.count.saturating_add(actor.count);
                if state.selected_unit == Some(actor.unit.id) {
                    *current = actor;
                }
                current.count = count;
            }
        }
    }
    let mut presented = groups.into_values().collect::<Vec<_>>();
    presented.sort_by_key(|actor| (actor.pose.position.1, actor.pose.position.0, actor.unit.id));
    presented
}
