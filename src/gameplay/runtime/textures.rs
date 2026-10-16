//! Original terrain composition, UI artwork and actor textures.

extern crate alloc;

use crate::gameplay::assets::goods;
use crate::gameplay::assets::sprite::SpriteFrame;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use sdl2::image::LoadSurface;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::{BlendMode, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::WindowContext;
use std::path::{Path, PathBuf};

use crate::gameplay::assets::rim::{RgbaImage, load_rim_rgba_entry};
use crate::gameplay::assets::tile::TileSet;
use crate::gameplay::assets::{PICTURE_ARCHIVE, load_unit_sprite};
use crate::gameplay::model::{FogState, GameState, MapPoint, TerrainKind, UnitKind};
use crate::gameplay::{GameplayError, asset_error, runtime_error};

/// Source-art names and pixel-accurate map icon colorization.
mod art;
#[cfg(test)]
mod tests;

/// Terrain cache key: family, variant, four edges, river and packed-channel fog.
type TerrainTextureKey = (u8, u16, [u8; 4], Option<usize>, bool);
/// Bounds persistent GPU/SDL terrain storage to about 14 MiB of RGBA art.
const TERRAIN_CACHE_LIMIT: usize = 512;
/// Original neighborhood order: W,NW,N,SW,center,NE,S,SE,E.
const NEIGHBOR_OFFSETS: [(i16, i16); 9] = [
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, 1),
    (0, 0),
    (0, -1),
    (1, 1),
    (1, 0),
    (1, -1),
];

/// Full-resolution actor frame and original anchor (never squeezed to a tile).
pub(super) struct SpriteTexture<'texture> {
    /// Original full-resolution frame.
    pub(super) texture: Texture<'texture>,
    /// Sprite hotspot relative to frame origin.
    pub(super) hotpoint: (i32, i32),
}

/// A unit frame with its additional shield/health anchor.
pub(super) struct UnitTexture<'texture> {
    /// Original unit frame and hotspot.
    pub(super) sprite: SpriteTexture<'texture>,
    /// Tile-relative stacking/health shield origin.
    pub(super) shield: (i32, i32),
}

/// Loaded original textures. Composed diamonds are cached by variant and edges,
/// not by map coordinate, and remain valid across authoritative projection updates.
pub(super) struct GameplayTextures<'texture> {
    /// Borrowed SDL creator for lazily uploaded terrain and colorized map icons.
    creator: Option<&'texture TextureCreator<WindowContext>>,
    /// Parsed transitions, transform rules, raw bases and terrain hats.
    tiles: Option<TileSet>,
    /// Cached composed terrain, keyed by chosen variant and four neighbors.
    terrain: BTreeMap<TerrainTextureKey, Texture<'texture>>,
    /// UI chrome keyed by original asset name.
    ui: Vec<(&'static str, Texture<'texture>)>,
    /// Source map icons, including brackets, flags and numbered heralds.
    icons: BTreeMap<u16, RgbaImage>,
    /// Uploaded map icons keyed by index and replacement color.
    colored_icons: BTreeMap<(u16, [u8; 3]), Texture<'texture>>,
    /// Idle actor textures keyed by client unit family.
    units: Vec<(UnitKind, UnitTexture<'texture>)>,
    /// Original resource namespace; this maps art, not deposit locations.
    goods_ids: BTreeMap<String, u16>,
    /// Lazily uploaded resource frames, shared by kind and remembered-fog variant.
    goods: BTreeMap<(u16, bool), SpriteTexture<'texture>>,
    /// Installation root for resource art first encountered during play.
    root: PathBuf,
}

impl<'texture> GameplayTextures<'texture> {
    /// Stubs art entirely in memory for renderer fallback tests.
    #[cfg(test)]
    pub(super) fn without_assets_for_test() -> Self {
        Self {
            creator: None,
            tiles: None,
            terrain: BTreeMap::new(),
            ui: Vec::new(),
            icons: BTreeMap::new(),
            colored_icons: BTreeMap::new(),
            units: Vec::new(),
            goods_ids: BTreeMap::new(),
            goods: BTreeMap::new(),
            root: PathBuf::new(),
        }
    }

