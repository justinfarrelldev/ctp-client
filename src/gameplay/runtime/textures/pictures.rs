//! Original loose TGA precedence with one case-insensitive installation inventory.
//!
//! CivPaths::FindFile checks localized/default loose art before project archives.
//! Original Windows lookup is case-insensitive: Linux must not mistake a real
//! mixed-case TGA for missing RIM art. Only filenames are indexed; decoded/uploaded
//! images remain in GameplayTextures' existing caches. No directory scans occur
//! on painting, hover, or a newly encountered management picture.

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::collections::btree_map::Entry;
use alloc::vec::Vec;
use sdl2::image::LoadSurface;
use sdl2::pixels::PixelFormatEnum;
use sdl2::surface::Surface;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::gameplay::assets::PICTURE_ARCHIVE;
use crate::gameplay::assets::rim::{RgbaImage, load_rim_rgba_entry};
use crate::gameplay::{GameplayError, asset_error};

#[cfg(test)]
mod tests;

/// Current English UI's original localized-then-default picture search paths.
const DIRECTORIES: [&str; 2] = [
    "ctp2_data/english/graphics/pictures",
    "ctp2_data/default/graphics/pictures",
];
/// Bounds startup directory enumeration, including unrelated installation files.
const MAX_FILES: usize = 0x4000_usize;

/// Immutable per-installation loose-picture index, never authoritative game data.
#[derive(Default)]
pub(super) struct PictureSources {
    /// Same read-only installation as the enclosing texture cache.
    root: PathBuf,
    /// First source layer's actual filename; None marks an ambiguous case collision.
    /// A missing key alone means no loose source. Never select an arbitrary neighbor.
    loose: BTreeMap<String, Option<PathBuf>>,
}

impl PictureSources {
    /// Enumerates each original source folder once; absent optional folders are OK.
    pub(super) fn discover(root: &Path) -> Result<Self, GameplayError> {
        Self::from_reader(root, read_names)
    }

    /// Injected directory listings keep tests entirely fileless.
    fn from_reader(
        root: &Path,
        mut read: impl FnMut(&Path) -> std::io::Result<Vec<String>>,
    ) -> Result<Self, GameplayError> {
        let mut sources = Self {
            root: root.to_path_buf(),
            loose: BTreeMap::new(),
        };
        for relative in DIRECTORIES {
            let directory = root.join(relative);
            let names = match read(&directory) {
                Ok(names) => names,
                Err(error) if error.kind() == ErrorKind::NotFound => Vec::new(),
                Err(error) => {
                    return Err(asset_error(format!(
                        "UI picture directory {}: {error}",
                        directory.display()
                    )));
                }
            };
            if names.len() > MAX_FILES {
                return Err(asset_error(format!(
                    "UI picture directory {} exceeds file limit {MAX_FILES}",
                    directory.display()
                )));
            }
            let mut layer = BTreeMap::new();
            for name in names {
                if !safe_name(&name) || !name.to_ascii_lowercase().ends_with(".tga") {
                    continue;
                }
                match layer.entry(name.to_ascii_lowercase()) {
                    Entry::Vacant(slot) => {
                        slot.insert(Some(directory.join(name)));
                    }
                    Entry::Occupied(mut slot) => {
                        slot.insert(None);
                    }
                }
            }
            for (key, choice) in layer {
                sources.loose.entry(key).or_insert(choice);
            }
        }
        Ok(sources)
    }

    /// Resolves logical RIM/TGA references without any filesystem access.
    fn resolve(&self, name: &str) -> Result<Option<&Path>, GameplayError> {
        let stem = name
            .rsplit_once('.')
            .filter(|&(stem, extension)| {
                !stem.is_empty()
                    && safe_name(name)
                    && (extension.eq_ignore_ascii_case("rim")
                        || extension.eq_ignore_ascii_case("tga"))
            })
            .map(|(stem, _extension)| stem)
            .ok_or_else(|| asset_error(format!("invalid UI picture basename {name:?}")))?;
        let key = format!("{}.tga", stem.to_ascii_lowercase());
        let Some(choice) = self.loose.get(&key) else {
            return Ok(None);
        };
        let path = choice.as_deref().ok_or_else(|| asset_error(format!(
            "ambiguous UI picture {name:?} in installation {}",
            self.root.display()
        )))?;
        Ok(Some(path))
    }

    /// Decodes exact selected loose art, otherwise the existing565/555 archives.
    pub(super) fn load(&self, name: &str) -> Result<RgbaImage, GameplayError> {
        self.load_using(name, decode_tga, |entry| {
            // Map icons use magenta as destination-shadow ink, not a chroma key.
            let icon = entry
                .get(..4_usize)
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case("upc0"));
            let chroma_key = (!icon).then_some((248_u8, 0_u8, 248_u8));
            load_rim_rgba_entry(&self.root, PICTURE_ARCHIVE, entry, chroma_key).or_else(|_| {
                load_rim_rgba_entry(
                    &self.root,
                    "ctp2_data/default/graphics/pictures/pic555.zfs",
                    entry,
                    chroma_key,
                )
            })
        })
    }

    /// Injected decoders prove precedence/errors without files, SDL or archives.
    fn load_using(
        &self,
        name: &str,
        loose: impl FnOnce(&Path) -> Result<RgbaImage, GameplayError>,
        archive: impl FnOnce(&str) -> Result<RgbaImage, GameplayError>,
    ) -> Result<RgbaImage, GameplayError> {
        self.resolve(name)?.map_or_else(|| archive(name), loose)
    }
}

/// Accepts flat original ASCII asset names, never paths/drives/NUL/traversal.
fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains("..")
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

/// Reads bounded file/symlink basenames; directory names are never image sources.
fn read_names(directory: &Path) -> std::io::Result<Vec<String>> {
    let mut names = Vec::new();
    for (index, entry) in std::fs::read_dir(directory)?
        .take(MAX_FILES.saturating_add(1_usize))
        .enumerate()
    {
        if index >= MAX_FILES {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "UI picture directory exceeds file limit",
            ));
        }
        let entry = entry?;
        let filename = entry.file_name();
        let Some(name) = filename.to_str() else {
            continue;
        };
        if !safe_name(name) || !name.to_ascii_lowercase().ends_with(".tga") {
            continue;
        }
        if !entry.path().metadata()?.is_file() {
            continue;
        }
        names.push(name.to_owned());
    }
    Ok(names)
}

/// Preserves existing SDL TGA alpha/pitch conversion; never substitutes art.
fn decode_tga(path: &Path) -> Result<RgbaImage, GameplayError> {
    let surface = Surface::from_file(path)
        .and_then(|surface| surface.convert_format(PixelFormatEnum::RGBA32))
        .map_err(|error| asset_error(format!("UI picture {}: {error}", path.display())))?;
    let bytes = surface
        .without_lock()
        .ok_or_else(|| asset_error("TGA surface requires locking".to_owned()))?;
    Ok(RgbaImage {
        width: surface.width(),
        height: surface.height(),
        pitch: usize::try_from(surface.pitch()).map_err(|error| asset_error(error.to_string()))?,
        data: bytes.to_vec(),
    })
}
