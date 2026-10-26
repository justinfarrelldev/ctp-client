//! High-resolution text rendering for the proportionally scaled SDL menu.

use super::typography::ShellTypography;
use crate::gameplay::ui_model::management::typography::SourceFont;
use alloc::collections::BTreeMap;
use std::path::{Path, PathBuf};

use sdl2::pixels::Color;
use sdl2::rect::Rect as SdlRect;
use sdl2::render::{Canvas, Texture};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::Window;

use crate::main_menu::{MainMenuError, MenuRect};

use super::viewport::menu_viewport_for_output_size;
use super::{BUTTON_LABEL_TEXT_COLOR, VERSION_TEXT_COLOR, runtime_error};

/// Cached source faces for the existing physical menu scale.
pub(super) struct MenuFontCache<'ttf> {
    /// Optional SDL context, borrowed for the session lifetime.
    context: Option<&'ttf Sdl2TtfContext>,
    /// Shared installation/platform discovery results, never copied font data.
    paths: BTreeMap<SourceFont, (PathBuf, bool)>,
    /// Cached original screen/control face identities.
    typography: ShellTypography,
    /// Original standard identity reused without allocating a name on every frame.
    standard_key: SourceFont,
    /// Current physical size, preserving the existing viewport scaling policy.
    point_size: Option<u16>,
    /// Fonts loaded only on first frame or physical-size change.
    faces: BTreeMap<SourceFont, Font<'ttf, 'static>>,
}

impl<'ttf> MenuFontCache<'ttf> {
    /// Resolves source names once, sharing gameplay's read-only discovery policy.
    pub(super) fn new(context: Option<&'ttf Sdl2TtfContext>, root: &Path) -> Self {
        let typography = ShellTypography::load(root);
        let paths = typography
            .keys()
            .into_iter()
            .filter_map(|key| {
                let found = crate::fonts::find(Some(root), &key.name, key.bold, key.italic);
                if let Some(resolved) = found.as_ref() {
                    if !resolved.1 {
                        log::warn!(
                            "Shell font {} unavailable; using {} (nonexact)",
                            key.name,
                            resolved.0.display()
                        );
                    }
                }
                found.map(|resolved| (key, resolved))
            })
            .collect();
        Self {
            context,
            paths,
            typography,
            standard_key: SourceFont::default(),
            point_size: None,
            faces: BTreeMap::new(),
        }
    }

    /// Loads all source faces at the current physical size, never per control/frame.
    pub(super) fn font_for_point_size(&mut self, point_size: u16) -> MenuFonts<'_, 'ttf, 'static> {
        let requested = point_size.max(1_u16);
        if self.point_size != Some(requested) {
            self.faces.clear();
            if let Some(context) = self.context {
                for (key, resolved) in &self.paths {
                    if let Ok(mut font) = context.load_font(&resolved.0, requested) {
                        // Open substitutes already select the combined style-specific face.
                        let mut style = sdl2::ttf::FontStyle::NORMAL;
                        if resolved.1 && key.bold {
                            style |= sdl2::ttf::FontStyle::BOLD;
                        }
                        if resolved.1 && key.italic {
                            style |= sdl2::ttf::FontStyle::ITALIC;
                        }
                        font.set_style(style);
                        self.faces.insert(key.clone(), font);
                    }
                }
            }
            self.point_size = Some(requested);
        }
        MenuFonts {
            standard: self.faces.get(&self.standard_key),
            typography: Some(&self.typography),
            faces: Some(&self.faces),
        }
    }
}

/// Borrowed source font set; explicit font-free preview tests remain supported.
#[derive(Clone, Copy)]
pub(super) struct MenuFonts<'draw, 'ttf, 'rwops> {
    /// Original Times Bold standard, or the test's explicit font.
    standard: Option<&'draw Font<'ttf, 'rwops>>,
    /// Resolved source inheritance when a production session owns the cache.
    typography: Option<&'draw ShellTypography>,
    /// Production faces, kept alive by the owning cache through painting.
    faces: Option<&'draw BTreeMap<SourceFont, Font<'ttf, 'static>>>,
}

impl<'draw, 'ttf, 'rwops> From<Option<&'draw Font<'ttf, 'rwops>>>
    for MenuFonts<'draw, 'ttf, 'rwops>
{
    fn from(standard: Option<&'draw Font<'ttf, 'rwops>>) -> Self {
        Self {
            standard,
            typography: None,
            faces: None,
        }
    }
}

impl<'draw, 'ttf, 'rwops> MenuFonts<'draw, 'ttf, 'rwops> {
    /// Landing labels retain the original standard Times Bold face.
    pub(super) const fn standard(self) -> Option<&'draw Font<'ttf, 'rwops>> {
        self.standard
    }

    /// Screen-local LDL identity, not label text or a guessed all-labels font.
    pub(super) fn control(self, screen: &str, control: &str) -> Option<&'draw Font<'ttf, 'rwops>> {
        self.typography
            .and_then(|source| source.face(screen, control))
            .and_then(|key| self.faces.and_then(|faces| faces.get(key)))
            .or(self.standard)
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

/// Style options for left-aligned shell text drawing.
#[derive(Clone, Copy)]
pub(super) struct LeftTextStyle {
    /// Horizontal physical inset in menu pixels before scaling.
    inset_x: i32,
    /// RGB text color.
    color: (u8, u8, u8),
    /// Text scale as a percentage of the loaded shell font.
    scale_percent: u32,
    /// Whether to draw a focused edit-control caret.
    focused: bool,
}

