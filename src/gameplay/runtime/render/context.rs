//! Original C3 context list and centered HelpTile window, composed in SDL.

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::Font;
use sdl2::video::Window;

use super::chrome::{draw_art_rect, draw_dropdown_chrome, tile_texture};
use super::text::{draw_centered_text, draw_text};
use crate::gameplay::model::{FogState, GameState};
use crate::gameplay::runtime::context_input::map_info_close_rect;
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::ui_model::InGameUiModel;
use crate::gameplay::{GameplayError, runtime_error};

/// Draws menus after all map/chrome layers but before the software pointer.
pub(super) fn draw_context(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
    ui: &InGameUiModel,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    if let Some(menu) = ui.context.as_ref() {
        let window = menu.rect;
        draw_dropdown_chrome(
            canvas,
            textures,
            window.x,
            window.y,
            window.width,
            window.height,
        )?;
        for (index, row) in menu.rows.iter().enumerate() {
            let Some(rect) = menu.row_rect(index) else {
                continue;
            };
            let hovered = if menu.keyboard_focus {
                index == menu.focused
            } else {
                ui.pointer.is_some_and(|(x, y)| rect.contains(x, y))
            };
            if hovered && row.enabled() {
                canvas.set_draw_color(Color::RGB(255, 198, 66));
                canvas
                    .fill_rect(Rect::new(rect.x, rect.y, rect.width, rect.height))
                    .map_err(runtime_error)?;
            }
            draw_text(
                canvas,
                font,
                row.label,
                rect.x.saturating_add(4),
                rect.y,
                if row.enabled() {
                    Color::RGB(50, 50, 50)
                } else {
                    Color::RGB(126, 118, 108)
                },
            )?;
        }
    }
    draw_map_info(canvas, textures, state, ui, font)
}

/// Draws only viewer-known terrain details in the original `HelpTile` geometry.
fn draw_map_info(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
    ui: &InGameUiModel,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    if let Some(point) = ui.map_info.filter(|point| {
        state
            .tile(*point)
            .is_some_and(|tile| tile.fog != FogState::Hidden)
    }) {
        let rect = Rect::new(127, 213, 546, 173);
        draw_gold_frame(canvas, textures, rect)?;
        let source = state.viewer.as_ref().and_then(|viewer| {
            viewer
                .tiles
                .iter()
                .find(|tile| tile.x == point.x && tile.y == point.y)
        });
        let title = source.map_or_else(
            || "Terrain".to_owned(),
            |tile| {
                tile.terrain
                    .trim_start_matches("TERRAIN_")
                    .replace('_', " ")
            },
        );
        draw_centered_text(
            canvas,
            font,
            &title,
            Rect::new(147, 233, 506, 20),
            Color::RGB(90, 70, 60),
            false,
        )?;
        for (label, field, x, y) in [
            ("Food:", "food", 152_i32, 283_i32),
            ("Production:", "production", 152_i32, 303_i32),
            ("Movement Cost:", "cost", 152_i32, 323_i32),
            ("Commerce:", "gold", 497_i32, 283_i32),
        ] {
            draw_text(canvas, font, label, x, y, Color::RGB(50, 50, 50))?;
            let value = crate::gameplay::ui_model::observed::tile_value(source, field);
            draw_text(
                canvas,
                font,
                &value,
                x.saturating_add(if x < 400 { 105 } else { 80 }),
                y,
                Color::RGB(50, 50, 50),
            )?;
        }
        draw_text(canvas, font, "Goods:", 497, 303, Color::RGB(50, 50, 50))?;
        let goods = state
            .tile(point)
            .and_then(|tile| tile.good_id.as_deref())
            .unwrap_or("—");
        draw_text(
            canvas,
            font,
            ui.management.book.label(goods),
            567,
            303,
            Color::RGB(50, 50, 50),
        )?;
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas
            .fill_rect(Rect::new(318, 273, 159, 80))
            .map_err(runtime_error)?;
        if let Some(texture) = textures.terrain_texture(state, point)? {
            canvas
                .copy(texture, None, Rect::new(350, 277, 94, 72))
                .map_err(runtime_error)?;
        }
        let close = map_info_close_rect();
        draw_art_rect(
            canvas,
            textures,
            "upbt02u.rim",
            Rect::new(close.x, close.y, close.width, close.height),
        )?;
        draw_centered_text(
            canvas,
            font,
            "Close",
            Rect::new(close.x, close.y, close.width, close.height),
            Color::RGB(50, 50, 50),
            false,
        )?;
    }
    Ok(())
}

/// `c3_PopupWindow`/`fancy.ldl` eight gold border pieces: 17px corners and
/// independently tiled edges, never a stretched copy of an entire background.
pub(super) fn draw_gold_frame(
    canvas: &mut Canvas<Window>,
    textures: &GameplayTextures<'_>,
    rect: Rect,
) -> Result<(), GameplayError> {
    if let Some(pattern) = textures.ui_texture("uptg20e.rim") {
        tile_texture(
            canvas,
            pattern,
            rect.x(),
            rect.y(),
            rect.width(),
            rect.height(),
        )?;
    }
    let right = rect.right().saturating_sub(17);
    let bottom = rect.bottom().saturating_sub(17);
    for (name, x, y, width, height) in [
        ("uptg03a.rim", rect.x(), rect.y(), 17_u32, 17_u32),
        ("uptg03c.rim", right, rect.y(), 17, 17),
        ("uptg03g.rim", rect.x(), bottom, 17, 17),
        ("uptg03i.rim", right, bottom, 17, 17),
        (
            "uptg03b.rim",
            rect.x().saturating_add(17),
            rect.y(),
            rect.width().saturating_sub(34),
            17,
        ),
        (
            "uptg03h.rim",
            rect.x().saturating_add(17),
            bottom,
            rect.width().saturating_sub(34),
            17,
        ),
        (
            "uptg03d.rim",
            rect.x(),
            rect.y().saturating_add(17),
            17,
            rect.height().saturating_sub(34),
        ),
        (
            "uptg03f.rim",
            right,
            rect.y().saturating_add(17),
            17,
            rect.height().saturating_sub(34),
        ),
    ] {
        if let Some(texture) = textures.ui_texture(name) {
            tile_texture(canvas, texture, x, y, width, height)?;
        }
    }
    Ok(())
}
