//! Installed CTP2 unit-to-art mapping, using `newsprite.txt` (not CTP1's stale spriteID).

extern crate alloc;

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

/// Reads original data tables, without extracting/copying any game artwork.
///
/// # Errors
/// Returns an asset error if required installed text data cannot be read.
pub fn load(root: &Path) -> Result<BTreeMap<String, UnitArt>, GameplayError> {
    let directory = root.join("ctp2_data/default/gamedata");
    let read = |name: &str| {
        std::fs::read_to_string(directory.join(name))
            .map_err(|error| asset_error(format!("{name}: {error}")))
    };
    let mut catalog = parse(
        &read("Units.txt")?,
        &read("newsprite.txt")?,
        &read("uniticon.txt")?,
    );
    let strings = super::text::decode(
        &std::fs::read(root.join("ctp2_data/english/gamedata/gl_str.txt"))
            .map_err(|error| asset_error(error.to_string()))?,
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

/// Finds an exact record field, not an incidental token in a comment.
fn field<'data>(words: &'data [String], name: &str) -> Option<&'data str> {
    words
        .windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].as_str())
}

/// Reads balanced top-level database records while ignoring comments/quoted braces.
fn records(text: &str) -> Vec<(String, Vec<String>)> {
    let tokens = tokenize(text);
    let mut records = Vec::new();
    let mut index = 0_usize;
    while index.saturating_add(1) < tokens.len() {
        if tokens[index.saturating_add(1)] != "{" {
            index = index.saturating_add(1);
            continue;
        }
        let name = tokens[index].clone();
        index = index.saturating_add(2);
        let start = index;
        let mut depth = 1_u32;
        while index < tokens.len() && depth > 0 {
            match tokens[index].as_str() {
                "{" => depth = depth.saturating_add(1),
                "}" => depth = depth.saturating_sub(1),
                _ => {}
            }
            index = index.saturating_add(1);
        }
        if depth == 0 {
            records.push((name, tokens[start..index.saturating_sub(1)].to_vec()));
        }
    }
    records
}

/// Minimal game-database lexer preserving strings and dropping line comments.
fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut word = String::new();
    let mut quoted = false;
    let mut comment = false;
    for character in text.chars() {
        if comment {
            if character == '\n' {
                comment = false;
            } else {
                continue;
            }
        }
        if character == '"' {
            quoted = !quoted;
            if !quoted {
                tokens.push(core::mem::take(&mut word));
            }
            continue;
        }
        if !quoted && character == '#' {
            comment = true;
            continue;
        }
        if !quoted && (character.is_whitespace() || matches!(character, '{' | '}')) {
            if !word.is_empty() {
                tokens.push(core::mem::take(&mut word));
            }
            if matches!(character, '{' | '}') {
                tokens.push(character.to_string());
            }
        } else {
            word.push(character);
        }
    }
    if !word.is_empty() {
        tokens.push(word);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::{load, parse};
    use std::path::Path;
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
