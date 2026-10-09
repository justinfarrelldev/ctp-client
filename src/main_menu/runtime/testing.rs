//! Headless SDL runtime smoke-test helpers.

use std::path::{Path, PathBuf};

use sdl2::pixels::PixelFormatEnum;

use crate::main_menu::shell;
use crate::main_menu::{ButtonAssetTriplet, MenuRect};

use super::viewport::prepare_menu_canvas;
use super::*;

/// Test summary for one original sound asset: archive, entry, byte length, and RIFF header.
pub(in crate::main_menu) type SoundAssetSummary = audio::SoundAssetSummary;

/// Named button layer render plan rows: layer, entry, width, height, and tiled flag.
type ButtonLayerRenderPlan = Vec<(String, String, u32, u32, bool)>;

/// Renderer state captured immediately after menu-canvas preparation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::main_menu) struct PreparedCanvasState {
    /// Physical output size reported by the renderer.
    pub(in crate::main_menu) output_size: (u32, u32),
    /// SDL logical size reported by the renderer.
    pub(in crate::main_menu) logical_size: (u32, u32),
    /// Physical viewport that receives the scaled menu frame.
    pub(in crate::main_menu) viewport: MenuRect,
}

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

        frame::draw_menu(&mut canvas, &textures, model, -1, -1, None)?;

        let pixels = canvas
            .read_pixels(None, MENU_TEXTURE_FORMAT)
            .map_err(runtime_error)?;
        average_rgba_pixels(&pixels)
    })
}

/// Captures renderer state after preparing a scaled menu frame.
#[cfg(test)]
pub(in crate::main_menu) fn prepared_canvas_state_for_test(
    _model: &MainMenuModel,
    output_size: (u32, u32),
) -> Result<PreparedCanvasState, MainMenuError> {
    with_sdl_test_lock(|| {
        let sdl = sdl2::init().map_err(runtime_error)?;
        let video = sdl.video().map_err(runtime_error)?;
        let window = video
            .window("Call to Power II", output_size.0, output_size.1)
            .position_centered()
            .build()
            .map_err(|error| runtime_error(error.to_string()))?;
        let mut canvas = window
            .into_canvas()
            .software()
            .build()
            .map_err(|error| runtime_error(error.to_string()))?;

        prepare_menu_canvas(&mut canvas)?;
        let actual_output_size = canvas.output_size().map_err(runtime_error)?;
        Ok(PreparedCanvasState {
            output_size: actual_output_size,
            logical_size: canvas.logical_size(),
            viewport: viewport::menu_viewport_for_output_size(
                actual_output_size.0,
                actual_output_size.1,
            ),
        })
    })
}

