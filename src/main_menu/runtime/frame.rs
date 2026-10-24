//! Frame composition for the SDL CTP2 main menu runtime.

use sdl2::pixels::Color;
use sdl2::rect::Rect as SdlRect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};

use crate::main_menu::{ButtonStyleId, MainMenuError, MainMenuModel, MenuRect};

use super::text::{draw_centered_text, draw_version_text_at};
use super::viewport::prepare_menu_canvas;
use super::{MenuTextures, button_art, runtime_error, shell_runtime};

/// Draws the current landing or shell screen.
#[expect(
    clippy::too_many_arguments,
    reason = "The menu frame renderer borrows SDL resources explicitly."
)]
pub(super) fn draw_current_menu(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    textures: &MenuTextures<'_>,
    model: &MainMenuModel,
    shell_state: &shell_runtime::ShellRuntimeState,
    mouse_x: i32,
    mouse_y: i32,
    font: Option<&Font<'_, '_>>,
    information:Option<&mut crate::gameplay::runtime::information::Information<'_, '_>>,
) -> Result<(), MainMenuError> {
    let viewport = prepare_menu_canvas(canvas)?;
    if viewport.width == 0 || viewport.height == 0 {
        canvas.present();
        return Ok(());
    }

    let mut frame_texture = texture_creator
        .create_texture_target(
            texture_creator.default_pixel_format(),
            super::super::MAIN_MENU_WIDTH,
            super::super::MAIN_MENU_HEIGHT,
        )
        .map_err(|error| runtime_error(error.to_string()))?;
    draw_menu_art_to_frame(
        canvas,
        &mut frame_texture,
        texture_creator,
        textures,
        model,
        shell_state,
        mouse_x,
        mouse_y,
    )?;
    canvas.set_viewport(None);
    canvas.set_scale(1.0_f32, 1.0_f32).map_err(runtime_error)?;
    canvas
        .copy(&frame_texture, None, to_sdl_rect(&viewport))
        .map_err(runtime_error)?;

    if shell_state.is_landing() {
        draw_menu_text_overlay(canvas, textures, model, mouse_x, mouse_y, font)?;
    } else {
        shell_state.draw_text_overlay(canvas, textures, mouse_x, mouse_y, font)?;
    }
    if let Some(information)=information {information.draw(canvas,to_sdl_rect(&viewport)).map_err(|error|runtime_error(error.to_string()))?;}
    canvas.present();
    Ok(())
}

/// Draws the low-resolution original-art frame into a texture before final scaling.
#[expect(
    clippy::too_many_arguments,
    reason = "The render-target pass must borrow SDL resources and current shell state."
)]
fn draw_menu_art_to_frame(
    canvas: &mut Canvas<Window>,
    frame_texture: &mut Texture<'_>,
    texture_creator: &TextureCreator<WindowContext>,
    textures: &MenuTextures<'_>,
    model: &MainMenuModel,
    shell_state: &shell_runtime::ShellRuntimeState,
    mouse_x: i32,
    mouse_y: i32,
) -> Result<(), MainMenuError> {
    let mut frame_result = Ok(());
    canvas
        .with_texture_canvas(frame_texture, |frame_canvas| {
            frame_result = draw_menu_art_frame(
                frame_canvas,
                texture_creator,
                textures,
                model,
                shell_state,
                mouse_x,
                mouse_y,
            );
        })
        .map_err(|error| runtime_error(error.to_string()))?;
    frame_result
}

/// Draws one complete original-size frame without text into the active render target.
fn draw_menu_art_frame(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    textures: &MenuTextures<'_>,
    model: &MainMenuModel,
    shell_state: &shell_runtime::ShellRuntimeState,
    mouse_x: i32,
    mouse_y: i32,
) -> Result<(), MainMenuError> {
    canvas
        .set_logical_size(0, 0)
        .map_err(|error| runtime_error(error.to_string()))?;
    canvas.set_scale(1.0_f32, 1.0_f32).map_err(runtime_error)?;
    canvas.set_viewport(None);
    if shell_state.is_landing() {
        draw_menu(canvas, textures, model, mouse_x, mouse_y, None)
    } else {
        shell_state.draw(
            canvas,
            texture_creator,
            textures,
            model,
            mouse_x,
            mouse_y,
            None,
        )
    }
}

/// Draws one original-size landing menu frame.
pub(super) fn draw_menu(
    canvas: &mut Canvas<Window>,
    textures: &MenuTextures<'_>,
    model: &MainMenuModel,
    mouse_x: i32,
    mouse_y: i32,
    font: Option<&Font<'_, '_>>,
) -> Result<(), MainMenuError> {
    match textures.background.as_ref() {
        Some(texture) => canvas
            .copy(
                texture,
                None,
                SdlRect::new(
                    0,
                    0,
                    super::super::MAIN_MENU_WIDTH,
                    super::super::MAIN_MENU_HEIGHT,
                ),
            )
            .map_err(runtime_error)?,
        None => draw_fallback_background(canvas)?,
    }

    for button in &model.buttons {
        let geometry = button_render_geometry(button.rect, button.style.id, true, mouse_x, mouse_y);
        let text_rect = if let Some(button_textures) = textures_for_style(textures, button.style.id)
        {
            button_art::draw_styled_button(
                canvas,
                &button.rect,
                button_textures,
                geometry.hovered(),
            )?;
            button_art::styled_button_text_rect(&button.rect, button_textures, geometry.hovered())
        } else {
            draw_fallback_button(
                canvas,
                &geometry.interaction_rect(),
                button.style.id,
                geometry.hovered(),
            )?;
            geometry.interaction_rect()
        };
        draw_centered_text(canvas, font, &text_rect, button.label)?;
    }

    draw_version_text_at(canvas, font, &model.version_rect, model.version_text)
}

