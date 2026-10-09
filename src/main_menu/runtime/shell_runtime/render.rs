//! Rendering helpers for CTP2 shell runtime screens.

use sdl2::pixels::Color;
use sdl2::rect::Rect as SdlRect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

use crate::main_menu::shell::{
    ShellControl, ShellControlKind, ShellLabelLayout, ShellScreen, ShellTextAlignment,
};
use crate::main_menu::{MainMenuError, MainMenuModel, MenuAssetReference, MenuRect};

use super::super::frame::{
    draw_fallback_background, draw_fallback_button, textures_for_style, to_sdl_rect,
};
use super::super::text::{draw_centered_text, draw_text_at};
use super::super::{
    MenuTextures, button_art, create_texture_from_rgba, load_rim_rgba, runtime_error,
};
use super::controls::{
    draw_disabled_button, draw_list_item, draw_slider, draw_switch, draw_text_field,
    draw_text_field_text,
};
use super::listbox::draw_listbox_chrome;
use super::screen_rect;

/// Draws one modeled CTP2 shell screen.
#[expect(
    clippy::too_many_arguments,
    reason = "The renderer keeps SDL objects borrowed separately to avoid hidden ownership."
)]
pub(super) fn draw_shell_screen(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    textures: &MenuTextures<'_>,
    model: &MainMenuModel,
    screen: &ShellScreen,
    mouse_x: i32,
    mouse_y: i32,
    font: Option<&sdl2::ttf::Font<'_, '_>>,
) -> Result<(), MainMenuError> {
    draw_shell_background(canvas, texture_creator, model, screen)?;
    draw_shell_pattern(canvas, texture_creator, model, screen)?;

    for control in screen
        .controls()
        .iter()
        .filter(|control| !draws_in_text_overlay_pass(control))
    {
        draw_shell_control(
            canvas,
            texture_creator,
            textures,
            model,
            screen,
            control,
            mouse_x,
            mouse_y,
            font,
        )?;
    }
    for control in screen
        .controls()
        .iter()
        .filter(|control| draws_in_text_overlay_pass(control))
    {
        draw_shell_control(
            canvas,
            texture_creator,
            textures,
            model,
            screen,
            control,
            mouse_x,
            mouse_y,
            font,
        )?;
    }

    Ok(())
}

/// Draws only text-bearing shell controls over the physically scaled frame.
pub(super) fn draw_shell_text_overlay(
    canvas: &mut Canvas<Window>,
    textures: &MenuTextures<'_>,
    screen: &ShellScreen,
    mouse_x: i32,
    mouse_y: i32,
    font: Option<&sdl2::ttf::Font<'_, '_>>,
) -> Result<(), MainMenuError> {
    for control in screen.controls() {
        draw_shell_control_text(canvas, textures, screen, control, mouse_x, mouse_y, font)?;
    }
    Ok(())
}

/// Draws the text associated with one shell control without redrawing its art.
fn draw_shell_control_text(
    canvas: &mut Canvas<Window>,
    textures: &MenuTextures<'_>,
    screen: &ShellScreen,
    control: &ShellControl,
    mouse_x: i32,
    mouse_y: i32,
    font: Option<&sdl2::ttf::Font<'_, '_>>,
) -> Result<(), MainMenuError> {
    let rect = screen_rect(screen, control.rect());
    match control.kind() {
        ShellControlKind::Button(style) => draw_button_text(
            canvas, textures, control, rect, style, mouse_x, mouse_y, font,
        ),
        ShellControlKind::Label { layout } => {
            draw_label(canvas, font, &rect, control.label(), layout)
        }
        ShellControlKind::TextField { style, focused } => {
            draw_text_field_text(canvas, font, &rect, style, focused, control.label())
        }
        ShellControlKind::Switch { .. } | ShellControlKind::ListItem { .. } => {
            draw_centered_text(canvas, font, &rect, control.label())
        }
        ShellControlKind::Slider { .. } | ShellControlKind::ListBoxChrome { .. } => Ok(()),
    }
}

