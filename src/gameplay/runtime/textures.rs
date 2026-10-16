//! Original CTP2 texture loading for the gameplay renderer.

extern crate alloc;

use alloc::string::ToString;
use alloc::vec::Vec;
use std::path::Path;

use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{BlendMode, Texture, TextureCreator};
use sdl2::video::WindowContext;

use crate::gameplay::assets::PICTURE_ARCHIVE;
use crate::gameplay::assets::load_unit_sprite;
use crate::gameplay::assets::rim::{RgbaImage, load_rim_rgba_entry};
use crate::gameplay::assets::tile::TileSet;
use crate::gameplay::model::{TerrainKind, UnitKind};
use crate::gameplay::{GameplayError, asset_error, runtime_error};

/// Alternate picture archive containing 555 RIM art.
const PICTURE_ARCHIVE_555: &str = "ctp2_data/default/graphics/pictures/pic555.zfs";
/// Chroma key used by original CTP2 UI RIM assets.
const UI_CHROMA_KEY: (u8, u8, u8) = (248, 0, 248);
/// Terrain tile number for plains.
const PLAINS_TILESET_INDEX: u8 = 1;
/// Terrain tile number for grassland.
const GRASSLAND_TILESET_INDEX: u8 = 4;
/// Terrain tile number for desert.
const DESERT_TILESET_INDEX: u8 = 5;
/// Terrain tile number for swamp.
const SWAMP_TILESET_INDEX: u8 = 6;
/// Terrain tile number for mountain.
const MOUNTAIN_TILESET_INDEX: u8 = 8;
/// Terrain tile number for hills.
const HILL_TILESET_INDEX: u8 = 9;
/// Terrain tile number for shallow water.
const WATER_TILESET_INDEX: u8 = 10;
/// Terrain tile number for forest.
const FOREST_TILESET_INDEX: u8 = 22;
/// Human settler sprite id from `spriteID.txt`.
const SETTLER_SPRITE_ID: u16 = 2;
/// Warrior sprite id from `spriteID.txt`.
const WARRIOR_SPRITE_ID: u16 = 3;

/// Loaded original textures used by the renderer.
pub(super) struct GameplayTextures<'texture> {
    /// Terrain textures keyed by render terrain family.
    terrain: Vec<(TerrainKind, Texture<'texture>)>,
    /// UI chrome textures keyed by asset name.
    ui: Vec<(&'static str, Texture<'texture>)>,
    /// Unit textures keyed by render unit family.
    units: Vec<(UnitKind, Texture<'texture>)>,
}

impl<'texture> GameplayTextures<'texture> {
    /// Stubs missing installation art without touching the filesystem.
    #[cfg(test)]
    pub(super) const fn without_assets_for_test() -> Self {
        Self {
            terrain: Vec::new(),
            ui: Vec::new(),
            units: Vec::new(),
        }
    }

    /// Loads original CTP2 terrain, UI chrome, and unit sprite textures.
    ///
    /// # Errors
    ///
    /// Returns [`GameplayError`] when strict launch requires original art and it cannot be loaded.
    pub(super) fn load(
        installation_root: &Path,
        texture_creator: &'texture TextureCreator<WindowContext>,
        strict: bool,
    ) -> Result<Self, GameplayError> {
        let mut textures = Self {
            terrain: Vec::new(),
            ui: Vec::new(),
            units: Vec::new(),
        };
        if let Err(error) = textures.load_terrain(installation_root, texture_creator) {
            if strict {
                return Err(error);
            }
        }
        textures.load_ui_chrome(installation_root, texture_creator);
        textures.load_unit_sprites(installation_root, texture_creator);
        if strict && textures.terrain.is_empty() {
            return Err(asset_error(
                "original CTP2 terrain art was not loaded; verify the ctp2 installation path contains ctp2_data/default/graphics/tiles/gtset565.til"
                    .to_owned(),
            ));
        }
        if strict && textures.ui_texture("upfg01.rim").is_none() {
            return Err(asset_error(
                "original CTP2 control-panel art was not loaded; verify pic565.zfs/pic555.zfs contain upfg01.rim"
                    .to_owned(),
            ));
        }
        Ok(textures)
    }

    /// Returns a terrain texture for per-cell fog modulation before drawing.
    pub(super) fn terrain_texture(&mut self, terrain: TerrainKind) -> Option<&mut Texture<'texture>> {
        self.terrain
            .iter_mut()
            .find(|entry| entry.0 == terrain)
            .map(|entry| &mut entry.1)
    }

    /// Returns a UI texture.
    pub(super) fn ui_texture(&self, name: &str) -> Option<&Texture<'texture>> {
        self.ui
            .iter()
            .find(|entry| entry.0.eq_ignore_ascii_case(name))
            .map(|entry| &entry.1)
    }

    /// Returns a unit texture.
    pub(super) fn unit_texture(&self, kind: UnitKind) -> Option<&Texture<'texture>> {
        self.units
            .iter()
            .find(|entry| entry.0 == kind)
            .map(|entry| &entry.1)
    }

    /// Loads terrain textures from `gtset565.til`.
    fn load_terrain(
        &mut self,
        installation_root: &Path,
        texture_creator: &'texture TextureCreator<WindowContext>,
    ) -> Result<(), GameplayError> {
        let tile_set = TileSet::load_from_installation(installation_root)?;
        for (terrain, base_type) in terrain_base_types() {
            if let Some(tile) = tile_set.tile_for_base_type(base_type) {
                let texture = texture_from_rgba(texture_creator, &tile.image)?;
                self.terrain.push((terrain, texture));
            }
        }
        Ok(())
    }

    /// Loads UI chrome textures from original picture archives.
    fn load_ui_chrome(
        &mut self,
        installation_root: &Path,
        texture_creator: &'texture TextureCreator<WindowContext>,
    ) {
        for name in [
            "uptb00.rim",
            "uptbtest.rim",
            "uptb02.rim",
            "uptb03.rim",
            "uptg01e.rim",
            "uptg05a.rim",
            "uptg05b.rim",
            "uptg05c.rim",
            "upic34.rim",
            "upic33.rim",
            "upic35.rim",
            "upic36.rim",
            "upfg01.rim",
            "upfg03.rim",
            "upfg04.rim",
            "upst01.rim",
            "upcb14u.rim",
            "upcb14d.rim",
            "upcb14h.rim",
            "upcb14x.rim",
            "uptg20e.rim",
        ] {
            if let Ok(image) = load_ui_rgba(installation_root, name) {
                if let Ok(texture) = texture_from_rgba(texture_creator, &image) {
                    self.ui.push((name, texture));
                }
            }
        }
    }

    /// Loads representative unit sprite textures.
    fn load_unit_sprites(
        &mut self,
        installation_root: &Path,
        texture_creator: &'texture TextureCreator<WindowContext>,
    ) {
        for (kind, sprite_id) in [
            (UnitKind::Settler, SETTLER_SPRITE_ID),
            (UnitKind::Warrior, WARRIOR_SPRITE_ID),
            (UnitKind::Barbarian, WARRIOR_SPRITE_ID),
        ] {
            if let Ok(image) = load_unit_sprite_rgba(installation_root, sprite_id) {
                if let Ok(texture) = texture_from_rgba(texture_creator, &image) {
                    self.units.push((kind, texture));
                }
            }
        }
    }
}

