//! Native SDL composition of original manager controls and nine-piece frames.
//!
//! Geometry/labels/hit testing come from the same resolved LDL descriptors. Art
//! is loaded from the installation on demand; no legacy renderer or embedded
//! copyrighted assets are used. Inactive panels are never painted over each other.

#![expect(
    clippy::default_numeric_fallback,
    reason = "Original pixel offsets are signed i32; texture dimensions are explicitly checked u32"
)]

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::Font;
use sdl2::video::Window;

use super::chrome::tile_texture;
use super::context::draw_gold_frame;
use super::text::{draw_centered_text, draw_text, draw_wrapped_text};
use crate::gameplay::model::GameState;
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::ui_model::InGameUiModel;
use crate::gameplay::ui_model::management::{ManagerKind, ManagerWidget};
use crate::gameplay::{GameplayError, runtime_error};

/// Draws an open manager after the map/panel and before the software cursor.
pub(super) fn draw_management(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
    ui: &InGameUiModel,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    let Some(view) = ui.management.view.as_ref() else {
        return Ok(());
    };
    let widgets = ui.management.widgets(state);
    let popup = matches!(view.kind, ManagerKind::Graphics | ManagerKind::Sound);
    if popup {
        if let Some(root) = widgets.first() {
            draw_gold_frame(canvas, textures, rect(root))?;
        }
    }
    for widget in &widgets {
        if !(popup && widget.id.is_empty()) {
            draw_widget(canvas, textures, widget, font)?;
        }
    }
    Ok(())
}

/// Draws one shared descriptor, retaining clipping around dynamic row text.
fn draw_widget(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    widget: &ManagerWidget,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    let bounds = rect(widget);
    if bounds.width() == 0 || bounds.height() == 0 {
        return Ok(());
    }
    let previous = canvas.clip_rect();
    let clip = previous.map_or(Some(bounds), |parent| parent.intersection(bounds));
    let Some(clip) = clip else {
        return Ok(());
    };
    canvas.set_clip_rect(clip);
    let result = paint_widget(canvas, textures, widget, font);
    canvas.set_clip_rect(previous);
    result
}

/// Paints within the source control's clip; oversized native art cannot erase siblings.
fn paint_widget(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    widget: &ManagerWidget,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    let bounds = rect(widget);
    if widget
        .node
        .field("objecttype")
        .is_some_and(|kind| kind.eq_ignore_ascii_case("ctp2_Textfield"))
    {
        return draw_text_field(canvas, widget, font);
    }
    if let Some(pattern) = widget
        .node
        .field("pattern")
        .filter(|name| !name.eq_ignore_ascii_case("error.tga"))
    {
        if let Some(texture) = textures.dynamic_ui(&art_name(pattern))? {
            if let (Some(x), Some(y), Some(width), Some(height)) = (
                widget.node.integer("srcxpix"),
                widget.node.integer("srcypix"),
                widget.node.integer("srcwidthpix"),
                widget.node.integer("srcheightpix"),
            ) {
                canvas
                    .copy(
                        texture,
                        Rect::new(
                            x,
                            y,
                            u32::try_from(width).unwrap_or(0_u32),
                            u32::try_from(height).unwrap_or(0_u32),
                        ),
                        bounds,
                    )
                    .map_err(runtime_error)?;
            } else {
                tile_texture(
                    canvas,
                    texture,
                    bounds.x(),
                    bounds.y(),
                    bounds.width(),
                    bounds.height(),
                )?;
            }
        }
    }
    draw_images(canvas, textures, widget)?;
    if widget.selected && widget.id.contains('/') {
        canvas.set_draw_color(Color::RGB(255, 198, 66));
        canvas.fill_rect(bounds).map_err(runtime_error)?;
    }
    if !widget.label.is_empty() {
        let previous_clip = canvas.clip_rect();
        canvas.set_clip_rect(previous_clip.map_or(Some(bounds), |clip| clip.intersection(bounds)));
        let disabled = widget.unavailable.is_some();
        let color = if disabled {
            Color::RGB(126, 118, 108)
        } else {
            label_color(widget)
        };
        let label = &widget.label;
        let result = if widget.node.field("wordwrap").is_some_and(truth) {
            draw_wrapped_text(canvas, font, label, bounds, color)
        } else if widget.id.contains('/') || widget.node.field("just") == Some("left") {
            draw_text(
                canvas,
                font,
                label,
                bounds
                    .x()
                    .saturating_add(if widget.node.field("richtext") == Some("true") {
                        0
                    } else {
                        3
                    }),
                bounds.y(),
                color,
            )
        } else {
            draw_centered_text(canvas, font, label, bounds, color, false)
        };
        canvas.set_clip_rect(previous_clip);
        result?;
        if widget.node.field("richtext") == Some("true") && widget.intent.is_some() {
            let y = bounds
                .y()
                .saturating_add(font.map_or(7_i32, |font| font.ascent().saturating_add(1_i32)))
                .min(bounds.bottom().saturating_sub(1_i32));
            canvas.set_draw_color(color);
            canvas
                .draw_line((bounds.x(), y), (bounds.right().saturating_sub(1_i32), y))
                .map_err(runtime_error)?;
        }
    }
    Ok(())
}

