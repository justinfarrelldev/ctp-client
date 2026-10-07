//! Headless SDL runtime smoke-test helpers.

use std::path::Path;

use sdl2::pixels::PixelFormatEnum;

use super::*;

/// Computes average RGB color for a rendered headless menu frame.
#[cfg(test)]
pub(in crate::main_menu) fn render_headless_average_color_for_test(
    model: &MainMenuModel,
) -> Result<(u8, u8, u8), MainMenuError> {
    let sdl = sdl2::init().map_err(runtime_error)?;
    let video = sdl.video().map_err(runtime_error)?;
    let window = video
        .window("Call to Power II", model.width, model.height)
        .position_centered()
        .build()
        .map_err(|error| runtime_error(error.to_string()))?;
    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|error| runtime_error(error.to_string()))?;
    let texture_creator = canvas.texture_creator();
    let textures = load_menu_textures(&texture_creator, model);

    draw_menu(&mut canvas, &textures, model, -1, -1, None)?;

    let pixels = canvas
        .read_pixels(None, MENU_TEXTURE_FORMAT)
        .map_err(runtime_error)?;
    average_rgba_pixels(&pixels)
}

/// Returns summaries for original button art entries used by a style.
#[cfg(test)]
pub(in crate::main_menu) fn button_style_asset_summaries_for_test(
    installation_root: &Path,
    style: ButtonVisualStyle,
) -> Result<Vec<(String, u32, u32, usize)>, MainMenuError> {
    let triplets = [
        style.assets.inactive,
        style.assets.up,
        style.assets.down,
        style.assets.hover,
    ];
    let mut summaries = Vec::new();

    for triplet in triplets {
        for asset_name in [triplet.left, triplet.middle, triplet.right] {
            let entry_name = button_asset_entry_name(asset_name);
            let image = load_rim_rgba_entry(
                installation_root,
                SHELL_ART_ARCHIVE,
                &entry_name,
                Some(BUTTON_CHROMA_KEY),
            )?;
            let transparent_pixels = image
                .data
                .chunks_exact(4)
                .filter(|pixel| pixel.get(3) == Some(&0))
                .count();
            summaries.push((entry_name, image.width, image.height, transparent_pixels));
        }
    }

    Ok(summaries)
}

/// Returns the packed RIM entry name for a button asset in tests.
#[cfg(test)]
pub(in crate::main_menu) fn button_asset_entry_name_for_test(ldl_asset_name: &str) -> String {
    button_asset_entry_name(ldl_asset_name)
}

/// Returns the texture format used for RGBA uploads in tests.
#[cfg(test)]
pub(in crate::main_menu) const fn menu_texture_format_for_test() -> PixelFormatEnum {
    MENU_TEXTURE_FORMAT
}

/// Computes average RGB from RGBA byte pixels.
#[cfg(test)]
fn average_rgba_pixels(pixels: &[u8]) -> Result<(u8, u8, u8), MainMenuError> {
    let mut red_total = 0_u64;
    let mut green_total = 0_u64;
    let mut blue_total = 0_u64;
    let mut pixel_count = 0_u64;

    for pixel in pixels.chunks_exact(4) {
        red_total = red_total.saturating_add(u64::from(pixel[0]));
        green_total = green_total.saturating_add(u64::from(pixel[1]));
        blue_total = blue_total.saturating_add(u64::from(pixel[2]));
        pixel_count = pixel_count.saturating_add(1);
    }

    if pixel_count == 0 {
        return Err(runtime_error(
            "headless render produced no readable pixels".to_owned(),
        ));
    }

    Ok((
        average_channel(red_total, pixel_count),
        average_channel(green_total, pixel_count),
        average_channel(blue_total, pixel_count),
    ))
}

/// Computes one bounded average color channel.
#[cfg(test)]
fn average_channel(total: u64, count: u64) -> u8 {
    let average = total.checked_div(count).map_or(0_u64, |value| value);
    u8::try_from(average.min(u64::from(u8::MAX))).map_or(u8::MAX, |value| value)
}
