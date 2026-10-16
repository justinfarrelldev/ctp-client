//! Bottom-left original radar frame, two toolbar rows and visibility-aware map.

#![expect(
    clippy::default_numeric_fallback,
    reason = "Radar frame offsets are original fixed i32 pixel coordinates"
)]

use super::chrome::{draw_art, draw_art_rect, tile_texture};
use super::tile_color;
use crate::gameplay::model::{FogState, GameState, MapPoint};
use crate::gameplay::runtime::layout::{RADAR_TOOLBAR_HEIGHT, radar_rect};
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::{GameplayError, runtime_error};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// Draws the release radar art rather than a labeled debug rectangle.
pub(super) fn draw_radar(
    canvas: &mut Canvas<Window>,
    textures: &GameplayTextures<'_>,
    state: &GameState,
) -> Result<(), GameplayError> {
    let rect = radar_rect();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.fill_rect(rect).map_err(runtime_error)?;
    let inner = Rect::new(12, rect.y().saturating_add(54_i32), 174, 174);
    if let Some(background) = textures.ui_texture("uptb00.rim") {
        tile_texture(
            canvas,
            background,
            12,
            rect.y().saturating_add(12_i32),
            174,
            RADAR_TOOLBAR_HEIGHT.saturating_sub(12_u32),
        )?;
    }
    for (name, target) in [
        ("uprb11.rim", Rect::new(12, rect.y(), 174, 12)),
        (
            "uprb44.rim",
            Rect::new(
                0,
                rect.y().saturating_add(12_i32),
                12,
                rect.height().saturating_sub(24_u32),
            ),
        ),
        (
            "uprb22.rim",
            Rect::new(
                186,
                rect.y().saturating_add(12_i32),
                12,
                rect.height().saturating_sub(24_u32),
            ),
        ),
        (
            "uprb33.rim",
            Rect::new(12, rect.bottom().saturating_sub(12_i32), 174, 12),
        ),
    ] {
        draw_art_rect(canvas, textures, name, target)?;
    }
    for (name, x, y) in [
        ("uprb01.rim", 0_i32, rect.y()),
        ("uprb02.rim", 186, rect.y()),
        ("uprb04.rim", 0, rect.bottom().saturating_sub(12_i32)),
        ("uprb03.rim", 186, rect.bottom().saturating_sub(12_i32)),
    ] {
        draw_art(canvas, textures, name, x, y)?;
    }
    for (index, name) in [
        "upcb20.rim",
        "upcb21.rim",
        "upcb22.rim",
        "upcb23u.rim",
        "upcb24.rim",
        "upcb25.rim",
        "upcb26.rim",
    ]
    .into_iter()
    .enumerate()
    {
        let x = if index == 6_usize {
            159_i32
        } else {
            12_i32.saturating_add(i32::try_from(index).map_or(0_i32, |v| v.saturating_mul(24_i32)))
        };
        let target = Rect::new(x, rect.y().saturating_add(14_i32), 24, 20);
        let background = if matches!(index, 3_usize | 6_usize) {
            "radar-off"
        } else {
            "radar-on"
        };
        draw_art_rect(canvas, textures, background, target)?;
        draw_art(canvas, textures, name, x.saturating_add(2_i32), target.y())?;
    }
    for (name, x) in [("upcb24u.rim", 12_i32), ("upcb25u.rim", 36_i32)] {
        let target = Rect::new(x, rect.y().saturating_add(34_i32), 24, 20);
        draw_art_rect(canvas, textures, "radar-off", target)?;
        draw_art(canvas, textures, name, x.saturating_add(2_i32), target.y())?;
    }
    draw_minimap(canvas, state, inner)
}

/// Hidden cells stay black; this view never reveals undiscovered server terrain.
fn draw_minimap(
    canvas: &mut Canvas<Window>,
    state: &GameState,
    rect: Rect,
) -> Result<(), GameplayError> {
    let width = i32::try_from(rect.width()).map_or(1_i32, |v| v);
    let height = i32::try_from(rect.height()).map_or(1_i32, |v| v);
    for y in 0_u16..state.height {
        for x in 0_u16..state.width {
            let Some(tile) = state
                .tile(MapPoint::new(x, y))
                .filter(|tile| tile.fog != FogState::Hidden)
            else {
                continue;
            };
            let color = tile_color(tile.terrain);
            canvas.set_draw_color(color);
            let px = rect.x().saturating_add(
                i32::from(x)
                    .saturating_mul(width)
                    .div_euclid(i32::from(state.width).max(1_i32)),
            );
            let py = rect.y().saturating_add(
                i32::from(y)
                    .saturating_mul(height)
                    .div_euclid(i32::from(state.height).max(1_i32)),
            );
            canvas
                .fill_rect(Rect::new(
                    px,
                    py,
                    rect.width()
                        .div_euclid(u32::from(state.width).max(1_u32))
                        .max(1_u32),
                    rect.height()
                        .div_euclid(u32::from(state.height).max(1_u32))
                        .max(1_u32),
                ))
                .map_err(runtime_error)?;
        }
    }
    if let Some(unit) = state.selected_unit() {
        let x = rect.x().saturating_add(
            i32::from(unit.position.x)
                .saturating_mul(width)
                .div_euclid(i32::from(state.width).max(1_i32)),
        );
        let y = rect.y().saturating_add(
            i32::from(unit.position.y)
                .saturating_mul(height)
                .div_euclid(i32::from(state.height).max(1_i32)),
        );
        canvas.set_draw_color(Color::RGB(192, 192, 192));
        canvas
            .draw_rect(Rect::new(
                x.saturating_sub(18_i32),
                y.saturating_sub(26_i32),
                36,
                52,
            ))
            .map_err(runtime_error)?;
        canvas.set_draw_color(Color::RGB(248, 220, 80));
        canvas
            .draw_rect(Rect::new(
                x.saturating_sub(2_i32),
                y.saturating_sub(2_i32),
                5,
                5,
            ))
            .map_err(runtime_error)?;
    }
    Ok(())
}