/// Replaces the platform-specific Windows EDIT drawing with SDL's clipped,
/// white text field while keeping the original logical bounds and Unicode value.
fn draw_text_field(
    canvas: &mut Canvas<Window>,
    widget: &ManagerWidget,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    let bounds = rect(widget);
    canvas.set_draw_color(Color::RGB(255_u8, 255_u8, 255_u8));
    canvas.fill_rect(bounds).map_err(runtime_error)?;
    canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
    canvas.draw_rect(bounds).map_err(runtime_error)?;
    let width = font.map_or_else(
        || {
            u32::try_from(widget.label.chars().count())
                .unwrap_or(u32::MAX)
                .saturating_mul(6_u32)
        },
        |font| font.size_of(&widget.label).map_or(0_u32, |size| size.0),
    );
    let excess = width.saturating_sub(bounds.width().saturating_sub(6_u32));
    let x = bounds
        .x()
        .saturating_add(3_i32)
        .saturating_sub(i32::try_from(excess).unwrap_or(i32::MAX));
    draw_text(
        canvas,
        font,
        &widget.label,
        x,
        bounds.y().saturating_add(2_i32),
        Color::RGB(0_u8, 0_u8, 0_u8),
    )?;
    if widget.node.field("textfocused") == Some("true") {
        let caret = x.saturating_add(i32::try_from(width).unwrap_or(i32::MAX));
        canvas
            .draw_line(
                (caret, bounds.y().saturating_add(3_i32)),
                (caret, bounds.bottom().saturating_sub(3_i32)),
            )
            .map_err(runtime_error)?;
    }
    Ok(())
}

/// AUI selects an enabled image layer; absent legacy layers are not invented.
fn draw_images(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    widget: &ManagerWidget,
) -> Result<(), GameplayError> {
    let node = &widget.node;
    let enabled = widget.intent.is_some();
    let count = node.integer("imagesperlayer").unwrap_or(1);
    let layer = if node.field("layeractive1").is_some_and(truth) {
        if widget.selected {
            1_u8
        } else if node.field("tabside") == Some("right") {
            2_u8
        } else {
            0_u8
        }
    } else if widget.selected && count == 3_i32 && node.field("image40").is_some() {
        4_u8
    } else {
        u8::from(enabled && node.field("image10").is_some())
    };
    match count {
        9 => draw_nine(canvas, textures, widget)?,
        3 => draw_three(canvas, textures, widget, layer)?,
        _ => {
            let name = node
                .field(&format!("image{layer}0"))
                .or_else(|| node.field("image0"))
                .or_else(|| node.field("image1"));
            if let Some(name) = name {
                let tiled = node.field("imageblttype00") == Some("tile");
                if node.field("imageblttype") == Some("stretch") {
                    if let Some(texture) = textures.dynamic_ui(&art_name(name))? {
                        canvas
                            .copy(texture, None, rect(widget))
                            .map_err(runtime_error)?;
                    }
                } else {
                    copy_image(
                        canvas,
                        textures,
                        name,
                        rect(widget),
                        tiled,
                        node.field("centerimage") == Some("true"),
                    )?;
                }
            } else if widget.id == "PopupClose" {
                copy_image(canvas, textures, "upbt02u.rim", rect(widget), false, false)?;
            }
        }
    }
    if count == 1_i32 {
        if let Some(name) = node.field("image40") {
            if node.field("imageblttype40") == Some("stretch") {
                if let Some(texture) = textures.dynamic_ui(&art_name(name))? {
                    canvas
                        .copy(texture, None, rect(widget))
                        .map_err(runtime_error)?;
                }
            } else {
                copy_image(canvas, textures, name, rect(widget), false, false)?;
            }
        }
    }
    Ok(())
}

