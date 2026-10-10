//! Parser for original CTP2 `.SPR` sprite groups.
//!
//! The implementation follows `ctp2_source/ctp2_code/gfx/spritesys/SpriteFile.cpp`:
//! `SPRF` files store a tag as bytes `FRPS`, a version, optional compression
//! mode, a file type, action offsets, sprite frame payloads, and animation rows.

#![expect(
    clippy::little_endian_bytes,
    reason = "The original SPR file format is explicitly little-endian on disk"
)]
#![expect(
    clippy::arithmetic_side_effects,
    reason = "Sprite decoding mirrors bounded byte-oriented CTP2 stream math with explicit size checks"
)]
#![expect(
    clippy::default_numeric_fallback,
    reason = "Sprite parser constants are fixed binary format field widths"
)]

extern crate alloc;

use alloc::string::ToString;
use alloc::vec::Vec;
use std::io::{Cursor, Read, Seek, SeekFrom};

use crate::gameplay::{GameplayError, asset_error};

/// Version `0x00010003`, original v1.3 layout.
const SPRITEFILE_VERSION0: u32 = 0x0001_0003;
/// Version `0x00020000`, v2.0 layout without explicit LZW header.
const SPRITEFILE_VERSION1: u32 = 0x0002_0000;
/// Version `0x00020001`, v2.0 layout with compression header.
const SPRITEFILE_VERSION2: u32 = 0x0002_0001;
/// The on-disk bytes produced by the original `'SPRF'` multi-character tag.
const SPRITE_TAG_REVERSED: &[u8; 4] = b"FRPS";
/// Alternate tag spelling accepted for hand-built fixtures.
const SPRITE_TAG_FORWARD: &[u8; 4] = b"SPRF";
/// Number of generic game actions in v2 sprite files.
const ACTION_MAX: usize = 16;
/// Number of unit actions stored in shield-point metadata.
const UNIT_ACTION_MAX: usize = 5;
/// Number of facings stored for CTP2 faced sprites.
const NUM_FACINGS: usize = 5;
/// Size of one `POINT` from the original Win32 code.
const POINT_BYTES: u64 = 8;
/// LZW1 header size.
const LZW1_FLAG_BYTES: usize = 4;
/// LZW1 copy flag.
const LZW1_FLAG_COPY: u8 = 1;

/// Original file type enum from `SpriteFile.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteFileType {
    /// Plain sprite.
    Plain,
    /// Faced sprite.
    Faced,
    /// Faced sprite with shadow.
    FacedWithShadow,
    /// Generic group.
    Group,
    /// Unit sprite group.
    Unit,
    /// Projectile sprite group.
    Projectile,
    /// Effect sprite group.
    Effect,
    /// Good/resource sprite group.
    Good,
    /// City sprite group.
    City,
}

impl SpriteFileType {
    /// Converts a raw on-disk type value.
    fn from_raw(value: u32) -> Result<Self, GameplayError> {
        match value {
            0 => Ok(Self::Plain),
            1 => Ok(Self::Faced),
            2 => Ok(Self::FacedWithShadow),
            3 => Ok(Self::Group),
            4 => Ok(Self::Unit),
            5 => Ok(Self::Projectile),
            6 => Ok(Self::Effect),
            7 => Ok(Self::Good),
            8 => Ok(Self::City),
            _ => Err(asset_error(format!("unknown SPR file type {value}"))),
        }
    }
}

/// Compression mode used by frame payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpriteCompression {
    /// Uncompressed/raw copy.
    Regular,
    /// Original CTP2 LZW1 stream.
    Lzw1,
}

impl SpriteCompression {
    /// Converts raw compression value.
    fn from_raw(value: u32) -> Result<Self, GameplayError> {
        match value {
            0 => Ok(Self::Regular),
            1 => Ok(Self::Lzw1),
            _ => Err(asset_error(format!("unknown SPR compression {value}"))),
        }
    }
}

/// Original sprite type stored before one sprite payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteImageKind {
    /// Non-directional sprite.
    Normal,
    /// Five-facing sprite.
    Faced,
}

