//! Five-image source dropdown buttons retain native caps and tiled fill strips.

use super::{copy_image, image_size, rect};
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::ui_model::management::ManagerWidget;
use crate::gameplay::{GameplayError, runtime_error};
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// Fit native cap widths to the declared vertical extent. Fill strips repeat
/// horizontally only: clipping a 24px image to a 15px control loses its bottom.
pub(super) fn strip(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    name: &str,
    bounds: Rect,
    repeat: bool,
) -> Result<(), GameplayError> {
    if bounds.width() == 0_u32 || bounds.height() == 0_u32 {
        return Ok(());
    }
    let Some(texture) = textures.dynamic_ui(&super::art_name(name))? else {
        return Ok(());
    };
    let size = texture.query();
    if size.width == 0_u32 || size.height == 0_u32 {
        return Ok(());
    }
    if !repeat {
        return canvas.copy(texture, None, bounds).map_err(runtime_error);
    }
    let mut offset = 0_u32;
    while offset < bounds.width() {
        let width = size.width.min(bounds.width().saturating_sub(offset));
        canvas
            .copy(
                texture,
                Rect::new(0_i32, 0_i32, width, size.height),
                Rect::new(
                    bounds
                        .x()
                        .saturating_add(i32::try_from(offset).unwrap_or(i32::MAX)),
                    bounds.y(),
                    width,
                    bounds.height(),
                ),
            )
            .map_err(runtime_error)?;
        offset = offset.saturating_add(width);
    }
    Ok(())
}

/// Original A/B/C/B/D layout: fixed native caps/center, two equal fill strips.
pub(super) fn five(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    widget: &ManagerWidget,
    layer: u8,
) -> Result<(), GameplayError> {
    let mut names = Vec::new();
    let mut native = Vec::new();
    for index in 0_u8..5_u8 {
        let name = widget
            .node
            .field(&format!("image{layer}{index}"))
            .map(str::to_owned);
        let width = name.as_deref().map_or(Ok(0_u32), |name| {
            image_size(textures, name).map(|size| size.0)
        })?;
        names.push(name);
        native.push(width);
    }
    let bounds = rect(widget);
    let left = native[0].min(bounds.width());
    let center = native[2].min(bounds.width().saturating_sub(left));
    let right = native[4].min(bounds.width().saturating_sub(left).saturating_sub(center));
    let remaining = bounds
        .width()
        .saturating_sub(left)
        .saturating_sub(center)
        .saturating_sub(right);
    let first = remaining.div_euclid(2_u32);
    // Do not leave an unpainted pixel when original FillWidth's integer division
    // truncates an odd remainder; give it to the final stretchable strip.
    let widths = [left, first, center, remaining.saturating_sub(first), right];
    let mut x = bounds.x();
    for (index, width) in widths.into_iter().enumerate() {
        if width > 0_u32 {
            if let Some(name) = names[index].as_deref() {
                copy_image(
                    canvas,
                    textures,
                    name,
                    Rect::new(x, bounds.y(), width, bounds.height()),
                    matches!(index, 1_usize | 3_usize),
                    false,
                )?;
            }
        }
        x = x.saturating_add(i32::try_from(width).unwrap_or(i32::MAX));
    }
    Ok(())
}
