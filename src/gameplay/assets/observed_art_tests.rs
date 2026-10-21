//! Art IDs are installation metadata, never authority about a tile or city.
use super::{cities::CityArtCatalog, observed_art::ImprovementArt};

#[test]
fn observed_art_uses_installed_age_order_terrain_recipes_and_construction_progress() {
    let city=CityArtCatalog::parse_with_ages("STYLE { AgeStyle FIRST AgeStyle SECOND }","FIRST { Sprites { Type 0 MinSize 1 MaxSize 99 Sprite 28 } } SECOND { Sprites { Type 0 MinSize 1 MaxSize 99 Sprite 29 } }","AGE_ONE {} AGE_TWO {}").expect("source metadata");
    assert_eq!(
        city.sprite_for_age("AGE_TWO", false, 4),
        Some(29_u16),
        "authoritative age uses installed order"
    );
    assert_eq!(
        city.sprite_for_age("MOD_UNKNOWN", false, 4),
        None,
        "do not infer unknown age/culture"
    );
    let art = ImprovementArt::parse(
        "FARM { ConstructionTiles 1 ConstructionTiles 3 TerrainEffect { Terrain PLAINS Terrain DESERT TilesetIndex 5 } TerrainEffect { Terrain WATER TilesetIndex 8 } }",
    );
    assert_eq!(
        art.index("FARM", "DESERT", 0, 2),
        Some(5_u16),
        "terrain-specific finished artwork"
    );
    assert_eq!(
        art.index("FARM", "WATER", 0, 2),
        Some(8_u16),
        "not the first terrain recipe"
    );
    assert_eq!(
        art.index("FARM", "PLAINS", 1, 2),
        Some(3_u16),
        "construction artwork is not completed improvement art"
    );
    assert_eq!(
        art.index("UNKNOWN", "PLAINS", 0, 2),
        None,
        "unknown IDs never become farms"
    );
}
