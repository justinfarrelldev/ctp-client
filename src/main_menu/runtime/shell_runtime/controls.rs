//! Drawing helpers for individual CTP2 shell controls.

use sdl2::pixels::Color;
use sdl2::rect::Rect as SdlRect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};

use crate::main_menu::shell::text_field::ShellTextFieldStyle;
use crate::main_menu::shell::{ShellControl, ShellImageSet};
use crate::main_menu::{MainMenuError, MainMenuModel, MenuRect};

use super::super::text::draw_left_centered_text;
use super::{create_texture_for_ldl_asset, draw_centered_text, runtime_error, to_sdl_rect};

/// Draws a disabled button fallback.
pub(super) fn draw_disabled_button(
    canvas: &mut Canvas<Window>,
    font: Option<&sdl2::ttf::Font<'_, '_>>,
    rect: &MenuRect,
    label: &str,
) -> Result<(), MainMenuError> {
    canvas.set_draw_color(Color::RGB(82, 78, 70));
    canvas.fill_rect(to_sdl_rect(rect)).map_err(runtime_error)?;
    canvas.set_draw_color(Color::RGB(35, 32, 28));
    canvas.draw_rect(to_sdl_rect(rect)).map_err(runtime_error)?;
    draw_centered_text(canvas, font, rect, label)
}

/// Draws the original CTP2 text-entry bevel without covering the patterned background.
pub(super) fn draw_text_field(
    canvas: &mut Canvas<Window>,
    rect: &MenuRect,
    style: ShellTextFieldStyle,
) -> Result<(), MainMenuError> {
    let bevel_rect = expanded_text_field_rect(rect);
    draw_inset_bevel(canvas, &bevel_rect, style.bevel_width())
}

/// Draws editable text inside an original CTP2 text-entry field.
pub(super) fn draw_text_field_text(
    canvas: &mut Canvas<Window>,
    font: Option<&sdl2::ttf::Font<'_, '_>>,
    rect: &MenuRect,
    style: ShellTextFieldStyle,
    text: &str,
) -> Result<(), MainMenuError> {
    draw_left_centered_text(canvas, font, rect, text, style.text_inset_x())
}

/// Expands an LDL field rectangle to the outer bevel drawn by `ctp2_TextField::DrawThis`.
const fn expanded_text_field_rect(rect: &MenuRect) -> MenuRect {
    MenuRect::new(
        rect.x.saturating_sub(2_i32),
        rect.y.saturating_sub(2_i32),
        rect.width.saturating_add(4_u32),
        rect.height.saturating_add(4_u32),
    )
}

/// Draws an inset CTP2-style bevel around a text field.
fn draw_inset_bevel(
    canvas: &mut Canvas<Window>,
    rect: &MenuRect,
    bevel_width: u32,
) -> Result<(), MainMenuError> {
    let levels = bevel_width.max(1_u32);
    for level in 0_u32..levels {
        let inset = i32::try_from(level).map_or(0_i32, |value| value);
        let left = rect.x.saturating_add(inset);
        let top = rect.y.saturating_add(inset);
        let shrink = level.saturating_mul(2_u32);
        let width = rect.width.saturating_sub(shrink);
        let height = rect.height.saturating_sub(shrink);
        if width == 0 || height == 0 {
            continue;
        }
        canvas.set_draw_color(Color::RGB(42, 34, 24));
        canvas
            .fill_rect(SdlRect::new(left, top, width, 1_u32))
            .map_err(runtime_error)?;
        canvas
            .fill_rect(SdlRect::new(left, top, 1_u32, height))
            .map_err(runtime_error)?;
        let bottom = top.saturating_add(
            i32::try_from(height.saturating_sub(1_u32)).map_or(0_i32, |value| value),
        );
        let right = left.saturating_add(
            i32::try_from(width.saturating_sub(1_u32)).map_or(0_i32, |value| value),
        );
        canvas.set_draw_color(Color::RGB(218, 202, 158));
        canvas
            .fill_rect(SdlRect::new(left, bottom, width, 1_u32))
            .map_err(runtime_error)?;
        canvas
            .fill_rect(SdlRect::new(right, top, 1_u32, height))
            .map_err(runtime_error)?;
    }
    Ok(())
}