/// Draws sharp text labels over the final physically scaled landing frame.
fn draw_menu_text_overlay(
    canvas: &mut Canvas<Window>,
    textures: &MenuTextures<'_>,
    model: &MainMenuModel,
    mouse_x: i32,
    mouse_y: i32,
    font: Option<&Font<'_, '_>>,
) -> Result<(), MainMenuError> {
    for button in &model.buttons {
        let geometry = button_render_geometry(button.rect, button.style.id, true, mouse_x, mouse_y);
        let text_rect = textures_for_style(textures, button.style.id).map_or(
            geometry.interaction_rect(),
            |button_textures| {
                button_art::styled_button_text_rect(
                    &button.rect,
                    button_textures,
                    geometry.hovered(),
                )
            },
        );
        draw_centered_text(canvas, font, &text_rect, button.label)?;
    }
    draw_version_text_at(canvas, font, &model.version_rect, model.version_text)
}

/// Button hit/hover geometry shared by landing and shell renderers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ButtonRenderGeometry {
    /// Rect covered by the rendered art, including native CTP2 button height.
    interaction_rect: MenuRect,
    /// Whether an enabled button is currently hovered by the pointer.
    hovered: bool,
}

impl ButtonRenderGeometry {
    /// Returns the rendered interaction rectangle.
    pub(super) const fn interaction_rect(self) -> MenuRect {
        self.interaction_rect
    }

    /// Returns whether the pointer hovers an enabled button.
    pub(super) const fn hovered(self) -> bool {
        self.hovered
    }
}

/// Computes shared button hover geometry from original CTP2 visual style bounds.
pub(super) fn button_render_geometry(
    rect: MenuRect,
    style: ButtonStyleId,
    enabled: bool,
    mouse_x: i32,
    mouse_y: i32,
) -> ButtonRenderGeometry {
    let interaction_rect = style.visual_style().rendered_rect(rect);
    ButtonRenderGeometry {
        interaction_rect,
        hovered: enabled && interaction_rect.contains_point(mouse_x, mouse_y),
    }
}

/// Draws a simple fallback background when original ZFS assets are unavailable.
pub(super) fn draw_fallback_background(canvas: &mut Canvas<Window>) -> Result<(), MainMenuError> {
    canvas.set_draw_color(Color::RGB(4, 9, 20));
    canvas
        .fill_rect(SdlRect::new(
            0,
            0,
            super::super::MAIN_MENU_WIDTH,
            super::super::MAIN_MENU_HEIGHT,
        ))
        .map_err(runtime_error)?;
    canvas.set_draw_color(Color::RGB(18, 28, 44));
    canvas
        .fill_rect(SdlRect::new(0, 430, super::super::MAIN_MENU_WIDTH, 170))
        .map_err(runtime_error)
}

/// Draws a button using colors chosen to approximate the original asset family.
pub(super) fn draw_fallback_button(
    canvas: &mut Canvas<Window>,
    rect: &MenuRect,
    style: ButtonStyleId,
    hovered: bool,
) -> Result<(), MainMenuError> {
    let color = match (style, hovered) {
        (ButtonStyleId::BlueLarge, false) => Color::RGB(33, 70, 112),
        (ButtonStyleId::BlueLarge, true) => Color::RGB(63, 105, 158),
        (ButtonStyleId::StoneLarge, false) => Color::RGB(89, 82, 70),
        (ButtonStyleId::StoneLarge, true) => Color::RGB(123, 112, 93),
        (ButtonStyleId::Ctp2Small, false) => Color::RGB(42, 78, 116),
        (ButtonStyleId::Ctp2Small, true) => Color::RGB(70, 112, 160),
    };
    let border = match style {
        ButtonStyleId::BlueLarge | ButtonStyleId::Ctp2Small => Color::RGB(211, 181, 85),
        ButtonStyleId::StoneLarge => Color::RGB(188, 174, 138),
    };
    let sdl_rect = to_sdl_rect(rect);

    canvas.set_draw_color(color);
    canvas.fill_rect(sdl_rect).map_err(runtime_error)?;
    canvas.set_draw_color(border);
    canvas.draw_rect(sdl_rect).map_err(runtime_error)
}

/// Converts a menu rectangle into an SDL rectangle.
pub(super) fn to_sdl_rect(rect: &MenuRect) -> SdlRect {
    SdlRect::new(rect.x, rect.y, rect.width, rect.height)
}

/// Selects cached textures for a button style.
pub(super) const fn textures_for_style<'texture>(
    textures: &'texture MenuTextures<'texture>,
    style: ButtonStyleId,
) -> Option<&'texture button_art::ButtonStateTextures<'texture>> {
    match style {
        ButtonStyleId::BlueLarge => textures.blue_large.as_ref(),
        ButtonStyleId::StoneLarge => textures.stone_large.as_ref(),
        ButtonStyleId::Ctp2Small => textures.ctp2_small.as_ref(),
    }
}