    /// Loads original art read-only. Strict mode never substitutes debug geometry.
    pub(super) fn load(
        root: &Path,
        creator: &'texture TextureCreator<WindowContext>,
        strict: bool,
    ) -> Result<Self, GameplayError> {
        let tiles = match TileSet::load_from_installation(root) {
            Ok(tiles) => Some(tiles),
            Err(error) if strict => return Err(error),
            Err(_) => None,
        };
        let mut textures = Self {
            creator: Some(creator),
            tiles,
            terrain: BTreeMap::new(),
            ui: Vec::new(),
            icons: BTreeMap::new(),
            colored_icons: BTreeMap::new(),
            units: Vec::new(),
            goods_ids: match goods::load_catalog(root) {
                Ok(catalog) => catalog,
                Err(error) if strict => return Err(error),
                Err(_) => BTreeMap::new(),
            },
            goods: BTreeMap::new(),
            root: root.to_path_buf(),
        };
        for &name in art::UI_NAMES {
            match load_ui_rgba(root, name).and_then(|mut image| {
                if name == "uptbtest.rim" {
                    art::apply_original_bevel(&mut image, 2_u32, false);
                }
                texture_from_rgba(creator, &image)
            }) {
                Ok(texture) => textures.ui.push((name, texture)),
                Err(error) if strict => return Err(error),
                Err(_) => {}
            }
        }
        if let Ok(pattern) = load_ui_rgba(root, "uptg32.rim") {
            for (pressed, key) in [(false, "radar-off"), (true, "radar-on")] {
                let background =
                    art::tiled_pattern(&pattern, 24_u32, 20_u32).and_then(|mut image| {
                        art::apply_original_bevel(&mut image, 3_u32, pressed);
                        texture_from_rgba(creator, &image)
                    });
                match background {
                    Ok(texture) => textures.ui.push((key, texture)),
                    Err(error) if strict => return Err(error),
                    Err(_) => {}
                }
            }
        }
        for index in 1_u16..=23_u16 {
            match load_ui_rgba(root, &format!("upc{index:03}.rim")) {
                Ok(image) => {
                    textures.icons.insert(index, image);
                }
                Err(error) if strict => return Err(error),
                Err(_) => {}
            }
        }
        for (kind, sprite_id) in [
            (UnitKind::Settler, 2_u16),
            (UnitKind::Warrior, 3_u16),
            (UnitKind::Barbarian, 3_u16),
        ] {
            match load_idle_actor(root, creator, sprite_id) {
                Ok(texture) => textures.units.push((kind, texture)),
                Err(error) if strict => return Err(error),
                Err(_) => {}
            }
        }
        Ok(textures)
    }

    /// Returns a cached terrain texture using the release neighborhood rules.
    pub(super) fn terrain_texture(
        &mut self,
        state: &GameState,
        point: MapPoint,
    ) -> Result<Option<&Texture<'texture>>, GameplayError> {
        let (Some(tiles), Some(creator)) = (self.tiles.as_ref(), self.creator) else {
            return Ok(None);
        };
        let neighborhood = terrain_neighborhood(state, point);
        let variant = u32::from(point.x)
            .saturating_mul(17_u32)
            .saturating_add(u32::from(point.y).saturating_mul(31_u32));
        let number = tiles.select_tile(neighborhood, variant);
        let neighbors = [
            neighborhood[3],
            neighborhood[1],
            neighborhood[5],
            neighborhood[7],
        ];
        let river_neighbors = NEIGHBOR_OFFSETS.map(|(dx, dy)| {
            point
                .offset(dx, dy, state.width, state.height)
                .and_then(|neighbor| state.tile(neighbor))
                .is_some_and(|tile| tile.river && tile.fog != FogState::Hidden)
        });
        let river = if river_neighbors[4] {
            tiles.river_piece(river_neighbors, neighborhood[4] == 13_u8)
        } else {
            None
        };
        let explored = state.filters.fog_of_war
            && state
                .tile(point)
                .is_some_and(|tile| tile.fog == FogState::Explored);
        let key = (neighborhood[4], number, neighbors, river, explored);
        if self.terrain.len() >= TERRAIN_CACHE_LIMIT && !self.terrain.contains_key(&key) {
            // A viewport uses far fewer entries than this. Clearing cold art
            // amortizes uploads while bounding long-running terraform sessions.
            self.terrain.clear();
        }
        if let alloc::collections::btree_map::Entry::Vacant(entry) = self.terrain.entry(key) {
            let mut image = tiles.compose_with_river(neighborhood[4], neighbors, number, river)?;
            if explored {
                crate::gameplay::assets::pixel::shade_explored(
                    &mut image.data,
                    tiles.format == crate::gameplay::assets::tile::PixelFormat::Rgb565,
                );
            }
            entry.insert(texture_from_rgba(creator, &image)?);
        }
        Ok(self.terrain.get(&key))
    }

    /// Returns original UI art by filename.
    pub(super) fn ui_texture(&self, name: &str) -> Option<&Texture<'texture>> {
        self.ui
            .iter()
            .find(|entry| entry.0.eq_ignore_ascii_case(name))
            .map(|entry| &entry.1)
    }

    /// Returns the original idle actor and its hotspot/shield metadata.
    pub(super) fn unit_texture(&self, kind: UnitKind) -> Option<&UnitTexture<'texture>> {
        self.units
            .iter()
            .find(|entry| entry.0 == kind)
            .map(|entry| &entry.1)
    }

    /// Returns original resource art only for an explicitly projected good ID.
    pub(super) fn good_texture(
        &mut self,
        good_id: &str,
        explored: bool,
    ) -> Result<Option<&SpriteTexture<'texture>>, GameplayError> {
        let Some(id) = self.goods_ids.get(&good_id.to_ascii_uppercase()).copied() else {
            return Ok(None);
        };
        let Some(creator) = self.creator else {
            return Ok(None);
        };
        let key = (id, explored);
        if let alloc::collections::btree_map::Entry::Vacant(entry) = self.goods.entry(key) {
            let sprite = goods::load_good_sprite(&self.root, id)?;
            let mut frame = sprite
                .actions
                .into_iter()
                .find_map(|action| action.frames.into_iter().next())
                .ok_or_else(|| asset_error(format!("resource sprite {id} has no idle frame")))?;
            if explored {
                crate::gameplay::assets::pixel::shade_explored(&mut frame.rgba, true);
            }
            entry.insert(texture_from_frame(creator, &frame)?);
        }
        Ok(self.goods.get(&key))
    }

    /// Colorizes only source cyan mask pixels, preserving black, white and shading.
    pub(super) fn map_icon(
        &mut self,
        index: u16,
        color: Color,
    ) -> Result<Option<&Texture<'texture>>, GameplayError> {
        let key = (index, [color.r, color.g, color.b]);
        if let alloc::collections::btree_map::Entry::Vacant(entry) = self.colored_icons.entry(key) {
            let (Some(creator), Some(image)) = (self.creator, self.icons.get(&index)) else {
                return Ok(None);
            };
            entry.insert(texture_from_rgba(
                creator,
                &art::colorize_map_icon(image, color),
            )?);
        }
        Ok(self.colored_icons.get(&key))
    }
}