/// Draws a functional slider approximation with original bounds.
#[expect(
    clippy::too_many_arguments,
    reason = "Slider rendering combines geometry, state, original art, and SDL resources."
)]
pub(super) fn draw_slider(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    model: &MainMenuModel,
    _font: Option<&sdl2::ttf::Font<'_, '_>>,
    rect: &MenuRect,
    _label: &str,
    min: i32,
    max: i32,
    value: i32,
) -> Result<(), MainMenuError> {
    let half_height = rect.height.checked_div(2_u32).map_or(0_u32, |half| half);
    let track_y = rect
        .y
        .saturating_add(i32::try_from(half_height).map_or(0_i32, |half| half));
    canvas.set_draw_color(Color::RGB(60, 50, 38));
    canvas
        .fill_rect(SdlRect::new(rect.x, track_y, rect.width, 3))
        .map_err(runtime_error)?;
    let range = max.saturating_sub(min).max(1_i32);
    let offset = value
        .saturating_sub(min)
        .saturating_mul(i32::try_from(rect.width).map_or(0_i32, |width| width))
        .checked_div(range)
        .map_or(0_i32, |computed| computed);
    let thumb_rect = MenuRect::new(rect.x.saturating_add(offset), rect.y, 9, rect.height);
    if let Ok(texture) = create_texture_for_ldl_asset(texture_creator, model, "upba2513.tga") {
        canvas
            .copy(&texture, None, to_sdl_rect(&thumb_rect))
            .map_err(runtime_error)?;
    } else {
        canvas.set_draw_color(Color::RGB(230, 215, 175));
        canvas
            .fill_rect(to_sdl_rect(&thumb_rect))
            .map_err(runtime_error)?;
    }
    Ok(())
}

/// Draws a switch with its label.
#[expect(
    clippy::too_many_arguments,
    reason = "Switch rendering combines state, original art, and SDL resources."
)]
pub(super) fn draw_switch(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    model: &MainMenuModel,
    font: Option<&sdl2::ttf::Font<'_, '_>>,
    rect: &MenuRect,
    label: &str,
    enabled: bool,
    images: Option<ShellImageSet>,
) -> Result<(), MainMenuError> {
    if let Some(image_set) = images {
        draw_shell_image(canvas, texture_creator, model, rect, image_set, enabled)?;
        return draw_centered_text(canvas, font, rect, label);
    }

    canvas.set_draw_color(if enabled {
        Color::RGB(190, 180, 130)
    } else {
        Color::RGB(95, 88, 70)
    });
    canvas.fill_rect(to_sdl_rect(rect)).map_err(runtime_error)?;
    canvas.set_draw_color(Color::RGB(45, 38, 28));
    canvas.draw_rect(to_sdl_rect(rect)).map_err(runtime_error)?;
    draw_centered_text(canvas, font, rect, label)
}

/// Draws a patterned dropdown/list item.
#[expect(
    clippy::too_many_arguments,
    reason = "List rendering combines optional original art and SDL resources."
)]
pub(super) fn draw_list_item(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    model: &MainMenuModel,
    font: Option<&sdl2::ttf::Font<'_, '_>>,
    rect: &MenuRect,
    control: &ShellControl,
    images: Option<ShellImageSet>,
    selected: bool,
) -> Result<(), MainMenuError> {
    if let Some(image_set) = images {
        draw_shell_image(canvas, texture_creator, model, rect, image_set, selected)?;
        return draw_centered_text(canvas, font, rect, control.label());
    }

    if selected {
        canvas.set_draw_color(Color::RGB(76, 92, 118));
        canvas.fill_rect(to_sdl_rect(rect)).map_err(runtime_error)?;
    }
    draw_centered_text(canvas, font, rect, control.label())
}

/// Draws one original single-image shell control.
fn draw_shell_image(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    model: &MainMenuModel,
    rect: &MenuRect,
    image_set: ShellImageSet,
    selected: bool,
) -> Result<(), MainMenuError> {
    let image_name = image_set.image_for_state(selected, true);
    if let Ok(texture) = create_texture_for_ldl_asset(texture_creator, model, image_name) {
        canvas
            .copy(&texture, None, to_sdl_rect(rect))
            .map_err(runtime_error)
    } else {
        canvas.set_draw_color(if selected {
            Color::RGB(190, 180, 130)
        } else {
            Color::RGB(95, 88, 70)
        });
        canvas.fill_rect(to_sdl_rect(rect)).map_err(runtime_error)
    }
}
