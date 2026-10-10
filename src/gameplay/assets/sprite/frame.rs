//! Decoding for original CTP2 encoded sprite frame payloads.

#![expect(
    clippy::little_endian_bytes,
    reason = "The original SPR frame format is explicitly little-endian on disk"
)]

extern crate alloc;

use alloc::vec::Vec;

use crate::gameplay::{GameplayError, asset_error};

/// Transparent skip run in encoded sprite frames.
const SPRITE_CHROMAKEY_RUN_ID: u16 = 0x0A;
/// Copy run in encoded sprite frames.
const SPRITE_COPY_RUN_ID: u16 = 0x0C;
/// Shadow run in encoded sprite frames.
const SPRITE_SHADOW_RUN_ID: u16 = 0x0E;
/// Feathered one-pixel run in encoded sprite frames.
const SPRITE_FEATHERED_RUN_ID: u16 = 0x0F;
/// End-of-line high nibble in encoded sprite frames.
const SPRITE_EOLN_MASK: u16 = 0xF000;
/// Empty row table value.
const SPRITE_EMPTY_TABLE_ENTRY: u16 = 0xFFFF;

/// Converts little-endian 565 sprite frame data to RGBA.
pub(super) fn rgba_from_565(
    bytes: &[u8],
    width: u16,
    height: u16,
) -> Result<Vec<u8>, GameplayError> {
    if looks_like_encoded_sprite(bytes, height) || bytes.len() < raw_frame_byte_len(width, height)?
    {
        return encoded_sprite_rgba_from_565(bytes, width, height);
    }
    raw_rgba_from_565(bytes, width, height)
}

/// Converts raw little-endian 565 pixels to RGBA.
fn raw_rgba_from_565(bytes: &[u8], width: u16, height: u16) -> Result<Vec<u8>, GameplayError> {
    let expected = raw_frame_byte_len(width, height)?;
    if bytes.len() < expected {
        return Err(asset_error(format!(
            "SPR frame is too short: expected {expected} bytes, got {}",
            bytes.len()
        )));
    }
    let mut rgba = Vec::with_capacity(expected.saturating_mul(2));
    for pixel_bytes in bytes[..expected].chunks_exact(2) {
        let pixel = u16::from(pixel_bytes[0]) | (u16::from(pixel_bytes[1]) << 8);
        rgba.extend_from_slice(&pixel_to_rgba_565(pixel, if pixel == 0 { 0 } else { 255 }));
    }
    Ok(rgba)
}

/// Converts original run-length encoded 565 sprite data to RGBA.
fn encoded_sprite_rgba_from_565(
    bytes: &[u8],
    width: u16,
    height: u16,
) -> Result<Vec<u8>, GameplayError> {
    let words = words_from_le_bytes(bytes)?;
    if words.first().copied() != Some(height) {
        return Err(asset_error(
            "encoded SPR frame height header is invalid".to_owned(),
        ));
    }
    let table_start = 1_usize;
    let data_start = table_start.saturating_add(usize::from(height));
    if words.len() < data_start {
        return Err(asset_error("encoded SPR row table is truncated".to_owned()));
    }
    let pixel_count = usize::from(width)
        .checked_mul(usize::from(height))
        .ok_or_else(|| asset_error("encoded SPR dimensions overflow".to_owned()))?;
    let mut rgba = vec![0_u8; pixel_count.saturating_mul(4)];
    for row in 0..usize::from(height) {
        let Some(&row_offset) = words.get(table_start.saturating_add(row)) else {
            continue;
        };
        if row_offset == SPRITE_EMPTY_TABLE_ENTRY {
            continue;
        }
        decode_encoded_sprite_row(
            &words,
            data_start.saturating_add(usize::from(row_offset)),
            row,
            width,
            &mut rgba,
        )?;
    }
    Ok(rgba)
}

