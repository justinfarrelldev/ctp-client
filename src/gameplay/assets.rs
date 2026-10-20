//! In-game original-asset references and loaders.

extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use std::fs;
use std::path::{Path, PathBuf};

use super::{GameplayError, asset_error};

/// Original city-style/age/population recipes and GC building sprites.
pub mod cities;
/// Shared record scanning for installed visual metadata.
mod database;
/// Original resource identifiers and resource SPR loading (not generation).
pub mod goods;
/// Installed source-layout inheritance and management UI geometry.
pub mod layout;
#[cfg(test)]
mod layout_tests;
/// Read-only original articles and non-executable rich text.
pub mod library;
#[cfg(test)]
mod library_tests;
/// Original channel expansion shared by terrain, actors and UI art.
pub(crate) mod pixel;
/// RIM/ZFS image loading for CTP2 UI art.
pub mod rim;
/// Sprite file parser for CTP2 `.SPR` files.
pub mod sprite;
/// Retail Windows-1252 and modern UTF-8 database text.
pub mod text;
/// Terrain tile-set decoder for CTP2 `.TIL` files.
pub mod tile;
/// Canonical installed unit-to-sprite/portrait mapping.
pub mod unit_catalog;

/// Default picture archive used by CTP2 in-game UI art.
pub const PICTURE_ARCHIVE: &str = "ctp2_data/default/graphics/pictures/pic565.zfs";
/// Default pattern archive used by terrain/UI patterns.
pub const PATTERN_ARCHIVE: &str = "ctp2_data/default/graphics/patterns/pat565.zfs";
/// Default sprite directory relative to the installation root.
pub const SPRITE_DIRECTORY: &str = "ctp2_data/default/graphics/sprites";
/// Sprite id database relative to the installation root.
pub const SPRITE_ID_DATABASE: &str = "ctp2_data/default/gamedata/spriteID.txt";

/// Reference to one original asset required by the in-game renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameplayAssetReference {
    /// Human-readable role of the asset.
    pub role: &'static str,
    /// Archive path relative to installation root, or `None` for loose files.
    pub archive_relative_path: Option<&'static str>,
    /// Entry or loose filename.
    pub entry_name: &'static str,
    /// Source LDL/source-file note.
    pub source: &'static str,
}

/// High-level manifest of assets researched for the in-game screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameplayAssetManifest {
    /// UI and map art references.
    pub art: Vec<GameplayAssetReference>,
    /// Unit sprite ids required by the first playable ancient turn.
    pub unit_sprite_ids: Vec<u16>,
    /// Sound database keys required by current interactions.
    pub sound_keys: Vec<&'static str>,
}

impl GameplayAssetManifest {
    /// Returns the manifest used by the current in-game implementation.
    #[must_use]
    pub fn original_ctp2() -> Self {
        Self {
            art: original_ctp2_art(),
            unit_sprite_ids: vec![2, 3],
            sound_keys: original_ctp2_sound_keys(),
        }
    }
}

/// Returns all researched CTP2 in-game art references.
fn original_ctp2_art() -> Vec<GameplayAssetReference> {
    let mut references = top_menu_art();
    references.extend(control_panel_art());
    references.extend(order_and_popup_art());
    references
}

