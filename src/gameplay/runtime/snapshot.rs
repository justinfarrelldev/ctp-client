//! Explicit, repeatable visual previews using the real gameplay SDL renderer.
//!
//! Preview mode never creates a server client. It is not a gameplay launch and
//! does not replace server-authoritative state during an interactive session.
//! Tests can inspect the RGBA result without invoking the separate file sink.

use super::fonts::{ManagementFonts, RenderFonts};
use super::{configure_gameplay_canvas, frame, load_gameplay_font, textures::GameplayTextures};
use crate::gameplay::ui_model::{InGameUiModel, MenuCategory};
use crate::gameplay::{GameplayError, runtime_error};
use sdl2::image::SaveSurface;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Canvas;
use sdl2::rwops::RWops;
use sdl2::surface::Surface;
use sdl2::video::Window;
use std::io::Seek;
use std::path::{Path, PathBuf};

/// Dense full-frame comparison reports; no internal gameplay region is masked.
pub mod comparison;
/// Explicit populated offline data for diagnostic managers, never live gameplay.
mod manager_fixture;
/// Explicit deterministic fixture selection, not live-game simulation.
pub mod scene;
use scene::{PreviewScene, preview_state};

/// Complete-frame (not masked-corner) comparison and malformed-buffer regressions.
#[cfg(test)]
mod comparison_tests;
#[cfg(test)]
mod png_tests;

/// Lossless physical-output framebuffer, including any aspect-ratio borders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameCapture {
    /// Output width in physical pixels.
    pub width: u32,
    /// Output height in physical pixels.
    pub height: u32,
    /// Row-major RGBA32 pixels.
    pub rgba: Vec<u8>,
}

impl FrameCapture {
    /// Encodes a lossless PNG entirely in memory, including in automated tests.
    ///
    /// # Errors
    /// Returns an error for invalid dimensions, byte count, or encoder failure.
    pub fn png_bytes(&mut self) -> Result<Vec<u8>, GameplayError> {
        let pitch = self
            .width
            .checked_mul(4_u32)
            .ok_or_else(|| runtime_error("snapshot pitch overflow".to_owned()))?;
        let required = usize::try_from(pitch)
            .ok()
            .and_then(|value| value.checked_mul(usize::try_from(self.height).ok()?));
        if self.width == 0_u32 || self.height == 0_u32 || required != Some(self.rgba.len()) {
            return Err(runtime_error(
                "invalid snapshot dimensions or byte count".to_owned(),
            ));
        }
        let capacity = self
            .rgba
            .len()
            .checked_add(self.rgba.len().div_euclid(100_usize))
            .and_then(|value| value.checked_add(0x0001_0000_usize))
            .ok_or_else(|| runtime_error("PNG buffer size overflow".to_owned()))?;
        let mut output = vec![0_u8; capacity];
        let length = {
            let surface = Surface::from_data(
                &mut self.rgba,
                self.width,
                self.height,
                pitch,
                PixelFormatEnum::RGBA32,
            )
            .map_err(runtime_error)?;
            let mut stream = RWops::from_bytes_mut(&mut output).map_err(runtime_error)?;
            surface.save_rw(&mut stream).map_err(runtime_error)?;
            usize::try_from(
                stream
                    .stream_position()
                    .map_err(|error| runtime_error(error.to_string()))?,
            )
            .map_err(|error| runtime_error(error.to_string()))?
        };
        output.truncate(length);
        Ok(output)
    }

    /// Writes an explicitly requested PNG; tests exercise `png_bytes`, never I/O.
    ///
    /// # Errors
    /// Returns an error if encoding or the filesystem sink fails.
    pub fn save_png(&mut self, path: &Path) -> Result<(), GameplayError> {
        std::fs::write(path, self.png_bytes()?).map_err(|error| runtime_error(error.to_string()))
    }
}

