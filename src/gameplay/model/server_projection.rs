//! Conversion from authoritative server visibility projections into render state.

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;

use crate::gameplay::server::{ServerProjection, ServerUnitProjection};

use super::generation::{generated_contacts, new_unit, start_popup};
use super::{
    City, CityId, FogState, GameState, GameplayPopup, MIN_WORLD_HEIGHT, MIN_WORLD_WIDTH,
    MapFilters, MapPoint, MapTile, PlayerId, PopupKind, TerrainKind, UnitId, UnitKind,
};

/// Converts an authoritative server terrain id into a render terrain family.
fn terrain_from_server_id(terrain: &str) -> TerrainKind {
    let lowered = terrain.to_ascii_lowercase();
    if lowered.contains("water") || lowered.contains("shelf") || lowered.contains("ocean") {
        TerrainKind::Water
    } else if lowered.contains("forest") || lowered.contains("jungle") {
        TerrainKind::Forest
    } else if lowered.contains("hill") {
        TerrainKind::Hills
    } else if lowered.contains("mountain") || lowered.contains("volcano") {
        TerrainKind::Mountain
    } else if lowered.contains("desert") {
        TerrainKind::Desert
    } else if lowered.contains("swamp") {
        TerrainKind::Swamp
    } else if lowered.contains("grass") {
        TerrainKind::Grassland
    } else {
        TerrainKind::Plains
    }
}

/// Retains every retail terrain family for transition and biome art selection.
fn terrain_art_base(terrain: &str) -> Option<u8> {
    let lower = terrain.to_ascii_lowercase();
    let name = lower.strip_prefix("terrain_").unwrap_or(&lower);
    match name {
        "forest" => Some(22),
        "plains" => Some(1),
        "tundra" => Some(2),
        "glacier" => Some(3),
        "grassland" => Some(4),
        "desert" => Some(5),
        "swamp" => Some(6),
        "jungle" => Some(7),
        "mountain" => Some(8),
        "hill" | "hills" => Some(9),
        "water_shallow" => Some(10),
        "water_deep" => Some(11),
        "water_volcano" => Some(12),
        "water_beach" => Some(13),
        "water_shelf" => Some(14),
        "water_trench" => Some(15),
        "water_rift" => Some(16),
        "dead" => Some(17),
        "brown_hill" => Some(18),
        "brown_mountain" => Some(19),
        "white_hill" => Some(20),
        "white_mountain" => Some(21),
        "water_kelp" => Some(23),
        "water_reef" => Some(24),
        "special1" => Some(25),
        "special2" => Some(26),
        _ => None,
    }
}

/// Uses canonical capabilities for opaque starter types, retaining known
/// retail type-name artwork fallbacks. Instance IDs never determine abilities.
fn unit_kind_from_server_unit(unit: &ServerUnitProjection) -> UnitKind {
    let lowered = unit.type_id.to_ascii_lowercase();
    if unit.effects.iter().any(|effect| effect.kind == "settler") || lowered.contains("settler") {
        UnitKind::Settler
    } else if lowered.contains("barbarian") {
        UnitKind::Barbarian
    } else {
        UnitKind::Warrior
    }
}

/// Converts an authoritative server owner id into a local render owner.
fn player_id_from_server_owner(owner_id: &str,roster:&super::super::server::roster::PlayerRoster) -> PlayerId {
    roster.viewer().map_or_else(||if roster.uses_server_slots(){PlayerId::authenticated(owner_id,"")}else{PlayerId::from_observed(owner_id)},|viewer|PlayerId::authenticated(owner_id,viewer))
}

