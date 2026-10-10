//! Decoder for original CTP2 `.til` terrain tile sets.
//!
//! CTP2 stores isometric terrain in `gtset565.til`/`gtset555.til` using the
//! run-length layout implemented by the original `TileSet` and `BaseTile`
//! classes.  This module decodes the base terrain tiles directly from the
//! installation so the renderer uses the game's real terrain art.

#![expect(
    clippy::little_endian_bytes,
    reason = "The original TIL file format is explicitly little-endian on disk"
)]
#![expect(
    clippy::default_numeric_fallback,
    reason = "Tile decoder constants are fixed original-format pixel dimensions"
)]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use std::io::{Cursor, Read};
use std::path::Path;

use crate::gameplay::{GameplayError, asset_error};

use super::rim::RgbaImage;

/// Original CTP2 tile-set path relative to an installation root.
pub const TILE_SET_565: &str = "ctp2_data/default/graphics/tiles/gtset565.til";
/// Original CTP2 fallback 555 tile-set path relative to an installation root.
pub const TILE_SET_555: &str = "ctp2_data/default/graphics/tiles/gtset555.til";
/// Original CTP2 terrain tile image width.
pub const TILE_PIXEL_WIDTH: u16 = 94;
/// Original CTP2 encoded tile grid height, including headroom rows.
pub const TILE_GRID_HEIGHT: u16 = 72;
/// Original CTP2 visual tile step height.
pub const TILE_PIXEL_HEIGHT: u16 = 48;
/// Original CTP2 tile headroom above the ground diamond.
pub const TILE_PIXEL_HEADROOM: u16 = 24;
/// Empty row table entry.
const EMPTY_TABLE_ENTRY: u16 = 0xFFFF;
/// Copy-run tag kind.
const COPY_RUN_ID: u16 = 0x0C;
/// Skip-run tag kind.
const SKIP_RUN_ID: u16 = 0x0A;
/// End-of-line tag mask.
const EOLN_MASK: u16 = 0xF000;
/// Terrain transition count per pair.
const TRANSITIONS_PER_TILE: usize = 4;
/// Tile transform row length from `TileSet.h`.
const TRANSFORM_SIZE: usize = 15;
/// River transform row length from `TileSet.h`.
const RIVER_TRANSFORM_SIZE: usize = 10;
/// Number of bytes in a mega-tile step.
const MEGA_TILE_STEP_BYTES: usize = 4;

/// One decoded base tile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedTile {
    /// Original tile number.
    pub tile_number: u16,
    /// Terrain base type from the original file.
    pub base_type: u8,
    /// Decoded tile image.
    pub image: RgbaImage,
}

/// Terrain tiles keyed by original tile number and base type.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TileSet {
    /// Tiles by exact tile number.
    pub by_tile_number: BTreeMap<u16, DecodedTile>,
    /// First tile for each terrain base type.
    pub first_by_base_type: BTreeMap<u8, u16>,
}

impl TileSet {
    /// Loads the original terrain tile set from an installation root.
    ///
    /// # Errors
    ///
    /// Returns [`GameplayError`] when neither terrain file can be read or decoded.
    pub fn load_from_installation(installation_root: &Path) -> Result<Self, GameplayError> {
        let primary = installation_root.join(TILE_SET_565);
        match std::fs::read(&primary) {
            Ok(bytes) => Self::parse(&bytes, PixelFormat::Rgb565),
            Err(primary_error) => {
                let fallback = installation_root.join(TILE_SET_555);
                let bytes = std::fs::read(&fallback).map_err(|fallback_error| {
                    asset_error(format!(
                        "failed to read terrain tile sets {} ({primary_error}) and {} ({fallback_error})",
                        primary.display(),
                        fallback.display()
                    ))
                })?;
                Self::parse(&bytes, PixelFormat::Rgb555)
            }
        }
    }

