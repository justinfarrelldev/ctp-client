//! Original 24px menu bar and right-to-left resource layout.

use super::chrome::{draw_art, draw_dropdown_chrome, tile_texture};
use super::text::{draw_centered_text, draw_text};
use super::{DROPDOWN_DISABLED_TEXT_COLOR, DROPDOWN_ENABLED_TEXT_COLOR, TOP_MENU_TEXT_COLOR};
use crate::gameplay::model::GameState;
use crate::gameplay::runtime::DEFAULT_WINDOW_WIDTH;
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::ui_model::{InGameUiModel, MENU_BAR_HEIGHT};
use crate::gameplay::{GameplayError, runtime_error};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::Font;
use sdl2::video::Window;

/// Draws source menu art at native size; no stretched font or pale bitmap fallback.
pub(super) fn draw_top_menu(
    canvas: &mut Canvas<Window>,
    textures: &GameplayTextures<'_>,
    state: &GameState,
    ui: &InGameUiModel,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    if let Some(texture) = textures.ui_texture("uptb00.rim") {
        tile_texture(canvas, texture, 0, 0, DEFAULT_WINDOW_WIDTH, MENU_BAR_HEIGHT)?;
    }
    for button in &ui.menu_buttons {
        let rect = Rect::new(button.x, 0, button.width, MENU_BAR_HEIGHT);
        if let Some(texture) = textures.ui_texture("uptbtest.rim") {
            tile_texture(
                canvas,
                texture,
                rect.x(),
                rect.y(),
                rect.width(),
                rect.height(),
            )?;
        }
        draw_centered_text(canvas, font, button.label, rect, TOP_MENU_TEXT_COLOR, true)?;
    }
    draw_resources(canvas, textures, state, font)?;
    if let Some(category) = ui.open_menu {
        if let Some(rect) = ui.dropdown_rect(category) {
            draw_dropdown_chrome(canvas, textures, rect.x, rect.y, rect.width, rect.height)?;
            let shortcut_right = rect
                .x
                .saturating_add(i32::try_from(rect.width).map_or(0_i32, |v| v))
                .saturating_sub(7_i32);
            for (index, item) in ui.items_for(category).iter().enumerate() {
                if let Some(row) = ui.dropdown_row_rect(category, index) {
                    let color = if item.enabled {
                        DROPDOWN_ENABLED_TEXT_COLOR
                    } else {
                        DROPDOWN_DISABLED_TEXT_COLOR
                    };
                    draw_text(
                        canvas,
                        font,
                        item.label,
                        row.x.saturating_add(7_i32),
                        row.y.saturating_add(1_i32),
                        color,
                    )?;
                    let shortcut_width = font
                        .and_then(|font| font.size_of(item.shortcut).ok())
                        .map_or_else(
                            || {
                                u32::try_from(item.shortcut.len())
                                    .map_or(0_u32, |v| v.saturating_mul(6_u32))
                            },
                            |size| size.0,
                        );
                    let shortcut_x = shortcut_right
                        .saturating_sub(i32::try_from(shortcut_width).map_or(0_i32, |v| v));
                    draw_text(
                        canvas,
                        font,
                        item.shortcut,
                        shortcut_x,
                        row.y.saturating_add(1_i32),
                        color,
                    )?;
                }
            }
        }
    }
    Ok(())
}

/// Mirrors `ctp2_MenuBar::AddChild`: counters and icons fill from the right.
fn draw_resources(
    canvas: &mut Canvas<Window>,
    textures: &GameplayTextures<'_>,
    state: &GameState,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    let mut right = i32::try_from(DEFAULT_WINDOW_WIDTH).map_or(800_i32, |v| v);
    for (icon, width, label, meter) in [
        ("upic36.rim", 36_u32, None, Some(Color::RGB(0, 0, 0))),
        ("upic35.rim", 36_u32, None, Some(Color::RGB(255, 255, 0))),
        ("upic33.rim", 82_u32, Some(state.public_works), None),
        ("upic34.rim", 82_u32, Some(state.gold), None),
    ] {
        right = right.saturating_sub(i32::try_from(width).map_or(0_i32, |v| v));
        if let Some(body) = textures.ui_texture("uptb02.rim") {
            tile_texture(canvas, body, right, 0, width, 24)?;
        }
        if let Some(cap) = textures.ui_texture("uptb03.rim") {
            let size = cap.query();
            let cap_x = right.saturating_add(
                i32::try_from(width.saturating_sub(size.width)).map_or(0_i32, |v| v),
            );
            draw_art(canvas, textures, "uptb03.rim", cap_x, 0)?;
        }
        if let Some(color) = meter {
            canvas.set_draw_color(color);
            canvas
                .fill_rect(Rect::new(
                    right.saturating_add(3_i32),
                    7,
                    width.saturating_sub(8_u32),
                    14,
                ))
                .map_err(runtime_error)?;
        }
        if let Some(value) = label {
            draw_centered_text(
                canvas,
                font,
                &value.to_string(),
                Rect::new(right, 0, width, 24),
                Color::RGB(50, 50, 50),
                false,
            )?;
        }
        if let Some(texture) = textures.ui_texture(icon) {
            let size = texture.query();
            right = right.saturating_sub(i32::try_from(size.width).map_or(0_i32, |v| v));
            let y = i32::try_from(
                MENU_BAR_HEIGHT
                    .saturating_sub(size.height)
                    .div_euclid(2_u32),
            )
            .map_or(0_i32, |v| v);
            draw_art(canvas, textures, icon, right, y)?;
        }
    }
    Ok(())
}
