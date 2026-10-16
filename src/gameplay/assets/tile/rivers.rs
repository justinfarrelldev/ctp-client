//! Original `TryRiver` edge-connectivity matching and overlay data.

use super::{
    PixelFormat, RgbaImage, asset_error, decode_tile_rgba, read_bytes, read_u16, read_u32,
};
use crate::gameplay::GameplayError;
use alloc::vec::Vec;
use std::io::Cursor;

/// One river connectivity rule and its transparent overlay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiverTile {
    /// W,NW,N,SW,center,NE,S,SE,E plus the source result field.
    pub pattern: [i16; 10],
    /// Decoded art; some source rules intentionally have no image.
    pub image: Option<RgbaImage>,
}

/// Loads all original river shapes, not just a guessed line between cells.
pub(super) fn read_rivers(
    reader: &mut Cursor<&[u8]>,
    format: PixelFormat,
) -> Result<Vec<RiverTile>, GameplayError> {
    if reader.position() >= u64::try_from(reader.get_ref().len()).map_or(u64::MAX, |v| v) {
        return Ok(Vec::new());
    }
    let count = read_u16(reader)?;
    let mut rivers = Vec::with_capacity(usize::from(count));
    for _ in 0..count {
        let mut pattern = [0_i16; 10];
        for value in &mut pattern {
            *value = i16::from_le_bytes(read_u16(reader)?.to_le_bytes());
        }
        let length =
            usize::try_from(read_u32(reader)?).map_err(|error| asset_error(error.to_string()))?;
        let image = if length == 0_usize {
            None
        } else {
            Some(decode_tile_rgba(&read_bytes(reader, length)?, format)?)
        };
        rivers.push(RiverTile { pattern, image });
    }
    Ok(rivers)
}

/// Matches source SW/NW/NE/SE connectivity, including beach river mouths.
pub(super) fn river_piece(
    rivers: &[RiverTile],
    neighborhood: [bool; 9],
    beach: bool,
) -> Option<usize> {
    rivers.iter().position(|river| {
        let center = river.pattern[4];
        river.image.is_some()
            && ((beach && center == 2_i16) || (neighborhood[4] && center == 1_i16))
            && [1_usize, 3_usize, 5_usize, 7_usize]
                .into_iter()
                .all(|edge| (river.pattern[edge] != 0_i16) == neighborhood[edge])
    })
}
