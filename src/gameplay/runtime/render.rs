//! Original CTP2 gameplay frame, composed with SDL (no legacy renderer hooks).

extern crate alloc;

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::ttf::Font;
use sdl2::video::Window;

use super::MovementInputState;
use super::textures::GameplayTextures;
use super::view::Camera;
use crate::gameplay::model::{GameState, TerrainKind};
use crate::gameplay::ui_model::InGameUiModel;
use crate::gameplay::{GameplayError, runtime_error};

/// Actor rendering and original sprite anchoring.
mod actors;
/// LDL chrome and original-art copy helpers.
mod chrome;
/// Menu bar, resource counters and weakly-modal dropdowns.
mod menus;
/// Drag-movement overlay drawing.
mod movement;
/// Complete lower-right control panel.
mod panel;
/// Lower-left radar and filter toolbar.
mod radar;
/// Ground/transition rendering and tile-local fog.
mod terrain;
/// Label rendering with installed Times-compatible fonts.
mod text;

use terrain::draw_map;
use text::draw_text;

/// Original `MENU_TITLE_FONT` foreground, not pale text over pale art.
const TOP_MENU_TEXT_COLOR: Color = Color::RGB(90, 70, 60);
/// Enabled dropdown row text over original pale chrome.
const DROPDOWN_ENABLED_TEXT_COLOR: Color = Color::RGB(50, 50, 50);
/// Disabled dropdown row text over original pale chrome.
const DROPDOWN_DISABLED_TEXT_COLOR: Color = Color::RGB(126, 118, 108);

/// Renders one frame without presenting, allowing lossless framebuffer capture.
pub(super) fn draw_frame(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
    ui: &InGameUiModel,
    camera: &Camera,
    input: &MovementInputState,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    draw_map(canvas, textures, state, camera, input)?;
    if let Some(point) = ui.hovered_tile {
        let (x, y) = camera.tile_to_screen(point);
        draw_tile_diamond(canvas, camera, x, y, Color::RGB(255, 255, 0))?;
    }
    movement::draw_planned_path(canvas, input, camera, font)?;
    panel::draw_panel(canvas, textures, state, ui, font)?;
    if state.filters.radar {
        radar::draw_radar(canvas, textures, state)?;
    }
    menus::draw_top_menu(canvas, textures, state, ui, font)?;
    draw_popups(canvas, textures, state, font)
}

/// Draws the diagnostic tile grid at its actual zoomed ground vertices.
fn draw_tile_diamond(
    canvas: &mut Canvas<Window>,
    camera: &Camera,
    screen_x: i32,
    screen_y: i32,
    color: Color,
) -> Result<(), GameplayError> {
    let width = i32::try_from(camera.tile_width()).map_or(94_i32, |v| v);
    let height = i32::try_from(camera.tile_height()).map_or(72_i32, |v| v);
    let half_width = width.div_euclid(2_i32);
    let step = height.div_euclid(3_i32);
    let points = [
        Point::new(
            screen_x,
            screen_y.saturating_add(step.saturating_mul(2_i32)),
        ),
        Point::new(
            screen_x.saturating_add(half_width),
            screen_y.saturating_add(step),
        ),
        Point::new(
            screen_x.saturating_add(width),
            screen_y.saturating_add(step.saturating_mul(2_i32)),
        ),
        Point::new(
            screen_x.saturating_add(half_width),
            screen_y.saturating_add(height),
        ),
        Point::new(
            screen_x,
            screen_y.saturating_add(step.saturating_mul(2_i32)),
        ),
    ];
    canvas.set_draw_color(color);
    canvas.draw_lines(points.as_slice()).map_err(runtime_error)
}

/// Draws pending gameplay messages above the chrome.
fn draw_popups(
    canvas: &mut Canvas<Window>,
    textures: &GameplayTextures<'_>,
    state: &GameState,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    let Some(popup) = state.popups.first() else {
        return Ok(());
    };
    canvas.set_draw_color(Color::RGB(200, 192, 174));
    canvas
        .fill_rect(Rect::new(150, 150, 500, 170))
        .map_err(runtime_error)?;
    chrome::draw_dropdown_chrome(canvas, textures, 150, 150, 500, 170)?;
    for (text, y) in [
        (popup.title.as_str(), 170_i32),
        (popup.body.as_str(), 204_i32),
        ("Click or Esc to continue", 280_i32),
    ] {
        draw_text(canvas, font, text, 170, y, DROPDOWN_ENABLED_TEXT_COLOR)?;
    }
    Ok(())
}

/// Returns fallback terrain colors when source art is deliberately absent in tests.
const fn tile_color(terrain: TerrainKind) -> Color {
    let (red, green, blue) = terrain.fallback_color();
    Color::RGB(red, green, blue)
}

#[cfg(test)]
mod pixel_tests;
#[cfg(test)]
mod resource_tests;
#[cfg(test)]
mod start_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod visual_tests;