/// Captures the current backbuffer before `present` can invalidate it.
pub(super) fn capture_canvas(canvas: &Canvas<Window>) -> Result<FrameCapture, GameplayError> {
    let (width, height) = canvas.output_size().map_err(runtime_error)?;
    let rgba = canvas
        .read_pixels(None, PixelFormatEnum::RGBA32)
        .map_err(runtime_error)?;
    Ok(FrameCapture {
        width,
        height,
        rgba,
    })
}

/// Saves an explicit F12 request from the live backbuffer, never from model tests.
pub(super) fn save_interactive_capture(canvas: &Canvas<Window>) -> Result<PathBuf, GameplayError> {
    let path = std::env::var_os("CTP_GAMEPLAY_CAPTURE")
        .map_or_else(|| PathBuf::from("images/gameplay.png"), PathBuf::from);
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| runtime_error(error.to_string()))?;
    }
    capture_canvas(canvas)?.save_png(&path)?;
    Ok(path)
}

/// Renders a deterministic, original-art preview directly into memory.
///
/// No launcher, extraction, intro, user configuration, live server or filesystem
/// writes are involved. `size` is the desired physical screenshot size; 640×480
/// is useful for comparing the supplied reference, and 800×600 shows native art.
///
/// # Errors
/// Returns an error if SDL, installed CTP2 artwork or rendering is unavailable.
pub fn render_preview(
    root: &Path,
    size: (u32, u32),
    menu: Option<MenuCategory>,
) -> Result<FrameCapture, GameplayError> {
    render_preview_scene(root, size, menu, PreviewScene::Generated)
}

/// Renders a specifically selected diagnostic scene using the production renderer.
///
/// # Errors
/// Returns an error for invalid dimensions, unavailable SDL, art or renderer.
pub fn render_preview_scene(
    root: &Path,
    size: (u32, u32),
    menu: Option<MenuCategory>,
    scene: PreviewScene,
) -> Result<FrameCapture, GameplayError> {
    if size.0 == 0_u32 || size.1 == 0_u32 {
        return Err(runtime_error(
            "snapshot dimensions must be positive".to_owned(),
        ));
    }
    let sdl = sdl2::init().map_err(runtime_error)?;
    let video = sdl.video().map_err(runtime_error)?;
    let window = video
        .window("CTP2 gameplay visual preview", size.0, size.1)
        .hidden()
        .build()
        .map_err(|error| runtime_error(error.to_string()))?;
    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|error| runtime_error(error.to_string()))?;
    configure_gameplay_canvas(&mut canvas)?;
    let creator = canvas.texture_creator();
    let mut textures = GameplayTextures::load(root, &creator, true)?;
    let ttf = sdl2::ttf::init().map_err(|error| runtime_error(error.to_string()))?;
    let font = load_gameplay_font(&ttf, Some(root));
    let mut state = preview_state(scene);
    let mut ui = InGameUiModel {
        open_menu: menu,
        ..InGameUiModel::default()
    };
    ui.management = crate::gameplay::ui_model::management::ManagementUi::new(
        crate::gameplay::assets::layout::LayoutBook::load(root)?,
    )?;
    if let Some(font) = font.as_ref() {
        ui.measure_menu_text(|text| font.size_of(text).map_or(0_u32, |metrics| metrics.0));
        ui.management.line_height = u32::try_from(font.height()).unwrap_or(16_u32);
    }
    let source_fonts = ManagementFonts::load(&ttf, &ui.management)?;
    source_fonts.measure(&mut ui.management);
    let (camera, input) = scene::preview_presentation(scene, &mut state, &textures, &mut ui);
    let mut frame_texture = creator
        .create_texture_target(
            None,
            super::DEFAULT_WINDOW_WIDTH,
            super::DEFAULT_WINDOW_HEIGHT,
        )
        .map_err(|error| runtime_error(error.to_string()))?;
    frame::draw_scaled_frame(
        &mut canvas,
        &mut frame_texture,
        &mut textures,
        &state,
        &ui,
        &camera,
        &input,
        RenderFonts {
            body: font.as_ref(),
            management: Some(&source_fonts),
        },
    )?;
    capture_canvas(&canvas)
}
