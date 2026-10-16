//! Render once at native LDL resolution, then scale the complete framebuffer.
//!
//! Scaling each SDL draw independently drops thin glyph strokes and causes
//! one-pixel gaps at fractional output sizes. A reusable native render target
//! keeps terrain, text, reticles, chrome and hit-test coordinates in agreement.

use super::{
    Camera, MovementInputState, configure_gameplay_canvas, render, textures::GameplayTextures,
};
use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::InGameUiModel;
use crate::gameplay::{GameplayError, runtime_error};
use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture};
use sdl2::ttf::Font;
use sdl2::video::Window;

/// Composes and uniformly scales one frame without presenting or writing files.
#[expect(
    clippy::too_many_arguments,
    reason = "Borrowed SDL resources and immutable scene state have independent owners"
)]
pub(super) fn draw_scaled_frame(
    canvas: &mut Canvas<Window>,
    frame: &mut Texture<'_>,
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
    ui: &InGameUiModel,
    camera: &Camera,
    input: &MovementInputState,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    let mut result = Ok(());
    canvas
        .with_texture_canvas(frame, |target| {
            result = configure_gameplay_canvas(target).and_then(|()| {
                render::draw_frame(target, textures, state, ui, camera, input, font)
            });
        })
        .map_err(|error| runtime_error(error.to_string()))?;
    result?;
    configure_gameplay_canvas(canvas)?;
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    // SAFETY: `frame` owns a live SDL texture on this renderer's thread. This
    // call only sets its sampling mode; SDL retains no borrowed Rust pointer.
    // rust-sdl2 0.37 does not yet wrap the SDL 2.0.12 texture filtering API.
    let status = unsafe {
        sdl2::sys::SDL_SetTextureScaleMode(
            frame.raw(),
            sdl2::sys::SDL_ScaleMode::SDL_ScaleModeLinear,
        )
    };
    if status != 0_i32 {
        return Err(runtime_error(sdl2::get_error()));
    }
    canvas.copy(frame, None, None).map_err(runtime_error)
}