/// Returns top menu and resource counter art references.
fn top_menu_art() -> Vec<GameplayAssetReference> {
    vec![
        art(
            "Menu bar",
            Some(PICTURE_ARCHIVE),
            "uptb00.rim",
            "controlpanel.ldl:2135",
        ),
        art(
            "Menu button",
            Some(PICTURE_ARCHIVE),
            "uptbtest.rim",
            "controlpanel.ldl:2125",
        ),
        art(
            "Dropdown item background",
            Some(PICTURE_ARCHIVE),
            "uptg01e.rim",
            "ctp_template.ldl:1013",
        ),
        art(
            "Dropdown left frame",
            Some(PICTURE_ARCHIVE),
            "uptg05a.rim",
            "ctp_template.ldl:1004",
        ),
        art(
            "Dropdown center frame",
            Some(PICTURE_ARCHIVE),
            "uptg05b.rim",
            "ctp_template.ldl:1008",
        ),
        art(
            "Dropdown right frame",
            Some(PICTURE_ARCHIVE),
            "uptg05c.rim",
            "ctp_template.ldl:1011",
        ),
        art(
            "Menu resource counter left",
            Some(PICTURE_ARCHIVE),
            "uptb02.rim",
            "controlpanel.ldl:2089",
        ),
        art(
            "Menu resource counter body",
            Some(PICTURE_ARCHIVE),
            "uptb03.rim",
            "controlpanel.ldl:2090",
        ),
        art(
            "Gold icon",
            Some(PICTURE_ARCHIVE),
            "upic34.rim",
            "controlpanel.ldl:2252",
        ),
        art(
            "Public works icon",
            Some(PICTURE_ARCHIVE),
            "upic33.rim",
            "controlpanel.ldl:2246",
        ),
        art(
            "Happiness icon",
            Some(PICTURE_ARCHIVE),
            "upic35.rim",
            "controlpanel.ldl:2237",
        ),
        art(
            "Pollution icon",
            Some(PICTURE_ARCHIVE),
            "upic36.rim",
            "controlpanel.ldl:2228",
        ),
    ]
}

/// Returns control panel and end-turn art references.
fn control_panel_art() -> Vec<GameplayAssetReference> {
    vec![
        art(
            "Control panel base",
            Some(PICTURE_ARCHIVE),
            "upfg01.rim",
            "controlpanel.ldl:526",
        ),
        art(
            "Control panel right cap",
            Some(PICTURE_ARCHIVE),
            "upfg03.rim",
            "controlpanel.ldl:529",
        ),
        art(
            "Control panel bottom cap",
            Some(PICTURE_ARCHIVE),
            "upfg04.rim",
            "controlpanel.ldl:532",
        ),
        art(
            "Control panel status bar",
            Some(PICTURE_ARCHIVE),
            "upst01.rim",
            "controlpanel.ldl:544",
        ),
        art(
            "End turn disabled",
            Some(PICTURE_ARCHIVE),
            "upcb14x.rim",
            "controlpanel.ldl:563",
        ),
        art(
            "End turn up",
            Some(PICTURE_ARCHIVE),
            "upcb14u.rim",
            "controlpanel.ldl:565",
        ),
        art(
            "End turn down",
            Some(PICTURE_ARCHIVE),
            "upcb14d.rim",
            "controlpanel.ldl:567",
        ),
        art(
            "End turn hover",
            Some(PICTURE_ARCHIVE),
            "upcb14h.rim",
            "controlpanel.ldl:569",
        ),
    ]
}

/// Returns order-button and popup art references.
fn order_and_popup_art() -> Vec<GameplayAssetReference> {
    vec![
        art(
            "Great Library shortcut",
            Some(PICTURE_ARCHIVE),
            "upcb60u.rim",
            "controlpanel.ldl:766",
        ),
        art(
            "Settle order icon",
            Some(PICTURE_ARCHIVE),
            "upa014b.rim",
            "buttonbank.ldl:100",
        ),
        art(
            "Order button pattern",
            Some(PICTURE_ARCHIVE),
            "upba0104.rim",
            "buttonbank.ldl:102",
        ),
        art(
            "Great Library window",
            Some(PICTURE_ARCHIVE),
            "uptg20e.rim",
            "greatlibrary.ldl:143",
        ),
    ]
}

/// Returns CTP2 sound database keys used by current gameplay interactions.
fn original_ctp2_sound_keys() -> Vec<&'static str> {
    vec![
        "SOUND_ACKNOWLEDGE_SETTLER",
        "SOUND_MOVE_SETTLER",
        "SOUND_WORK_SETTLER",
        "SOUND_ATTACK_SETTLER",
        "SOUND_DEATH_SETTLER",
    ]
}

/// Builds one art manifest row.
const fn art(
    role: &'static str,
    archive_relative_path: Option<&'static str>,
    entry_name: &'static str,
    source: &'static str,
) -> GameplayAssetReference {
    GameplayAssetReference {
        role,
        archive_relative_path,
        entry_name,
        source,
    }
}