impl SpriteImageKind {
    /// Converts raw type.
    fn from_raw(value: u16) -> Result<Self, GameplayError> {
        match value {
            0 => Ok(Self::Normal),
            1 => Ok(Self::Faced),
            _ => Err(asset_error(format!("unsupported SPR image kind {value}"))),
        }
    }
}

/// RGBA sprite frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteFrame {
    /// Width in pixels.
    pub width: u16,
    /// Height in pixels.
    pub height: u16,
    /// Facing index, or `0` for normal sprites.
    pub facing: u8,
    /// Frame index in animation order.
    pub frame_index: u16,
    /// RGBA bytes.
    pub rgba: Vec<u8>,
}

/// Animation metadata from one action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteAnimation {
    /// Original animation type.
    pub animation_type: u16,
    /// Frame indices.
    pub frames: Vec<u16>,
    /// Frame movement deltas.
    pub deltas: Vec<(i32, i32)>,
    /// Frame transparencies.
    pub transparencies: Vec<u16>,
    /// Playback time.
    pub playback_time: u16,
    /// Start/end delay.
    pub delay: u16,
}

/// One action in a unit sprite group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteActionAsset {
    /// Action index (`0` move, `1` attack, `2` idle, etc.).
    pub action_index: usize,
    /// Sprite image kind.
    pub image_kind: SpriteImageKind,
    /// Frames for every facing/frame.
    pub frames: Vec<SpriteFrame>,
    /// Animation metadata.
    pub animation: SpriteAnimation,
}

/// Parsed sprite group asset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteGroupAsset {
    /// File version.
    pub version: u32,
    /// File type.
    pub file_type: SpriteFileType,
    /// Parsed actions.
    pub actions: Vec<SpriteActionAsset>,
    /// Whether the unit has death animation metadata.
    pub has_death: bool,
    /// Whether the unit has directional attack metadata.
    pub has_directional: bool,
}

impl SpriteGroupAsset {
    /// Parses a full sprite group from bytes.
    ///
    /// # Errors
    ///
    /// Returns [`GameplayError::Asset`] if the bytes are not a supported CTP2 sprite.
    pub fn parse(bytes: &[u8]) -> Result<Self, GameplayError> {
        let mut reader = Cursor::new(bytes);
        let header = SpriteHeader::read(&mut reader)?;
        if header.file_type != SpriteFileType::Unit {
            return Err(asset_error(format!(
                "only unit SPR groups are currently supported, got {:?}",
                header.file_type
            )));
        }
        let mut actions = match header.version {
            SPRITEFILE_VERSION0 => read_unit_v13(&mut reader, header.compression)?,
            SPRITEFILE_VERSION1 | SPRITEFILE_VERSION2 => {
                read_unit_v20(&mut reader, header.compression)?
            }
            _ => {
                return Err(asset_error(format!(
                    "unsupported SPR version {:08x}",
                    header.version
                )));
            }
        };
        actions.sort_by_key(|action| action.action_index);
        let (has_death, has_directional) = read_unit_tail(&mut reader).unwrap_or((false, false));
        Ok(Self {
            version: header.version,
            file_type: header.file_type,
            actions,
            has_death,
            has_directional,
        })
    }

    /// Returns total loaded frame count.
    #[must_use]
    pub fn frame_count(&self) -> usize {
        self.actions.iter().map(|action| action.frames.len()).sum()
    }

    /// Returns whether an action exists.
    #[must_use]
    pub fn has_action(&self, action_index: usize) -> bool {
        self.actions
            .iter()
            .any(|action| action.action_index == action_index)
    }
}

/// Parsed file header.
struct SpriteHeader {
    /// File version.
    version: u32,
    /// Compression.
    compression: SpriteCompression,
    /// File type.
    file_type: SpriteFileType,
}

impl SpriteHeader {
    /// Reads a file header.
    fn read(reader: &mut Cursor<&[u8]>) -> Result<Self, GameplayError> {
        let mut tag = [0_u8; 4];
        reader.read_exact(&mut tag).map_err(io_error)?;
        if &tag != SPRITE_TAG_REVERSED && &tag != SPRITE_TAG_FORWARD {
            return Err(asset_error("invalid SPR tag".to_owned()));
        }
        let version = read_u32(reader)?;
        let compression = if version == SPRITEFILE_VERSION2 {
            SpriteCompression::from_raw(read_u32(reader)?)?
        } else {
            SpriteCompression::Regular
        };
        let file_type = SpriteFileType::from_raw(read_u32(reader)?)?;
        Ok(Self {
            version,
            compression,
            file_type,
        })
    }
}

