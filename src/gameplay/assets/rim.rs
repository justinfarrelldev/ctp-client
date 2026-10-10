//! RIM/ZFS image loading for original CTP2 in-game art.

extern crate alloc;

use alloc::vec::Vec;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use unciv::ZfsFile;

use crate::gameplay::{GameplayError, asset_error};

/// Image decoded into RGBA bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbaImage {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Bytes per row.
    pub pitch: usize,
    /// RGBA pixels.
    pub data: Vec<u8>,
}

/// Loads one RIM image from a CTP2 ZFS archive.
///
/// # Errors
///
/// Returns [`GameplayError`] when the archive or entry cannot be read.
pub fn load_rim_rgba_entry(
    installation_root: &Path,
    archive_relative_path: &str,
    entry_name: &str,
    chroma_key: Option<(u8, u8, u8)>,
) -> Result<RgbaImage, GameplayError> {
    let archive_path = installation_root.join(archive_relative_path);
    let file = File::open(&archive_path).map_err(|error| {
        asset_error(format!(
            "failed to open RIM archive {}: {error}",
            archive_path.display()
        ))
    })?;
    let mut reader = BufReader::new(file);
    let zfs = ZfsFile::from_stream(&mut reader).map_err(|error| {
        asset_error(format!(
            "failed to parse RIM archive {}: {error}",
            archive_path.display()
        ))
    })?;
    let entry = zfs
        .files
        .iter()
        .find(|candidate| candidate.name.eq_ignore_ascii_case(entry_name))
        .cloned()
        .ok_or_else(|| {
            asset_error(format!(
                "{entry_name} was not found in {}",
                archive_path.display()
            ))
        })?;
    let rim = entry.read_rim_image(&mut reader).map_err(|error| {
        asset_error(format!(
            "failed to decode {entry_name} from {}: {error}",
            archive_path.display()
        ))
    })?;
    let mut data = rim.to_rgba_bytes();
    if let Some((key_red, key_green, key_blue)) = chroma_key {
        apply_chroma_key_alpha(&mut data, key_red, key_green, key_blue);
    }
    Ok(RgbaImage {
        width: u32::from(rim.width),
        height: u32::from(rim.height),
        pitch: usize::from(rim.width)
            .checked_mul(4)
            .ok_or_else(|| asset_error("RIM pitch overflow".to_owned()))?,
        data,
    })
}

/// Applies transparent alpha to pixels matching the supplied chroma key.
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::gameplay::assets::PICTURE_ARCHIVE;

    use super::{apply_chroma_key_alpha, load_rim_rgba_entry};

    #[test]
    fn chroma_key_makes_matching_pixels_transparent() {
        let mut data = vec![248_u8, 0, 248, 255, 1, 2, 3, 255];

        apply_chroma_key_alpha(&mut data, 248, 0, 248);

        assert_eq!(
            data[3], 0,
            "matching chroma pixel should become transparent"
        );
        assert_eq!(data[7], 255, "non-matching pixel should remain opaque");
    }

    #[test]
    fn loads_local_ctp2_control_panel_rim_when_available() {
        let installation_root = Path::new("ctp2");
        if !installation_root.join(PICTURE_ARCHIVE).exists() {
            return;
        }

        let image = load_rim_rgba_entry(
            installation_root,
            PICTURE_ARCHIVE,
            "upfg01.rim",
            Some((248, 0, 248)),
        )
        .expect("local CTP2 control panel RIM should decode");

        assert!(image.width > 0, "decoded RIM width should be non-zero");
        assert!(image.height > 0, "decoded RIM height should be non-zero");
    }
}