    /// Parses a CTP2 terrain tile set from bytes.
    ///
    /// # Errors
    ///
    /// Returns [`GameplayError`] when the stream is truncated or malformed.
    pub fn parse(bytes: &[u8], format: PixelFormat) -> Result<Self, GameplayError> {
        let mut reader = Cursor::new(bytes);
        skip_transforms(&mut reader)?;
        skip_transitions(&mut reader)?;
        let base_tile_count = read_u32(&mut reader)?;
        let mut by_tile_number = BTreeMap::new();
        let mut first_by_base_type = BTreeMap::new();
        for _ in 0..base_tile_count {
            let tile_number = read_u16(&mut reader)?;
            let base_type = read_u8(&mut reader)?;
            let _flags = read_u8(&mut reader)?;
            let tile_data_len = read_u16(&mut reader)?;
            let tile_data = read_bytes(&mut reader, usize::from(tile_data_len))?;
            let image = decode_tile_rgba(&tile_data, format, base_type)?;
            let hat_data_len = read_u16(&mut reader)?;
            skip_bytes(&mut reader, usize::from(hat_data_len))?;
            first_by_base_type.entry(base_type).or_insert(tile_number);
            by_tile_number.insert(
                tile_number,
                DecodedTile {
                    tile_number,
                    base_type,
                    image,
                },
            );
        }
        skip_river_transforms(&mut reader)?;
        skip_improvements(&mut reader)?;
        skip_mega_tiles(&mut reader)?;
        Ok(Self {
            by_tile_number,
            first_by_base_type,
        })
    }

    /// Returns a decoded tile for an original terrain base type.
    #[must_use]
    pub fn tile_for_base_type(&self, base_type: u8) -> Option<&DecodedTile> {
        let tile_number = self.first_by_base_type.get(&base_type)?;
        self.by_tile_number.get(tile_number)
    }
}

/// Pixel format used by the tile file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// 5 red, 5 green, 5 blue bits.
    Rgb555,
    /// 5 red, 6 green, 5 blue bits.
    Rgb565,
}

/// Decodes one original base-tile image to RGBA.
fn decode_tile_rgba(
    data: &[u8],
    format: PixelFormat,
    base_type: u8,
) -> Result<RgbaImage, GameplayError> {
    let words = words_from_le_bytes(data)?;
    if words.is_empty() {
        return Err(asset_error("empty tile payload".to_owned()));
    }
    if words.len() == raw_base_tile_word_count() {
        return decode_raw_base_tile_rgba(&words, format, base_type);
    }
    let start_row = usize::from(words[0]);
    let end_row = usize::from(
        *words
            .get(1)
            .ok_or_else(|| asset_error("tile row table is truncated".to_owned()))?,
    );
    if start_row >= usize::from(TILE_GRID_HEIGHT) || end_row >= usize::from(TILE_GRID_HEIGHT) {
        return Err(asset_error(
            "tile row table is outside the tile grid".to_owned(),
        ));
    }
    if end_row < start_row {
        return Err(asset_error("tile row table end precedes start".to_owned()));
    }
    let table_len = end_row
        .checked_sub(start_row)
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| asset_error("tile row table length overflow".to_owned()))?;
    if words.len() < table_len.saturating_add(2) {
        return Err(asset_error("tile row table is truncated".to_owned()));
    }
    let table_start = 2_usize;
    let data_start = table_start.saturating_add(table_len);
    let pixel_count = usize::from(TILE_PIXEL_WIDTH)
        .checked_mul(usize::from(TILE_GRID_HEIGHT))
        .ok_or_else(|| asset_error("tile pixel count overflow".to_owned()))?;
    let mut pixels = vec![0_u16; pixel_count];
    for row in start_row..=end_row {
        let table_index = row.saturating_sub(start_row);
        let Some(&row_offset) = words.get(table_start.saturating_add(table_index)) else {
            continue;
        };
        if row_offset == EMPTY_TABLE_ENTRY {
            continue;
        }
        decode_tile_row(
            &words,
            data_start.saturating_add(usize::from(row_offset)),
            row,
            &mut pixels,
        )?;
    }
    rgba_image_from_pixels(&pixels, format)
}