/// Converts RGBA into a pixel-exact alpha-blended SDL texture.
fn texture_from_rgba<'texture>(
    creator: &'texture TextureCreator<WindowContext>,
    image: &RgbaImage,
) -> Result<Texture<'texture>, GameplayError> {
    texture_from_pixels(creator, image.width, image.height, &image.data, image.pitch)
}

/// Uploads borrowed RGBA pixels without cloning decoded sprite frames.
fn texture_from_pixels<'texture>(
    creator: &'texture TextureCreator<WindowContext>,
    width: u32,
    height: u32,
    pixels: &[u8],
    pitch: usize,
) -> Result<Texture<'texture>, GameplayError> {
    let mut texture = creator
        .create_texture_static(PixelFormatEnum::RGBA32, width, height)
        .map_err(|error| runtime_error(error.to_string()))?;
    texture.set_blend_mode(BlendMode::Blend);
    texture
        .update(None, pixels, pitch)
        .map_err(|error| runtime_error(error.to_string()))?;
    Ok(texture)
}

/// Tries both release picture archives, preserving source chroma-key transparency.
fn load_ui_rgba(root: &Path, name: &str) -> Result<RgbaImage, GameplayError> {
    // TileSet::LoadMapIcons prefers loose TGA files. In the retail install
    // UPC008–011 exist ONLY as TGAs, not as entries in either picture archive.
    if let Some(stem) = name.strip_suffix(".rim") {
        let directory = root.join("ctp2_data/default/graphics/pictures");
        for filename in [
            format!("{}.TGA", stem.to_ascii_uppercase()),
            format!("{stem}.tga"),
        ] {
            let path = directory.join(filename);
            if path
                .try_exists()
                .map_err(|error| asset_error(error.to_string()))?
            {
                let surface = Surface::from_file(path)
                    .and_then(|surface| surface.convert_format(PixelFormatEnum::RGBA32))
                    .map_err(asset_error)?;
                let bytes = surface
                    .without_lock()
                    .ok_or_else(|| asset_error("TGA surface requires locking".to_owned()))?;
                return Ok(RgbaImage {
                    width: surface.width(),
                    height: surface.height(),
                    pitch: usize::try_from(surface.pitch())
                        .map_err(|error| asset_error(error.to_string()))?,
                    data: bytes.to_vec(),
                });
            }
        }
    }
    // Map icons encode magenta as a destination-shadow run, unlike UI chrome
    // where it is a chroma key. Preserve it for the map-icon conversion pass.
    let chroma_key = (!name.starts_with("upc0")).then_some((248_u8, 0_u8, 248_u8));
    load_rim_rgba_entry(root, PICTURE_ARCHIVE, name, chroma_key).or_else(|_| {
        load_rim_rgba_entry(
            root,
            "ctp2_data/default/graphics/pictures/pic555.zfs",
            name,
            chroma_key,
        )
    })
}

