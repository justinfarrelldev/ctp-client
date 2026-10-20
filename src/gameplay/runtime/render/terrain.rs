//! Ground rendering and tile-local fog, using original CTP2 diamond coverage.

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use super::actors::{draw_goods, draw_settlement_preview, draw_units};
use super::cities::draw_cities;
use super::{draw_tile_diamond, tile_color};
use crate::gameplay::assets::tile::{TILE_PIXEL_HEADROOM, TILE_PIXEL_HEIGHT, TILE_PIXEL_WIDTH};
use crate::gameplay::model::{FogState, GameState, TerrainKind, TileFeature};
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::runtime::view::painter_order;
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
    // Painter order follows screen depth, not map row-major storage. Terrain
    // hats can overhang their neighbors and must never be erased by a back row.
    for point in painter_order(state.width, state.height) {
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
        if target.x() >= i32::try_from(DEFAULT_WINDOW_WIDTH).map_or(800_i32, |value| value)
            || target.y() >= i32::try_from(DEFAULT_WINDOW_HEIGHT).map_or(600_i32, |value| value)
            || target.right() <= 0_i32
            || target.bottom() <= 0_i32
        {
            continue;
        }
        let fog =
            if state.filters.fog_of_war || (state.authoritative && tile.fog == FogState::Hidden) {
                tile.fog
            } else {
                FogState::Visible
            };
        if fog == FogState::Hidden {
            // Unknown land is an opaque ground diamond, not a dim preview
            // of the terrain or a black mountain-shaped information leak.
            draw_fallback_tile(
                canvas,
                camera,
                tile.terrain,
                screen_x,
                screen_y,
                FogState::Hidden,
            )?;
            continue;
        }
        if let Some(texture) = textures.terrain_texture(state, point)? {
            // Cached fog variants reproduce the source 16/32 packed-channel
            // blend while preserving diamond/hat alpha. Never shade a tile's
            // whole bounding rectangle or mutate a shared texture's tint.
            canvas.copy(texture, None, target).map_err(runtime_error)?;
        } else {
            draw_fallback_tile(canvas, camera, tile.terrain, screen_x, screen_y, fog)?;
        }
        if state.filters.grid {
            draw_tile_diamond(canvas, camera, screen_x, screen_y, Color::RGB(20, 20, 20))?;
        }
        if tile.feature == Some(TileFeature::Ruin) && tile.fog == FogState::Visible {
            canvas.set_draw_color(Color::RGB(210, 185, 100));
            canvas
                .fill_rect(Rect::new(
                    screen_x.saturating_add(40_i32),
                    screen_y.saturating_add(34_i32),
                    12,
                    12,
                ))
                .map_err(runtime_error)?;
        }
    }
    draw_goods(canvas, textures, state, camera)?;
    draw_cities(canvas, textures, state, camera)?;
    draw_settlement_preview(canvas, state, camera)?;
    draw_units(canvas, textures, state, camera, input)
}

/// Draws fallback ground when original tiles are unavailable.
fn draw_fallback_tile(
    canvas: &mut Canvas<Window>,
    camera: &Camera,
    terrain: TerrainKind,
    screen_x: i32,
    screen_y: i32,
    fog: FogState,
) -> Result<(), GameplayError> {
    let color = tile_color(terrain);
    let mut pixel = [color.r, color.g, color.b, 255_u8];
    match fog {
        FogState::Visible => {}
        FogState::Explored => crate::gameplay::assets::pixel::shade_explored(&mut pixel, true),
        FogState::Hidden => pixel = [0_u8, 0_u8, 0_u8, 255_u8],
    }
    canvas.set_draw_color(Color::RGBA(pixel[0], pixel[1], pixel[2], pixel[3]));
    let zoom = camera.tile_width().div_euclid(u32::from(TILE_PIXEL_WIDTH));
    let scale = i32::try_from(zoom).map_or(1_i32, |value| value);
    let width = i32::from(TILE_PIXEL_WIDTH);
    let height = i32::from(TILE_PIXEL_HEIGHT);
    let half_height = height.div_euclid(2_i32);
    // Use the original ground raster from tileutils_BorkifyTile, including its
    // two-pixel staircase edges. A shrunken rectangle leaves holes between cells;
    // stretching a full rectangle instead would paint into neighbors/headroom.
    for row in 0_i32..height {
        let inset = if row < half_height {
            half_height
                .saturating_sub(1_i32)
                .saturating_sub(row)
                .saturating_mul(2_i32)
        } else {
            row.saturating_sub(half_height).saturating_mul(2_i32)
        };
        let span_width = u32::try_from(width.saturating_sub(inset.saturating_mul(2_i32)))
            .map_or(0_u32, |value| value);
        canvas
            .fill_rect(Rect::new(
                screen_x.saturating_add(inset.saturating_mul(scale)),
                screen_y.saturating_add(
                    i32::from(TILE_PIXEL_HEADROOM)
                        .saturating_add(row)
                        .saturating_mul(scale),
                ),
                span_width.saturating_mul(zoom),
                zoom,
            ))
            .map_err(runtime_error)?;
    }
    Ok(())
}
