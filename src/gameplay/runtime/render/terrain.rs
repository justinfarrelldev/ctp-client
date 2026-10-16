//! Ground rendering and tile-local fog, using original CTP2 diamond coverage.

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use super::{draw_cities, draw_tile_diamond, draw_units, tile_color};
use crate::gameplay::assets::tile::{TILE_PIXEL_HEADROOM, TILE_PIXEL_HEIGHT, TILE_PIXEL_WIDTH};
use crate::gameplay::model::{FogState, GameState, MapPoint, TerrainKind, TileFeature};
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::runtime::{
    Camera, DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH, MovementInputState,
};
use crate::gameplay::{GameplayError, runtime_error};

/// Draws the world map, retaining partially visible tiles at every zoom level.
pub(super) fn draw_map(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
    camera: &Camera,
    input: &MovementInputState,
) -> Result<(), GameplayError> {
    for y in 0..state.height {
        for x in 0..state.width {
            let point = MapPoint::new(x, y);
            let Some(tile) = state.tile(point) else {
                continue;
            };
            let (screen_x, screen_y) = camera.tile_to_screen(point);
            let target = Rect::new(
                screen_x,
                screen_y,
                camera.tile_width(),
                camera.tile_height(),
            );
            if target.x() >= i32::try_from(DEFAULT_WINDOW_WIDTH).map_or(1024, |value| value)
                || target.y() >= i32::try_from(DEFAULT_WINDOW_HEIGHT).map_or(768, |value| value)
                || target.right() <= 0
                || target.bottom() <= 0
            {
                continue;
            }
            let fog = if state.filters.fog_of_war {
                tile.fog
            } else {
                FogState::Visible
            };
            let brightness = fog_brightness(fog);
            if let Some(texture) = textures.terrain_texture(tile.terrain) {
                // TiledMap::DrawBlendedTile shades the diamond's pixels, not its
                // bounding rectangle. SDL modulation preserves the decoded alpha
                // silhouette, so fog never paints over a neighboring ground tile.
                // Always reset the modulation, including for visible tiles: each
                // terrain texture is shared by cells with different visibility.
                texture.set_color_mod(brightness, brightness, brightness);
                canvas.copy(texture, None, target).map_err(runtime_error)?;
            } else {
                draw_fallback_tile(canvas, camera, tile.terrain, screen_x, screen_y, brightness)?;
            }
            if state.filters.grid {
                draw_tile_diamond(canvas, camera, screen_x, screen_y, Color::RGB(20, 20, 20))?;
            }
            if tile.feature == Some(TileFeature::Ruin) && tile.fog == FogState::Visible {
                canvas.set_draw_color(Color::RGB(210, 185, 100));
                canvas
                    .fill_rect(Rect::new(screen_x + 40, screen_y + 34, 12, 12))
                    .map_err(runtime_error)?;
            }
        }
    }
    draw_cities(canvas, state, camera)?;
    draw_units(canvas, textures, state, camera, input)
}

/// Returns the original overlay's attenuation, now applied only to terrain pixels.
const fn fog_brightness(fog: FogState) -> u8 {
    match fog {
        FogState::Visible => 255,
        FogState::Explored => 135,
        FogState::Hidden => 35,
    }
}

/// Draws fallback ground when original tiles are unavailable.
fn draw_fallback_tile(
    canvas: &mut Canvas<Window>,
    camera: &Camera,
    terrain: TerrainKind,
    screen_x: i32,
    screen_y: i32,
    brightness: u8,
) -> Result<(), GameplayError> {
    let color = tile_color(terrain);
    canvas.set_draw_color(Color::RGB(
        attenuate(color.r, brightness),
        attenuate(color.g, brightness),
        attenuate(color.b, brightness),
    ));
    let zoom = camera.tile_width().div_euclid(u32::from(TILE_PIXEL_WIDTH));
    let scale = i32::try_from(zoom).map_or(1, |value| value);
    let width = i32::from(TILE_PIXEL_WIDTH);
    let height = i32::from(TILE_PIXEL_HEIGHT);
    let half_height = height.div_euclid(2);
    // Use the original ground raster from tileutils_BorkifyTile, including its
    // two-pixel staircase edges. A shrunken rectangle leaves holes between cells;
    // stretching a full rectangle instead would paint into neighbors/headroom.
    for row in 0..height {
        let inset = if row < half_height {
            (half_height - 1 - row) * 2
        } else {
            (row - half_height) * 2
        };
        let span_width = u32::try_from(width - inset * 2).map_or(0, |value| value);
        canvas
            .fill_rect(Rect::new(
                screen_x + inset * scale,
                screen_y + (i32::from(TILE_PIXEL_HEADROOM) + row) * scale,
                span_width.saturating_mul(zoom),
                zoom,
            ))
            .map_err(runtime_error)?;
    }
    Ok(())
}

/// Matches SDL's integer RGB modulation for fallback colors.
fn attenuate(channel: u8, brightness: u8) -> u8 {
    u8::try_from(
        u16::from(channel)
            .saturating_mul(u16::from(brightness))
            .div_euclid(255_u16),
    )
    .map_or(0_u8, |value| value)
}
