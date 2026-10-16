//! Original resource-good identifiers and SPR artwork, never resource generation.

extern crate alloc;

use super::SPRITE_DIRECTORY;
use super::sprite::{SpriteFileType, SpriteGroupAsset};
use crate::gameplay::{GameplayError, asset_error};
use alloc::collections::BTreeMap;
use alloc::string::String;
use std::path::Path;

/// Resource identifier-to-sprite database from the retail installation.
const GOODS_DATABASE: &str = "ctp2_data/default/gamedata/goodsID.txt";

/// Parses original resource identifiers, excluding intentionally undefined ID 0.
///
/// # Errors
/// Returns an asset error for malformed IDs or duplicate symbolic names.
pub fn parse_catalog(text: &str) -> Result<BTreeMap<String, u16>, GameplayError> {
    let mut catalog = BTreeMap::new();
    for line in text.lines() {
        let content = line.split('#').next().unwrap_or_default().trim();
        if content.is_empty() {
            continue;
        }
        let mut parts = content.split_whitespace();
        let Some(name) = parts.next() else {
            continue;
        };
        let Some(raw) = parts.next() else {
            if name.parse::<u16>().is_ok() {
                continue;
            }
            return Err(asset_error(format!("resource {name} has no sprite ID")));
        };
        let id = raw.parse::<u16>().map_err(|error| {
            asset_error(format!("resource {name} has an invalid sprite ID: {error}"))
        })?;
        if id == 0_u16 {
            continue;
        }
        if catalog.insert(name.to_ascii_uppercase(), id).is_some() {
            return Err(asset_error(format!("duplicate resource identifier {name}")));
        }
    }
    Ok(catalog)
}

/// Reads original artwork mappings without inventing any map resource locations.
///
/// # Errors
/// Returns an asset error if the installation database cannot be read or parsed.
pub fn load_catalog(root: &Path) -> Result<BTreeMap<String, u16>, GameplayError> {
    let text = std::fs::read_to_string(root.join(GOODS_DATABASE))
        .map_err(|error| asset_error(format!("resource database: {error}")))?;
    parse_catalog(&text)
}

/// Loads a resource group's original idle animation, including its sprite hotspot.
///
/// # Errors
/// Returns an asset error for missing, malformed or wrong-kind SPR files.
pub fn load_good_sprite(root: &Path, id: u16) -> Result<SpriteGroupAsset, GameplayError> {
    let directory = root.join(SPRITE_DIRECTORY);
    for name in [
        format!("GG{id:03}.SPR"),
        format!("Gg{id:03}.spr"),
        format!("gg{id:03}.spr"),
    ] {
        if let Ok(bytes) = std::fs::read(directory.join(name)) {
            let sprite = SpriteGroupAsset::parse(&bytes)?;
            if sprite.file_type != SpriteFileType::Good {
                return Err(asset_error(format!(
                    "resource sprite {id} is not a good group"
                )));
            }
            return Ok(sprite);
        }
    }
    Err(asset_error(format!(
        "resource sprite GG{id:03}.SPR is missing from {}",
        directory.display()
    )))
}

#[cfg(test)]
mod tests {
    use super::{load_catalog, load_good_sprite, parse_catalog};
    use std::path::Path;

    #[test]
    fn catalog_handles_headers_comments_placeholders_and_case() {
        let catalog =
            parse_catalog("3\nTERRAIN_PLAINS_GOOD_TWO 8 # Elephant\nunused 0\nother 13\n")
                .expect("original catalog syntax");
        assert_eq!(
            catalog.get("TERRAIN_PLAINS_GOOD_TWO"),
            Some(&8_u16),
            "elephant reference"
        );
        assert_eq!(
            catalog.get("OTHER"),
            Some(&13_u16),
            "canonical symbol normalization"
        );
        assert_eq!(
            catalog.len(),
            2_usize,
            "undefined placeholders are not assets"
        );
        for invalid in ["good", "good invalid", "good 8\nGOOD 9"] {
            assert!(
                parse_catalog(invalid).is_err(),
                "malformed/duplicate data rejected: {invalid}"
            );
        }
    }

    #[test]
    fn every_defined_retail_resource_sprite_is_readable_without_copying_assets() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let catalog = load_catalog(&root).expect("installed read-only resource mapping");
        for (symbol, id) in catalog {
            let sprite = load_good_sprite(&root, id).expect("every defined resource SPR decodes");
            assert!(
                sprite.frame_count() > 0_usize,
                "resource {symbol} has real frames"
            );
        }
    }
}
