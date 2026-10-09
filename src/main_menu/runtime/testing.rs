//! Headless SDL runtime smoke-test helpers.

use std::path::{Path, PathBuf};

use sdl2::pixels::PixelFormatEnum;

use crate::main_menu::ButtonAssetTriplet;

use super::*;

/// Test summary for one original sound asset: archive, entry, byte length, and RIFF header.
pub(in crate::main_menu) type SoundAssetSummary = audio::SoundAssetSummary;

/// Named button layer render plan rows: layer, entry, width, height, and tiled flag.
type ButtonLayerRenderPlan = Vec<(String, String, u32, u32, bool)>;

/// Computes average RGB color for a rendered headless menu frame.
#[cfg(test)]
pub(in crate::main_menu) fn render_headless_average_color_for_test(
    model: &MainMenuModel,
) -> Result<(u8, u8, u8), MainMenuError> {
    with_sdl_test_lock(|| {
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
    })
}

/// Returns a summary for the original button-click sound asset.
#[cfg(test)]
pub(in crate::main_menu) fn button_click_sound_summary_for_test(
    installation_root: &Path,
) -> Result<SoundAssetSummary, MainMenuError> {
    audio::button_click_sound_summary_for_test(installation_root)
}

/// Returns summaries for the original CTP2 slider sound assets.
#[cfg(test)]
pub(in crate::main_menu) fn slider_sound_summaries_for_test(
    installation_root: &Path,
) -> Result<Vec<SoundAssetSummary>, MainMenuError> {
    audio::slider_sound_summaries_for_test(installation_root)
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
            let entry_name = button_art::button_asset_entry_name(asset_name);
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
    button_art::button_asset_entry_name(ldl_asset_name)
}

/// Returns the original dark button label color used by the renderer.
#[cfg(test)]
pub(in crate::main_menu) const fn button_label_text_color_for_test() -> (u8, u8, u8) {
    BUTTON_LABEL_TEXT_COLOR
}

/// Returns a native/tiled render plan for a button triplet loaded from original assets.
#[cfg(test)]
pub(in crate::main_menu) fn button_triplet_render_plan_for_test(
    installation_root: &Path,
    triplet: ButtonAssetTriplet,
    target_width: u32,
) -> Result<Vec<(String, u32, u32, bool)>, MainMenuError> {
    let left = button_piece_dimensions_for_test(installation_root, triplet.left)?;
    let middle = button_piece_dimensions_for_test(installation_root, triplet.middle)?;
    let right = button_piece_dimensions_for_test(installation_root, triplet.right)?;
    let middle_width =
        button_art::tiled_middle_width(target_width, left.dimensions.width, right.dimensions.width);

    Ok(vec![
        (
            left.entry_name,
            left.dimensions.width,
            left.dimensions.height,
            false,
        ),
        (
            middle.entry_name,
            middle_width,
            middle.dimensions.height,
            true,
        ),
        (
            right.entry_name,
            right.dimensions.width,
            right.dimensions.height,
            false,
        ),
    ])
}

/// Returns the original disabled layer stack for a styled button.
#[cfg(test)]
pub(in crate::main_menu) fn disabled_button_layer_render_plan_for_test(
    installation_root: &Path,
    style: ButtonVisualStyle,
    target_width: u32,
) -> Result<ButtonLayerRenderPlan, MainMenuError> {
    let mut plan = Vec::new();
    append_named_triplet_plan(
        &mut plan,
        "disabled",
        installation_root,
        style.assets.inactive,
        target_width,
    )?;
    Ok(plan)
}

/// Returns the original layer stack for a styled button in one interaction state.
#[cfg(test)]
pub(in crate::main_menu) fn button_layer_render_plan_for_test(
    installation_root: &Path,
    style: ButtonVisualStyle,
    hovered: bool,
    target_width: u32,
) -> Result<ButtonLayerRenderPlan, MainMenuError> {
    let mut plan = Vec::new();
    append_named_triplet_plan(
        &mut plan,
        "always",
        installation_root,
        style.assets.inactive,
        target_width,
    )?;
    append_named_triplet_plan(
        &mut plan,
        "up",
        installation_root,
        style.assets.up,
        target_width,
    )?;
    if hovered {
        append_named_triplet_plan(
            &mut plan,
            "highlight",
            installation_root,
            style.assets.hover,
            target_width,
        )?;
    }
    Ok(plan)
}

/// Appends one named original-art triplet render plan to a layer plan.
#[cfg(test)]
fn append_named_triplet_plan(
    plan: &mut ButtonLayerRenderPlan,
    layer_name: &str,
    installation_root: &Path,
    triplet: ButtonAssetTriplet,
    target_width: u32,
) -> Result<(), MainMenuError> {
    for (entry_name, width, height, tiled) in
        button_triplet_render_plan_for_test(installation_root, triplet, target_width)?
    {
        plan.push((layer_name.to_owned(), entry_name, width, height, tiled));
    }
    Ok(())
}

/// Original button piece metadata needed for render-plan tests.
#[cfg(test)]
struct ButtonPieceDimensions {
    /// Packed RIM entry name.
    entry_name: String,
    /// Original image dimensions.
    dimensions: button_art::TextureDimensions,
}

/// Loads the original dimensions for one packed button piece.
#[cfg(test)]
fn button_piece_dimensions_for_test(
    installation_root: &Path,
    ldl_asset_name: &str,
) -> Result<ButtonPieceDimensions, MainMenuError> {
    let entry_name = button_art::button_asset_entry_name(ldl_asset_name);
    let image = load_rim_rgba_entry(
        installation_root,
        SHELL_ART_ARCHIVE,
        &entry_name,
        Some(BUTTON_CHROMA_KEY),
    )?;

    Ok(ButtonPieceDimensions {
        entry_name,
        dimensions: button_art::TextureDimensions {
            width: image.width,
            height: image.height,
        },
    })
}

/// Returns the texture format used for RGBA uploads in tests.
#[cfg(test)]
pub(in crate::main_menu) const fn menu_texture_format_for_test() -> PixelFormatEnum {
    MENU_TEXTURE_FORMAT
}

/// Converts an option volume to `SDL_mixer` scale in tests.
#[cfg(test)]
pub(in crate::main_menu) fn option_volume_to_mixer_volume_for_test(value: i32) -> i32 {
    option_volume_to_mixer_volume(value)
}

/// Parses a resolution label in tests.
#[cfg(test)]
pub(in crate::main_menu) fn parse_resolution_label_for_test(label: &str) -> Option<(u32, u32)> {
    parse_resolution_label(label)
}

/// Returns a deterministic initial music track for tests.
#[cfg(test)]
pub(in crate::main_menu) fn initial_music_track_for_test(
    selected_track: usize,
    random_order: bool,
    seed: u64,
) -> usize {
    audio::initial_music_track_number_from_seed(selected_track, random_order, seed)
}

/// Returns a track path for tests.
#[cfg(test)]
pub(in crate::main_menu) fn music_track_relative_path_for_test(track_number: usize) -> PathBuf {
    music_track_relative_path(track_number)
}

/// Returns the low-latency mixer chunk size for tests.
#[cfg(test)]
pub(in crate::main_menu) const fn mixer_chunk_size_for_test() -> i32 {
    audio::mixer_chunk_size_for_test()
}

/// Returns the mixer channel count for tests.
#[cfg(test)]
pub(in crate::main_menu) const fn mixer_allocated_channels_for_test() -> i32 {
    audio::mixer_allocated_channels_for_test()
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
