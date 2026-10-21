//! Original tile-improvement artwork mappings; no placement or yield simulation.
use super::{
    database::{field, records},
    text,
};
use crate::gameplay::{GameplayError, asset_error};
use alloc::collections::BTreeMap;
use std::path::Path;

/// Construction sequence plus terrain-specific source texture references.
#[derive(Debug, Clone, Default)]
struct Recipe {
    /// Original in-progress overlay sequence.
    construction: Vec<u16>,
    /// Canonical terrain names and corresponding finished overlay index.
    terrain: BTreeMap<String, u16>,
}
/// Session-cached source art lookup for server-observed improvement IDs only.
#[derive(Debug, Clone, Default)]
pub struct ImprovementArt {
    /// Pinned visual recipes, not authoritative construction locations.
    recipes: BTreeMap<String, Recipe>,
}
impl ImprovementArt {
    /// Parses original record syntax in memory without opening any file.
    #[must_use]
    pub fn parse(source: &str) -> Self {
        let recipes = records(source)
            .into_iter()
            .map(|(id, words)| {
                let construction = words
                    .windows(2)
                    .filter(|pair| pair[0] == "ConstructionTiles")
                    .filter_map(|pair| pair[1].parse().ok())
                    .collect();
                let mut terrain = BTreeMap::new();
                for effect in words.split(|word| word == "TerrainEffect").skip(1) {
                    if let Some(index) =
                        field(effect, "TilesetIndex").and_then(|index| index.parse::<u16>().ok())
                    {
                        for pair in effect.windows(2).filter(|pair| pair[0] == "Terrain") {
                            terrain.insert(pair[1].clone(), index);
                        }
                    }
                }
                (
                    id,
                    Recipe {
                        construction,
                        terrain,
                    },
                )
            })
            .collect();
        Self { recipes }
    }
    /// Loads immutable installed metadata once per session.
    ///
    /// # Errors
    /// Returns contextual original database IO failures.
    pub fn load(root: &Path) -> Result<Self, GameplayError> {
        let bytes = std::fs::read(root.join("ctp2_data/default/gamedata/tileimp.txt"))
            .map_err(|error| asset_error(format!("tileimp.txt: {error}")))?;
        Ok(Self::parse(&text::decode(&bytes)))
    }
    /// Selects art only for an already observed improvement and known terrain.
    #[must_use]
    pub fn index(&self, id: &str, terrain: &str, remaining: u32, total: u32) -> Option<u16> {
        let recipe = self.recipes.get(id)?;
        if remaining > 0 {
            let count = u32::try_from(recipe.construction.len()).ok()?;
            let index = total
                .saturating_sub(remaining)
                .saturating_mul(count)
                .checked_div(total.max(1))?
                .min(count.saturating_sub(1));
            recipe
                .construction
                .get(usize::try_from(index).ok()?)
                .copied()
        } else {
            recipe.terrain.get(terrain).copied()
        }
    }
}