/// Uploads a full-size frame and retains its original source anchor.
fn texture_from_frame<'texture>(
    creator: &'texture TextureCreator<WindowContext>,
    frame: &SpriteFrame,
) -> Result<SpriteTexture<'texture>, GameplayError> {
    Ok(SpriteTexture {
        texture: texture_from_pixels(
            creator,
            u32::from(frame.width),
            u32::from(frame.height),
            &frame.rgba,
            usize::from(frame.width).saturating_mul(4_usize),
        )?,
        hotpoint: frame.hotpoint,
    })
}

/// Uses `UNITACTION_IDLE` and default facing 3, falling back to MOVE like `UnitActor`.
fn load_idle_actor<'texture>(
    root: &Path,
    creator: &'texture TextureCreator<WindowContext>,
    sprite_id: u16,
) -> Result<UnitTexture<'texture>, GameplayError> {
    let sprite = load_unit_sprite(root, sprite_id)?;
    let action = sprite
        .actions
        .iter()
        .find(|action| action.action_index == 2_usize)
        .or_else(|| {
            sprite
                .actions
                .iter()
                .find(|action| action.action_index == 0_usize)
        })
        .ok_or_else(|| asset_error(format!("unit {sprite_id} has no idle/move action")))?;
    let frame = action
        .frames
        .iter()
        .find(|frame| frame.facing == 3_u8)
        .or_else(|| action.frames.first())
        .ok_or_else(|| asset_error(format!("unit {sprite_id} has no idle frame")))?;
    let shield = sprite
        .shield_points
        .get(action.action_index)
        .map_or((70_i32, 10_i32), |points| points[0]);
    Ok(UnitTexture {
        sprite: texture_from_frame(creator, frame)?,
        shield,
    })
}

/// Returns the release tileset index of the supported authoritative terrain family.
const fn base_type(terrain: TerrainKind) -> u8 {
    match terrain {
        TerrainKind::Plains => 1,
        TerrainKind::Grassland => 4,
        TerrainKind::Desert => 5,
        TerrainKind::Swamp => 6,
        TerrainKind::Mountain => 8,
        TerrainKind::Hills => 9,
        TerrainKind::Water => 10,
        TerrainKind::Forest => 22,
    }
}

/// Chooses source shore artwork at known land/water boundaries without altering
/// gameplay terrain or inferring any terrain fact from undiscovered neighbors.
fn visual_base_type(state: &GameState, point: MapPoint) -> u8 {
    let Some(tile) = state.tile(point) else {
        return 4_u8;
    };
    let base = tile
        .art_base_type
        .unwrap_or_else(|| base_type(tile.terrain));
    if matches!(base, 10_u8 | 11_u8 | 14_u8 | 15_u8)
        && NEIGHBOR_OFFSETS.iter().any(|&(dx, dy)| {
            point
                .offset(dx, dy, state.width, state.height)
                .and_then(|neighbor| state.tile(neighbor))
                .is_some_and(|neighbor| {
                    neighbor.fog != FogState::Hidden
                        && !matches!(
                            neighbor
                                .art_base_type
                                .unwrap_or_else(|| base_type(neighbor.terrain)),
                            10_u8..=16_u8 | 23_u8 | 24_u8
                        )
                })
        })
    {
        13_u8
    } else {
        base
    }
}

/// Maps client diagonal-grid neighbors to the release's W,NW,N,SW,C,NE,S,SE,E.
fn terrain_neighborhood(state: &GameState, point: MapPoint) -> [u8; 9] {
    let center = visual_base_type(state, point);
    NEIGHBOR_OFFSETS.map(|(dx, dy)| {
        point
            .offset(dx, dy, state.width, state.height)
            .filter(|neighbor| {
                !state.authoritative
                    || state
                        .tile(*neighbor)
                        .is_some_and(|tile| tile.fog != FogState::Hidden)
            })
            .map_or(center, |neighbor| visual_base_type(state, neighbor))
    })
}