/// Decodes the raw 48-row terrain diamond stored by base tiles.
fn decode_raw_base_tile_rgba(
    words: &[u16],
    format: PixelFormat,
    base_type: u8,
) -> Result<RgbaImage, GameplayError> {
    let pixel_count = usize::from(TILE_PIXEL_WIDTH)
        .checked_mul(usize::from(TILE_GRID_HEIGHT))
        .ok_or_else(|| asset_error("tile pixel count overflow".to_owned()))?;
    let mut pixels = vec![0_u16; pixel_count];
    let mut source_index = 0_usize;
    for row in 0..usize::from(TILE_PIXEL_HEIGHT) {
        let start_x = if row <= 23 {
            23_usize.saturating_sub(row).saturating_mul(2)
        } else {
            row.saturating_sub(24).saturating_mul(2)
        };
        let end_x = usize::from(TILE_PIXEL_WIDTH).saturating_sub(start_x);
        let dest_row = row.saturating_add(usize::from(TILE_PIXEL_HEADROOM));
        for x in start_x..end_x {
            let Some(&source_pixel) = words.get(source_index) else {
                return Err(asset_error("raw base tile is truncated".to_owned()));
            };
            source_index = source_index.saturating_add(1);
            let pixel = if source_pixel <= 3 {
                placeholder_pixel(base_type, format)
            } else {
                source_pixel
            };
            let dest = dest_row
                .checked_mul(usize::from(TILE_PIXEL_WIDTH))
                .and_then(|value| value.checked_add(x))
                .ok_or_else(|| asset_error("raw base tile destination overflow".to_owned()))?;
            if let Some(target) = pixels.get_mut(dest) {
                *target = pixel;
            }
        }
    }
    rgba_image_from_pixels(&pixels, format)
}

/// Returns the word count of one raw 94x48 isometric terrain diamond.
fn raw_base_tile_word_count() -> usize {
    let mut count = 0_usize;
    for row in 0..usize::from(TILE_PIXEL_HEIGHT) {
        let start_x = if row <= 23 {
            23_usize.saturating_sub(row).saturating_mul(2)
        } else {
            row.saturating_sub(24).saturating_mul(2)
        };
        let end_x = usize::from(TILE_PIXEL_WIDTH).saturating_sub(start_x);
        count = count.saturating_add(end_x.saturating_sub(start_x));
    }
    count
}

/// Builds an image from decoded 16-bit pixels.
fn rgba_image_from_pixels(pixels: &[u16], format: PixelFormat) -> Result<RgbaImage, GameplayError> {
    Ok(RgbaImage {
        width: u32::from(TILE_PIXEL_WIDTH),
        height: u32::from(TILE_GRID_HEIGHT),
        pitch: usize::from(TILE_PIXEL_WIDTH)
            .checked_mul(4)
            .ok_or_else(|| asset_error("tile pitch overflow".to_owned()))?,
        data: pixels_to_rgba(pixels, format),
    })
}

/// Provides a same-terrain fill for transition placeholder pixels in raw base tiles.
fn placeholder_pixel(base_type: u8, format: PixelFormat) -> u16 {
    let (red, green, blue) = match base_type {
        4 => (74, 143, 54),
        5 => (190, 165, 92),
        6 => (64, 105, 82),
        8 => (112, 112, 112),
        9 => (118, 103, 74),
        10..=16 | 23 | 24 => (42, 84, 145),
        22 => (40, 104, 42),
        _ => (148, 132, 72),
    };
    rgb_to_format(red, green, blue, format)
}

/// Converts RGB888 into the selected 16-bit format.
fn rgb_to_format(red: u8, green: u8, blue: u8, format: PixelFormat) -> u16 {
    match format {
        PixelFormat::Rgb555 => {
            (u16::from(red >> 3) << 10) | (u16::from(green >> 3) << 5) | u16::from(blue >> 3)
        }
        PixelFormat::Rgb565 => {
            (u16::from(red >> 3) << 11) | (u16::from(green >> 2) << 5) | u16::from(blue >> 3)
        }
    }
}