impl LeftTextStyle {
    /// Creates a left-aligned text style.
    pub(super) const fn new(
        inset_x: i32,
        color: (u8, u8, u8),
        scale_percent: u32,
        focused: bool,
    ) -> Self {
        Self {
            inset_x,
            color,
            scale_percent,
            focused,
        }
    }
}

/// Draws left-aligned, vertically centered shell text inside a menu rectangle.
pub(super) fn draw_left_centered_text(
    canvas: &mut Canvas<Window>,
    font: Option<&Font<'_, '_>>,
    rect: &MenuRect,
    text: &str,
    style: LeftTextStyle,
) -> Result<(), MainMenuError> {
    let Some(font) = font else {
        return Ok(());
    };

    let physical_rect = physical_rect_for_menu_rect(canvas, rect)?;
    let output_size = canvas.output_size().map_err(runtime_error)?;
    let viewport = menu_viewport_for_output_size(output_size.0, output_size.1);
    let physical_inset_x =
        scale_signed_axis(style.inset_x, viewport.width, super::super::MAIN_MENU_WIDTH);
    if text.is_empty() {
        return draw_optional_caret(
            canvas,
            style.focused,
            &physical_rect,
            physical_inset_x,
            0_u32,
            style.color,
        );
    }

    let surface = font
        .render(text)
        .blended(rgb_color(style.color))
        .map_err(|error| runtime_error(error.to_string()))?;
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|error| runtime_error(error.to_string()))?;
    let scaled_width = scale_percent_extent(surface.width(), style.scale_percent);
    let scaled_height = scale_percent_extent(surface.height(), style.scale_percent);
    let rect_height = i32::try_from(physical_rect.height).map_or(0_i32, |value| value);
    let text_height = i32::try_from(scaled_height).map_or(0_i32, |value| value);
    let offset_y = rect_height
        .saturating_sub(text_height)
        .checked_div(2_i32)
        .map_or(0_i32, |value| value);
    let destination = SdlRect::new(
        physical_rect.x.saturating_add(physical_inset_x),
        physical_rect.y.saturating_add(offset_y),
        scaled_width,
        scaled_height,
    );

    copy_texture_at_physical_resolution(canvas, &texture, destination)?;
    draw_optional_caret(
        canvas,
        style.focused,
        &physical_rect,
        physical_inset_x,
        scaled_width,
        style.color,
    )
}

/// Draws a native edit-control caret when a text field is focused.
fn draw_optional_caret(
    canvas: &mut Canvas<Window>,
    focused: bool,
    physical_rect: &MenuRect,
    inset_x: i32,
    text_width: u32,
    color: (u8, u8, u8),
) -> Result<(), MainMenuError> {
    if !focused {
        return Ok(());
    }
    let caret_x = physical_rect
        .x
        .saturating_add(inset_x)
        .saturating_add(i32::try_from(text_width).map_or(i32::MAX, |value| value))
        .saturating_add(1_i32);
    let caret_height = physical_rect.height.saturating_sub(4_u32).max(1_u32);
    let caret_y = physical_rect.y.saturating_add(2_i32);
    canvas.set_draw_color(rgb_color(color));
    canvas
        .fill_rect(SdlRect::new(caret_x, caret_y, 2_u32, caret_height))
        .map_err(runtime_error)
}

/// Draws shell text starting at the menu rectangle origin at physical output resolution.
pub(super) fn draw_text_at(
    canvas: &mut Canvas<Window>,
    font: Option<&Font<'_, '_>>,
    rect: &MenuRect,
    text: &str,
) -> Result<(), MainMenuError> {
    draw_text_at_with_color(canvas, font, rect, text, BUTTON_LABEL_TEXT_COLOR)
}

/// Draws version text starting at the menu rectangle origin at physical output resolution.
pub(super) fn draw_version_text_at(
    canvas: &mut Canvas<Window>,
    font: Option<&Font<'_, '_>>,
    rect: &MenuRect,
    text: &str,
) -> Result<(), MainMenuError> {
    draw_text_at_with_color(canvas, font, rect, text, VERSION_TEXT_COLOR)
}

/// Draws text starting at the menu rectangle origin using the supplied RGB color.
fn draw_text_at_with_color(
    canvas: &mut Canvas<Window>,
    font: Option<&Font<'_, '_>>,
    rect: &MenuRect,
    text: &str,
    color: (u8, u8, u8),
) -> Result<(), MainMenuError> {
    let Some(font) = font else {
        return Ok(());
    };

    let surface = font
        .render(text)
        .blended(rgb_color(color))
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

/// Scales a physical text extent by a percentage.
fn scale_percent_extent(value: u32, percent: u32) -> u32 {
    let scaled = u64::from(value)
        .saturating_mul(u64::from(percent))
        .checked_div(100_u64)
        .map_or(1_u64, |extent| extent.max(1_u64));
    u32::try_from(scaled).map_or(u32::MAX, |extent| extent)
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

/// Returns the left/top shell label text color for tests.
#[cfg(test)]
pub(super) const fn shell_left_top_text_color_for_test() -> (u8, u8, u8) {
    BUTTON_LABEL_TEXT_COLOR
}

/// Converts an RGB tuple into an SDL color.
const fn rgb_color((red, green, blue): (u8, u8, u8)) -> Color {
    Color::RGB(red, green, blue)
}
