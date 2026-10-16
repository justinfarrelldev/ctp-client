//! Read-only retail art validation; no assets or screenshots are copied to disk.

use super::{PixelFormat, TILE_SET_555, TILE_SET_565, TileSet};
use std::path::Path;

#[test]
fn every_retail_biome_and_neighbor_pair_composes_in_both_pixel_formats() {
    for (path, format) in [
        (TILE_SET_565, PixelFormat::Rgb565),
        (TILE_SET_555, PixelFormat::Rgb555),
    ] {
        let bytes = std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("ctp2")
                .join(path),
        )
        .expect("installed original art is read-only test input");
        let tiles = TileSet::parse(&bytes, format)
            .expect("every ground, hat, shadow and river must decode");
        assert_eq!(
            tiles.transforms.len(),
            218_usize,
            "all release transform rules loaded"
        );
        assert_eq!(
            tiles.transitions.len(),
            221_usize,
            "all four streams for all release terrain pairs loaded"
        );
        for center in 1_u8..=24_u8 {
            for neighbor in 1_u8..=24_u8 {
                let mut neighborhood = [neighbor; 9_usize];
                neighborhood[4_usize] = center;
                for variation in 0_u32..5_u32 {
                    let number = tiles.select_tile(neighborhood, variation);
                    let image = tiles
                        .compose_tile(center, [neighbor; 4_usize], number)
                        .expect("every selected original terrain combination must compose");
                    assert_eq!(
                        (image.width, image.height, image.data.len()),
                        (94_u32, 72_u32, 27_072_usize),
                        "full 94×72 art canvas retained"
                    );
                    assert!(
                        image
                            .data
                            .chunks_exact(4_usize)
                            .any(|pixel| pixel[3_usize] == 255_u8),
                        "no empty/missing terrain placeholder"
                    );
                }
            }
        }
    }
}

#[test]
fn every_original_river_connection_and_mouth_uses_decoded_art() {
    let tiles =
        TileSet::load_from_installation(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"))
            .expect("read original river art");
    for mask in 0_u8..16_u8 {
        let mut neighborhood = [false; 9_usize];
        neighborhood[4_usize] = true;
        for (bit, edge) in [1_usize, 3_usize, 5_usize, 7_usize].into_iter().enumerate() {
            neighborhood[edge] = mask & (1_u8 << bit) != 0_u8;
        }
        let piece = tiles
            .river_piece(neighborhood, false)
            .expect("all sixteen land-river masks exist");
        let dry = tiles
            .compose_tile(1_u8, [1_u8; 4_usize], 1_u16)
            .expect("plain ground");
        let wet = tiles
            .compose_with_river(1_u8, [1_u8; 4_usize], 1_u16, Some(piece))
            .expect("river over ground");
        assert_ne!(
            dry.data, wet.data,
            "river art actually changes the frame, mask {mask}"
        );
        if mask != 0_u8 {
            let mouth = tiles
                .river_piece(neighborhood, true)
                .expect("all fifteen nonempty shore-river masks exist");
            assert_eq!(
                tiles.rivers[mouth].pattern[4_usize], 2_i16,
                "beaches select distinct river-mouth art"
            );
        }
    }
    assert_eq!(
        tiles.river_piece([false; 9_usize], false),
        None,
        "ordinary dry cells never gain rivers"
    );
}