/// Counts unpainted black pixels inside the active menu viewport after a resize path.
#[cfg(test)]
pub(in crate::main_menu) fn black_pixels_inside_viewport_after_resize_for_test(
    model: &MainMenuModel,
    first_size: (u32, u32),
    second_size: (u32, u32),
) -> Result<usize, MainMenuError> {
    with_sdl_test_lock(|| {
        let sdl = sdl2::init().map_err(runtime_error)?;
        let video = sdl.video().map_err(runtime_error)?;
        let window = video
            .window("Call to Power II", first_size.0, first_size.1)
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
        let shell_state = shell_runtime::ShellRuntimeState::new();

        frame::draw_current_menu(
            &mut canvas,
            &texture_creator,
            &textures,
            model,
            &shell_state,
            -1_i32,
            -1_i32,
            None,
        )?;
        apply_resolution(&mut canvas, second_size.0, second_size.1)?;
        frame::draw_current_menu(
            &mut canvas,
            &texture_creator,
            &textures,
            model,
            &shell_state,
            -1_i32,
            -1_i32,
            None,
        )?;

        let output_size = canvas.output_size().map_err(runtime_error)?;
        let viewport = viewport::menu_viewport_for_output_size(output_size.0, output_size.1);
        canvas.set_scale(1.0_f32, 1.0_f32).map_err(runtime_error)?;
        canvas.set_viewport(None);
        let pixels = canvas
            .read_pixels(None, MENU_TEXTURE_FORMAT)
            .map_err(runtime_error)?;
        Ok(count_black_pixels_in_viewport(
            &pixels,
            output_size.0,
            &viewport,
        ))
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

/// Returns the original dark left/top shell label color used by the renderer.
#[cfg(test)]
pub(in crate::main_menu) const fn shell_left_top_text_color_for_test() -> (u8, u8, u8) {
    text::shell_left_top_text_color_for_test()
}

/// Edits the focused runtime leader-name field through real SDL-style events.
#[cfg(test)]
pub(in crate::main_menu) fn leader_name_after_keydown_edit_for_test() -> String {
    let model = MainMenuModel::ctp2_landing(PathBuf::from("/tmp/ctp2-keydown-test"));
    let mut shell_state = shell_runtime::ShellRuntimeState::new();
    let mut selected_action = None;
    let mut suppressed_keydown_text = None;
    for event in leader_name_edit_events_for_test() {
        let _sound = handle_menu_event(
            &event,
            &model,
            &mut shell_state,
            &mut selected_action,
            (
                super::super::MAIN_MENU_WIDTH,
                super::super::MAIN_MENU_HEIGHT,
            ),
            &mut suppressed_keydown_text,
        );
        assert_eq!(
            selected_action, None,
            "leader-name editing events should not leave the shell"
        );
    }
    shell_state.leader_name_for_test().to_owned()
}

/// Returns the SDL-style event sequence for focusing and editing the leader name.
#[cfg(test)]
fn leader_name_edit_events_for_test() -> Vec<sdl2::event::Event> {
    vec![
        mouse_down_event_for_test(600_i32, 470_i32),
        mouse_down_event_for_test(644_i32, 246_i32),
        mouse_down_event_for_test(300_i32, 173_i32),
        key_down_event_for_test(sdl2::keyboard::Keycode::N, sdl2::keyboard::Mod::LSHIFTMOD),
        text_input_event_for_test("N"),
        key_down_event_for_test(sdl2::keyboard::Keycode::I, sdl2::keyboard::Mod::NOMOD),
        key_down_event_for_test(sdl2::keyboard::Keycode::N, sdl2::keyboard::Mod::NOMOD),
        key_down_event_for_test(sdl2::keyboard::Keycode::J, sdl2::keyboard::Mod::NOMOD),
        key_down_event_for_test(sdl2::keyboard::Keycode::A, sdl2::keyboard::Mod::NOMOD),
        key_down_event_for_test(sdl2::keyboard::Keycode::B, sdl2::keyboard::Mod::NOMOD),
        key_down_event_for_test(sdl2::keyboard::Keycode::O, sdl2::keyboard::Mod::NOMOD),
        key_down_event_for_test(sdl2::keyboard::Keycode::Y, sdl2::keyboard::Mod::NOMOD),
    ]
}

/// Creates a deterministic mouse-down event for runtime text-entry tests.
#[cfg(test)]
const fn mouse_down_event_for_test(x: i32, y: i32) -> sdl2::event::Event {
    sdl2::event::Event::MouseButtonDown {
        timestamp: 0,
        window_id: 0,
        which: 0,
        mouse_btn: sdl2::mouse::MouseButton::Left,
        clicks: 1,
        x,
        y,
    }
}

/// Creates a deterministic keydown event for runtime text-entry tests.
#[cfg(test)]
const fn key_down_event_for_test(
    keycode: sdl2::keyboard::Keycode,
    keymod: sdl2::keyboard::Mod,
) -> sdl2::event::Event {
    sdl2::event::Event::KeyDown {
        timestamp: 0,
        window_id: 0,
        keycode: Some(keycode),
        scancode: None,
        keymod,
        repeat: false,
    }
}

/// Creates a deterministic text-input event for runtime text-entry tests.
#[cfg(test)]
fn text_input_event_for_test(text: &str) -> sdl2::event::Event {
    sdl2::event::Event::TextInput {
        timestamp: 0,
        window_id: 0,
        text: String::from(text),
    }
}

/// Returns the shell control draw order for one screen.
#[cfg(test)]
pub(in crate::main_menu) fn shell_control_draw_order_for_test(
    screen_id: shell::ShellScreenId,
) -> Vec<&'static str> {
    let screen = shell::screen_for_test_defaults(screen_id);
    shell_runtime::control_draw_order_for_test(&screen)
}

/// Returns visible scrollbar arrow overlay rectangles for tests.
#[cfg(test)]
pub(in crate::main_menu) fn listbox_scrollbar_arrow_glyph_plan_for_test(
    listbox_rect: MenuRect,
) -> Vec<(&'static str, MenuRect)> {
    shell_runtime::listbox::arrow_glyph_plan_for_test(listbox_rect)
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

/// Returns the next track after natural completion using original CTP2 music rules.
#[cfg(test)]
pub(in crate::main_menu) fn next_music_track_after_finish_for_test(
    current_track: usize,
    auto_repeat: bool,
    random_order: bool,
    user_selected: bool,
    seed: u64,
) -> Option<usize> {
    audio::next_music_track_after_finish(
        current_track,
        auto_repeat,
        random_order,
        user_selected,
        seed,
    )
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

/// Counts pure black pixels inside a rendered viewport.
#[cfg(test)]
fn count_black_pixels_in_viewport(pixels: &[u8], output_width: u32, viewport: &MenuRect) -> usize {
    let output_width_usize = usize::try_from(output_width).map_or(0_usize, |value| value);
    let viewport_width = usize::try_from(viewport.width).map_or(0_usize, |value| value);
    let viewport_height = usize::try_from(viewport.height).map_or(0_usize, |value| value);
    let viewport_x = usize::try_from(viewport.x.max(0_i32)).map_or(0_usize, |value| value);
    let viewport_y = usize::try_from(viewport.y.max(0_i32)).map_or(0_usize, |value| value);
    let mut black_pixels = 0_usize;

    for y in 0..viewport_height {
        for x in 0..viewport_width {
            let pixel_index = viewport_y
                .saturating_add(y)
                .saturating_mul(output_width_usize)
                .saturating_add(viewport_x.saturating_add(x))
                .saturating_mul(4_usize);
            let Some(pixel) = pixels.get(pixel_index..pixel_index.saturating_add(4_usize)) else {
                continue;
            };
            if pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0 {
                black_pixels = black_pixels.saturating_add(1_usize);
            }
        }
    }

    black_pixels
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