/// Creates a renderable state from an authoritative server projection.
#[must_use]
pub(in crate::gameplay::model) fn game_state_from_server_projection(
    projection: &ServerProjection,
    empire_count: u8,
    seed: u64,
) -> GameState {
    let width = projection
        .tiles
        .iter()
        .map(|tile| tile.x)
        .max()
        .map_or(MIN_WORLD_WIDTH, |value| {
            value.saturating_add(1).max(MIN_WORLD_WIDTH)
        });
    let height = projection
        .tiles
        .iter()
        .map(|tile| tile.y)
        .max()
        .map_or(MIN_WORLD_HEIGHT, |value| {
            value.saturating_add(1).max(MIN_WORLD_HEIGHT)
        });
    let (width, height) = projection.interface.world.as_ref().and_then(super::super::server::v2::World::dimensions).unwrap_or((width, height));
    let tiles = viewer_tiles(projection, width, height);
    let contacts = generated_contacts(empire_count);
    let tile_lookup = projection
        .tiles
        .iter()
        .map(|tile| (tile.id.clone(), MapPoint::new(tile.x, tile.y)))
        .collect::<BTreeMap<_, _>>();
    let roster=super::super::server::roster::PlayerRoster::from_projection(&projection.interface).unwrap_or_default();
    let units = projection
        .units
        .iter()
        .filter_map(|unit| {
            let position = *tile_lookup.get(&unit.tile_id)?;
            let mut converted = new_unit(
                UnitId::from_server_id(&unit.id),
                unit_kind_from_server_unit(unit),
                player_id_from_server_owner(&unit.owner_id,&roster),
                position,
            );
            converted.movement_points = unit.movement;
            converted.max_movement_points = unit.max_movement.max(unit.movement);
            if unit.facts["asleep"].as_bool() == Some(true) {
                converted.order = super::UnitOrder::Sentinel;
            }
            Some(converted)
        })
        .collect::<Vec<_>>();
    let cities = projection
        .cities
        .iter()
        .filter_map(|city| {
            Some(City {
                id: CityId(stable_id_from_text(&city.id)),
                owner: player_id_from_server_owner(&city.owner_id,&roster),
                position: *tile_lookup.get(&city.tile_id)?,
                name: city.facts["name"]
                    .as_str()
                    .filter(|name| !name.is_empty())
                    .unwrap_or("—")
                    .to_owned(),
                build_item: city.facts["queue"]
                    .as_array()
                    .and_then(|queue| queue.first())
                    .and_then(|item| item["id"].as_str())
                    .unwrap_or_default()
                    .to_owned(),
            })
        })
        .collect::<Vec<_>>();
    let selected_unit = units
        .iter()
        .find(|unit| unit.owner == PlayerId::Human && unit.alive)
        .map(|unit| unit.id);
    GameState {
        authoritative: true,
        width,
        height,
        tiles,
        units,
        cities,
        contacts,
        turn: projection.turn.max(1),
        gold: projection
            .diplomacy
            .as_ref()
            .map_or(0, |diplomacy| diplomacy.gold),
        public_works: projection
            .diplomacy
            .as_ref()
            .map_or(0, |diplomacy| diplomacy.public_works),
        selected_unit,
        popups: vec![start_popup()],
        filters: MapFilters::default(),
        seed,
        city_build_menu: None,
        network: super::NetworkStatus::Ready,
        viewer: Some(projection.clone()),
        roster,
    }
}

/// Fills only viewer-exposed facts; the remaining cells are opaque unknowns,
/// never procedurally generated ruins, resources, rivers or discovered terrain.
fn viewer_tiles(projection: &ServerProjection, width: u16, height: u16) -> Vec<MapTile> {
    let mut tiles = vec![
        MapTile {
            terrain: TerrainKind::Grassland,
            art_base_type: None,
            river: false,
            good_id: None,
            feature: None,
            fog: FogState::Hidden
        };
        usize::from(width).saturating_mul(usize::from(height))
    ];
    for source in &projection.tiles {
        let point = MapPoint::new(source.x, source.y);
        if let Some(tile) =
            tile_index_for_dimensions(width, point).and_then(|index| tiles.get_mut(index))
        {
            tile.terrain = terrain_from_server_id(&source.terrain);
            tile.art_base_type = terrain_art_base(&source.terrain);
            tile.river = source.river == Some(true);
            tile.good_id = source.good_id.clone().filter(|id| !id.is_empty());
            tile.feature = (source.ruin == Some(true)).then_some(super::TileFeature::Ruin);
            tile.fog = if source.stale {
                FogState::Explored
            } else {
                FogState::Visible
            };
        }
    }
    tiles
}

/// Queues a popup confirming server authority for this game.
pub(in crate::gameplay::model) fn queue_server_connected_popup(
    state: &mut GameState,
    game_id: i64,
) {
    state.popups.push(GameplayPopup {
        title: "Authoritative Server Connected".to_owned(),
        body: format!("Open CTP server game #{game_id} is the command authority for this session."),
        kind: PopupKind::GameStart,
    });
}

/// Returns row-major tile index for arbitrary dimensions.
fn tile_index_for_dimensions(width: u16, point: MapPoint) -> Option<usize> {
    usize::from(point.y)
        .checked_mul(usize::from(width))?
        .checked_add(usize::from(point.x))
}

/// Builds a stable numeric id from a server string id.
pub(super) fn stable_id_from_text(text: &str) -> u32 {
    let mut hash = 2_166_136_261_u32;
    for byte in text.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(16_777_619_u32);
    }
    hash.max(1)
}