/// Converts RGBA bytes into an SDL texture.
fn texture_from_rgba<'texture>(
    texture_creator: &'texture TextureCreator<WindowContext>,
    image: &RgbaImage,
) -> Result<Texture<'texture>, GameplayError> {
    let mut texture = texture_creator
        .create_texture_static(PixelFormatEnum::RGBA32, image.width, image.height)
        .map_err(|error| runtime_error(error.to_string()))?;
    texture.set_blend_mode(BlendMode::Blend);
    texture
        .update(None, &image.data, image.pitch)
        .map_err(|error| runtime_error(error.to_string()))?;
    Ok(texture)
}

/// Loads a RIM image from either CTP2 picture archive.
fn load_ui_rgba(installation_root: &Path, entry_name: &str) -> Result<RgbaImage, GameplayError> {
    load_rim_rgba_entry(
        installation_root,
        PICTURE_ARCHIVE,
        entry_name,
        Some(UI_CHROMA_KEY),
    )
    .or_else(|_| {
        load_rim_rgba_entry(
            installation_root,
            PICTURE_ARCHIVE_555,
            entry_name,
            Some(UI_CHROMA_KEY),
        )
    })
}

/// Loads the first usable unit sprite frame as RGBA.
fn load_unit_sprite_rgba(
    installation_root: &Path,
    sprite_id: u16,
) -> Result<RgbaImage, GameplayError> {
    let sprite = load_unit_sprite(installation_root, sprite_id)?;
    let frame = sprite
        .actions
        .iter()
        .find_map(|action| action.frames.first())
        .ok_or_else(|| asset_error(format!("unit sprite {sprite_id} had no frames")))?;
    Ok(RgbaImage {
        width: u32::from(frame.width),
        height: u32::from(frame.height),
        pitch: usize::from(frame.width).saturating_mul(4),
        data: frame.rgba.clone(),
    })
}

/// Original terrain base-type mapping.
const fn terrain_base_types() -> [(TerrainKind, u8); 8] {
    [
        (TerrainKind::Grassland, GRASSLAND_TILESET_INDEX),
        (TerrainKind::Plains, PLAINS_TILESET_INDEX),
        (TerrainKind::Forest, FOREST_TILESET_INDEX),
        (TerrainKind::Hills, HILL_TILESET_INDEX),
        (TerrainKind::Mountain, MOUNTAIN_TILESET_INDEX),
        (TerrainKind::Desert, DESERT_TILESET_INDEX),
        (TerrainKind::Swamp, SWAMP_TILESET_INDEX),
        (TerrainKind::Water, WATER_TILESET_INDEX),
    ]
}
