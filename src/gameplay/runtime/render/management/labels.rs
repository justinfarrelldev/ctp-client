//! Source text alignment, shadow and underline, separate from image clipping.

use super::super::text::{draw_text, draw_wrapped_text};
use crate::gameplay::ui_model::management::ManagerWidget;
use crate::gameplay::{GameplayError, runtime_error};

/// Source alignment/title clipping regressions with independent expected pixels.
#[cfg(test)]
mod tests;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::Font;
use sdl2::video::Window;

/// Honors original just/vertcenter defaults, including right-aligned field labels.
pub(super) fn draw(
    canvas: &mut Canvas<Window>,
    widget: &ManagerWidget,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    if widget.label.is_empty() || widget.node.field("row-container") == Some("true") {
        return Ok(());
    }
    let bounds = super::rect(widget);
    let color = if widget.unavailable.is_some() {
        Color::RGB(126_u8, 118_u8, 108_u8)
    } else {
        super::label_color(widget)
    };
    let wrapped = widget.node.field("wordwrap").is_some_and(super::truth);
    if wrapped {
        if shadow(widget) {
            draw_wrapped_text(
                canvas,
                font,
                &widget.label,
                Rect::new(
                    bounds.x().saturating_add(1_i32),
                    bounds.y().saturating_add(1_i32),
                    bounds.width(),
                    bounds.height(),
                ),
                shadow_color(widget),
            )?;
        }
        return draw_wrapped_text(canvas, font, &widget.label, bounds, color);
    }
    let (width, height) = font
        .and_then(|font| font.size_of(&widget.label).ok())
        .unwrap_or_else(|| {
            (
                u32::try_from(widget.label.chars().count())
                    .unwrap_or(u32::MAX)
                    .saturating_mul(6_u32),
                7_u32,
            )
        });
    let just = widget.node.field("just").unwrap_or("center");
    let horizontal = i32::try_from(bounds.width())
        .unwrap_or(i32::MAX)
        .saturating_sub(i32::try_from(width).unwrap_or(i32::MAX));
    let x = match just {
        "left" => bounds.x().saturating_add(
            widget
                .node
                .integer("text-inset-left")
                .unwrap_or(0_i32)
                .max(0_i32),
        ),
        "right" => bounds.x().saturating_add(horizontal),
        _ => bounds.x().saturating_add(horizontal.div_euclid(2_i32)),
    };
    let centered = widget
        .node
        .field("vertcenter")
        .map_or(widget.node.field("just").is_none(), super::truth);
    let y = if centered {
        bounds.y().saturating_add(
            i32::try_from(bounds.height())
                .unwrap_or(i32::MAX)
                .saturating_sub(i32::try_from(height).unwrap_or(i32::MAX))
                .div_euclid(2_i32),
        )
    } else {
        bounds.y()
    };
    if shadow(widget) {
        draw_text(
            canvas,
            font,
            &widget.label,
            x.saturating_add(1_i32),
            y.saturating_add(1_i32),
            shadow_color(widget),
        )?;
    }
    draw_text(canvas, font, &widget.label, x, y, color)?;
    if (widget.node.field("richtext") == Some("true") && widget.intent.is_some())
        || widget.node.field("underlinefont").is_some_and(super::truth)
    {
        let underline = y
            .saturating_add(font.map_or(7_i32, |font| font.ascent().saturating_add(1_i32)))
            .min(bounds.bottom().saturating_sub(1_i32));
        canvas.set_draw_color(color);
        canvas
            .draw_line(
                (x, underline),
                (
                    x.saturating_add(i32::try_from(width).unwrap_or(i32::MAX))
                        .saturating_sub(1_i32),
                    underline,
                ),
            )
            .map_err(runtime_error)?;
    }
    Ok(())
}

/// Disabled source text is shaded rather than receiving an enabled title shadow.
fn shadow(widget: &ManagerWidget) -> bool {
    widget.unavailable.is_none() && widget.node.field("shadowfont").is_some_and(super::truth)
}
/// Declared source RGB shadow values, not a universal black shadow.
fn shadow_color(widget: &ManagerWidget) -> Color {
    let channel = |name| {
        widget
            .node
            .integer(name)
            .and_then(|value| u8::try_from(value).ok())
            .unwrap_or(255_u8)
    };
    Color::RGB(
        channel("fontshadowcolorred"),
        channel("fontshadowcolorgreen"),
        channel("fontshadowcolorblue"),
    )
}
