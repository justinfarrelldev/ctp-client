//! Original CTP2 shell button art loading and rendering.

use sdl2::rect::Rect as SdlRect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

use crate::main_menu::{
    ButtonAssetTriplet, ButtonVisualStyle, MainMenuError, MainMenuModel, MenuRect,
};

use super::{
    BUTTON_CHROMA_KEY, SHELL_ART_ARCHIVE, create_texture_from_rgba, load_rim_rgba_entry,
    runtime_error,
};

/// Texture triplets for available button interaction states.
pub(super) struct ButtonStateTextures<'texture> {
    /// Textures for the original always/background layer.
    always: ButtonTextureTriplet<'texture>,
    /// Textures for default/up state.
    up: ButtonTextureTriplet<'texture>,
    /// Textures for hover highlight overlay state.
    hover: ButtonTextureTriplet<'texture>,
}

/// Three-piece button textures matching the original CTP2 shell assets.
struct ButtonTextureTriplet<'texture> {
    /// Left cap texture.
    left: Texture<'texture>,
    /// Tiled center texture.
    middle: Texture<'texture>,
    /// Right cap texture.
    right: Texture<'texture>,
}

/// Width and height reported by an SDL texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TextureDimensions {
    /// Texture width in pixels.
    pub(super) width: u32,
    /// Texture height in pixels.
    pub(super) height: u32,
}

/// Draws a button with original three-piece art and layer flags.
pub(super) fn draw_styled_button(
    canvas: &mut Canvas<Window>,
    rect: &MenuRect,
    textures: &ButtonStateTextures<'_>,
    hovered: bool,
) -> Result<(), MainMenuError> {
    draw_texture_triplet(canvas, rect, &textures.always)?;
    draw_texture_triplet(canvas, rect, &textures.up)?;
    if hovered {
        draw_texture_triplet(canvas, rect, &textures.hover)?;
    }
    Ok(())
}

/// Draws a three-piece CTP2 shell button using native endcaps and a tiled center.
fn draw_texture_triplet(
    canvas: &mut Canvas<Window>,
    rect: &MenuRect,
    triplet: &ButtonTextureTriplet<'_>,
) -> Result<(), MainMenuError> {
    let left_dimensions = texture_dimensions(&triplet.left);
    let middle_dimensions = texture_dimensions(&triplet.middle);
    let right_dimensions = texture_dimensions(&triplet.right);
    let middle_width =
        tiled_middle_width(rect.width, left_dimensions.width, right_dimensions.width);
    let middle_x = rect.x.saturating_add(u32_to_i32(left_dimensions.width));
    let right_x = middle_x.saturating_add(u32_to_i32(middle_width));

    canvas
        .copy(
            &triplet.left,
            None,
            SdlRect::new(
                rect.x,
                rect.y,
                left_dimensions.width,
                left_dimensions.height,
            ),
        )
        .map_err(runtime_error)?;
    draw_tiled_texture(
        canvas,
        &triplet.middle,
        middle_x,
        rect.y,
        middle_width,
        middle_dimensions,
    )?;
    canvas
        .copy(
            &triplet.right,
            None,
            SdlRect::new(
                right_x,
                rect.y,
                right_dimensions.width,
                right_dimensions.height,
            ),
        )
        .map_err(runtime_error)
}

/// Draws a texture repeatedly across the requested destination width.
fn draw_tiled_texture(
    canvas: &mut Canvas<Window>,
    texture: &Texture<'_>,
    destination_x: i32,
    destination_y: i32,
    destination_width: u32,
    dimensions: TextureDimensions,
) -> Result<(), MainMenuError> {
    if destination_width == 0 || dimensions.width == 0 || dimensions.height == 0 {
        return Ok(());
    }

    let mut remaining_width = destination_width;
    let mut current_x = destination_x;
    while remaining_width > 0 {
        let copy_width = remaining_width.min(dimensions.width);
        canvas
            .copy(
                texture,
                SdlRect::new(0, 0, copy_width, dimensions.height),
                SdlRect::new(current_x, destination_y, copy_width, dimensions.height),
            )
            .map_err(runtime_error)?;
        remaining_width = remaining_width.saturating_sub(copy_width);
        current_x = current_x.saturating_add(u32_to_i32(copy_width));
    }

    Ok(())
}

