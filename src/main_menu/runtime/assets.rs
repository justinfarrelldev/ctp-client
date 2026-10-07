//! Original CTP2 RIM/ZFS image loading helpers for menu rendering.

extern crate alloc;

use alloc::vec::Vec;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use sdl2::render::{BlendMode, Texture, TextureCreator};
use sdl2::video::WindowContext;
use unciv::ZfsFile;

use crate::main_menu::{MainMenuError, MenuAssetReference};

use super::{MENU_TEXTURE_FORMAT, runtime_error};

/// Image decoded from a RIM entry into RGBA bytes.
pub(super) struct RgbaImage {
    /// Width in pixels.
    pub(super) width: u32,
    /// Height in pixels.
    pub(super) height: u32,
    /// Bytes per image row.
    pub(super) pitch: usize,
    /// Pixel data in RGBA order.
    pub(super) data: Vec<u8>,
}

/// Creates an SDL texture from RGBA bytes.
pub(super) fn create_texture_from_rgba<'texture>(
    texture_creator: &'texture TextureCreator<WindowContext>,
    image: &RgbaImage,
) -> Result<Texture<'texture>, MainMenuError> {
    let mut texture = texture_creator
        .create_texture_streaming(MENU_TEXTURE_FORMAT, image.width, image.height)
        .map_err(|error| runtime_error(error.to_string()))?;
    texture
        .update(None, &image.data, image.pitch)
        .map_err(|error| runtime_error(error.to_string()))?;
    texture.set_blend_mode(BlendMode::Blend);
    Ok(texture)
}

/// Loads and converts a RIM image from a ZFS archive without copying assets to the repository.
pub(super) fn load_rim_rgba(
    installation_root: &Path,
    asset: &MenuAssetReference,
) -> Result<RgbaImage, MainMenuError> {
    load_rim_rgba_entry(
        installation_root,
        asset.archive_relative_path,
        asset.entry_name,
        None,
    )
}

/// Loads one RIM entry as RGBA bytes, optionally applying a chroma key.
pub(super) fn load_rim_rgba_entry(
    installation_root: &Path,
    archive_relative_path: &str,
    entry_name: &str,
    chroma_key: Option<(u8, u8, u8)>,
) -> Result<RgbaImage, MainMenuError> {
    let archive_path = installation_root.join(archive_relative_path);
    let file = File::open(&archive_path).map_err(|error| {
        runtime_error(format!(
            "failed to open {}: {error}",
            archive_path.display()
        ))
    })?;
    let mut reader = BufReader::new(file);
    let zfs = ZfsFile::from_stream(&mut reader).map_err(|error| {
        runtime_error(format!(
            "failed to parse {}: {error}",
            archive_path.display()
        ))
    })?;
    let entry = zfs
        .files
        .iter()
        .find(|candidate| candidate.name.eq_ignore_ascii_case(entry_name))
        .cloned()
        .ok_or_else(|| {
            runtime_error(format!(
                "{} was not found in {}",
                entry_name,
                archive_path.display()
            ))
        })?;
    let rim = entry.read_rim_image(&mut reader).map_err(|error| {
        runtime_error(format!(
            "failed to decode {} from {}: {error}",
            entry_name,
            archive_path.display()
        ))
    })?;
    let width = u32::from(rim.width);
    let height = u32::from(rim.height);
    let pitch = usize::from(rim.width)
        .checked_mul(4)
        .ok_or_else(|| runtime_error("RIM pitch overflow".to_owned()))?;

    Ok(RgbaImage {
        width,
        height,
        pitch,
        data: rim_rgba_bytes_with_chroma_key(&rim, chroma_key),
    })
}

/// Converts RIM image pixels to RGBA and applies optional chroma transparency.
fn rim_rgba_bytes_with_chroma_key(
    rim: &unciv::RimImage,
    chroma_key: Option<(u8, u8, u8)>,
) -> Vec<u8> {
    let mut data = rim.to_rgba_bytes();
    if let Some((key_red, key_green, key_blue)) = chroma_key {
        apply_chroma_key_alpha(&mut data, key_red, key_green, key_blue);
    }
    data
}

/// Applies transparent alpha to pixels matching the supplied RGB chroma key.
fn apply_chroma_key_alpha(data: &mut [u8], key_red: u8, key_green: u8, key_blue: u8) {
    for pixel in data.chunks_exact_mut(4) {
        if pixel.first() == Some(&key_red)
            && pixel.get(1) == Some(&key_green)
            && pixel.get(2) == Some(&key_blue)
        {
            if let Some(alpha) = pixel.get_mut(3) {
                *alpha = 0;
            }
        }
    }
}
