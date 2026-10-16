//! In-memory fixtures for the original four-stream terrain compositor.

use super::{PixelFormat, TileSet};

#[test]
fn all_four_transition_streams_advance_independently() {
    let tiles = TileSet::parse(&fixture(false), PixelFormat::Rgb565).expect("fixture parses");
    let image = tiles
        .compose_tile(4, [5, 1, 6, 2], 4)
        .expect("all streams compose");
    // First ground row has two pixels, second has six. Each marker consumes
    // only its own stream; fixed pixels must not advance any transition.
    let first = (24_usize * 94_usize + 46_usize) * 4_usize;
    let next = (25_usize * 94_usize + 44_usize) * 4_usize;
    assert_eq!(
        &image.data[first..first + 4],
        &[255, 0, 0, 255],
        "SW stream"
    );
    assert_eq!(
        &image.data[first + 4..first + 8],
        &[0, 255, 0, 255],
        "NW stream"
    );
    assert_eq!(&image.data[next..next + 4], &[0, 0, 255, 255], "NE stream");
    assert_eq!(
        &image.data[next + 4..next + 8],
        &[255, 255, 0, 255],
        "SE stream"
    );
    assert_eq!(
        &image.data[next + 8..next + 12],
        &[255, 0, 0, 255],
        "SW advances independently"
    );
}

#[test]
fn missing_transition_uses_same_position_in_original_99_tile() {
    let tiles = TileSet::parse(&fixture(false), PixelFormat::Rgb565).expect("fixture parses");
    let image = tiles
        .compose_tile(4, [4; 4], 4)
        .expect("self terrain composes");
    let first = (24_usize * 94_usize + 46_usize) * 4_usize;
    assert_eq!(
        &image.data[first..first + 8],
        &[255, 255, 255, 255, 0, 255, 255, 255],
        "fallback must preserve textured 499 pixels, not a flat approximate color"
    );
}

#[test]
fn truncated_transition_is_rejected_instead_of_drawing_corruption() {
    let tiles = TileSet::parse(&fixture(true), PixelFormat::Rgb565).expect("container parses");
    assert!(
        tiles.compose_tile(4, [5, 1, 6, 2], 4).is_err(),
        "short per-edge pixel streams must fail explicitly"
    );
}

#[test]
fn terrain_hat_shadow_runs_decode_without_copy_payload_or_runtime_crash() {
    for format in [PixelFormat::Rgb565, PixelFormat::Rgb555] {
        let bytes: Vec<u8> = [0_u16, 0_u16, 0_u16, 0xED04_u16]
            .into_iter()
            .flat_map(u16::to_le_bytes)
            .collect();
        let image = super::decode_tile_rgba(&bytes, format)
            .expect("original type-13 shadow run must decode");
        assert_eq!(
            &image.data[..16_usize],
            &[0_u8, 0, 0, 128, 0, 0, 0, 128, 0, 0, 0, 128, 0, 0, 0, 128],
            "shadows are translucent black, not magenta, opaque pixels, or copied words"
        );
        assert_eq!(
            image.data[19_usize], 0_u8,
            "uncovered hat pixels stay transparent"
        );
    }
}

/// Constructs two raw diamonds plus four terrain-pair records, without I/O.
fn fixture(short: bool) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&0_u16.to_le_bytes());
    bytes.extend_from_slice(&4_u32.to_le_bytes());
    let words = if short { 1_usize } else { 576_usize };
    bytes.extend_from_slice(
        &u32::try_from(words.saturating_mul(2_usize))
            .expect("small fixture")
            .to_le_bytes(),
    );
    for (to, pixel) in [(5_u16, 0xF800_u16), (1, 0x07E0), (6, 0x001F), (2, 0xFFE0)] {
        bytes.extend_from_slice(&4_u16.to_le_bytes());
        bytes.extend_from_slice(&to.to_le_bytes());
        for _ in 0_usize..4_usize.saturating_mul(words) {
            bytes.extend_from_slice(&pixel.to_le_bytes());
        }
    }
    bytes.extend_from_slice(&2_u32.to_le_bytes());
    for number in [4_u16, 499_u16] {
        bytes.extend_from_slice(&number.to_le_bytes());
        bytes.extend_from_slice(&[4_u8, 15_u8]);
        bytes.extend_from_slice(&4608_u16.to_le_bytes());
        for index in 0_u16..2304_u16 {
            let pixel = if number == 4_u16 {
                index % 4_u16
            } else if index % 2_u16 == 0_u16 {
                0xFFFF_u16
            } else {
                0x07FF_u16
            };
            bytes.extend_from_slice(&pixel.to_le_bytes());
        }
        bytes.extend_from_slice(&0_u16.to_le_bytes());
    }
    bytes.extend_from_slice(&[0_u8; 6]);
    bytes
}
