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
/// Original city sprites and viewer-safe map caption strips.
mod cities;
/// Source-aligned context menus and terrain information.
mod context;
/// Original Statistics graph pane with only authorized own history.
mod history;
/// Original source-layout manager windows and controls.
mod management;
/// Menu bar, resource counters and weakly-modal dropdowns.
mod menus;
/// Native private notification rows and original gold-framed message windows.
mod messages;
/// Drag-movement overlay drawing.
mod movement;
/// Observed improvement art, remembered ruins and political edges.
mod observations;
/// Explicit source-style operation/target confirmation forms.
mod operations;
/// Complete lower-right control panel.
mod panel;
/// Lower-left radar and filter toolbar.
mod radar;
/// Expanded unit/army portrait grid.
mod stack;
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
pub(super) fn draw_frame<'draw, 'ttf: 'draw, 'rwops: 'draw>(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
    ui: &InGameUiModel,
    camera: &Camera,
    input: &MovementInputState,
    font_source: impl Into<super::fonts::RenderFonts<'draw, 'ttf, 'rwops>>,
) -> Result<(), GameplayError> {
    let fonts = font_source.into();
    let font = fonts.body;
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    draw_map(canvas, textures, state, camera, input)?;
    if let Some(point) = ui.hovered_tile {
        let (x, y) = camera.tile_to_screen(point);
        draw_tile_diamond(canvas, camera, x, y, Color::RGB(255, 255, 0))?;
    }
    movement::draw_planned_path(canvas, input, state, camera, font)?;
    cities::draw_labels(canvas, textures, state, ui, camera, font)?;
    panel::draw_panel(canvas, textures, state, ui, fonts)?;
    if state.filters.radar {
        radar::draw_radar(canvas, textures, state)?;
    }
    menus::draw_top_menu(canvas, textures, state, ui, font)?;
    context::draw_context(canvas, textures, state, ui, font)?;
    management::draw_management(canvas, textures, state, ui, fonts)?;
    history::draw(canvas, state, ui, font)?;
    draw_popups(canvas, textures, state, font)?;
    messages::draw(canvas, textures, state, ui, font)?;
    operations::draw(canvas, textures, state, ui, fonts)?;
    if state.network != crate::gameplay::model::NetworkStatus::Ready {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas
            .fill_rect(Rect::new(230, 28, 340, 24))
            .map_err(runtime_error)?;
        let message = if state.network == crate::gameplay::model::NetworkStatus::Pending {
            "Waiting for server…"
        } else {
            "Resynchronizing…"
        };
        draw_text(canvas, font, message, 240, 31, Color::RGB(255, 230, 140))?;
    }
    draw_cursor(canvas, textures, state, ui, input)
}

/// Captures last-presented own actor identities with the identical source geometry.
pub(super) fn actor_hit_targets(
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
    camera: &Camera,
    input: &MovementInputState,
) -> Result<Vec<super::actor_hits::ActorHit>, GameplayError> {
    actors::hit_targets(textures, state, camera, input)
}

/// Draws the retail pointer last, so captures include the same visible cursor.
fn draw_cursor(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
    ui: &InGameUiModel,
    input: &MovementInputState,
) -> Result<(), GameplayError> {
    let Some((x, y)) = ui.pointer else {
        return Ok(());
    };
    let kind = super::cursor::cursor_for_frame(state, ui, input);
    let (_, hotspot) = kind.art();
    if let Some(texture) = textures.cursor_texture(kind)? {
        let size = texture.query();
        canvas
            .copy(
                texture,
                None,
                Rect::new(
                    x.saturating_sub(hotspot.0),
                    y.saturating_sub(hotspot.1),
                    size.width,
                    size.height,
                ),
            )
            .map_err(runtime_error)?;
    }
    Ok(())
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

/// Source pixel masks and native moving-frame pointer identities.
#[cfg(test)]
mod actor_hit_tests;
#[cfg(test)]
mod animation_tests;
#[cfg(test)]
mod chart_tests;
/// Original GC building-art and remembered-city privacy regressions.
#[cfg(test)]
mod city_tests;
#[cfg(test)]
mod context_tests;
/// Original MessageBoxDialog art/extent readback, no snapshot writes.
#[cfg(test)]
mod confirmation_tests;
#[cfg(test)]
mod cursor_tests;
#[cfg(test)]
mod editor_tests;
#[cfg(test)]
mod fog_tests;
#[cfg(test)]
mod health_tests;
#[cfg(test)]
mod management_tests;
/// Original option slider pixels and popup lifecycle.
#[cfg(test)]
mod option_tests;
/// Native lower-panel portraits and semantic category selection.
#[cfg(test)]
mod panel_tests;
#[cfg(test)]
mod pixel_tests;
/// Original player palette and exact-owner native flags/city/fog rendering.
#[cfg(test)]
mod player_color_tests;
/// Repaired server national territory, city influence and palette identity.
#[cfg(test)]
mod border_tests;
/// Source solid/dashed borders from authoritative observed assignments.
mod borders;
/// Independent source-art checks for native policy sliders and percentage controls.
#[cfg(test)]
mod policy_tests;
/// In-memory masked comparisons against supplied release captures.
#[cfg(test)]
mod reference_tests;
#[cfg(test)]
mod resource_tests;
/// Independent source list-frame/content and discovery-column native pixel checks.
#[cfg(test)]
mod row_tests;
#[cfg(test)]
mod scrollbar_tests;
/// Native five-part city dropdown artwork and detached-popup regressions.
#[cfg(test)]
mod selector_tests;
#[cfg(test)]
mod start_tests;
/// Native sizable-strip clipping regressions.
#[cfg(test)]
mod strip_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod text_tests;
/// Native source-art and newly populated contract-v2 UI regressions.
#[cfg(test)]
mod v2_tests;
#[cfg(test)]
mod visual_tests;