/// Decodes one tile row from run-length tags.
fn decode_tile_row(
    words: &[u16],
    mut word_index: usize,
    row: usize,
    pixels: &mut [u16],
) -> Result<(), GameplayError> {
    let mut x = 0_usize;
    loop {
        let Some(&tag) = words.get(word_index) else {
            return Err(asset_error("tile row is truncated".to_owned()));
        };
        word_index = word_index.saturating_add(1);
        let run_kind = (tag & 0x0F00) >> 8;
        let run_len = usize::from(tag & 0x00FF);
        match run_kind {
            0 if (tag & EOLN_MASK) != 0 => {}
            SKIP_RUN_ID => x = x.saturating_add(run_len),
            COPY_RUN_ID => {
                for _ in 0..run_len {
                    let Some(&pixel) = words.get(word_index) else {
                        return Err(asset_error("tile copy run is truncated".to_owned()));
                    };
                    word_index = word_index.saturating_add(1);
                    if x < usize::from(TILE_PIXEL_WIDTH) {
                        let dest = row
                            .checked_mul(usize::from(TILE_PIXEL_WIDTH))
                            .and_then(|value| value.checked_add(x))
                            .ok_or_else(|| asset_error("tile destination overflow".to_owned()))?;
                        if let Some(target) = pixels.get_mut(dest) {
                            *target = pixel;
                        }
                    }
                    x = x.saturating_add(1);
                }
            }
            _ => return Err(asset_error(format!("unsupported tile run kind {run_kind}"))),
        }
        if (tag & EOLN_MASK) != 0 || x >= usize::from(TILE_PIXEL_WIDTH) {
            break;
        }
    }
    Ok(())
}

/// Converts 16-bit tile pixels to RGBA.
fn pixels_to_rgba(pixels: &[u16], format: PixelFormat) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(pixels.len().saturating_mul(4));
    for &pixel in pixels {
        if pixel == 0 {
            rgba.extend_from_slice(&[0, 0, 0, 0]);
            continue;
        }
        let (red, green, blue) = match format {
            PixelFormat::Rgb555 => (
                u8::try_from(((pixel >> 10) & 0x1F) << 3).map_or(u8::MAX, |value| value),
                u8::try_from(((pixel >> 5) & 0x1F) << 3).map_or(u8::MAX, |value| value),
                u8::try_from((pixel & 0x1F) << 3).map_or(u8::MAX, |value| value),
            ),
            PixelFormat::Rgb565 => (
                u8::try_from(((pixel >> 11) & 0x1F) << 3).map_or(u8::MAX, |value| value),
                u8::try_from(((pixel >> 5) & 0x3F) << 2).map_or(u8::MAX, |value| value),
                u8::try_from((pixel & 0x1F) << 3).map_or(u8::MAX, |value| value),
            ),
        };
        rgba.extend_from_slice(&[red, green, blue, 255]);
    }
    rgba
}

/// Skips transform table.
fn skip_transforms(reader: &mut Cursor<&[u8]>) -> Result<(), GameplayError> {
    let count = usize::from(read_u16(reader)?);
    skip_bytes(
        reader,
        count.saturating_mul(TRANSFORM_SIZE).saturating_mul(2),
    )
}

/// Skips terrain transition table.
fn skip_transitions(reader: &mut Cursor<&[u8]>) -> Result<(), GameplayError> {
    let count = usize::try_from(read_u32(reader)?)
        .map_err(|error| asset_error(format!("transition count does not fit usize: {error}")))?;
    let size = usize::try_from(read_u32(reader)?)
        .map_err(|error| asset_error(format!("transition size does not fit usize: {error}")))?;
    for _ in 0..count {
        skip_bytes(reader, 4)?;
        skip_bytes(reader, size.saturating_mul(TRANSITIONS_PER_TILE))?;
    }
    Ok(())
}