/// Decodes one encoded sprite row.
fn decode_encoded_sprite_row(
    words: &[u16],
    mut word_index: usize,
    row: usize,
    width: u16,
    rgba: &mut [u8],
) -> Result<(), GameplayError> {
    let mut x = 0_usize;
    loop {
        let Some(&tag) = words.get(word_index) else {
            return Err(asset_error("encoded SPR row is truncated".to_owned()));
        };
        word_index = word_index.saturating_add(1);
        let run_kind = (tag & 0x0F00) >> 8;
        let run_len = usize::from(tag & 0x00FF);
        match run_kind {
            0 if (tag & SPRITE_EOLN_MASK) != 0 => {}
            SPRITE_CHROMAKEY_RUN_ID => x = x.saturating_add(run_len),
            SPRITE_COPY_RUN_ID => {
                for _ in 0..run_len {
                    let Some(&pixel) = words.get(word_index) else {
                        return Err(asset_error("encoded SPR copy run is truncated".to_owned()));
                    };
                    word_index = word_index.saturating_add(1);
                    set_rgba_pixel(rgba, width, row, x, pixel, 255)?;
                    x = x.saturating_add(1);
                }
            }
            SPRITE_SHADOW_RUN_ID => {
                for _ in 0..run_len {
                    set_shadow_pixel(rgba, width, row, x)?;
                    x = x.saturating_add(1);
                }
            }
            SPRITE_FEATHERED_RUN_ID => {
                let Some(&pixel) = words.get(word_index) else {
                    return Err(asset_error(
                        "encoded SPR feather run is truncated".to_owned(),
                    ));
                };
                word_index = word_index.saturating_add(1);
                let alpha = u8::try_from((tag & 0x00FF).min(0x00FF)).map_or(u8::MAX, |value| value);
                set_rgba_pixel(rgba, width, row, x, pixel, alpha)?;
                x = x.saturating_add(1);
            }
            _ => return Err(asset_error(format!("unsupported SPR run kind {run_kind}"))),
        }
        if (tag & SPRITE_EOLN_MASK) != 0 || x >= usize::from(width) {
            break;
        }
    }
    Ok(())
}

/// Sets one decoded RGBA pixel.
fn set_rgba_pixel(
    rgba: &mut [u8],
    width: u16,
    row: usize,
    x: usize,
    pixel: u16,
    alpha: u8,
) -> Result<(), GameplayError> {
    if x >= usize::from(width) {
        return Ok(());
    }
    let offset = row
        .checked_mul(usize::from(width))
        .and_then(|value| value.checked_add(x))
        .and_then(|value| value.checked_mul(4))
        .ok_or_else(|| asset_error("encoded SPR destination overflow".to_owned()))?;
    let color = pixel_to_rgba_565(pixel, alpha);
    let end = offset.saturating_add(4);
    if let Some(target) = rgba.get_mut(offset..end) {
        target.copy_from_slice(&color);
    }
    Ok(())
}

/// Sets one shadow pixel.
fn set_shadow_pixel(
    rgba: &mut [u8],
    width: u16,
    row: usize,
    x: usize,
) -> Result<(), GameplayError> {
    if x >= usize::from(width) {
        return Ok(());
    }
    let offset = row
        .checked_mul(usize::from(width))
        .and_then(|value| value.checked_add(x))
        .and_then(|value| value.checked_mul(4))
        .ok_or_else(|| asset_error("encoded SPR shadow destination overflow".to_owned()))?;
    let end = offset.saturating_add(4);
    if let Some(target) = rgba.get_mut(offset..end) {
        target.copy_from_slice(&[0, 0, 0, 120]);
    }
    Ok(())
}

/// Returns true when bytes carry an encoded sprite row table.
fn looks_like_encoded_sprite(bytes: &[u8], height: u16) -> bool {
    bytes
        .get(0..2)
        .is_some_and(|header| u16::from_le_bytes([header[0], header[1]]) == height)
}

/// Returns raw frame byte length.
fn raw_frame_byte_len(width: u16, height: u16) -> Result<usize, GameplayError> {
    usize::from(width)
        .checked_mul(usize::from(height))
        .and_then(|pixels| pixels.checked_mul(2))
        .ok_or_else(|| asset_error("SPR frame dimensions overflow".to_owned()))
}

/// Converts one 565 pixel to RGBA.
fn pixel_to_rgba_565(pixel: u16, alpha: u8) -> [u8; 4] {
    let red5 = (pixel >> 11) & 0x1F;
    let green6 = (pixel >> 5) & 0x3F;
    let blue5 = pixel & 0x1F;
    [
        u8::try_from(red5 << 3).map_or(u8::MAX, |value| value),
        u8::try_from(green6 << 2).map_or(u8::MAX, |value| value),
        u8::try_from(blue5 << 3).map_or(u8::MAX, |value| value),
        if pixel == 0 { 0 } else { alpha },
    ]
}

/// Reads all little-endian words from a byte slice.
fn words_from_le_bytes(bytes: &[u8]) -> Result<Vec<u16>, GameplayError> {
    if bytes.len() % 2 != 0 {
        return Err(asset_error("SPR frame has an odd byte length".to_owned()));
    }
    let mut words = Vec::with_capacity(bytes.len().checked_div(2).map_or(0, |value| value));
    for chunk in bytes.chunks_exact(2) {
        words.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    Ok(words)
}
