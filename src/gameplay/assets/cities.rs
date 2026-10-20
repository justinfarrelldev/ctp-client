//! Installed city artwork recipes, independent of authoritative gameplay rules.
//!
//! `CityData::GetDesiredSpriteIndex` uses style, age, land/water and population.
//! Retail GC files are UnitSpriteGroup containers (`SpriteGroupList.cpp`), not
//! the unused CitySpriteGroup reader. Only visual metadata is loaded here.

use super::{
    SPRITE_DIRECTORY,
    database::{field, records},
    sprite::SpriteGroupAsset,
    text,
};
use crate::gameplay::{GameplayError, asset_error};
use alloc::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Original population range and installed sprite identity for one city recipe.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Recipe {
    /// Zero means land; nonzero means water.
    kind: u32,
    /// Inclusive source population minimum.
    minimum: u32,
    /// Inclusive source population maximum.
    maximum: u32,
    /// GC file number, separate from GU unit artwork with the same number.
    sprite: u16,
}

/// Read-only source art catalogue; no game facts are inferred from its defaults.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CityArtCatalog {
    /// Style name and ordered source age-style references.
    styles: Vec<(String, Vec<String>)>,
    /// Age-style recipes in source declaration order.
    ages: BTreeMap<String, Vec<Recipe>>,
}

impl CityArtCatalog {
    /// Parses cached source metadata without any filesystem access.
    ///
    /// # Errors
    /// Rejects malformed ranges or missing visual identifiers.
    pub fn parse(styles: &str, ages: &str) -> Result<Self, GameplayError> {
        let styles = records(styles)
            .into_iter()
            .map(|(id, words)| {
                let age_names = words
                    .windows(2_usize)
                    .filter(|pair| pair[0_usize] == "AgeStyle")
                    .map(|pair| pair[1_usize].clone())
                    .collect();
                (id, age_names)
            })
            .collect();
        let mut catalog = Self {
            styles,
            ages: BTreeMap::new(),
        };
        for (id, words) in records(ages) {
            let mut recipes = Vec::new();
            for part in words.split(|word| word == "Sprites").skip(1_usize) {
                let number = |key| {
                    field(part, key)
                        .and_then(|value| value.parse::<u32>().ok())
                        .ok_or_else(|| asset_error(format!("city art {id}: missing/invalid {key}")))
                };
                let recipe = Recipe {
                    kind: number("Type")?,
                    minimum: number("MinSize")?,
                    maximum: number("MaxSize")?,
                    sprite: u16::try_from(number("Sprite")?)
                        .map_err(|error| asset_error(error.to_string()))?,
                };
                if recipe.minimum > recipe.maximum {
                    return Err(asset_error(format!(
                        "city art {id}: reversed population range"
                    )));
                }
                recipes.push(recipe);
            }
            catalog.ages.insert(id, recipes);
        }
        Ok(catalog)
    }

    /// Reads original source tables without altering or extracting assets.
    ///
    /// # Errors
    /// Returns contextual I/O or metadata errors.
    pub fn load(root: &Path) -> Result<Self, GameplayError> {
        let read = |name| {
            std::fs::read(root.join("ctp2_data/default/gamedata").join(name))
                .map(|bytes| text::decode(&bytes))
                .map_err(|error| asset_error(format!("{name}: {error}")))
        };
        Self::parse(&read("citystyle.txt")?, &read("agecitystyle.txt")?)
    }

    /// Exact source selection, including water-to-land fallback and largest size.
    #[must_use]
    pub fn sprite(&self, style: usize, age: usize, water: bool, population: u32) -> Option<u16> {
        let recipes = self.ages.get(self.styles.get(style)?.1.get(age)?)?;
        let choose = |is_water| {
            let mut last = None;
            for recipe in recipes
                .iter()
                .filter(|recipe| (recipe.kind != 0_u32) == is_water)
            {
                last = Some(recipe.sprite);
                if (recipe.minimum..=recipe.maximum).contains(&population) {
                    return last;
                }
            }
            last
        };
        choose(water).or_else(|| choose(false))
    }

