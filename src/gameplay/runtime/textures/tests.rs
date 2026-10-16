//! Read-only source-art tests and pure, viewer-safe terrain classification.

use super::{GameplayTextures, art, load_ui_rgba, terrain_neighborhood, visual_base_type};
use crate::gameplay::model::{FogState, GameState, MapPoint, TerrainKind};
use crate::gameplay::runtime::test_support::with_canvas;
use sdl2::pixels::Color;
use std::path::Path;

#[test]
fn shore_art_uses_known_land_only_and_never_changes_authoritative_terrain() {
    let mut state = GameState::new(24_u16, 18_u16, 5_u8, 42_u64);
    for tile in &mut state.tiles {
        tile.terrain = TerrainKind::Water;
        tile.art_base_type = Some(10_u8);
        tile.fog = FogState::Visible;
    }
    let point = MapPoint::new(4_u16, 4_u16);
    assert_eq!(
        visual_base_type(&state, point),
        10_u8,
        "open shallows retain water art"
    );
    let neighbor = 4_usize * 24_usize + 5_usize;
    state.tiles[neighbor].terrain = TerrainKind::Plains;
    state.tiles[neighbor].art_base_type = Some(1_u8);
    let before = state.clone();
    assert_eq!(
        visual_base_type(&state, point),
        13_u8,
        "visible land/water boundaries select original shore/surf rules"
    );
    assert_eq!(
        state, before,
        "shore classification is visual, not a client-side terraforming command"
    );
    state.tiles[neighbor].fog = FogState::Hidden;
    assert_eq!(
        visual_base_type(&state, point),
        10_u8,
        "hidden land cannot leak through coastline graphics"
    );
}

#[test]
fn authoritative_transition_rules_never_inspect_unexplored_neighbor_terrain() {
    let mut state = GameState::new(24_u16, 18_u16, 5_u8, 42_u64);
    state.authoritative = true;
    for tile in &mut state.tiles {
        tile.terrain = TerrainKind::Mountain;
        tile.art_base_type = Some(8_u8);
        tile.fog = FogState::Hidden;
    }
    let point = MapPoint::new(4_u16, 4_u16);
    state.tiles[100_usize].terrain = TerrainKind::Plains;
    state.tiles[100_usize].art_base_type = Some(1_u8);
    state.tiles[100_usize].fog = FogState::Visible;
    assert_eq!(
        terrain_neighborhood(&state, point),
        [1_u8; 9_usize],
        "all unknown neighbors use the known center's art, never hidden mountains"
    );
    state.tiles[101_usize].fog = FogState::Explored;
    assert!(
        terrain_neighborhood(&state, point).contains(&8_u8),
        "legally remembered neighbor art is available for transitions"
    );
}

#[test]
fn all_four_original_reticle_tgas_load_and_preserve_their_shaped_masks() {
    let result = with_canvas((800_u32, 600_u32), |_canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        for index in 8_u16..=11_u16 {
            let source = load_ui_rgba(&root, &format!("upc{index:03}.rim"))
                .map_err(|error| error.to_string())?;
            let green = art::colorize_map_icon(&source, Color::RGB(0_u8, 252_u8, 0_u8));
            let masks = green
                .data
                .chunks_exact(4_usize)
                .filter(|pixel| *pixel == [0_u8, 252_u8, 0_u8, 255_u8])
                .count();
            let transparent = green
                .data
                .chunks_exact(4_usize)
                .filter(|pixel| pixel[3_usize] == 0_u8)
                .count();
            if (source.width, source.height, masks, transparent)
                != (20_u32, 20_u32, 109_usize, 285_usize)
            {
                return Err(format!("reticle {index} lost its original curved mask"));
            }
        }
        Ok(())
    });
    assert!(result.is_ok(), "retail reticle regression: {result:?}");
}

#[test]
fn composed_texture_cache_tracks_visual_edges_and_rivers_not_coordinates() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let creator = canvas.texture_creator();
        let mut textures = GameplayTextures::load(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"),
            &creator,
            true,
        )
        .map_err(|error| error.to_string())?;
        let mut state = GameState::new(24_u16, 18_u16, 5_u8, 42_u64);
        for tile in &mut state.tiles {
            tile.terrain = TerrainKind::Plains;
            tile.art_base_type = Some(1_u8);
            tile.fog = FogState::Visible;
        }
        let point = MapPoint::new(4_u16, 4_u16);
        if textures
            .terrain_texture(&state, point)
            .map_err(|error| error.to_string())?
            .is_none()
        {
            return Err("terrain texture missing".to_owned());
        }
        let count = textures.terrain.len();
        for _ in 0_u8..3_u8 {
            textures
                .terrain_texture(&state, point)
                .map_err(|error| error.to_string())?;
        }
        if textures.terrain.len() != count {
            return Err("unchanged terrain reallocated textures".to_owned());
        }
        state.tiles[100_usize].river = true;
        textures
            .terrain_texture(&state, point)
            .map_err(|error| error.to_string())?;
        if textures.terrain.len() != count.saturating_add(1_usize) {
            return Err("river changes were incorrectly served stale cached art".to_owned());
        }
        Ok(())
    });
    assert!(result.is_ok(), "terrain cache regression: {result:?}");
}