/// Reads a v1.3 unit group.
fn read_unit_v13(
    reader: &mut Cursor<&[u8]>,
    compression: SpriteCompression,
) -> Result<Vec<SpriteActionAsset>, GameplayError> {
    skip_bytes(
        reader,
        u64::try_from(UNIT_ACTION_MAX).map_or(0, |count| count.saturating_mul(4)),
    )?;
    let mut actions = Vec::new();
    for action_index in 0..UNIT_ACTION_MAX {
        let present = read_u32(reader)? != 0;
        if present {
            actions.push(read_sprite_action(reader, compression, action_index)?);
        }
    }
    Ok(actions)
}

/// Reads a v2 unit group.
fn read_unit_v20(
    reader: &mut Cursor<&[u8]>,
    compression: SpriteCompression,
) -> Result<Vec<SpriteActionAsset>, GameplayError> {
    let mut offsets = Vec::with_capacity(ACTION_MAX.saturating_add(1));
    for _ in 0..=ACTION_MAX {
        offsets.push(read_i32(reader)?);
    }
    let mut actions = Vec::new();
    for (action_index, offset) in offsets.iter().take(ACTION_MAX).enumerate() {
        if *offset > 0 {
            actions.push(read_sprite_action(reader, compression, action_index)?);
        }
    }
    Ok(actions)
}

/// Reads the shared unit tail, if enough bytes remain.
fn read_unit_tail(reader: &mut Cursor<&[u8]>) -> Result<(bool, bool), GameplayError> {
    skip_bytes(
        reader,
        u64::try_from(UNIT_ACTION_MAX)
            .map_or(0, |actions| {
                actions.saturating_mul(u64::try_from(NUM_FACINGS).map_or(0, |facings| facings))
            })
            .saturating_mul(POINT_BYTES),
    )?;
    let has_death = read_u16(reader)? != 0;
    let has_directional = read_u16(reader)? != 0;
    Ok((has_death, has_directional))
}

/// Reads one sprite+animation action.
fn read_sprite_action(
    reader: &mut Cursor<&[u8]>,
    compression: SpriteCompression,
    action_index: usize,
) -> Result<SpriteActionAsset, GameplayError> {
    let image_kind = SpriteImageKind::from_raw(read_u16(reader)?)?;
    let frames = match image_kind {
        SpriteImageKind::Normal => read_normal_sprite(reader, compression)?,
        SpriteImageKind::Faced => read_faced_sprite(reader, compression)?,
    };
    let animation = read_animation(reader)?;
    Ok(SpriteActionAsset {
        action_index,
        image_kind,
        frames,
        animation,
    })
}

/// Reads a normal sprite frame set.
fn read_normal_sprite(
    reader: &mut Cursor<&[u8]>,
    compression: SpriteCompression,
) -> Result<Vec<SpriteFrame>, GameplayError> {
    let width = read_u16(reader)?;
    let height = read_u16(reader)?;
    skip_bytes(reader, 8)?;
    let _first_frame = read_u16(reader)?;
    let num_frames = read_u16(reader)?;
    let sprite_sizes = read_u32_vec(reader, usize::from(num_frames))?;
    let mini_sizes = read_u32_vec(reader, usize::from(num_frames))?;
    let mut frames = Vec::new();
    for (frame_index, size) in sprite_sizes.into_iter().enumerate() {
        let rgba = read_frame_rgba(reader, compression, size, width, height)?;
        frames.push(SpriteFrame {
            width,
            height,
            facing: 0,
            frame_index: u16::try_from(frame_index).map_or(u16::MAX, |value| value),
            rgba,
        });
    }
    skip_mini_frames(reader, mini_sizes)?;
    Ok(frames)
}

