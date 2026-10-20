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
use crate::gameplay::model::GameState;
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::ui_model::InGameUiModel;
use crate::gameplay::ui_model::management::{ManagerKind, ManagerWidget};
use crate::gameplay::{GameplayError, runtime_error};

/// Source face-specific alignment, title shadows and link underlines.
mod labels;
/// Platform-independent native search editing visuals.
mod text_field;

/// Draws an open manager after the map/panel and before the software cursor.
pub(super) fn draw_management<'draw, 'ttf: 'draw, 'rwops: 'draw>(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
    ui: &InGameUiModel,
    font: impl Into<crate::gameplay::runtime::fonts::RenderFonts<'draw, 'ttf, 'rwops>>,
) -> Result<(), GameplayError> {
    let fonts = font.into();
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
            draw_widget(canvas, textures, widget, fonts.control(&widget.node))?;
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
    // SDL Rect::new coerces zero extents to one: reject before conversion.
    if widget.rect.width == 0_u32 || widget.rect.height == 0_u32 {
        return Ok(());
    }
    let bounds = rect(widget);
    let previous = canvas.clip_rect();
    // AUI centers titles in their bound but clips them to the window, permitting
    // horizontal text overhang. Images still retain their own control clipping.
    let text_clip = if widget.node.field("overflow-title") == Some("true") {
        Rect::new(
            canvas.viewport().x(),
            bounds.y(),
            canvas.viewport().width(),
            bounds.height().saturating_add(1_u32),
        )
    } else {
        bounds
    };
    let clip = previous.map_or(Some(text_clip), |parent| parent.intersection(text_clip));
    let clip = content_clip(&widget.node).map_or(clip, |content| {
        clip.and_then(|clip| clip.intersection(content))
    });
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
    if let Some(kind) = widget.node.field("primitive") {
        return draw_primitive(canvas, widget, kind);
    }
    if widget
        .node
        .field("objecttype")
        .is_some_and(|kind| kind.eq_ignore_ascii_case("ctp2_Textfield"))
    {
        return text_field::draw(canvas, widget, font);
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
        canvas.set_draw_color(Color::RGB(255_u8, 179_u8, 69_u8));
        let selected = Rect::new(
            bounds.x().saturating_add(1_i32),
            bounds.y(),
            bounds.width().saturating_sub(2_u32),
            bounds.height(),
        );
        if selected.width() > 0_u32 && selected.height() > 0_u32 {
            canvas.fill_rect(selected).map_err(runtime_error)?;
        }
    }
    labels::draw(canvas, widget, font)
}

/// Native chart connectors/borders use the same bounded descriptors as buttons.
fn draw_primitive(
    canvas: &mut Canvas<Window>,
    widget: &ManagerWidget,
    kind: &str,
) -> Result<(), GameplayError> {
    canvas.set_draw_color(label_color(widget));
    if kind == "frame" {
        return canvas.draw_rect(rect(widget)).map_err(runtime_error);
    }
    if kind == "fill" {
        return canvas.fill_rect(rect(widget)).map_err(runtime_error);
    }
    if kind != "line" {
        return Err(runtime_error("Unknown manager primitive".to_owned()));
    }
    let coordinate = |name| {
        widget
            .node
            .integer(name)
            .ok_or_else(|| runtime_error(format!("Missing chart coordinate {name}")))
    };
    canvas
        .draw_line(
            (coordinate("x1")?, coordinate("y1")?),
            (coordinate("x2")?, coordinate("y2")?),
        )
        .map_err(runtime_error)
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

/// Source item children clip to their content pane, never over adjacent controls.
fn content_clip(node: &crate::gameplay::assets::layout::LayoutNode) -> Option<Rect> {
    Some(Rect::new(
        node.field("clip-x")?.parse::<i32>().ok()?,
        node.field("clip-y")?.parse::<i32>().ok()?,
        node.field("clip-width")?.parse::<u32>().ok()?,
        node.field("clip-height")?.parse::<u32>().ok()?,
    ))
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
