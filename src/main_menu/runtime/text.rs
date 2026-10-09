//! High-resolution text rendering for the proportionally scaled SDL menu.

use std::path::PathBuf;

use sdl2::pixels::Color;
use sdl2::rect::Rect as SdlRect;
use sdl2::render::{Canvas, Texture};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::Window;

use crate::main_menu::{MainMenuError, MenuRect};

use super::viewport::menu_viewport_for_output_size;
use super::{BUTTON_LABEL_TEXT_COLOR, VERSION_TEXT_COLOR, runtime_error};

/// Cached TTF font for the current physical menu scale.
pub(super) struct MenuFontCache<'ttf> {
    /// `SDL_ttf` context used to load fonts when available.
    context: Option<&'ttf Sdl2TtfContext>,
    /// Chosen system font path.
    path: Option<PathBuf>,
    /// Point size currently stored in [`Self::font`].
    point_size: Option<u16>,
    /// Font loaded for the current viewport scale.
    font: Option<Font<'ttf, 'static>>,
}

impl<'ttf> MenuFontCache<'ttf> {
    /// Creates an empty scaled-font cache.
    pub(super) const fn new(context: Option<&'ttf Sdl2TtfContext>, path: Option<PathBuf>) -> Self {
        Self {
            context,
            path,
            point_size: None,
            font: None,
        }
    }

    /// Returns a font loaded at `point_size`, reloading only when the scale changes.
    pub(super) fn font_for_point_size(&mut self, point_size: u16) -> Option<&Font<'ttf, 'static>> {
        let requested_point_size = point_size.max(1_u16);
        if self.point_size != Some(requested_point_size) {
            let loaded = self.context.and_then(|context| {
                self.path
                    .as_ref()
                    .and_then(|path| context.load_font(path, requested_point_size).ok())
            });
            self.font = loaded;
            self.point_size = Some(requested_point_size);
        }
        self.font.as_ref()
    }
}

/// Draws centered text inside a menu rectangle at physical output resolution.
pub(super) fn draw_centered_text(
    canvas: &mut Canvas<Window>,
    font: Option<&Font<'_, '_>>,
    rect: &MenuRect,
    text: &str,
) -> Result<(), MainMenuError> {
    let Some(font) = font else {
        return Ok(());
    };

    let surface = font
        .render(text)
        .blended(rgb_color(BUTTON_LABEL_TEXT_COLOR))
        .map_err(|error| runtime_error(error.to_string()))?;
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|error| runtime_error(error.to_string()))?;
    let physical_rect = physical_rect_for_menu_rect(canvas, rect)?;
    let rect_width = i32::try_from(physical_rect.width).map_or(0_i32, |value| value);
    let rect_height = i32::try_from(physical_rect.height).map_or(0_i32, |value| value);
    let text_width = i32::try_from(surface.width()).map_or(0_i32, |value| value);
    let text_height = i32::try_from(surface.height()).map_or(0_i32, |value| value);
    let offset_x = rect_width
        .saturating_sub(text_width)
        .checked_div(2_i32)
        .map_or(0_i32, |value| value);
    let offset_y = rect_height
        .saturating_sub(text_height)
        .checked_div(2_i32)
        .map_or(0_i32, |value| value);
    let destination = SdlRect::new(
        physical_rect.x.saturating_add(offset_x),
        physical_rect.y.saturating_add(offset_y),
        surface.width(),
        surface.height(),
    );

    copy_texture_at_physical_resolution(canvas, &texture, destination)
}

/// Draws text starting at the menu rectangle origin at physical output resolution.
pub(super) fn draw_text_at(
    canvas: &mut Canvas<Window>,
    font: Option<&Font<'_, '_>>,
    rect: &MenuRect,
    text: &str,
) -> Result<(), MainMenuError> {
    let Some(font) = font else {
        return Ok(());
    };

    let surface = font
        .render(text)
        .blended(rgb_color(VERSION_TEXT_COLOR))
        .map_err(|error| runtime_error(error.to_string()))?;
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|error| runtime_error(error.to_string()))?;
    let physical_rect = physical_rect_for_menu_rect(canvas, rect)?;
    let destination = SdlRect::new(
        physical_rect.x,
        physical_rect.y,
        surface.width(),
        surface.height(),
    );

    copy_texture_at_physical_resolution(canvas, &texture, destination)
}

/// Copies a text texture with renderer scaling temporarily disabled.
fn copy_texture_at_physical_resolution(
    canvas: &mut Canvas<Window>,
    texture: &Texture<'_>,
    destination: SdlRect,
) -> Result<(), MainMenuError> {
    canvas.set_viewport(None);
    canvas.set_scale(1.0_f32, 1.0_f32).map_err(runtime_error)?;
    canvas
        .copy(texture, None, destination)
        .map_err(runtime_error)
}

/// Converts a menu-space text rectangle into physical output pixels.
fn physical_rect_for_menu_rect(
    canvas: &Canvas<Window>,
    rect: &MenuRect,
) -> Result<MenuRect, MainMenuError> {
    let output_size = canvas.output_size().map_err(runtime_error)?;
    let viewport = menu_viewport_for_output_size(output_size.0, output_size.1);
    Ok(MenuRect::new(
        viewport.x.saturating_add(scale_signed_axis(
            rect.x,
            viewport.width,
            super::super::MAIN_MENU_WIDTH,
        )),
        viewport.y.saturating_add(scale_signed_axis(
            rect.y,
            viewport.height,
            super::super::MAIN_MENU_HEIGHT,
        )),
        scale_unsigned_axis(rect.width, viewport.width, super::super::MAIN_MENU_WIDTH),
        scale_unsigned_axis(rect.height, viewport.height, super::super::MAIN_MENU_HEIGHT),
    ))
}

/// Scales a signed menu coordinate into a physical viewport coordinate.
fn scale_signed_axis(value: i32, viewport_extent: u32, menu_extent: u32) -> i32 {
    if menu_extent == 0 {
        return 0_i32;
    }
    let scaled = i64::from(value)
        .saturating_mul(i64::from(viewport_extent))
        .checked_div(i64::from(menu_extent))
        .map_or(0_i64, |coordinate| coordinate);
    i32::try_from(scaled).unwrap_or_else(|_| {
        if scaled.is_negative() {
            i32::MIN
        } else {
            i32::MAX
        }
    })
}

/// Scales an unsigned menu extent into physical viewport pixels.
fn scale_unsigned_axis(value: u32, viewport_extent: u32, menu_extent: u32) -> u32 {
    if menu_extent == 0 {
        return 0_u32;
    }
    let scaled = u64::from(value)
        .saturating_mul(u64::from(viewport_extent))
        .checked_div(u64::from(menu_extent))
        .map_or(0_u64, |extent| extent);
    u32::try_from(scaled).map_or(u32::MAX, |extent| extent)
}

/// Converts an RGB tuple into an SDL color.
const fn rgb_color((red, green, blue): (u8, u8, u8)) -> Color {
    Color::RGB(red, green, blue)
}
