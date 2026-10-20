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

use crate::gameplay::assets::PICTURE_ARCHIVE;
use crate::gameplay::assets::rim::{RgbaImage, load_rim_rgba_entry};
use crate::gameplay::assets::tile::TileSet;
use crate::gameplay::model::{FogState, GameState, MapPoint, TerrainKind, UnitKind};
use crate::gameplay::{GameplayError, asset_error, runtime_error};

/// Source-art names and pixel-accurate map icon colorization.
mod art;
#[cfg(test)]
mod tests;
/// Directional action/frame uploads and sequence sampling.
mod units;

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
    /// Full directional/action sprite groups with lazily uploaded frames.
    units: units::UnitTextures<'texture>,
    /// Original city visual recipes, separate from authoritative gameplay facts.
    city_catalog: crate::gameplay::assets::cities::CityArtCatalog,
    /// GC city frames, distinct from GU units and keyed by remembered shading.
    cities: BTreeMap<(u16, bool), SpriteTexture<'texture>>,
    /// Original resource namespace; this maps art, not deposit locations.
    goods_ids: BTreeMap<String, u16>,
    /// Lazily uploaded resource frames, shared by kind and remembered-fog variant.
    goods: BTreeMap<(u16, bool), SpriteTexture<'texture>>,
    /// Installation root for resource art first encountered during play.
    root: PathBuf,
    /// Original software cursors, uploaded on first use with black chroma key.
    cursors: BTreeMap<super::cursor::CursorKind, Texture<'texture>>,
    /// Complete installed CTP2 roster from Units/newsprite/uniticon databases.
    unit_catalog: BTreeMap<String, crate::gameplay::assets::unit_catalog::UnitArt>,
    /// Lazily uploaded portraits and other original management artwork.
    dynamic_ui: BTreeMap<String, Texture<'texture>>,
    /// Decoded standard tile overlays uploaded only when needed.
    improvements: BTreeMap<u16, Texture<'texture>>,
    /// Live sessions reject missing art; explicit asset-free previews may stub it.
    strict: bool,
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
            units: units::UnitTextures::default(),
            city_catalog: crate::gameplay::assets::cities::CityArtCatalog::default(),
            cities: BTreeMap::new(),
            goods_ids: BTreeMap::new(),
            goods: BTreeMap::new(),
            root: PathBuf::new(),
            cursors: BTreeMap::new(),
            unit_catalog: BTreeMap::new(),
            dynamic_ui: BTreeMap::new(),
            improvements: BTreeMap::new(),
            strict: false,
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
            units: units::UnitTextures::default(),
            city_catalog: match crate::gameplay::assets::cities::CityArtCatalog::load(root) {
                Ok(catalog) => catalog,
                Err(error) if strict => return Err(error),
                Err(_) => crate::gameplay::assets::cities::CityArtCatalog::default(),
            },
            cities: BTreeMap::new(),
            goods_ids: match goods::load_catalog(root) {
                Ok(catalog) => catalog,
                Err(error) if strict => return Err(error),
                Err(_) => BTreeMap::new(),
            },
            goods: BTreeMap::new(),
            root: root.to_path_buf(),
            cursors: BTreeMap::new(),
            unit_catalog: match crate::gameplay::assets::unit_catalog::load(root) {
                Ok(catalog) => catalog,
                Err(error) if strict => return Err(error),
                Err(_) => BTreeMap::new(),
            },
            dynamic_ui: BTreeMap::new(),
            improvements: BTreeMap::new(),
            strict,
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
        textures.units = units::UnitTextures::load(root, strict)?;
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

    /// Loads installed cursor art read-only; no copyrighted data is embedded.
    pub(super) fn cursor_texture(
        &mut self,
        kind: super::cursor::CursorKind,
    ) -> Result<Option<&Texture<'texture>>, GameplayError> {
        let Some(creator) = self.creator else {
            return Ok(None);
        };
        if let alloc::collections::btree_map::Entry::Vacant(entry) = self.cursors.entry(kind) {
            let (number, _) = kind.art();
            let path = self
                .root
                .join("ctp2_data/default/graphics/cursors")
                .join(format!("UC{number:03}.tga"));
            let surface = Surface::from_file(path)
                .and_then(|image| image.convert_format(PixelFormatEnum::RGBA32))
                .map_err(asset_error)?;
            let mut data = surface
                .without_lock()
                .ok_or_else(|| asset_error("cursor surface requires locking".to_owned()))?
                .to_vec();
            for pixel in data.chunks_exact_mut(4) {
                pixel[0] = crate::gameplay::assets::pixel::expand5(pixel[0] >> 3);
                pixel[1] = crate::gameplay::assets::pixel::expand6(pixel[1] >> 2);
                pixel[2] = crate::gameplay::assets::pixel::expand5(pixel[2] >> 3);
                if pixel[..3] == [0, 0, 0] {
                    pixel[3] = 0;
                }
            }
            entry.insert(texture_from_pixels(
                creator,
                surface.width(),
                surface.height(),
                &data,
                usize::try_from(surface.pitch()).map_err(|error| asset_error(error.to_string()))?,
            )?);
        }
        Ok(self.cursors.get(&kind))
    }

    /// Returns original UI art by filename.
    pub(super) fn ui_texture(&self, name: &str) -> Option<&Texture<'texture>> {
        self.ui
            .iter()
            .find(|entry| entry.0.eq_ignore_ascii_case(name))
            .map(|entry| &entry.1)
    }

    /// Shared duration for actor travel and its MOVE animation.
    pub(super) fn move_duration(&self, kind: UnitKind, speed: i32, enabled: bool) -> u64 {
        self.units.duration(kind.sprite_id(), speed, enabled)
    }

    /// Returns the sampled original actor with directional and sequence metadata.
    pub(super) fn unit_texture(
        &mut self,
        sprite_id: u16,
        pose: super::animation::ActorPose,
        duration: u64,
        enabled: bool,
    ) -> Result<Option<units::UnitDraw<'_, 'texture>>, GameplayError> {
        let Some(creator) = self.creator else {
            return Ok(None);
        };
        self.units
            .sample(creator, sprite_id, pose, duration, enabled)
    }

    /// Resolves exact canonical artwork; unknown unit types never become Warriors.
    pub(super) fn unit_art(
        &mut self,
        state: &GameState,
        unit: &crate::gameplay::model::Unit,
    ) -> Result<Option<crate::gameplay::assets::unit_catalog::UnitArt>, GameplayError> {
        let source = state.viewer.as_ref().and_then(|viewer| {
            viewer.units.iter().find(|source| {
                crate::gameplay::model::UnitId::from_server_id(&source.id) == unit.id
            })
        });
        let art = source
            .and_then(|source| self.unit_catalog.get(&source.type_id))
            .cloned()
            .or_else(|| {
                (source.is_none() || unit.kind == UnitKind::Settler).then(|| {
                    crate::gameplay::assets::unit_catalog::UnitArt {
                        sprite: unit.kind.sprite_id(),
                        portrait: format!("upup{:03}l.rim", unit.kind.sprite_id()),
                        icon: format!("upup{:03}a.rim", unit.kind.sprite_id()),
                        label: unit.kind.label().to_owned(),
                    }
                })
            });
        if let Some(art) = art.as_ref().filter(|_| self.creator.is_some()) {
            if let Err(error) = self.units.ensure(&self.root, art.sprite) {
                if self.strict {
                    return Err(error);
                }
                return Ok(None);
            }
        }
        Ok(art)
    }

    /// Duration for an exact retail sprite, not a three-family artwork fallback.
    pub(super) fn unit_duration(&self, id: u16, speed: i32, enabled: bool) -> u64 {
        self.units.duration(id, speed, enabled)
    }

    /// Caches the source `C3Slider` pattern/groove once per source size and pattern.
    pub(super) fn policy_slider_track(
        &mut self,
        pattern: &str,
        width: u32,
        height: u32,
        ticks: Option<(u32, u32, u32)>,
    ) -> Result<Option<&Texture<'texture>>, GameplayError> {
        let Some(creator) = self.creator else {
            return Ok(None);
        };
        let key = format!("c3-slider:{pattern}:{width}x{height}:{ticks:?}");
        if !self.dynamic_ui.contains_key(&key) {
            let name = pattern.to_ascii_lowercase().replace(".tga", ".rim");
            let mut image = match load_ui_rgba(&self.root, &name) {
                Ok(source) => art::c3_slider_track(&source, width, height)?,
                Err(error) if self.strict => return Err(error),
                Err(_error) => return Ok(None),
            };
            if let Some((maximum, spacing, thumb)) = ticks {
                art::c3_slider_ticks(&mut image, maximum, spacing, thumb);
            }
            self.dynamic_ui
                .insert(key.clone(), texture_from_rgba(creator, &image)?);
        }
        Ok(self.dynamic_ui.get(&key))
    }

    /// Caches a native patterned static's packed-channel bevel, not stretched art.
    pub(super) fn beveled_pattern(
        &mut self,
        pattern: &str,
        size: (u32, u32),
        bevel: (u32, bool),
    ) -> Result<Option<&Texture<'texture>>, GameplayError> {
        let Some(creator) = self.creator else {
            return Ok(None);
        };
        let key = format!("static-bevel:{pattern}:{size:?}:{bevel:?}");
        if !self.dynamic_ui.contains_key(&key) {
            let name = pattern.to_ascii_lowercase().replace(".tga", ".rim");
            let mut image = match load_ui_rgba(&self.root, &name) {
                Ok(source) => art::tiled_pattern(&source, size.0, size.1)?,
                Err(error) if self.strict => return Err(error),
                Err(_error) => return Ok(None),
            };
            art::apply_original_bevel(&mut image, bevel.0, bevel.1);
            self.dynamic_ui
                .insert(key.clone(), texture_from_rgba(creator, &image)?);
        }
        Ok(self.dynamic_ui.get(&key))
    }

    /// Loads an original portrait or dialog image on demand without copying assets.
    pub(super) fn dynamic_ui(
        &mut self,
        name: &str,
    ) -> Result<Option<&Texture<'texture>>, GameplayError> {
        let Some(creator) = self.creator else {
            return Ok(None);
        };
        if !self.dynamic_ui.contains_key(name) {
            let image = match load_ui_rgba(&self.root, name) {
                Ok(image) => image,
                Err(error) if self.strict => return Err(error),
                Err(_) => return Ok(None),
            };
            let texture = texture_from_rgba(creator, &image)?;
            self.dynamic_ui.insert(name.to_owned(), texture);
        }
        Ok(self.dynamic_ui.get(name))
    }

    /// Uploads original fortification/standard improvement art on first use.
    pub(super) fn improvement_texture(
        &mut self,
        id: u16,
    ) -> Result<Option<&Texture<'texture>>, GameplayError> {
        let (Some(creator), Some(tiles)) = (self.creator, self.tiles.as_ref()) else {
            return Ok(None);
        };
        if let alloc::collections::btree_map::Entry::Vacant(entry) = self.improvements.entry(id) {
            let Some(image) = tiles.improvement_image(id)? else {
                return Ok(None);
            };
            entry.insert(texture_from_rgba(creator, &image)?);
        }
        Ok(self.improvements.get(&id))
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

    /// Uses source ancient/default styling until viewer-safe style/age metadata
    /// exists. This is visual fallback art, never a claim about a city's nation.
    pub(super) fn city_texture(&mut self, population: u32, water: bool, explored: bool) -> Result<Option<&SpriteTexture<'texture>>, GameplayError> {
        let Some(id) = self.city_catalog.sprite(0_usize, 0_usize, water, population).or_else(|| self.city_catalog.fallback()) else { return Ok(None); };
        let Some(creator) = self.creator else { return Ok(None); };
        if let alloc::collections::btree_map::Entry::Vacant(entry) = self.cities.entry((id, explored)) {
            let sprite = crate::gameplay::assets::cities::load_sprite(&self.root, id)?;
            let mut frame = sprite.actions.into_iter().find(|action| action.action_index == 2_usize).and_then(|action| action.frames.into_iter().next()).ok_or_else(|| asset_error(format!("city sprite GC{id:03} has no idle frame")))?;
            if explored { crate::gameplay::assets::pixel::shade_explored(&mut frame.rgba, true); }
            entry.insert(texture_from_frame(creator, &frame)?);
        }
        Ok(self.cities.get(&(id, explored)))
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
