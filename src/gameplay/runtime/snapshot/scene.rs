//! Deterministic visual fixtures, explicitly separate from authoritative games.

use crate::gameplay::model::{FogState, GameState, MapPoint, PlayerId, TerrainKind};

/// Scene to inspect with the real renderer; neither variant contacts a server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewScene {
    /// The existing application-generated preview map.
    Generated,
    /// Known land, connected river, shore, stacked settlers and explored terrain.
    /// This is diagnostic fixture data, never a substitute for a server projection.
    Terrain,
}

/// Creates a reproducible scene without reading or writing configuration files.
pub(in crate::gameplay::runtime) fn preview_state(scene: PreviewScene) -> GameState {
    let mut state = GameState::new(64_u16, 48_u16, 5_u8, 42_u64);
    state.popups.clear();
    if scene == PreviewScene::Generated {
        return state;
    }
    for tile in &mut state.tiles {
        tile.terrain = TerrainKind::Plains;
        tile.art_base_type = Some(1_u8);
        tile.feature = None;
        tile.fog = FogState::Hidden;
        tile.river = false;
    }
    state.units.retain(|unit| unit.owner == PlayerId::Human);
    let center = MapPoint::new(30_u16, 22_u16);
    for unit in &mut state.units {
        unit.position = center;
        unit.movement_points = 100_i16;
    }
    for y in 21_u16..=24_u16 {
        for x in 29_u16..=32_u16 {
            let index = usize::from(y)
                .saturating_mul(usize::from(state.width))
                .saturating_add(usize::from(x));
            let tile = &mut state.tiles[index];
            tile.fog = if y == 21_u16 && x > 30_u16 {
                FogState::Explored
            } else {
                FogState::Visible
            };
            if x == 32_u16 || y == 24_u16 {
                tile.terrain = TerrainKind::Water;
                tile.art_base_type = Some(13_u8);
            } else if x == 31_u16 && y == 21_u16 {
                tile.terrain = TerrainKind::Mountain;
                tile.art_base_type = Some(8_u8);
            }
            tile.river = matches!(
                (x, y),
                (30_u16, 21_u16..=23_u16) | (31_u16 | 32_u16, 23_u16)
            );
        }
    }
    state.gold = 100_i32;
    state
}