    /// Original `UnitActor`'s first-style/first-age/first-sprite visual fallback.
    #[must_use]
    pub fn fallback(&self) -> Option<u16> {
        let age = self.styles.first()?.1.first()?;
        Some(self.ages.get(age)?.first()?.sprite)
    }

    /// Distinct installed city files for read-only source-art validation.
    #[must_use]
    pub fn sprite_ids(&self) -> BTreeSet<u16> {
        self.ages
            .values()
            .flatten()
            .map(|recipe| recipe.sprite)
            .collect()
    }
}

/// Loads the actual GC namespace; a city must never accidentally become GU028.
///
/// # Errors
/// Returns a missing/malformed sprite error with its city identity.
pub fn load_sprite(root: &Path, id: u16) -> Result<SpriteGroupAsset, GameplayError> {
    for name in [
        format!("GC{id:03}.SPR"),
        format!("Gc{id:03}.spr"),
        format!("gc{id:03}.spr"),
    ] {
        match std::fs::read(root.join(SPRITE_DIRECTORY).join(&name)) {
            Ok(bytes) => {
                return SpriteGroupAsset::parse(&bytes)
                    .map_err(|error| asset_error(format!("{name}: {error}")));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(asset_error(format!("{name}: {error}"))),
        }
    }
    Err(asset_error(format!("city sprite GC{id:03}.SPR is missing")))
}

#[cfg(test)]
mod tests {
    use super::{CityArtCatalog, load_sprite};
    use std::path::Path;

    #[test]
    fn source_city_recipes_keep_boundaries_age_style_water_fallback_and_largest_size() {
        let catalog=CityArtCatalog::parse("STYLE { AgeStyle ANCIENT AgeStyle MODERN }","ANCIENT { Sprites { Type 0 MinSize 0 MaxSize 5 Sprite 28 } Sprites { Type 0 MinSize 6 MaxSize 10 Sprite 29 } } MODERN { Sprites { Type 0 MinSize 0 MaxSize 5 Sprite 57 } Sprites { Type 1 MinSize 0 MaxSize 5 Sprite 89 } }").expect("source city recipes");
        for (age, water, population, id) in [
            (0_usize, false, 5_u32, 28_u16),
            (0_usize, false, 6_u32, 29_u16),
            (0_usize, true, 3_u32, 28_u16),
            (0_usize, false, 100_u32, 29_u16),
            (1_usize, false, 1_u32, 57_u16),
            (1_usize, true, 1_u32, 89_u16),
            (1_usize, true, 100_u32, 89_u16),
        ] {
            assert_eq!(
                catalog.sprite(0_usize, age, water, population),
                Some(id),
                "source recipe boundary"
            );
        }
        assert_eq!(
            catalog.sprite(1_usize, 0_usize, false, 1_u32),
            None,
            "unknown styles are not a real civilization"
        );
        assert_eq!(
            catalog.fallback(),
            Some(28_u16),
            "original visual fallback, not a gameplay fact"
        );
        assert!(
            CityArtCatalog::parse("", "A { Sprites { Type 0 MinSize 9 MaxSize 1 Sprite 28 } }")
                .is_err(),
            "reject malformed source metadata"
        );
    }

    #[test]
    fn installed_city_groups_are_real_building_art_in_the_gc_namespace() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let catalog = CityArtCatalog::load(&root).expect("read-only city metadata");
        assert_eq!(catalog.fallback(), Some(28_u16), "actual original fallback");
        for id in catalog.sprite_ids() {
            let sprite = load_sprite(&root, id).expect("installed city group");
            assert!(
                sprite
                    .actions
                    .iter()
                    .any(|action| action.action_index == 2_usize && !action.frames.is_empty()),
                "GC{id:03} has original idle city artwork"
            );
        }
    }
}