/// Draws sharp text for a shell button using original art text bounds.
#[expect(
    clippy::too_many_arguments,
    reason = "Button text bounds depend on style assets, state, and pointer hover."
)]
fn draw_button_text(
    canvas: &mut Canvas<Window>,
    textures: &MenuTextures<'_>,
    control: &ShellControl,
    rect: MenuRect,
    style: crate::main_menu::ButtonStyleId,
    mouse_x: i32,
    mouse_y: i32,
    font: Option<&sdl2::ttf::Font<'_, '_>>,
) -> Result<(), MainMenuError> {
    let interaction_rect = style.visual_style().rendered_rect(rect);
    let hovered = control.enabled() && interaction_rect.contains_point(mouse_x, mouse_y);
    let text_rect =
        textures_for_style(textures, style).map_or(interaction_rect, |button_textures| {
            if control.enabled() {
                button_art::styled_button_text_rect(&rect, button_textures, hovered)
            } else {
                button_art::disabled_button_text_rect(&rect, button_textures)
            }
        });
    draw_centered_text(canvas, font, &text_rect, control.label())
}

/// Returns modeled control drawing order for tests.
#[cfg(test)]
pub(super) fn control_draw_order_for_test(screen: &ShellScreen) -> alloc::vec::Vec<&'static str> {
    let mut order = alloc::vec::Vec::new();
    for control in screen
        .controls()
        .iter()
        .filter(|control| !draws_in_text_overlay_pass(control))
    {
        order.push(control.ldl_name());
    }
    for control in screen
        .controls()
        .iter()
        .filter(|control| draws_in_text_overlay_pass(control))
    {
        order.push(control.ldl_name());
    }
    order
}

/// Returns whether a control belongs in the final text-overlay pass.
const fn draws_in_text_overlay_pass(control: &ShellControl) -> bool {
    matches!(control.kind(), ShellControlKind::Label { .. })
}

/// Draws either the screen background or the single-player hub underlay for popups.
fn draw_shell_background(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    model: &MainMenuModel,
    screen: &ShellScreen,
) -> Result<(), MainMenuError> {
    let background = screen.background().unwrap_or(MenuAssetReference {
        archive_relative_path: "ctp2_data/default/graphics/pictures/pic565.zfs",
        entry_name: "upsg002.rim",
    });
    match create_texture_for_asset(texture_creator, model, &background) {
        Ok(texture) => canvas
            .copy(
                &texture,
                None,
                SdlRect::new(
                    0,
                    0,
                    super::super::super::MAIN_MENU_WIDTH,
                    super::super::super::MAIN_MENU_HEIGHT,
                ),
            )
            .map_err(runtime_error),
        Err(_) => draw_fallback_background(canvas),
    }
}

/// Draws the screen's tiled popup pattern when one exists.
fn draw_shell_pattern(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    model: &MainMenuModel,
    screen: &ShellScreen,
) -> Result<(), MainMenuError> {
    let Some(pattern) = screen.pattern() else {
        return Ok(());
    };
    let destination = screen.rect();
    if let Ok(texture) = create_texture_for_asset(texture_creator, model, &pattern) {
        tile_texture(canvas, &texture, destination)
    } else {
        canvas.set_draw_color(Color::RGB(160, 145, 108));
        canvas
            .fill_rect(to_sdl_rect(&destination))
            .map_err(runtime_error)
    }
}