/// Nine cells retain native corners/side art; only declared middle strips tile.
fn draw_nine(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    widget: &ManagerWidget,
) -> Result<(), GameplayError> {
    let node = &widget.node;
    let bounds = rect(widget);
    let names: Vec<_> = (0_u8..9)
        .map(|index| node.field(&format!("image0{index}")).map(str::to_owned))
        .collect();
    let dimensions = names
        .iter()
        .map(|name| {
            name.as_ref()
                .map_or(Ok((0_u32, 0_u32)), |name| image_size(textures, name))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let left = dimensions[0].0.max(dimensions[3].0).min(bounds.width());
    let right = dimensions[2]
        .0
        .max(dimensions[5].0)
        .min(bounds.width().saturating_sub(left));
    let top = dimensions[0].1.max(dimensions[2].1).min(bounds.height());
    let bottom = dimensions[6]
        .1
        .max(dimensions[8].1)
        .min(bounds.height().saturating_sub(top));
    let widths = [
        left,
        bounds.width().saturating_sub(left).saturating_sub(right),
        right,
    ];
    let heights = [
        top,
        bounds.height().saturating_sub(top).saturating_sub(bottom),
        bottom,
    ];
    let mut y = bounds.y();
    for (row, &height) in heights.iter().enumerate() {
        let mut x = bounds.x();
        for (column, &width) in widths.iter().enumerate() {
            let index = row.saturating_mul(3).saturating_add(column);
            if let Some(name) = names[index].as_deref() {
                let key = format!("imageblttype0{index}");
                let tiled = node.field(&key) == Some("tile")
                    || node
                        .field(&format!("imagestretchx0{index}"))
                        .is_some_and(truth)
                    || node
                        .field(&format!("imagestretchy0{index}"))
                        .is_some_and(truth);
                copy_image(
                    canvas,
                    textures,
                    name,
                    Rect::new(x, y, width, height),
                    tiled,
                    false,
                )?;
            }
            x = x.saturating_add(i32::try_from(width).unwrap_or(0_i32));
        }
        y = y.saturating_add(i32::try_from(height).unwrap_or(0_i32));
    }
    Ok(())
}

/// Three-part sizable text buttons preserve original endcaps and tile the middle.
fn draw_three(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    widget: &ManagerWidget,
    layer: u8,
) -> Result<(), GameplayError> {
    let names: Vec<_> = (0_u8..3)
        .map(|index| {
            widget
                .node
                .field(&format!("image{layer}{index}"))
                .map(str::to_owned)
        })
        .collect();
    let bounds = rect(widget);
    let left = names[0]
        .as_deref()
        .map_or(Ok(0), |name| image_size(textures, name).map(|size| size.0))?
        .min(bounds.width());
    let right = names[2]
        .as_deref()
        .map_or(Ok(0), |name| image_size(textures, name).map(|size| size.0))?
        .min(bounds.width().saturating_sub(left));
    let widths = [
        left,
        bounds.width().saturating_sub(left).saturating_sub(right),
        right,
    ];
    let mut x = bounds.x();
    for (index, name) in names.iter().enumerate() {
        if let Some(name) = name.as_deref() {
            copy_image(
                canvas,
                textures,
                name,
                Rect::new(x, bounds.y(), widths[index], bounds.height()),
                index == 1,
                false,
            )?;
        }
        x = x.saturating_add(i32::try_from(widths[index]).unwrap_or(0));
    }
    Ok(())
}

/// Shared lazy texture sizing does not extract or write installation content.
fn image_size(
    textures: &mut GameplayTextures<'_>,
    name: &str,
) -> Result<(u32, u32), GameplayError> {
    Ok(textures
        .dynamic_ui(&art_name(name))?
        .map_or((0, 0), |texture| {
            let query = texture.query();
            (query.width, query.height)
        }))
}

/// Copies/tiles original art with native size for non-stretch layers.
fn copy_image(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    name: &str,
    bounds: Rect,
    tiled: bool,
    centered: bool,
) -> Result<(), GameplayError> {
    if bounds.width() == 0 || bounds.height() == 0 {
        return Ok(());
    }
    if let Some(texture) = textures.dynamic_ui(&art_name(name))? {
        if tiled {
            tile_texture(
                canvas,
                texture,
                bounds.x(),
                bounds.y(),
                bounds.width(),
                bounds.height(),
            )?;
        } else {
            let size = texture.query();
            let x = if centered {
                bounds.x().saturating_add(
                    i32::try_from(bounds.width().saturating_sub(size.width))
                        .unwrap_or(0)
                        .div_euclid(2),
                )
            } else {
                bounds.x()
            };
            let y = if centered {
                bounds.y().saturating_add(
                    i32::try_from(bounds.height().saturating_sub(size.height))
                        .unwrap_or(0)
                        .div_euclid(2),
                )
            } else {
                bounds.y()
            };
            canvas
                .copy(texture, None, Rect::new(x, y, size.width, size.height))
                .map_err(runtime_error)?;
        }
    }
    Ok(())
}

/// Original grayscale text color unless a source control explicitly overrides it.
fn label_color(widget: &ManagerWidget) -> Color {
    let channel = |name| u8::try_from(widget.node.integer(name).unwrap_or(50)).unwrap_or(50);
    Color::RGB(
        channel("fontcolorred"),
        channel("fontcolorgreen"),
        channel("fontcolorblue"),
    )
}

/// Maps installed source TGA names to the already-supported RIM/TGA lookup.
fn art_name(name: &str) -> String {
    name.to_ascii_lowercase().replace(".tga", ".rim")
}

/// Converts only shared model geometry; no rendering-only offsets.
fn rect(widget: &ManagerWidget) -> Rect {
    Rect::new(
        widget.rect.x,
        widget.rect.y,
        widget.rect.width,
        widget.rect.height,
    )
}

/// LDL booleans are case-insensitive in the original parser.
fn truth(value: &str) -> bool {
    value.eq_ignore_ascii_case("true")
}