/// Parses `spriteID.txt` and returns the value for a symbolic sprite name.
#[must_use]
pub fn parse_sprite_id(database_text: &str, symbol: &str) -> Option<u16> {
    database_text.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let name = parts.next()?;
        let value = parts.next()?;
        if name.eq_ignore_ascii_case(symbol) {
            value.parse::<u16>().ok()
        } else {
            None
        }
    })
}

/// Returns candidate sprite filenames for a CTP2 unit sprite id.
#[must_use]
pub fn unit_sprite_candidates(sprite_id: u16) -> Vec<String> {
    vec![
        format!("Gu{sprite_id:03}.spr"),
        format!("GU{sprite_id:03}.SPR"),
        format!("gu{sprite_id:03}.spr"),
        format!("Gu{sprite_id:02}.spr"),
        format!("GU{sprite_id:02}.SPR"),
        format!("gu{sprite_id:02}.spr"),
    ]
}

/// Loads a full unit sprite group from the installation.
///
/// # Errors
///
/// Returns [`GameplayError::Asset`] if no candidate file can be read or parsed.
pub fn load_unit_sprite(
    installation_root: &Path,
    sprite_id: u16,
) -> Result<sprite::SpriteGroupAsset, GameplayError> {
    let mut last_error = String::new();
    for candidate in unit_sprite_candidates(sprite_id) {
        let path = installation_root.join(SPRITE_DIRECTORY).join(&candidate);
        match fs::read(&path) {
            Ok(bytes) => return sprite::SpriteGroupAsset::parse(&bytes),
            Err(error) => {
                last_error = format!("{}: {error}", path.display());
            }
        }
    }
    Err(asset_error(format!(
        "unit sprite {sprite_id} could not be read; last error: {last_error}"
    )))
}

/// Returns all loose sprite paths that should be loadable for launch gameplay.
#[must_use]
pub fn required_unit_sprite_paths(installation_root: &Path) -> Vec<PathBuf> {
    GameplayAssetManifest::original_ctp2()
        .unit_sprite_ids
        .into_iter()
        .flat_map(unit_sprite_candidates)
        .map(|candidate| installation_root.join(SPRITE_DIRECTORY).join(candidate))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{GameplayAssetManifest, parse_sprite_id, unit_sprite_candidates};

    #[test]
    fn manifest_includes_control_panel_menu_and_settler_assets() {
        let manifest = GameplayAssetManifest::original_ctp2();

        assert!(
            manifest
                .art
                .iter()
                .any(|asset| asset.entry_name == "uptb00.rim"),
            "top menu bar art should be tracked"
        );
        assert!(
            manifest
                .art
                .iter()
                .any(|asset| asset.entry_name == "uptg05a.rim"),
            "original CTP2 dropdown frame art should be tracked"
        );
        assert!(
            manifest
                .art
                .iter()
                .any(|asset| asset.entry_name == "upcb14u.rim"),
            "end turn button up art should be tracked"
        );
        assert!(
            manifest.unit_sprite_ids.contains(&2),
            "settler sprite id should be required"
        );
    }

    #[test]
    fn sprite_id_database_parser_reads_symbol_values() {
        let text = "SPRITE_SETTLER 2\nSPRITE_WARRIOR 03\n";

        assert_eq!(parse_sprite_id(text, "SPRITE_SETTLER"), Some(2));
        assert_eq!(parse_sprite_id(text, "sprite_warrior"), Some(3));
        assert_eq!(parse_sprite_id(text, "SPRITE_MISSING"), None);
    }

    #[test]
    fn unit_sprite_candidates_prefer_ctp2_three_digit_mixed_case_names() {
        let candidates = unit_sprite_candidates(2);

        assert_eq!(
            candidates.first(),
            Some(&"Gu002.spr".to_owned()),
            "CTP2 installs include Gu002.spr; it must be preferred over legacy GU02.SPR"
        );
        assert!(candidates.contains(&"GU002.SPR".to_owned()));
        assert!(candidates.contains(&"GU02.SPR".to_owned()));
    }
}