/// Draws one shell control in screen coordinates.
#[expect(
    clippy::too_many_arguments,
    reason = "Control rendering needs SDL texture resources and current mouse state."
)]
fn draw_shell_control(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    textures: &MenuTextures<'_>,
    model: &MainMenuModel,
    screen: &ShellScreen,
    control: &ShellControl,
    mouse_x: i32,
    mouse_y: i32,
    font: Option<&sdl2::ttf::Font<'_, '_>>,
) -> Result<(), MainMenuError> {
    let rect = screen_rect(screen, control.rect());
    match control.kind() {
        ShellControlKind::Button(style) => {
            let enabled = control.enabled();
            let interaction_rect = style.visual_style().rendered_rect(rect);
            let hovered = enabled && interaction_rect.contains_point(mouse_x, mouse_y);
            if enabled {
                if let Some(button_textures) = textures_for_style(textures, style) {
                    button_art::draw_styled_button(canvas, &rect, button_textures, hovered)?;
                    let text_rect =
                        button_art::styled_button_text_rect(&rect, button_textures, hovered);
                    draw_centered_text(canvas, font, &text_rect, control.label())
                } else {
                    draw_fallback_button(canvas, &interaction_rect, style, hovered)?;
                    draw_centered_text(canvas, font, &interaction_rect, control.label())
                }
            } else if let Some(button_textures) = textures_for_style(textures, style) {
                button_art::draw_disabled_styled_button(canvas, &rect, button_textures)?;
                let text_rect = button_art::disabled_button_text_rect(&rect, button_textures);
                draw_centered_text(canvas, font, &text_rect, control.label())
            } else {
                draw_disabled_button(canvas, font, &interaction_rect, control.label())
            }
        }
        ShellControlKind::Label { layout } => {
            draw_label(canvas, font, &rect, control.label(), layout)
        }
        ShellControlKind::TextField { style, .. } => draw_text_field(canvas, &rect, style),
        ShellControlKind::ListBoxChrome {
            scroll_offset,
            item_count,
            visible_rows,
        } => draw_listbox_chrome(
            canvas,
            texture_creator,
            model,
            &rect,
            scroll_offset,
            item_count,
            visible_rows,
        ),
        ShellControlKind::Slider { min, max, value } => draw_slider(
            canvas,
            texture_creator,
            model,
            font,
            &rect,
            control.label(),
            min,
            max,
            value,
        ),
        ShellControlKind::Switch { enabled, images } => draw_switch(
            canvas,
            texture_creator,
            model,
            font,
            &rect,
            control.label(),
            enabled,
            images,
        ),
        ShellControlKind::ListItem { images, selected } => draw_list_item(
            canvas,
            texture_creator,
            model,
            font,
            &rect,
            control,
            images,
            selected,
        ),
    }
}

/// Draws label text according to the modeled LDL text layout.
fn draw_label(
    canvas: &mut Canvas<Window>,
    font: Option<&sdl2::ttf::Font<'_, '_>>,
    rect: &MenuRect,
    label: &str,
    layout: ShellLabelLayout,
) -> Result<(), MainMenuError> {
    let text_rect = layout.text_rect(*rect);
    match layout.alignment() {
        ShellTextAlignment::Center => draw_centered_text(canvas, font, &text_rect, label),
        ShellTextAlignment::LeftTop => draw_text_at(canvas, font, &text_rect, label),
    }
}

/// Creates a texture for a menu asset.
fn create_texture_for_asset<'texture>(
    texture_creator: &'texture TextureCreator<WindowContext>,
    model: &MainMenuModel,
    asset: &MenuAssetReference,
) -> Result<Texture<'texture>, MainMenuError> {
    let image = load_rim_rgba(&model.installation_root, asset)?;
    create_texture_from_rgba(texture_creator, &image)
}

/// Tiles one texture inside the destination rectangle.
fn tile_texture(
    canvas: &mut Canvas<Window>,
    texture: &Texture<'_>,
    destination: MenuRect,
) -> Result<(), MainMenuError> {
    let query = texture.query();
    if query.width == 0 || query.height == 0 {
        return Ok(());
    }
    let right = destination
        .x
        .saturating_add(i32::try_from(destination.width).map_or(0_i32, |width| width));
    let bottom = destination
        .y
        .saturating_add(i32::try_from(destination.height).map_or(0_i32, |height| height));
    let mut y = destination.y;
    while y < bottom {
        let mut x = destination.x;
        let remaining_y = u32::try_from(bottom.saturating_sub(y)).map_or(0, |value| value);
        let copy_height = remaining_y.min(query.height);
        while x < right {
            let remaining_x = u32::try_from(right.saturating_sub(x)).map_or(0, |value| value);
            let copy_width = remaining_x.min(query.width);
            canvas
                .copy(
                    texture,
                    SdlRect::new(0, 0, copy_width, copy_height),
                    SdlRect::new(x, y, copy_width, copy_height),
                )
                .map_err(runtime_error)?;
            x = x.saturating_add(i32::try_from(copy_width).map_or(0_i32, |value| value));
        }
        y = y.saturating_add(i32::try_from(copy_height).map_or(0_i32, |value| value));
    }
    Ok(())
}
