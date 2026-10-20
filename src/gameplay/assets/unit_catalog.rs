//! Installed CTP2 unit-to-art mapping, using `newsprite.txt` (not CTP1's stale spriteID).

extern crate alloc;

use super::database::{field, records};
use crate::gameplay::{GameplayError, asset_error};
use alloc::collections::BTreeMap;
use std::path::Path;

/// Visual identity only; capabilities and game rules always belong to the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitArt {
    /// Retail SPR number from `DefaultSprite` and `newsprite.txt`.
    pub sprite: u16,
    /// Original large-portrait archive entry.
    pub portrait: String,
    /// Original medium icon used by multiple-unit selection.
    pub icon: String,
    /// Installed localized unit name, not an unrelated family label.
    pub label: String,
}

/// Reads original Windows-1252 or modded UTF-8 tables without changing assets.
///
/// # Errors
/// Returns an asset error if required installed text data cannot be read.
pub fn load(root: &Path) -> Result<BTreeMap<String, UnitArt>, GameplayError> {
    load_with_reader(root, |path| std::fs::read(path))
}

/// Injectable read-only byte boundary for real installation and fileless regressions.
fn load_with_reader(
    root: &Path,
    mut reader: impl FnMut(&Path) -> std::io::Result<Vec<u8>>,
) -> Result<BTreeMap<String, UnitArt>, GameplayError> {
    let directory = root.join("ctp2_data/default/gamedata");
    let mut read = |name: &str| -> Result<String, GameplayError> {
        let bytes = reader(&directory.join(name))
            .map_err(|error| asset_error(format!("{name}: {error}")))?;
        Ok(super::text::decode(&bytes))
    };
    let mut catalog = parse(
        &read("Units.txt")?,
        &read("newsprite.txt")?,
        &read("uniticon.txt")?,
    );
    let strings = super::text::decode(
        &reader(&root.join("ctp2_data/english/gamedata/gl_str.txt"))
            .map_err(|error| asset_error(format!("gl_str.txt: {error}")))?,
    );
    for line in strings.lines() {
        let Some((prefix, quoted)) = line.split_once('"') else {
            continue;
        };
        let Some((label, _)) = quoted.rsplit_once('"') else {
            continue;
        };
        if let Some(art) = prefix
            .split_whitespace()
            .next()
            .and_then(|key| catalog.get_mut(key))
        {
            label.clone_into(&mut art.label);
        }
    }
    Ok(catalog)
}