/// Skips river transform table.
fn skip_river_transforms(reader: &mut Cursor<&[u8]>) -> Result<(), GameplayError> {
    if reader.position() >= u64::try_from(reader.get_ref().len()).map_or(u64::MAX, |value| value) {
        return Ok(());
    }
    let count = usize::from(read_u16(reader)?);
    for _ in 0..count {
        skip_bytes(reader, RIVER_TRANSFORM_SIZE.saturating_mul(2))?;
        let len = usize::try_from(read_u32(reader)?).map_err(|error| {
            asset_error(format!("river image size does not fit usize: {error}"))
        })?;
        skip_bytes(reader, len)?;
    }
    Ok(())
}

/// Skips improvement images.
fn skip_improvements(reader: &mut Cursor<&[u8]>) -> Result<(), GameplayError> {
    if reader.position() >= u64::try_from(reader.get_ref().len()).map_or(u64::MAX, |value| value) {
        return Ok(());
    }
    let count = usize::from(read_u16(reader)?);
    for _ in 0..count {
        skip_bytes(reader, 2)?;
        let len = usize::try_from(read_u32(reader)?).map_err(|error| {
            asset_error(format!("improvement size does not fit usize: {error}"))
        })?;
        skip_bytes(reader, len)?;
    }
    Ok(())
}

/// Skips mega-tile metadata.
fn skip_mega_tiles(reader: &mut Cursor<&[u8]>) -> Result<(), GameplayError> {
    if reader.position() >= u64::try_from(reader.get_ref().len()).map_or(u64::MAX, |value| value) {
        return Ok(());
    }
    let count = usize::from(read_u16(reader)?);
    for _ in 0..count {
        let len = usize::from(read_u16(reader)?);
        skip_bytes(reader, len.saturating_mul(MEGA_TILE_STEP_BYTES))?;
    }
    Ok(())
}

/// Reads all little-endian u16 words from bytes.
fn words_from_le_bytes(data: &[u8]) -> Result<Vec<u16>, GameplayError> {
    if data.len() % 2 != 0 {
        return Err(asset_error(
            "tile payload has an odd byte length".to_owned(),
        ));
    }
    let mut words = Vec::with_capacity(data.len().checked_div(2).map_or(0, |value| value));
    for chunk in data.chunks_exact(2) {
        let bytes = [chunk[0], chunk[1]];
        words.push(u16::from_le_bytes(bytes));
    }
    Ok(words)
}

/// Reads one byte.
fn read_u8(reader: &mut Cursor<&[u8]>) -> Result<u8, GameplayError> {
    let mut byte = [0_u8; 1];
    reader.read_exact(&mut byte).map_err(io_error)?;
    Ok(byte[0])
}

/// Reads one little-endian u16.
fn read_u16(reader: &mut Cursor<&[u8]>) -> Result<u16, GameplayError> {
    let mut bytes = [0_u8; 2];
    reader.read_exact(&mut bytes).map_err(io_error)?;
    Ok(u16::from_le_bytes(bytes))
}

/// Reads one little-endian u32.
fn read_u32(reader: &mut Cursor<&[u8]>) -> Result<u32, GameplayError> {
    let mut bytes = [0_u8; 4];
    reader.read_exact(&mut bytes).map_err(io_error)?;
    Ok(u32::from_le_bytes(bytes))
}

/// Reads a fixed number of bytes.
fn read_bytes(reader: &mut Cursor<&[u8]>, count: usize) -> Result<Vec<u8>, GameplayError> {
    let mut bytes = vec![0_u8; count];
    reader.read_exact(&mut bytes).map_err(io_error)?;
    Ok(bytes)
}

