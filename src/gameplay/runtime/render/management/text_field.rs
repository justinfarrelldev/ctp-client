//! Clipped SDL replacement for the original platform-specific Windows EDIT field.

use super::super::text::draw_text;
use crate::gameplay::ui_model::management::ManagerWidget;
use crate::gameplay::{GameplayError, runtime_error};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::Font;
use sdl2::video::Window;

/// Paints selection, horizontal caret scrolling and uncommitted IME separately.
pub(super) fn draw(
    canvas: &mut Canvas<Window>,
    widget: &ManagerWidget,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    let bounds = super::rect(widget);
    canvas.set_draw_color(Color::RGB(255_u8, 255_u8, 255_u8));
    canvas.fill_rect(bounds).map_err(runtime_error)?;
    canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
    canvas.draw_rect(bounds).map_err(runtime_error)?;
    let caret = offset(widget, "edit-caret", widget.label.len());
    let caret_width = width(font, widget.label.get(..caret).unwrap_or(""));
    let excess = caret_width.saturating_sub(bounds.width().saturating_sub(6_u32));
    let x = bounds
        .x()
        .saturating_add(3_i32)
        .saturating_sub(i32::try_from(excess).unwrap_or(i32::MAX));
    let y = bounds.y().saturating_add(2_i32);
    draw_text(
        canvas,
        font,
        &widget.label,
        x,
        y,
        Color::RGB(0_u8, 0_u8, 0_u8),
    )?;
    if widget.node.field("textfocused") != Some("true") {
        return Ok(());
    }
    selection(canvas, widget, font, x, y)?;
    let caret_x = x.saturating_add(i32::try_from(caret_width).unwrap_or(i32::MAX));
    let composition = widget.node.field("composition").unwrap_or("");
    let composed_caret = if composition.is_empty() {
        caret_x
    } else {
        draw_composition(canvas, widget, font, caret_x, y, composition)?
    };
    canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
    canvas
        .draw_line(
            (composed_caret, bounds.y().saturating_add(3_i32)),
            (composed_caret, bounds.bottom().saturating_sub(3_i32)),
        )
        .map_err(runtime_error)
}

/// Selection uses real glyph widths and restores even a partially intersecting clip.
fn selection(
    canvas: &mut Canvas<Window>,
    widget: &ManagerWidget,
    font: Option<&Font<'_, '_>>,
    x: i32,
    y: i32,
) -> Result<(), GameplayError> {
    let start = offset(widget, "selection-start", 0_usize);
    let end = offset(widget, "selection-end", start);
    if start >= end {
        return Ok(());
    }
    let left = width(font, widget.label.get(..start).unwrap_or(""));
    let right = width(font, widget.label.get(..end).unwrap_or(""));
    let selected = Rect::new(
        x.saturating_add(i32::try_from(left).unwrap_or(i32::MAX)),
        y,
        right.saturating_sub(left),
        widget.rect.height.saturating_sub(4_u32),
    );
    let previous = canvas.clip_rect();
    let Some(clip) = previous
        .unwrap_or_else(|| super::rect(widget))
        .intersection(selected)
    else {
        return Ok(());
    };
    canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 128_u8));
    canvas.fill_rect(selected).map_err(runtime_error)?;
    canvas.set_clip_rect(clip);
    let result = draw_text(
        canvas,
        font,
        &widget.label,
        x,
        y,
        Color::RGB(255_u8, 255_u8, 255_u8),
    );
    canvas.set_clip_rect(previous);
    result
}

/// Composition is visual-only until SDL emits a committed text-input event.
fn draw_composition(
    canvas: &mut Canvas<Window>,
    widget: &ManagerWidget,
    font: Option<&Font<'_, '_>>,
    x: i32,
    y: i32,
    text: &str,
) -> Result<i32, GameplayError> {
    let extent = width(font, text);
    let background = Rect::new(x, y, extent, widget.rect.height.saturating_sub(4_u32));
    canvas.set_draw_color(Color::RGB(255_u8, 255_u8, 255_u8));
    canvas.fill_rect(background).map_err(runtime_error)?;
    draw_text(canvas, font, text, x, y, Color::RGB(0_u8, 0_u8, 0_u8))?;
    let underline =
        y.saturating_add(font.map_or(7_i32, |font| font.ascent().saturating_add(1_i32)));
    canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
    canvas
        .draw_line(
            (x, underline),
            (
                x.saturating_add(i32::try_from(extent).unwrap_or(i32::MAX)),
                underline,
            ),
        )
        .map_err(runtime_error)?;
    let prefix: String = text
        .chars()
        .take(offset(widget, "composition-caret", 0_usize))
        .collect();
    Ok(x.saturating_add(i32::try_from(width(font, &prefix)).unwrap_or(i32::MAX)))
}

/// Only validated descriptor offsets are used; malformed metadata cannot panic.
fn offset(widget: &ManagerWidget, key: &str, fallback: usize) -> usize {
    widget
        .node
        .integer(key)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(fallback)
}
/// Production and pointer metrics both use this same native font's prefix widths.
fn width(font: Option<&Font<'_, '_>>, text: &str) -> u32 {
    font.map_or_else(
        || {
            u32::try_from(text.chars().count())
                .unwrap_or(u32::MAX)
                .saturating_mul(6_u32)
        },
        |font| font.size_of(text).map_or(0_u32, |size| size.0),
    )
}
