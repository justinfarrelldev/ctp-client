//! Pure viewer-safe fixtures matching server events/visibility/start contracts.
//!
//! No network or filesystem access. These are not copied live player/world data.

use serde_json::{Value, json};

/// Models retail start for account 7 at (41,27), away from preview-map defaults.
/// `visibility.addSight` uses Manhattan radius max(2, `visibility_range`), hence
/// thirteen unobstructed plains cells. Go emits nil `Cities` as JSON null.
pub fn starting_projection_json() -> Value {
    let mut tiles = Vec::new();
    for x in 39_u16..=43_u16 {
        for y in 25_u16..=29_u16 {
            if x.abs_diff(41_u16).saturating_add(y.abs_diff(27_u16)) > 2_u16 {
                continue;
            }
            tiles.push(json!({
                "id": format!("tile-{x}-{y}"), "x": x, "y": y,
                "terrain": "TERRAIN_PLAINS", "stale": false, "observed_turn": 1_u32,
                "planning": {"cost": 100_u16, "domains": ["land"], "accessible": true, "enemy_zoc": false}
            }));
        }
    }
    json!({
        "revision": 1_i64, "turn": 1_u32,
        "tiles": tiles,
        "units": [{"id": "settler-7-1", "type_id": "UNIT_SETTLER", "owner_id": "account-7",
            "tile_id": "tile-41-27", "fortified": false, "domain": "land",
            "movement": 100_i16, "max_movement": 100_i16,
            "health": 10_u16, "max_health": 10_u16, "visibility_range": 1_u16}],
        "cities": null, "diplomacy": {}, "planning_status": "open"
    })
}

/// Models the running server's default synthetic starter, which is a settler
/// because of its canonical `settler` effect, not its opaque type name.
pub fn synthetic_starting_resync_event() -> String {
    let mut projection = starting_projection_json();
    if let Some(tiles) = projection["tiles"].as_array_mut() {
        for tile in tiles {
            tile["terrain"] = json!("plains");
            tile["planning"]["cost"] = json!(1_u16);
        }
    }
    let unit = &mut projection["units"][0_usize];
    unit["type_id"] = json!("UNIT_SYNTHETIC");
    unit["movement"] = json!(2_i16);
    unit["max_movement"] = json!(2_i16);
    unit["visibility_range"] = json!(2_u16);
    unit["effects"] = json!([
        {"id":"settler", "kind":"settler"},
        {"id":"capture", "kind":"special_capture"},
        {"id":"embassy", "kind":"special_embassy"},
        {"id":"steal", "kind":"special_steal", "value":100_u16},
        {"id":"terraform", "kind":"special_terraform", "value":1_u16},
        {"id":"terraform-turns", "kind":"special_terraform_turns", "value":1_u16}
    ]);
    format!(
        "id: 1\nevent: resync\ndata: {}\n\n",
        json!({"current_revision":1_i64, "snapshot":projection})
    )
}

/// Matches `events.Store.resync`, not the unrelated direct command-receipt shape.
pub fn starting_resync_event() -> String {
    format!(
        "id: 1\nevent: resync\ndata: {}\n\n",
        json!({"current_revision": 1_i64, "snapshot": starting_projection_json()})
    )
}
