//! Viewer-safe terrain detail must survive conversion to the render model.

use crate::gameplay::model::GameState;
use crate::gameplay::server::ServerProjection;

#[test]
fn empty_authoritative_projection_never_invents_settlers_or_explored_geography() {
    let state = GameState::from_server_projection(&ServerProjection::default(), 5_u8, 42_u64);
    assert!(
        state.units.is_empty(),
        "empty viewer data must not fall back to a locally generated starting army"
    );
    assert!(
        state
            .tiles
            .iter()
            .all(|tile| tile.fog == crate::gameplay::model::FogState::Hidden
                && tile.feature.is_none()),
        "unobserved cells must not contain generated ruins or explored land"
    );
}

#[test]
fn projection_preserves_original_biomes_and_optional_river_metadata() {
    let projection: ServerProjection = serde_json::from_str(
        r#"{
        "tiles":[
            {"id":"tile-0-0","x":0,"y":0,"terrain":"TERRAIN_WATER_BEACH","river":true},
            {"id":"tile-1-0","x":1,"y":0,"terrain":"TERRAIN_WHITE_MOUNTAIN"},
            {"id":"tile-2-0","x":2,"y":0,"terrain":"TERRAIN_JUNGLE"},
            {"id":"tile-3-0","x":3,"y":0,"terrain":"TERRAIN_PLAINS","good_id":"TERRAIN_PLAINS_GOOD_TWO"}
        ]
    }"#,
    )
    .expect("accurate optional projection fixture parses");
    let state = GameState::from_server_projection(&projection, 5_u8, 42_u64);
    assert_eq!(
        state.tiles[0_usize].art_base_type,
        Some(13_u8),
        "beach must not collapse into a square of generic shallow water"
    );
    assert!(
        state.tiles[0_usize].river,
        "exposed river metadata must survive"
    );
    assert_eq!(
        state.tiles[1_usize].art_base_type,
        Some(21_u8),
        "snow mountain artwork is distinct from regular mountain"
    );
    assert_eq!(
        state.tiles[2_usize].art_base_type,
        Some(7_u8),
        "jungle artwork is distinct from forest"
    );
    assert_eq!(
        state.tiles[3_usize].good_id.as_deref(),
        Some("TERRAIN_PLAINS_GOOD_TWO"),
        "resource identities are preserved, never inferred from base terrain"
    );
    assert!(
        state.tiles[2_usize].good_id.is_none(),
        "omission never invents a jungle resource"
    );
    assert!(
        !state.tiles[1_usize].river,
        "omitted river metadata never creates invented rivers"
    );
}