/// Reads a five-facing sprite frame set.
fn read_faced_sprite(
    reader: &mut Cursor<&[u8]>,
    compression: SpriteCompression,
) -> Result<Vec<SpriteFrame>, GameplayError> {
    let width = read_u16(reader)?;
    let height = read_u16(reader)?;
    skip_bytes(
        reader,
        u64::try_from(NUM_FACINGS).map_or(0, |facings| facings.saturating_mul(POINT_BYTES)),
    )?;
    let _first_frame = read_u16(reader)?;
    let num_frames = read_u16(reader)?;
    let frame_count = usize::from(num_frames);
    let mut sprite_sizes = Vec::new();
    let mut mini_sizes = Vec::new();
    for _ in 0..NUM_FACINGS {
        sprite_sizes.push(read_u32_vec(reader, frame_count)?);
        mini_sizes.push(read_u32_vec(reader, frame_count)?);
    }
    let mut frames = Vec::new();
    for facing in 0..NUM_FACINGS {
        for frame_index in 0..frame_count {
            let size = sprite_sizes
                .get(facing)
                .and_then(|row| row.get(frame_index))
                .copied()
                .ok_or_else(|| asset_error("faced SPR size table is incomplete".to_owned()))?;
            let rgba = read_frame_rgba(reader, compression, size, width, height)?;
            frames.push(SpriteFrame {
                width,
                height,
                facing: u8::try_from(facing).map_or(u8::MAX, |value| value),
                frame_index: u16::try_from(frame_index).map_or(u16::MAX, |value| value),
                rgba,
            });
        }
        if let Some(row) = mini_sizes.get(facing) {
            skip_mini_frames(reader, row.clone())?;
        }
    }
    Ok(frames)
}

/// Reads and decodes one full-size frame.
fn read_frame_rgba(
    reader: &mut Cursor<&[u8]>,
    compression: SpriteCompression,
    compressed_size: u32,
    width: u16,
    height: u16,
) -> Result<Vec<u8>, GameplayError> {
    let actual_size = if compression == SpriteCompression::Lzw1 {
        read_u32(reader)?
    } else {
        compressed_size
    };
    let bytes = read_bytes(
        reader,
        usize::try_from(compressed_size).map_err(|error| {
            asset_error(format!(
                "compressed SPR frame size does not fit usize: {error}"
            ))
        })?,
    )?;
    let raw = decompress_frame(
        &bytes,
        compression,
        usize::try_from(actual_size).map_err(|error| {
            asset_error(format!("actual SPR frame size does not fit usize: {error}"))
        })?,
    )?;
    rgba_from_565(&raw, width, height)
}

/// Skips mini-frame payloads.
fn skip_mini_frames(reader: &mut Cursor<&[u8]>, sizes: Vec<u32>) -> Result<(), GameplayError> {
    for size in sizes {
        skip_bytes(reader, u64::from(size))?;
    }
    Ok(())
}

/// Reads animation metadata.
fn read_animation(reader: &mut Cursor<&[u8]>) -> Result<SpriteAnimation, GameplayError> {
    let animation_type = read_u16(reader)?;
    let num_frames = usize::from(read_u16(reader)?);
    let playback_time = read_u16(reader)?;
    let delay = read_u16(reader)?;
    let frames = read_u16_vec(reader, num_frames)?;
    let mut deltas = Vec::with_capacity(num_frames);
    for _ in 0..num_frames {
        deltas.push((read_i32(reader)?, read_i32(reader)?));
    }
    let transparencies = read_u16_vec(reader, num_frames)?;
    Ok(SpriteAnimation {
        animation_type,
        frames,
        deltas,
        transparencies,
        playback_time,
        delay,
    })
}

/// Decompresses a frame payload.
fn decompress_frame(
    bytes: &[u8],
    compression: SpriteCompression,
    actual_size: usize,
) -> Result<Vec<u8>, GameplayError> {
    match compression {
        SpriteCompression::Regular => Ok(bytes.to_vec()),
        SpriteCompression::Lzw1 => decompress_lzw1(bytes, actual_size),
    }
}

