//! Viewer-safe terrain detail must survive conversion to the render model.

use crate::gameplay::model::{GameState, UnitKind};
use crate::gameplay::server::ServerProjection;
use serde_json::{Value, json};

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
fn opaque_unit_types_use_the_exact_server_settler_capability_not_id_heuristics() {
    for (effects, expected) in [
        (
            json!([{"id":"settler","kind":"settler"}]),
            UnitKind::Settler,
        ),
        (Value::Null, UnitKind::Warrior),
        (json!([]), UnitKind::Warrior),
        (json!([{"id":"settler","kind":"combat"}]), UnitKind::Warrior),
        (
            json!([{"id":"raid","kind":"special_settler_slave_raid"}]),
            UnitKind::Warrior,
        ),
    ] {
        let projection: ServerProjection = serde_json::from_value(json!({
            "revision":1_i64,"turn":1_u32,
            "tiles":[{"id":"tile-0-0","x":0_u16,"y":0_u16,"terrain":"plains"}],
            "units":[{"id":"settler-7-1","type_id":"UNIT_SYNTHETIC","owner_id":"account-7","tile_id":"tile-0-0","effects":effects}],
            "cities":null
        })).expect("canonical effects and Go nil slices parse");
        let state = GameState::from_server_projection(&projection, 2_u8, 42_u64);
        assert_eq!(
            state.units[0_usize].kind, expected,
            "only exact settler capability classifies an opaque unit as a settler"
        );
    }
}

#[test]
fn projection_preserves_original_biomes_and_optional_river_metadata() {
    let projection: ServerProjection = serde_json::from_str(
        r#"{
        "revision":1,"turn":1,
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
