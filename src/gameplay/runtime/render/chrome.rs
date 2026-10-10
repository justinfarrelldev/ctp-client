//! CTP2 UI chrome drawing helpers for gameplay rendering.

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

use crate::gameplay::ui_model::MENU_DROPDOWN_BORDER;
use crate::gameplay::{GameplayError, runtime_error};

use super::super::textures::GameplayTextures;

/// Draws the CTP2 dropdown listbox chrome from `CTP2_MENU`.
pub(super) fn draw_dropdown_chrome(
    canvas: &mut Canvas<Window>,
    textures: &GameplayTextures<'_>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(), GameplayError> {
    canvas.set_draw_color(Color::RGB(196, 184, 142));
    canvas
        .fill_rect(Rect::new(x, y, width, height))
        .map_err(runtime_error)?;
    let border = i32::try_from(MENU_DROPDOWN_BORDER).map_or(0_i32, |value| value);
    let inner_width = width.saturating_sub(MENU_DROPDOWN_BORDER.saturating_mul(2));
    let inner_height = height.saturating_sub(MENU_DROPDOWN_BORDER.saturating_mul(2));
    if let Some(texture) = textures.ui_texture("uptg01e.rim") {
        tile_texture(
            canvas,
            texture,
            x.saturating_add(border),
            y.saturating_add(border),
            inner_width,
            inner_height,
        )?;
    }
    if let (Some(left), Some(center), Some(right)) = (
        textures.ui_texture("uptg05a.rim"),
        textures.ui_texture("uptg05b.rim"),
        textures.ui_texture("uptg05c.rim"),
    ) {
        draw_three_part_horizontal(canvas, left, center, right, Rect::new(x, y, width, height))?;
    } else {
        canvas.set_draw_color(Color::RGB(84, 70, 46));
        canvas
            .draw_rect(Rect::new(x, y, width, height))
            .map_err(runtime_error)?;
    }
    Ok(())
}

/// Tiles a texture across a rectangle.
fn tile_texture(
    canvas: &mut Canvas<Window>,
    texture: &Texture<'_>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(), GameplayError> {
    let query = texture.query();
    let tile_width = query.width.max(1);
    let tile_height = query.height.max(1);
    let mut drawn_y = 0_u32;
    while drawn_y < height {
        let copy_height = tile_height.min(height.saturating_sub(drawn_y));
        let mut drawn_x = 0_u32;
        while drawn_x < width {
            let copy_width = tile_width.min(width.saturating_sub(drawn_x));
            canvas
                .copy(
                    texture,
                    Rect::new(0, 0, copy_width, copy_height),
                    Rect::new(
                        x.saturating_add(i32::try_from(drawn_x).map_or(0_i32, |value| value)),
                        y.saturating_add(i32::try_from(drawn_y).map_or(0_i32, |value| value)),
                        copy_width,
                        copy_height,
                    ),
                )
                .map_err(runtime_error)?;
            drawn_x = drawn_x.saturating_add(copy_width);
        }
        drawn_y = drawn_y.saturating_add(copy_height);
    }
    Ok(())
}

/// Draws a left/stretch/right chrome strip.
fn draw_three_part_horizontal(
    canvas: &mut Canvas<Window>,
    left: &Texture<'_>,
    center: &Texture<'_>,
    right: &Texture<'_>,
    dest: Rect,
) -> Result<(), GameplayError> {
    let left_width = left.query().width.min(dest.width());
    let right_width = right
        .query()
        .width
        .min(dest.width().saturating_sub(left_width));
    let center_width = dest
        .width()
        .saturating_sub(left_width)
        .saturating_sub(right_width);
    canvas
        .copy(
            left,
            None,
            Rect::new(dest.x(), dest.y(), left_width, dest.height()),
        )
        .map_err(runtime_error)?;
    if center_width > 0 {
        canvas
            .copy(
                center,
                None,
                Rect::new(
                    dest.x()
                        .saturating_add(i32::try_from(left_width).map_or(0_i32, |value| value)),
                    dest.y(),
                    center_width,
                    dest.height(),
                ),
            )
            .map_err(runtime_error)?;
    }
    if right_width > 0 {
        canvas
            .copy(
                right,
                None,
                Rect::new(
                    dest.x().saturating_add(
                        i32::try_from(left_width.saturating_add(center_width))
                            .map_or(0_i32, |value| value),
                    ),
                    dest.y(),
                    right_width,
                    dest.height(),
                ),
            )
            .map_err(runtime_error)?;
    }
    Ok(())
}
