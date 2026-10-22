//! Pure pixel/identity regression; injected bytes/state, no filesystem or transport.
use super::{ActorHit, ActorHits, HitMask};
use crate::gameplay::{
    model::{GameState, PlayerId, Unit, UnitId},
    server::ServerProjection,
    ui_model::MenuRect,
};
use alloc::sync::Arc;
use serde_json::json;
#[test]
fn alpha_masks_follow_exact_scaled_flipped_pixel_bounds_and_reject_malformed_rasters() {
    let bytes = [
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 255_u8, 0_u8, 0_u8, 0_u8, 128_u8, 0_u8, 0_u8,
        0_u8, 0_u8,
    ];
    let mask = HitMask::from_rgba(2_u32, 2_u32, &bytes).expect("2x2");
    let bounds = MenuRect {
        x: 100_i32,
        y: 200_i32,
        width: 4_u32,
        height: 4_u32,
    };
    for (point, normal, flipped) in [
        ((100_i32, 200_i32), false, true),
        ((103_i32, 200_i32), true, false),
        ((100_i32, 203_i32), true, false),
        ((103_i32, 203_i32), false, true),
        ((99_i32, 200_i32), false, false),
        ((104_i32, 203_i32), false, false),
    ] {
        assert_eq!(
            mask.hit(bounds, point.0, point.1, false),
            normal,
            "scaled mask {point:?}"
        );
        assert_eq!(
            mask.hit(bounds, point.0, point.1, true),
            flipped,
            "reflected mask {point:?}"
        );
    }
    assert!(
        mask.hit(
            MenuRect {
                x: -2_i32,
                y: 100_i32,
                ..bounds
            },
            0_i32,
            100_i32,
            false
        ),
        "partially clipped left-edge sprite remains hittable at visible opaque pixel; do not port original whole-frame edge rejection"
    );
    assert!(
        !mask.hit(
            MenuRect {
                width: 0_u32,
                ..bounds
            },
            100_i32,
            200_i32,
            false
        ),
        "zero destination cannot divide"
    );
    assert!(
        HitMask::from_rgba(0_u32, 1_u32, &[]).is_err(),
        "zero source invalid"
    );
    assert!(
        HitMask::from_rgba(2_u32, 2_u32, &bytes[..15_usize]).is_err(),
        "truncation invalid"
    );
    assert!(
        HitMask::from_rgba(u32::MAX, u32::MAX, &[]).is_err(),
        "overflow dimensions rejected before allocation"
    );
}
/// A current own moving glyph, kept independent of private/raw unit data.
fn fixture() -> (GameState, ActorHits, ActorHit) {
    let mut state = GameState::new(24_u16, 18_u16, 1_u8, 42_u64);
    state.viewer = Some(
        serde_json::from_value::<ServerProjection>(json!({"revision":1_i64,"turn":1_u32}))
            .expect("viewer"),
    );
    let unit = state.selected_unit().expect("starter");
    let hit = ActorHit {
        id: unit.id,
        canonical: unit.position,
        bounds: MenuRect {
            x: 100_i32,
            y: 100_i32,
            width: 4_u32,
            height: 4_u32,
        },
        mask: Some(Arc::new(
            HitMask::from_rgba(1_u32, 1_u32, &[0_u8, 0_u8, 0_u8, 255_u8]).expect("opaque"),
        )),
        visible: true,
        flipped: false,
        moving: true,
    };
    let mut cache = ActorHits::default();
    cache.replace(&state, vec![hit.clone()]);
    (state, cache, hit)
}
/// Only fixture mutation, not a live transport or gameplay write.
fn own(state: &mut GameState, id: UnitId) -> &mut Unit {
    state
        .units
        .iter_mut()
        .find(|unit| unit.id == id)
        .expect("fixture own unit")
}
#[test]
fn presented_identity_is_owned_alive_and_position_exact_not_neighbor_retargeting() {
    let (mut state, mut cache, hit) = fixture();
    let id = hit.id;
    assert_eq!(
        cache.at(&state, 101_i32, 101_i32),
        Some(id),
        "exact drawn ID"
    );
    assert!(
        cache.moving(&state, id),
        "canonical destination must not masquerade as traveling actor"
    );
    assert!(!cache.stale(&state), "current frame");
    own(&mut state, id).owner = PlayerId::Empire(1_u8);
    assert_eq!(
        cache.at(&state, 101_i32, 101_i32),
        None,
        "foreign actor cannot gain selection"
    );
    own(&mut state, id).owner = PlayerId::Human;
    own(&mut state, id).alive = false;
    assert_eq!(
        cache.at(&state, 101_i32, 101_i32),
        None,
        "dead actor unavailable"
    );
    own(&mut state, id).alive = true;
    own(&mut state, id).position.x = hit.canonical.x.saturating_add(1_u16);
    assert_eq!(
        cache.at(&state, 101_i32, 101_i32),
        None,
        "same-ID position disagreement cannot rebase"
    );
    own(&mut state, id).position = hit.canonical;
    cache.replace(
        &state,
        vec![ActorHit {
            id: UnitId(99_999_u32),
            ..hit
        }],
    );
    assert_eq!(
        cache.at(&state, 101_i32, 101_i32),
        None,
        "vanished target never selects neighboring owner"
    );
}
#[test]
fn stale_revision_and_invisible_frame_never_gain_clicks_or_imply_current_motion() {
    let (mut state, mut cache, hit) = fixture();
    cache.replace(
        &state,
        vec![ActorHit {
            visible: false,
            ..hit.clone()
        }],
    );
    assert_eq!(
        cache.at(&state, 101_i32, 101_i32),
        None,
        "fully transparent frame cannot create opaque clickable box"
    );
    assert!(
        cache.moving(&state, hit.id),
        "invisible traveling glyph still cannot become phantom resting destination"
    );
    state.viewer.as_mut().expect("viewer").revision = 2_i64;
    assert!(
        cache.stale(&state),
        "stale presentation blocks map dispatch until rendered"
    );
    assert_eq!(cache.at(&state, 101_i32, 101_i32), None, "no stale IDs");
    assert!(
        !cache.moving(&state, hit.id),
        "stale motion is not inferred current"
    );
    cache.replace(&state, vec![hit]);
    assert!(
        !cache.stale(&state),
        "complete new presented frame clears only presentation staleness"
    );
}
