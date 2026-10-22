//! Presented grouping is not canonical destination/hidden-membership inference.
use super::stacks::{Presented, group};
use crate::gameplay::{
    model::{GameState, UnitId},
    runtime::animation::{ActorPose, Facing},
    server::ServerProjection,
};
use serde_json::json;
/// Exact received army membership; no renderer-local grouping command exists.
fn fixture() -> GameState {
    let viewer:ServerProjection=serde_json::from_value(json!({"revision":1_i64,"turn":1_u32,"tiles":[{"id":"cell","x":1_u16,"y":1_u16,"terrain":"TERRAIN_PLAINS"}],"units":[{"id":"a","owner_id":"human","tile_id":"cell","army_id":"army-1"},{"id":"b","owner_id":"human","tile_id":"cell","army_id":"army-1"}]})).expect("viewer-safe own membership");
    GameState::from_server_projection(&viewer, 1_u8, 1_u64)
}
/// Synthetic clock/screen positions only; no original art/engine facts copied.
fn posed(unit: &crate::gameplay::model::Unit, moving: bool, position: (i32, i32)) -> Presented<'_> {
    Presented {
        unit,
        sprite: None,
        duration: 100_u64,
        pose: ActorPose {
            position,
            facing: Facing::from_delta((1_i32, 1_i32)),
            moving,
            elapsed: 0_u64,
        },
        count: 1_usize,
    }
}
#[test]
fn sprite_only_lookup_preserves_unknown_authoritative_type_and_explicit_settler_fallback_without_cloning_art_strings()
 {
    let state = fixture();
    let mut textures =
        crate::gameplay::runtime::textures::GameplayTextures::without_assets_for_test();
    let source = &state.viewer.as_ref().expect("viewer").units[0];
    let unit = &state.units[0];
    assert!(
        textures
            .unit_sprite(unit, Some(source))
            .expect("unknown source")
            .is_none(),
        "unknown authoritative warrior-like type cannot silently become a retail Warrior"
    );
    assert_eq!(
        textures
            .unit_sprite(unit, None)
            .expect("explicit asset-free model"),
        Some(unit.kind.sprite_id()),
        "non-authoritative fallback remains source-compatible"
    );
    let mut settler = unit.clone();
    settler.kind = crate::gameplay::model::UnitKind::Settler;
    assert_eq!(
        textures
            .unit_sprite(&settler, Some(source))
            .expect("explicit settler role"),
        Some(settler.kind.sprite_id()),
        "existing opaque synthetic settler role artwork remains explicit fallback"
    );
}
#[test]
fn canonical_co_location_does_not_hide_visually_separate_motion_or_infer_a_moving_army() {
    let mut state = fixture();
    state.selected_unit = Some(UnitId::from_server_id("b"));
    let members = &state.units;
    let separate = group(
        &state,
        vec![
            posed(&members[0], true, (100_i32, 100_i32)),
            posed(&members[1], false, (200_i32, 150_i32)),
        ],
    );
    assert_eq!(
        separate.len(),
        2_usize,
        "selected destination member must not suppress traveling counterpart"
    );
    assert!(
        separate.iter().all(|actor| actor.count == 1_usize),
        "display groups count only actually presented members"
    );
    let stationary = group(
        &state,
        vec![
            posed(&members[0], false, (200_i32, 150_i32)),
            posed(&members[1], false, (200_i32, 150_i32)),
        ],
    );
    assert_eq!(
        stationary.len(),
        1_usize,
        "once stationary same cell merges normally"
    );
    assert_eq!(
        (stationary[0].unit.id, stationary[0].count),
        (UnitId::from_server_id("b"), 2_usize),
        "selected representative only wins inside its real presented group"
    );
    let together = group(
        &state,
        vec![
            posed(&members[0], true, (100_i32, 100_i32)),
            posed(&members[1], true, (100_i32, 100_i32)),
        ],
    );
    assert_eq!(
        together.len(),
        1_usize,
        "known same received army traveling together shares herald"
    );
    assert_eq!(together[0].count, 2_usize, "observed members only");
    state.viewer.as_mut().expect("viewer").units[1].facts["army_id"] = json!(null);
    let independent = group(
        &state,
        vec![
            posed(&state.units[0], true, (100_i32, 100_i32)),
            posed(&state.units[1], true, (100_i32, 100_i32)),
        ],
    );
    assert_eq!(
        independent.len(),
        2_usize,
        "coinciding animation positions never invent army membership"
    );
}