/// Skips a fixed number of bytes.
fn skip_bytes(reader: &mut Cursor<&[u8]>, count: usize) -> Result<(), GameplayError> {
    let target = reader
        .position()
        .checked_add(u64::try_from(count).map_err(|error| {
            asset_error(format!("skip size does not fit stream position: {error}"))
        })?)
        .ok_or_else(|| asset_error("tile stream seek overflow".to_owned()))?;
    if target > u64::try_from(reader.get_ref().len()).map_or(u64::MAX, |value| value) {
        return Err(asset_error("tile stream is truncated".to_owned()));
    }
    reader.set_position(target);
    Ok(())
}

/// Converts an I/O error to a gameplay asset error.
#[expect(
    clippy::needless_pass_by_value,
    reason = "std::io::Error is consumed by map_err adapters and converted into owned gameplay errors"
)]
fn io_error(error: std::io::Error) -> GameplayError {
    asset_error(format!("failed to read tile set: {error}"))
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use std::path::Path;

    use super::{PixelFormat, TILE_GRID_HEIGHT, TILE_PIXEL_WIDTH, TILE_SET_565, TileSet};

    #[test]
    fn parses_minimal_tile_set_and_decodes_copy_run() {
        let mut bytes = Vec::new();
        push_u16(&mut bytes, 0);
        push_u32(&mut bytes, 0);
        push_u32(&mut bytes, 0);
        push_u32(&mut bytes, 1);
        push_u16(&mut bytes, 1);
        bytes.push(4);
        bytes.push(0);
        let tile_payload = encoded_single_pixel_tile();
        push_u16(
            &mut bytes,
            u16::try_from(tile_payload.len()).expect("fixture length should fit u16"),
        );
        bytes.extend_from_slice(&tile_payload);
        push_u16(&mut bytes, 0);
        push_u16(&mut bytes, 0);
        push_u16(&mut bytes, 0);
        push_u16(&mut bytes, 0);

        let tile_set = TileSet::parse(&bytes, PixelFormat::Rgb565).expect("tile set should parse");
        let tile = tile_set
            .tile_for_base_type(4)
            .expect("base type should map to decoded tile");

        assert_eq!(tile.tile_number, 1, "tile number should be preserved");
        assert_eq!(tile.image.width, u32::from(TILE_PIXEL_WIDTH));
        assert_eq!(tile.image.height, u32::from(TILE_GRID_HEIGHT));
        let first_pixel = (usize::from(TILE_GRID_HEIGHT) - 1) * usize::from(TILE_PIXEL_WIDTH) * 4;
        assert_eq!(
            &tile.image.data[first_pixel..first_pixel + 4],
            &[248, 252, 248, 255],
            "565 white pixel should decode to opaque RGBA"
        );
    }

    #[test]
    fn loads_local_ctp2_tile_set_when_available() {
        let installation_root = Path::new("ctp2");
        if !installation_root.join(TILE_SET_565).exists() {
            return;
        }

        let tile_set = TileSet::load_from_installation(installation_root)
            .expect("local CTP2 gtset565.til should decode");

        assert!(
            tile_set.tile_for_base_type(4).is_some(),
            "grassland base tile should be available"
        );
        assert!(
            tile_set.tile_for_base_type(10).is_some(),
            "water base tile should be available"
        );
    }

    /// Builds one encoded tile with a single copied bottom-row pixel.
    fn encoded_single_pixel_tile() -> Vec<u8> {
        let mut bytes = Vec::new();
        push_u16(&mut bytes, TILE_GRID_HEIGHT.saturating_sub(1));
        push_u16(&mut bytes, TILE_GRID_HEIGHT.saturating_sub(1));
        push_u16(&mut bytes, 0);
        push_u16(&mut bytes, 0x0C01);
        push_u16(&mut bytes, 0xFFFF);
        push_u16(&mut bytes, 0xE000);
        bytes
    }

    /// Appends little-endian u16 fixture bytes.
    fn push_u16(bytes: &mut Vec<u8>, value: u16) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    /// Appends little-endian u32 fixture bytes.
    fn push_u32(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
}