/// Resolves all top-level unit definitions from the installed database.
#[must_use]
pub fn parse(units: &str, sprites: &str, icons: &str) -> BTreeMap<String, UnitArt> {
    let sprite_ids: BTreeMap<_, _> = sprites
        .lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            let name = words.next()?;
            let id = words.next()?.parse::<u16>().ok()?;
            Some((name.to_owned(), id))
        })
        .collect();
    let portraits: BTreeMap<_, _> = records(icons)
        .into_iter()
        .filter_map(|(name, words)| {
            Some((
                name,
                (
                    field(&words, "LargeIcon")
                        .or_else(|| field(&words, "FirstFrame"))?
                        .to_ascii_lowercase()
                        .replace(".tga", ".rim"),
                    field(&words, "Icon")
                        .or_else(|| field(&words, "LargeIcon"))?
                        .to_ascii_lowercase()
                        .replace(".tga", ".rim"),
                ),
            ))
        })
        .collect();
    records(units)
        .into_iter()
        .filter_map(|(name, words)| {
            let sprite = *sprite_ids.get(field(&words, "DefaultSprite")?)?;
            let (portrait, icon) = portraits.get(field(&words, "DefaultIcon")?)?.clone();
            let label = name.trim_start_matches("UNIT_").replace('_', " ");
            Some((
                name,
                UnitArt {
                    sprite,
                    portrait,
                    icon,
                    label,
                },
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{load, load_with_reader, parse};
    use alloc::collections::BTreeMap;
    use std::io::{Error, ErrorKind};
    use std::path::Path;
    /// Minimal installation bytes; these mocks never create or edit real files.
    fn installation_bytes() -> BTreeMap<&'static str, Vec<u8>> {
        [
            (
                "Units.txt",
                b"UNIT_ARCHER { DefaultSprite SPRITE_ARCHER DefaultIcon ICON_ARCHER }\n".to_vec(),
            ),
            ("newsprite.txt", b"SPRITE_ARCHER 13\n".to_vec()),
            (
                "uniticon.txt",
                b"ICON_ARCHER { LargeIcon \"UPUP013L.TGA\" Icon \"UPUP013A.TGA\" }\n".to_vec(),
            ),
            ("gl_str.txt", b"UNIT_ARCHER \"Archer\"\n".to_vec()),
        ]
        .into_iter()
        .collect()
    }

    #[test]
    fn original_ansi_unit_databases_load_through_the_real_reader_boundary_without_asset_edits() {
        for filename in ["Units.txt", "newsprite.txt", "uniticon.txt", "gl_str.txt"] {
            let mut files = installation_bytes();
            files
                .get_mut(filename)
                .expect("mock file")
                .extend_from_slice(b"\n# Archer\x92s notes \x96 original ANSI text\n");
            let mut reads = Vec::new();
            let catalog = load_with_reader(Path::new("read-only-installation"), |path| {
                reads.push(path.to_path_buf());
                path.file_name()
                    .and_then(|name| name.to_str())
                    .and_then(|name| files.get(name))
                    .cloned()
                    .ok_or_else(|| Error::new(ErrorKind::NotFound, "unexpected asset read"))
            })
            .expect("retail Windows-1252 is valid game text, not a launch error");
            assert_eq!(
                catalog["UNIT_ARCHER"].sprite, 13_u16,
                "{filename}: real sprite association"
            );
            assert_eq!(
                catalog["UNIT_ARCHER"].portrait, "upup013l.rim",
                "{filename}: real portrait"
            );
            assert_eq!(
                catalog["UNIT_ARCHER"].label, "Archer",
                "{filename}: unchanged localized label"
            );
            assert_eq!(
                reads.len(),
                4_usize,
                "only required metadata files are read"
            );
            assert!(
                reads
                    .iter()
                    .all(|path| path.starts_with("read-only-installation/ctp2_data")),
                "reader must remain under the installation root"
            );
        }
    }

    #[test]
    fn installation_unit_labels_preserve_cp1252_and_modern_utf8_without_replacement_characters() {
        for (bytes, expected) in [
            (
                b"UNIT_ARCHER \"Archer\x92s \x96 cit\xe9\"\n".to_vec(),
                "Archer’s – cité",
            ),
            (
                "UNIT_ARCHER \"Łucznik 弓兵\"\n".as_bytes().to_vec(),
                "Łucznik 弓兵",
            ),
        ] {
            let mut files = installation_bytes();
            files.insert("gl_str.txt", bytes);
            let catalog = load_with_reader(Path::new("unused-root"), |path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .and_then(|name| files.get(name))
                    .cloned()
                    .ok_or_else(|| Error::new(ErrorKind::NotFound, "mock input missing"))
            })
            .expect("encoded original or mod labels");
            assert_eq!(
                catalog["UNIT_ARCHER"].label, expected,
                "preserve label encoding exactly"
            );
        }
    }

    #[test]
    fn installation_metadata_io_failures_keep_the_failing_filename_and_cause() {
        for filename in ["Units.txt", "newsprite.txt", "uniticon.txt", "gl_str.txt"] {
            let mut files = installation_bytes();
            files.remove(filename);
            let error = load_with_reader(Path::new("unused-root"), |path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .and_then(|name| files.get(name))
                    .cloned()
                    .ok_or_else(|| Error::new(ErrorKind::PermissionDenied, "mock read denied"))
            })
            .expect_err("real I/O failures must still stop asset loading")
            .to_string();
            assert!(
                error.contains(filename) && error.contains("mock read denied"),
                "asset and cause required for {filename}: {error}"
            );
        }
    }

    #[test]
    fn catalog_uses_exact_retail_sprite_and_icon_records() {
        let catalog = parse(
            "# UNIT_FAKE { DefaultSprite WRONG }\nUNIT_ARCHER { DefaultSprite SPRITE_ARCHER DefaultIcon ICON_ARCHER nested { Ignore 1 } }",
            "SPRITE_ARCHER 13\nSPRITE_ARTILLERY 82",
            "ICON_ARCHER { LargeIcon \"UPUP013L.TGA\" }",
        );
        assert_eq!(
            catalog["UNIT_ARCHER"].sprite, 13_u16,
            "retail newsprite, not stale CTP1 table"
        );
        assert_eq!(
            catalog["UNIT_ARCHER"].portrait, "upup013l.rim",
            "exact installed portrait"
        );
        assert!(
            !catalog.contains_key("UNIT_FAKE"),
            "comments do not create phantom definitions"
        );
    }
    #[test]
    fn every_catalog_sprite_loads_all_actions_and_all_stored_directions() -> Result<(), String> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let catalog = load(&root).expect("catalog");
        for (name, art) in catalog {
            let group = crate::gameplay::assets::load_unit_sprite(&root, art.sprite)
                .map_err(|error| format!("{name}: {error}"))?;
            if group.actions.is_empty() {
                return Err(format!("{name} has no actions"));
            }
            for action in group.actions {
                for frame in &action.animation.frames {
                    for facing in 0_u8..5_u8 {
                        if action.frame_for_facing(facing, *frame).is_none() {
                            return Err(format!(
                                "{name} action {} cannot resolve frame {frame} facing {facing}",
                                action.action_index
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn every_installed_unit_resolves_and_references_have_distinct_art() {
        let catalog =
            load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2")).expect("installed catalog");
        assert!(
            catalog.len() == 72_usize,
            "all 72 installed retail unit records, not a three-unit hardcoded roster"
        );
        assert_eq!(catalog["UNIT_HOPLITE"].sprite, 8_u16, "Hoplite");
        assert_eq!(catalog["UNIT_ARCHER"].sprite, 13_u16, "Archer");
        assert_eq!(
            catalog["UNIT_ARTILLERY"].sprite, 82_u16,
            "CTP2 artillery differs from stale CTP1 table"
        );
    }
}
