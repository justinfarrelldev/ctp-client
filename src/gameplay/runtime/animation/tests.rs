//! Direction, animation-table and confirmed-path clock contracts.

use super::{ActorAnimations, Facing, frame_sample, move_duration};
use crate::gameplay::assets::sprite::SpriteAnimation;
use crate::gameplay::model::{GameState, MapPoint};
use crate::gameplay::runtime::Camera;

#[test]
fn all_eight_directions_use_original_five_facings_and_mirroring() {
    let cases: [((i32, i32), u8, u8, bool); 8] = [
        ((0_i32, -48_i32), 0_u8, 0_u8, false),
        ((47, -24), 1, 1, false),
        ((94, 0), 2, 2, false),
        ((47, 24), 3, 3, false),
        ((0, 48), 4, 4, false),
        ((-47, 24), 5, 3, true),
        ((-94, 0), 6, 2, true),
        ((-47, -24), 7, 1, true),
    ];
    for (delta, index, stored, flipped) in cases {
        let facing = Facing::from_delta(delta);
        assert_eq!(facing.index(), index, "spriteutils_DeltaToFacing {delta:?}");
        assert_eq!(
            (facing.stored(), facing.flipped()),
            (stored, flipped),
            "FacedSprite::Draw {delta:?}"
        );
        assert_eq!(
            facing.hotpoint((17_i32, 29_i32), 96),
            if flipped {
                (79_i32, 29_i32)
            } else {
                (17_i32, 29_i32)
            },
            "mirror anchor as well as pixels"
        );
    }
}

#[test]
fn sequence_frames_deltas_transparency_delay_and_speed_share_one_clock() {
    let animation = SpriteAnimation {
        animation_type: 1,
        frames: vec![2, 0, 1],
        deltas: vec![(0_i32, 0_i32), (1_i32, -2_i32), (3_i32, 4_i32)],
        transparencies: vec![15, 10, 5],
        playback_time: 300,
        delay: 200,
    };
    assert_eq!(
        frame_sample(&animation, 100, 300, true),
        (0_u16, (1_i32, -2_i32), 170_u8),
        "frame table, not sequential image index"
    );
    assert_eq!(
        frame_sample(&animation, 300, 300, true),
        (2_u16, (0_i32, 0_i32), 255_u8),
        "idle delay holds the original first frame"
    );
    assert_eq!(
        frame_sample(&animation, 500, 300, true),
        (2_u16, (0_i32, 0_i32), 255_u8),
        "loop restarts after delay"
    );
    assert_eq!(
        frame_sample(&animation, u64::MAX, 300, false),
        (1_u16, (3_i32, 4_i32), 85_u8),
        "sequential playback clamps safely"
    );
    assert_eq!(
        move_duration(300, 0, true),
        600,
        "slow profile doubles default duration"
    );
    assert_eq!(move_duration(300, 5, true), 300, "default speed");
    assert_eq!(
        move_duration(300, 9, true),
        60,
        "fast profile scales motion and frames together"
    );
    assert_eq!(
        move_duration(0, 99, false),
        1,
        "animations off completes immediately"
    );
}

#[test]
fn disabling_animation_displays_confirmed_endpoint_on_the_first_frame() {
    let mut state = GameState::new(24, 18, 1, 42);
    let before = state.units.clone();
    let target = state.units[0_usize]
        .position
        .offset(1, 0, state.width, state.height)
        .expect("target");
    state.units[0_usize].position = target;
    let mut actors = ActorAnimations {
        enabled: false,
        ..ActorAnimations::default()
    };
    actors.confirm(&before, &state, &[target]);
    actors.begin_frame(1000);
    let camera = Camera::for_test_identity();
    let pose = actors.pose(&state.units[0_usize], &camera, 1);
    assert_eq!(
        pose.position,
        camera.tile_to_screen(target),
        "disabled animation never briefly rewinds the accepted unit"
    );
    assert!(
        !pose.moving,
        "disabled animations do not render a MOVE cycle"
    );
}

#[test]
fn network_wait_does_not_consume_confirmed_animation_before_first_frame() {
    let mut state = GameState::new(24, 18, 1, 42);
    let before = state.units.clone();
    let origin = state.units[0_usize].position;
    let target = origin
        .offset(1, 0, state.width, state.height)
        .expect("target");
    state.units[0_usize].position = target;
    let mut actors = ActorAnimations::default();
    actors.confirm(&before, &state, &[target]);
    actors.begin_frame(5000);
    let camera = Camera::for_test_identity();
    assert_eq!(
        actors.pose(&state.units[0_usize], &camera, 300).position,
        camera.tile_to_screen(origin),
        "server response latency must not skip travel"
    );
    actors.begin_frame(5150);
    assert!(
        actors.pose(&state.units[0_usize], &camera, 300).moving,
        "second frame halfway through travel"
    );
    actors.begin_frame(5300);
    assert!(
        !actors.pose(&state.units[0_usize], &camera, 300).moving,
        "arrival uses receipt-time clock"
    );
}

#[test]
fn only_confirmed_path_prefix_animates_and_rejected_orders_never_move_pixels() {
    let mut state = GameState::new(24, 18, 1, 42);
    let before = state.units.clone();
    let id = state.selected_unit.expect("selected");
    let origin = state.selected_unit().expect("starter").position;
    let first = origin
        .offset(1, 0, state.width, state.height)
        .expect("first");
    let last = origin
        .offset(2, 0, state.width, state.height)
        .expect("last");
    let camera = Camera::for_test_identity();
    let mut actors = ActorAnimations::default();
    actors.confirm(&before, &state, &[first, last]);
    assert_eq!(
        actors
            .pose(state.selected_unit().expect("unit"), &camera, 300)
            .position,
        camera.tile_to_screen(origin),
        "server rejection/no change never animates"
    );
    state
        .units
        .iter_mut()
        .find(|unit| unit.id == id)
        .expect("unit")
        .position = first;
    actors.confirm(&before, &state, &[first, last]);
    let unit = state.selected_unit().expect("unit");
    actors.now_ms = 150;
    let middle = actors.pose(unit, &camera, 300);
    let from = camera.tile_to_screen(origin);
    let to = camera.tile_to_screen(first);
    assert_eq!(
        middle.position,
        (from.0.saturating_add(23), from.1.saturating_add(12)),
        "halfway through confirmed first leg"
    );
    assert!(middle.moving, "same clock still selects MOVE action");
    actors.now_ms = 300;
    let done = actors.pose(unit, &camera, 300);
    assert_eq!(
        done.position, to,
        "must stop at accepted prefix, not requested last tile"
    );
    assert!(!done.moving, "switch to idle exactly at arrival");
    actors.now_ms = u64::MAX;
    assert_eq!(
        actors.pose(unit, &camera, 300).position,
        to,
        "large clock is bounded"
    );
    assert_eq!(
        unit.position, first,
        "presentation never changes authoritative position"
    );
    assert_ne!(first, MapPoint::new(0, 0), "fixture has real coordinates");
}