/// Decompresses original CTP2 LZW1 data.
fn decompress_lzw1(bytes: &[u8], actual_size: usize) -> Result<Vec<u8>, GameplayError> {
    if bytes.len() < LZW1_FLAG_BYTES {
        return Err(asset_error(
            "LZW1 stream is shorter than its header".to_owned(),
        ));
    }
    if bytes[0] == LZW1_FLAG_COPY {
        return Ok(bytes[LZW1_FLAG_BYTES..].to_vec());
    }
    let mut output = Vec::with_capacity(actual_size);
    let mut source_index = LZW1_FLAG_BYTES;
    let mut control = 0_u16;
    let mut control_bits = 0_u8;
    while source_index < bytes.len() {
        if control_bits == 0 {
            if source_index.saturating_add(2) > bytes.len() {
                break;
            }
            control = u16::from(bytes[source_index]) | (u16::from(bytes[source_index + 1]) << 8);
            source_index = source_index.saturating_add(2);
            control_bits = 16;
        }
        if (control & 1) != 0 {
            if source_index.saturating_add(2) > bytes.len() {
                return Err(asset_error("truncated LZW1 back-reference".to_owned()));
            }
            let high = bytes[source_index];
            source_index = source_index.saturating_add(1);
            let low = bytes[source_index];
            source_index = source_index.saturating_add(1);
            let offset = usize::from((u16::from(high & 0xF0) << 4) | u16::from(low));
            let length = usize::from((high & 0x0F).saturating_add(1));
            if offset == 0 || offset > output.len() {
                return Err(asset_error("invalid LZW1 back-reference offset".to_owned()));
            }
            let start = output.len().saturating_sub(offset);
            for index in 0..length {
                let value = *output
                    .get(start.saturating_add(index))
                    .ok_or_else(|| asset_error("invalid LZW1 overlap copy".to_owned()))?;
                output.push(value);
            }
        } else if let Some(value) = bytes.get(source_index) {
            output.push(*value);
            source_index = source_index.saturating_add(1);
        }
        control >>= 1;
        control_bits = control_bits.saturating_sub(1);
    }
    Ok(output)
}

mod frame;

use frame::rgba_from_565;

/// Reads a vector of u32 values.
fn read_u32_vec(reader: &mut Cursor<&[u8]>, count: usize) -> Result<Vec<u32>, GameplayError> {
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(read_u32(reader)?);
    }
    Ok(values)
}

/// Reads a vector of u16 values.
fn read_u16_vec(reader: &mut Cursor<&[u8]>, count: usize) -> Result<Vec<u16>, GameplayError> {
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(read_u16(reader)?);
    }
    Ok(values)
}

/// Reads bytes from the cursor.
fn read_bytes(reader: &mut Cursor<&[u8]>, count: usize) -> Result<Vec<u8>, GameplayError> {
    let mut bytes = vec![0_u8; count];
    reader.read_exact(&mut bytes).map_err(io_error)?;
    Ok(bytes)
}

/// Skips bytes in the cursor.
fn skip_bytes(reader: &mut Cursor<&[u8]>, count: u64) -> Result<(), GameplayError> {
    let position = reader.position().saturating_add(count);
    reader.seek(SeekFrom::Start(position)).map_err(io_error)?;
    Ok(())
}

/// Reads little-endian u16.
fn read_u16(reader: &mut Cursor<&[u8]>) -> Result<u16, GameplayError> {
    let mut bytes = [0_u8; 2];
    reader.read_exact(&mut bytes).map_err(io_error)?;
    Ok(u16::from_le_bytes(bytes))
}

/// Reads little-endian u32.
fn read_u32(reader: &mut Cursor<&[u8]>) -> Result<u32, GameplayError> {
    let mut bytes = [0_u8; 4];
    reader.read_exact(&mut bytes).map_err(io_error)?;
    Ok(u32::from_le_bytes(bytes))
}

/// Reads little-endian i32.
fn read_i32(reader: &mut Cursor<&[u8]>) -> Result<i32, GameplayError> {
    let mut bytes = [0_u8; 4];
    reader.read_exact(&mut bytes).map_err(io_error)?;
    Ok(i32::from_le_bytes(bytes))
}

/// Converts an I/O error into an asset error.
#[expect(
    clippy::needless_pass_by_value,
    reason = "std::io::Error is consumed by map_err adapters and converted into owned gameplay errors"
)]
fn io_error(error: std::io::Error) -> GameplayError {
    asset_error(error.to_string())
}

#[cfg(test)]
mod tests;