/// Computes the center-piece destination width for original CTP2 sizable buttons.
pub(super) const fn tiled_middle_width(total_width: u32, left_width: u32, right_width: u32) -> u32 {
    total_width.saturating_sub(left_width.saturating_add(right_width))
}

/// Returns the texture dimensions used for native button-piece rendering.
fn texture_dimensions(texture: &Texture<'_>) -> TextureDimensions {
    let query = texture.query();
    TextureDimensions {
        width: query.width,
        height: query.height,
    }
}

/// Builds the text rectangle after original button art expands the native layer height.
pub(super) fn styled_button_text_rect(
    rect: &MenuRect,
    textures: &ButtonStateTextures<'_>,
    hovered: bool,
) -> MenuRect {
    let highlighted_height = if hovered {
        triplet_height(&textures.hover)
    } else {
        0
    };
    MenuRect::new(
        rect.x,
        rect.y,
        rect.width,
        triplet_height(&textures.always)
            .max(triplet_height(&textures.up))
            .max(highlighted_height)
            .max(rect.height),
    )
}

/// Returns the maximum native height of a three-piece button texture set.
fn triplet_height(triplet: &ButtonTextureTriplet<'_>) -> u32 {
    texture_dimensions(&triplet.left)
        .height
        .max(texture_dimensions(&triplet.middle).height)
        .max(texture_dimensions(&triplet.right).height)
}

/// Converts a `u32` pixel count into a saturating signed coordinate offset.
fn u32_to_i32(value: u32) -> i32 {
    i32::try_from(value).map_or(i32::MAX, |converted| converted)
}

/// Loads original button textures for the style states used by the landing menu.
pub(super) fn load_button_state_textures<'texture>(
    texture_creator: &'texture TextureCreator<WindowContext>,
    model: &MainMenuModel,
    style: ButtonVisualStyle,
) -> Result<ButtonStateTextures<'texture>, MainMenuError> {
    Ok(ButtonStateTextures {
        always: load_button_triplet(texture_creator, model, style.assets.inactive)?,
        up: load_button_triplet(texture_creator, model, style.assets.up)?,
        hover: load_button_triplet(texture_creator, model, style.assets.hover)?,
    })
}

/// Loads one three-piece button texture triplet from immutable original assets.
fn load_button_triplet<'texture>(
    texture_creator: &'texture TextureCreator<WindowContext>,
    model: &MainMenuModel,
    triplet: ButtonAssetTriplet,
) -> Result<ButtonTextureTriplet<'texture>, MainMenuError> {
    Ok(ButtonTextureTriplet {
        left: load_button_texture(texture_creator, model, triplet.left)?,
        middle: load_button_texture(texture_creator, model, triplet.middle)?,
        right: load_button_texture(texture_creator, model, triplet.right)?,
    })
}

/// Loads one button image, applying the original chroma key as alpha.
fn load_button_texture<'texture>(
    texture_creator: &'texture TextureCreator<WindowContext>,
    model: &MainMenuModel,
    ldl_asset_name: &str,
) -> Result<Texture<'texture>, MainMenuError> {
    let entry_name = button_asset_entry_name(ldl_asset_name);
    let image = load_rim_rgba_entry(
        &model.installation_root,
        SHELL_ART_ARCHIVE,
        &entry_name,
        Some(BUTTON_CHROMA_KEY),
    )?;
    create_texture_from_rgba(texture_creator, &image)
}

/// Converts an LDL/TGA asset reference to its packed RIM entry name.
pub(super) fn button_asset_entry_name(ldl_asset_name: &str) -> String {
    ldl_asset_name
        .to_ascii_lowercase()
        .strip_suffix(".tga")
        .map_or_else(
            || format!("{}.rim", ldl_asset_name.to_ascii_lowercase()),
            |stem| format!("{stem}.rim"),
        )
}
