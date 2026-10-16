//! Tests for original CTP2 SPR parsing.

use alloc::vec::Vec;
use std::path::Path;

use crate::gameplay::assets::{SPRITE_DIRECTORY, load_unit_sprite, unit_sprite_candidates};

use super::{NUM_FACINGS, SpriteGroupAsset, UNIT_ACTION_MAX, decompress_lzw1, rgba_from_565};

#[test]
fn lzw1_copy_stream_returns_payload() {
    let data = [1_u8, 0, 0, 0, 10, 20, 30];
    let decoded = decompress_lzw1(&data, 3).expect("copy stream should decode");

    assert_eq!(decoded, vec![10, 20, 30]);
}

#[test]
fn rgba_conversion_uses_zero_alpha_for_transparency() {
    let bytes = [0_u8, 0, 0xFF, 0xFF];
    let rgba = rgba_from_565(&bytes, 2, 1).expect("rgba conversion should succeed");

    assert_eq!(rgba[3], 0, "zero pixel should be transparent");
    assert_eq!(rgba[7], 255, "non-zero pixel should be opaque");
}

#[test]
fn parses_minimal_regular_unit_sprite() {
    let bytes = minimal_regular_unit_sprite();
    let parsed = SpriteGroupAsset::parse(&bytes).expect("minimal sprite should parse");

    assert_eq!(parsed.actions.len(), 1, "one action should be present");
    assert_eq!(parsed.frame_count(), 1, "one frame should be loaded");
    assert!(parsed.has_action(0), "move action should exist");
    assert_eq!(
        parsed.actions[0].frames[0].hotpoint,
        (3_i32, 7_i32),
        "SPR hotspot must survive parsing; bounding-box centering distorts actors"
    );
}

#[test]
fn decodes_original_elephant_good_sprite_without_a_unit_shield_tail() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("ctp2")
        .join(SPRITE_DIRECTORY)
        .join("GG008.SPR");
    let bytes = std::fs::read(path).expect("read-only original elephant art");
    let sprite = SpriteGroupAsset::parse(&bytes)
        .expect("resource SPR groups use their own one-action layout");
    assert_eq!(
        sprite.file_type,
        super::SpriteFileType::Good,
        "not misread as a unit"
    );
    assert!(sprite.frame_count() > 0_usize, "elephant frame decoded");
    assert!(
        sprite.shield_points.is_empty(),
        "resources do not have unit shield metadata"
    );
    let frame = &sprite.actions[0_usize].frames[0_usize];
    assert_eq!(
        (frame.width, frame.height),
        (96_u16, 72_u16),
        "original resource canvas retained"
    );
}

#[test]
fn loads_local_ctp2_settler_sprite_when_available() {
    let candidates = unit_sprite_candidates(2);
    let Some(first_candidate) = candidates.first() else {
        return;
    };
    let path = Path::new("ctp2")
        .join(SPRITE_DIRECTORY)
        .join(first_candidate);
    if !path.exists() {
        return;
    }

    let sprite = load_unit_sprite(Path::new("ctp2"), 2)
        .expect("local CTP2 settler sprite should decode through the shared loader");

    assert!(
        sprite.frame_count() > 0,
        "settler sprite should contain frames"
    );
}

/// Builds a minimal v1.3 unit sprite containing one normal 1x1 frame.
fn minimal_regular_unit_sprite() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"FRPS");
    bytes.extend_from_slice(&0x0001_0003_u32.to_le_bytes());
    bytes.extend_from_slice(&4_u32.to_le_bytes());
    for _ in 0..5 {
        bytes.extend_from_slice(&0_u32.to_le_bytes());
    }
    bytes.extend_from_slice(&1_u32.to_le_bytes());
    bytes.extend_from_slice(&0_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&3_i32.to_le_bytes());
    bytes.extend_from_slice(&7_i32.to_le_bytes());
    bytes.extend_from_slice(&0_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&2_u32.to_le_bytes());
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    bytes.extend_from_slice(&0xFFFF_u16.to_le_bytes());
    bytes.extend_from_slice(&0_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&0_u16.to_le_bytes());
    bytes.extend_from_slice(&0_u16.to_le_bytes());
    bytes.extend_from_slice(&0_i32.to_le_bytes());
    bytes.extend_from_slice(&0_i32.to_le_bytes());
    bytes.extend_from_slice(&255_u16.to_le_bytes());
    for _ in 1..5 {
        bytes.extend_from_slice(&0_u32.to_le_bytes());
    }
    bytes.extend_from_slice(&vec![0_u8; 2 + 8 * NUM_FACINGS * 8]);
    bytes.extend_from_slice(&vec![0_u8; 2 + 8 * NUM_FACINGS * 8 + NUM_FACINGS * 8]);
    bytes.extend_from_slice(&[0_u8; UNIT_ACTION_MAX * NUM_FACINGS * 8]);
    bytes.extend_from_slice(&0_u16.to_le_bytes());
    bytes.extend_from_slice(&0_u16.to_le_bytes());
    bytes
}
