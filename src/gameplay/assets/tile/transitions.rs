//! Release neighborhood rules and four-stream ground composition.
//!
//! Markers 0–3 consume independent SW/NW/NE/SE streams. Missing terrain pairs
//! sample the corresponding pixel of the terrain's *99 texture, not a flat RGB.

use super::{
    RgbaImage, TILE_GRID_HEIGHT, TILE_PIXEL_HEADROOM, TILE_PIXEL_HEIGHT, TILE_PIXEL_WIDTH, TileSet,
    asset_error, raw_base_tile_word_count, read_bytes, read_u16, read_u32, rgba_image_from_pixels,
    words_from_le_bytes,
};
use crate::gameplay::GameplayError;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use std::io::Cursor;

/// Four ordered pixel streams for every from/to terrain pair.
pub type TransitionTable = BTreeMap<(u8, u8), [Vec<u16>; 4]>;

/// Reads ordered 3×3 rules and their five-entry variant lists.
pub(super) fn read_transforms(reader: &mut Cursor<&[u8]>) -> Result<Vec<[i16; 15]>, GameplayError> {
    let count = read_u16(reader)?;
    let mut transforms = Vec::with_capacity(usize::from(count));
    for _ in 0..count {
        let mut rule = [0_i16; 15];
        for value in &mut rule {
            *value = i16::from_le_bytes(read_u16(reader)?.to_le_bytes());
        }
        transforms.push(rule);
    }
    Ok(transforms)
}

/// Reads all four original edge textures, rejecting malformed lengths.
pub(super) fn read_transitions(
    reader: &mut Cursor<&[u8]>,
) -> Result<TransitionTable, GameplayError> {
    let count = read_u32(reader)?;
    let size =
        usize::try_from(read_u32(reader)?).map_err(|error| asset_error(error.to_string()))?;
    let mut transitions = BTreeMap::new();
    for _ in 0..count {
        let from =
            u8::try_from(read_u16(reader)?).map_err(|error| asset_error(error.to_string()))?;
        let to = u8::try_from(read_u16(reader)?).map_err(|error| asset_error(error.to_string()))?;
        let mut edges = core::array::from_fn(|_| Vec::new());
        for edge in &mut edges {
            *edge = words_from_le_bytes(&read_bytes(reader, size)?)?;
        }
        transitions.insert((from, to), edges);
    }
    Ok(transitions)
}

/// Matches every original wildcard class and the full ordered variant list.
pub(super) fn select_tile(tiles: &TileSet, neighborhood: [u8; 9], variation: u32) -> u16 {
    let center = neighborhood[4];
    for rule in &tiles.transforms {
        if !rule
            .iter()
            .take(9)
            .zip(neighborhood)
            .all(|(&pattern, terrain)| matches_terrain(pattern, terrain, center))
        {
            continue;
        }
        if let Ok(number) = u16::try_from(rule[9]) {
            if tiles.by_tile_number.contains_key(&number) {
                return number;
            }
        } else {
            let start = usize::try_from(variation % 5_u32).map_or(0_usize, |value| value);
            // Check all slots once. The source decrements past zero to index 5;
            // that out-of-bounds access is deliberately not preserved.
            for offset in 0_usize..5_usize {
                let index = start.saturating_add(5_usize).saturating_sub(offset) % 5_usize;
                if let Ok(number) = u16::try_from(rule[10_usize.saturating_add(index)]) {
                    if tiles.by_tile_number.contains_key(&number) {
                        return number;
                    }
                }
            }
        }
    }
    u16::from(center)
}

/// Resolves raw ground, then river, then terrain hat in the source draw order.
pub(super) fn compose_tile(
    tiles: &TileSet,
    base: u8,
    neighbors: [u8; 4],
    number: u16,
    river: Option<usize>,
) -> Result<RgbaImage, GameplayError> {
    let tile = tiles
        .by_tile_number
        .get(&number)
        .ok_or_else(|| asset_error(format!("missing terrain tile {number}")))?;
    let mut image = if tile.raw_words.len() == raw_base_tile_word_count() {
        compose_ground(tiles, base, neighbors, &tile.raw_words)?
    } else {
        tile.image.clone()
    };
    if let Some(overlay) = river
        .and_then(|index| tiles.rivers.get(index))
        .and_then(|river| river.image.as_ref())
    {
        blend_overlay(&mut image, overlay);
    }
    if let Some(hat) = tile.hat.as_ref() {
        blend_overlay(&mut image, hat);
    }
    Ok(image)
}

/// Consumes marker streams exactly as `DrawTransitionTile`, preserving all art.
fn compose_ground(
    tiles: &TileSet,
    base: u8,
    neighbors: [u8; 4],
    raw: &[u16],
) -> Result<RgbaImage, GameplayError> {
    let fallback_number = u16::from(base)
        .saturating_mul(100_u16)
        .saturating_add(99_u16);
    let fallback = tiles.by_tile_number.get(&fallback_number);
    let mut cursors = [0_usize; 4];
    let mut source = 0_usize;
    let mut pixels =
        vec![0_u16; usize::from(TILE_PIXEL_WIDTH).saturating_mul(usize::from(TILE_GRID_HEIGHT))];
    for row in 0_u16..TILE_PIXEL_HEIGHT {
        let inset = if row < 24_u16 {
            23_u16.saturating_sub(row).saturating_mul(2_u16)
        } else {
            row.saturating_sub(24_u16).saturating_mul(2_u16)
        };
        for column in inset..TILE_PIXEL_WIDTH.saturating_sub(inset) {
            let marker = raw[source];
            let pixel = if marker < 4_u16 {
                let edge = usize::from(marker);
                if let Some(stream) = tiles.transitions.get(&(base, neighbors[edge])) {
                    let value = stream[edge].get(cursors[edge]).copied().ok_or_else(|| {
                        asset_error(format!(
                            "short terrain transition {base} -> {}, edge {edge}",
                            neighbors[edge]
                        ))
                    })?;
                    cursors[edge] = cursors[edge].saturating_add(1_usize);
                    value
                } else {
                    fallback
                        .and_then(|buffer| buffer.raw_words.get(source))
                        .copied()
                        .filter(|value| *value >= 4_u16)
                        .ok_or_else(|| {
                            asset_error(format!(
                                "missing terrain fallback {fallback_number} at pixel {source}"
                            ))
                        })?
                }
            } else {
                marker
            };
            let dest = usize::from(row.saturating_add(TILE_PIXEL_HEADROOM))
                .saturating_mul(usize::from(TILE_PIXEL_WIDTH))
                .saturating_add(usize::from(column));
            pixels[dest] = pixel;
            source = source.saturating_add(1_usize);
        }
    }
    rgba_image_from_pixels(&pixels, tiles.format)
}

/// Applies opaque hat/river pixels and true shadow runs without discarding ground.
fn blend_overlay(image: &mut RgbaImage, overlay: &RgbaImage) {
    for (target, pixel) in image
        .data
        .chunks_exact_mut(4_usize)
        .zip(overlay.data.chunks_exact(4_usize))
    {
        if pixel[3] == 128_u8 && target[3] == 255_u8 {
            for channel in target.iter_mut().take(3_usize) {
                *channel = channel.saturating_div(2_u8);
            }
        } else if pixel[3] != 0_u8 {
            target.copy_from_slice(pixel);
        }
    }
}

/// Release wildcard classes interpreted in tileset-index space, not server enums.
fn matches_terrain(pattern: i16, terrain: u8, center: u8) -> bool {
    let water = matches!(terrain, 10..=16 | 23 | 24);
    match pattern {
        -1 => true,
        -2 => !water,
        -3 => water,
        -4 => matches!(terrain, 11 | 12 | 14 | 15),
        -5 => matches!(terrain, 10 | 13 | 16 | 23 | 24),
        -6 => matches!(terrain, 14 | 15),
        -7 => terrain != center,
        -8 => !water && terrain != center,
        -9 => water && terrain != center,
        value => value == i16::from(terrain),
    }
}
