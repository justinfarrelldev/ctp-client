//! SDL rendering and interaction for CTP2 shell screens below the landing page.

use sdl2::pixels::Color;
use sdl2::rect::Rect as SdlRect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

use crate::main_menu::shell::{
    self, ShellAction, ShellControl, ShellControlKind, ShellFlowOutcome, ShellOptionField,
    ShellScreen, ShellScreenId, SinglePlayerShellModel,
};
use crate::main_menu::{
    MainMenuAction, MainMenuError, MainMenuModel, MenuAssetReference, MenuRect,
};

use super::{
    MenuTextures, button_art, create_texture_from_rgba, draw_centered_text,
    draw_fallback_background, draw_fallback_button, load_rim_rgba, runtime_error,
    textures_for_style, to_sdl_rect,
};

/// Runtime state for the navigable single-player shell.
pub(super) struct ShellRuntimeState {
    /// Testable shell model.
    model: SinglePlayerShellModel,
}

impl ShellRuntimeState {
    /// Creates shell runtime state at the landing page.
    pub(super) fn new() -> Self {
        Self {
            model: SinglePlayerShellModel::default(),
        }
    }

    /// Returns whether the landing page is currently active.
    pub(super) fn is_landing(&self) -> bool {
        self.model.current() == ShellScreenId::Landing
    }

    /// Navigates into the single-player hub.
    pub(super) fn open_single_player(&mut self) {
        let outcome = self
            .model
            .apply_action(ShellAction::Navigate(ShellScreenId::SinglePlayerHub));
        debug_assert!(
            matches!(outcome, ShellFlowOutcome::Continue),
            "opening single-player hub should keep the shell active"
        );
    }

    /// Handles a click in the current shell screen.
    pub(super) fn click_shell(&mut self, point_x: i32, point_y: i32) -> Option<MainMenuAction> {
        let screen = current_screen(&self.model);
        let control = screen.find_at_point(point_x, point_y)?;
        let action = control.action()?;
        if let Some(field) = slider_field(action, control.kind()) {
            let rect = screen_rect(&screen, control.rect());
            let value = slider_value_at(&rect, control.kind(), point_x);
            let updated = self.model.options_mut().set_numeric(field, value);
            debug_assert!(updated, "slider controls must target numeric options");
            return None;
        }
        match self.model.apply_action(action) {
            ShellFlowOutcome::Continue => None,
            ShellFlowOutcome::LaunchGame => Some(MainMenuAction::LaunchSinglePlayer),
            ShellFlowOutcome::Quit => Some(MainMenuAction::Quit),
        }
    }

    /// Draws the current shell screen.
    #[expect(
        clippy::too_many_arguments,
        reason = "The renderer keeps SDL objects borrowed separately to avoid hidden ownership."
    )]
    pub(super) fn draw(
        &self,
        canvas: &mut Canvas<Window>,
        texture_creator: &TextureCreator<WindowContext>,
        textures: &MenuTextures<'_>,
        model: &MainMenuModel,
        mouse_x: i32,
        mouse_y: i32,
        font: Option<&sdl2::ttf::Font<'_, '_>>,
    ) -> Result<(), MainMenuError> {
        let screen = current_screen(&self.model);
        draw_shell_screen(
            canvas,
            texture_creator,
            textures,
            model,
            &screen,
            mouse_x,
            mouse_y,
            font,
        )
    }
}

/// Builds the current shell screen from runtime state.
fn current_screen(state: &SinglePlayerShellModel) -> ShellScreen {
    shell::screen_for(state.current(), state.setup(), state.options())
}

/// Draws one modeled CTP2 shell screen.
#[expect(
    clippy::too_many_arguments,
    reason = "The renderer keeps SDL objects borrowed separately to avoid hidden ownership."
)]
fn draw_shell_screen(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    textures: &MenuTextures<'_>,
    model: &MainMenuModel,
    screen: &ShellScreen,
    mouse_x: i32,
    mouse_y: i32,
    font: Option<&sdl2::ttf::Font<'_, '_>>,
) -> Result<(), MainMenuError> {
    draw_shell_background(canvas, texture_creator, model, screen)?;
    draw_shell_pattern(canvas, texture_creator, model, screen)?;

    for control in screen.controls() {
        draw_shell_control(canvas, textures, screen, control, mouse_x, mouse_y, font)?;
    }

    canvas.present();
    Ok(())
}

/// Draws either the screen background or the single-player hub underlay for popups.
fn draw_shell_background(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    model: &MainMenuModel,
    screen: &ShellScreen,
) -> Result<(), MainMenuError> {
    let background = screen.background().unwrap_or(MenuAssetReference {
        archive_relative_path: "ctp2_data/default/graphics/pictures/pic565.zfs",
        entry_name: "upsg002.rim",
    });
    match create_texture_for_asset(texture_creator, model, &background) {
        Ok(texture) => canvas.copy(&texture, None, None).map_err(runtime_error),
        Err(_) => draw_fallback_background(canvas),
    }
}

/// Draws the screen's tiled popup pattern when one exists.
fn draw_shell_pattern(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    model: &MainMenuModel,
    screen: &ShellScreen,
) -> Result<(), MainMenuError> {
    let Some(pattern) = screen.pattern() else {
        return Ok(());
    };
    let destination = screen.rect();
    if let Ok(texture) = create_texture_for_asset(texture_creator, model, &pattern) {
        tile_texture(canvas, &texture, destination)
    } else {
        canvas.set_draw_color(Color::RGB(160, 145, 108));
        canvas
            .fill_rect(to_sdl_rect(&destination))
            .map_err(runtime_error)
    }
}

/// Draws one shell control in screen coordinates.
fn draw_shell_control(
    canvas: &mut Canvas<Window>,
    textures: &MenuTextures<'_>,
    screen: &ShellScreen,
    control: &ShellControl,
    mouse_x: i32,
    mouse_y: i32,
    font: Option<&sdl2::ttf::Font<'_, '_>>,
) -> Result<(), MainMenuError> {
    let rect = screen_rect(screen, control.rect());
    match control.kind() {
        ShellControlKind::Button(style) => {
            let hovered = rect.contains_point(mouse_x, mouse_y);
            if let Some(button_textures) = textures_for_style(textures, style) {
                button_art::draw_styled_button(canvas, &rect, button_textures, hovered)?;
                let text_rect =
                    button_art::styled_button_text_rect(&rect, button_textures, hovered);
                draw_centered_text(canvas, font, &text_rect, control.label())
            } else {
                draw_fallback_button(canvas, &rect, style, hovered)?;
                draw_centered_text(canvas, font, &rect, control.label())
            }
        }
        ShellControlKind::Label => draw_centered_text(canvas, font, &rect, control.label()),
        ShellControlKind::TextField => draw_text_field(canvas, font, &rect, control.label()),
        ShellControlKind::Slider { min, max, value } => {
            draw_slider(canvas, font, &rect, control.label(), min, max, value)
        }
        ShellControlKind::Switch { enabled } => {
            draw_switch(canvas, font, &rect, control.label(), enabled)
        }
        ShellControlKind::Decoration => draw_decoration(canvas, font, &rect, control),
        ShellControlKind::ListItem => draw_list_item(canvas, font, &rect, control),
    }
}

/// Converts a control rectangle to screen-space.
const fn screen_rect(screen: &ShellScreen, rect: MenuRect) -> MenuRect {
    let screen_rect = screen.rect();
    MenuRect::new(
        screen_rect.x.saturating_add(rect.x),
        screen_rect.y.saturating_add(rect.y),
        rect.width,
        rect.height,
    )
}

/// Returns the numeric option controlled by a slider action.
const fn slider_field(action: ShellAction, kind: ShellControlKind) -> Option<ShellOptionField> {
    match (action, kind) {
        (ShellAction::OpenOption(field), ShellControlKind::Slider { .. }) => Some(field),
        (
            ShellAction::Navigate(_)
            | ShellAction::OpenPicker(_)
            | ShellAction::SelectGameSetup(_, _)
            | ShellAction::SelectOption(_, _)
            | ShellAction::ToggleOption(_)
            | ShellAction::LaunchGame
            | ShellAction::Back
            | ShellAction::Quit
            | ShellAction::Disabled
            | ShellAction::OpenOption(_),
            _,
        ) => None,
    }
}

/// Maps a click position to the original slider's integer range.
fn slider_value_at(rect: &MenuRect, kind: ShellControlKind, point_x: i32) -> i32 {
    let ShellControlKind::Slider { min, max, .. } = kind else {
        return 0_i32;
    };
    let width = i32::try_from(rect.width).map_or(1_i32, |value| value.max(1_i32));
    let range = max.saturating_sub(min).max(1_i32);
    let offset = point_x.saturating_sub(rect.x).clamp(0_i32, width);
    let scaled = offset
        .saturating_mul(range)
        .checked_div(width)
        .map_or(0_i32, |value| value);
    min.saturating_add(scaled).clamp(min, max)
}

/// Draws a simple text-entry field.
fn draw_text_field(
    canvas: &mut Canvas<Window>,
    font: Option<&sdl2::ttf::Font<'_, '_>>,
    rect: &MenuRect,
    text: &str,
) -> Result<(), MainMenuError> {
    canvas.set_draw_color(Color::RGB(210, 200, 165));
    canvas.fill_rect(to_sdl_rect(rect)).map_err(runtime_error)?;
    canvas.set_draw_color(Color::RGB(65, 55, 40));
    canvas.draw_rect(to_sdl_rect(rect)).map_err(runtime_error)?;
    draw_centered_text(canvas, font, rect, text)
}

/// Draws a functional slider approximation with original bounds.
fn draw_slider(
    canvas: &mut Canvas<Window>,
    font: Option<&sdl2::ttf::Font<'_, '_>>,
    rect: &MenuRect,
    label: &str,
    min: i32,
    max: i32,
    value: i32,
) -> Result<(), MainMenuError> {
    let half_height = rect.height.checked_div(2_u32).map_or(0_u32, |half| half);
    let track_y = rect
        .y
        .saturating_add(i32::try_from(half_height).map_or(0_i32, |half| half));
    canvas.set_draw_color(Color::RGB(60, 50, 38));
    canvas
        .fill_rect(SdlRect::new(rect.x, track_y, rect.width, 3))
        .map_err(runtime_error)?;
    let range = max.saturating_sub(min).max(1_i32);
    let offset = value
        .saturating_sub(min)
        .saturating_mul(i32::try_from(rect.width).map_or(0_i32, |width| width))
        .checked_div(range)
        .map_or(0_i32, |value| value);
    canvas.set_draw_color(Color::RGB(230, 215, 175));
    canvas
        .fill_rect(SdlRect::new(
            rect.x.saturating_add(offset),
            rect.y,
            9,
            rect.height,
        ))
        .map_err(runtime_error)?;
    draw_centered_text(canvas, font, rect, label)
}

/// Draws a switch with its label.
fn draw_switch(
    canvas: &mut Canvas<Window>,
    font: Option<&sdl2::ttf::Font<'_, '_>>,
    rect: &MenuRect,
    label: &str,
    enabled: bool,
) -> Result<(), MainMenuError> {
    canvas.set_draw_color(if enabled {
        Color::RGB(190, 180, 130)
    } else {
        Color::RGB(95, 88, 70)
    });
    canvas.fill_rect(to_sdl_rect(rect)).map_err(runtime_error)?;
    canvas.set_draw_color(Color::RGB(45, 38, 28));
    canvas.draw_rect(to_sdl_rect(rect)).map_err(runtime_error)?;
    draw_centered_text(canvas, font, rect, label)
}

/// Draws a non-button decoration, promoting actionable decorations to framed controls.
fn draw_decoration(
    canvas: &mut Canvas<Window>,
    font: Option<&sdl2::ttf::Font<'_, '_>>,
    rect: &MenuRect,
    control: &ShellControl,
) -> Result<(), MainMenuError> {
    if control.is_actionable() {
        canvas.set_draw_color(Color::RGB(180, 165, 125));
        canvas.fill_rect(to_sdl_rect(rect)).map_err(runtime_error)?;
        canvas.set_draw_color(Color::RGB(55, 45, 35));
        canvas.draw_rect(to_sdl_rect(rect)).map_err(runtime_error)?;
        return draw_centered_text(canvas, font, rect, control.label());
    }
    Ok(())
}

/// Draws a patterned dropdown/list item.
fn draw_list_item(
    canvas: &mut Canvas<Window>,
    font: Option<&sdl2::ttf::Font<'_, '_>>,
    rect: &MenuRect,
    control: &ShellControl,
) -> Result<(), MainMenuError> {
    canvas.set_draw_color(Color::RGB(165, 150, 112));
    canvas.fill_rect(to_sdl_rect(rect)).map_err(runtime_error)?;
    canvas.set_draw_color(Color::RGB(70, 60, 45));
    canvas.draw_rect(to_sdl_rect(rect)).map_err(runtime_error)?;
    draw_centered_text(canvas, font, rect, control.label())
}

/// Creates a texture for a menu asset.
fn create_texture_for_asset<'texture>(
    texture_creator: &'texture TextureCreator<WindowContext>,
    model: &MainMenuModel,
    asset: &MenuAssetReference,
) -> Result<Texture<'texture>, MainMenuError> {
    let image = load_rim_rgba(&model.installation_root, asset)?;
    create_texture_from_rgba(texture_creator, &image)
}

/// Tiles one texture inside the destination rectangle.
fn tile_texture(
    canvas: &mut Canvas<Window>,
    texture: &Texture<'_>,
    destination: MenuRect,
) -> Result<(), MainMenuError> {
    let query = texture.query();
    if query.width == 0 || query.height == 0 {
        return Ok(());
    }
    let right = destination
        .x
        .saturating_add(i32::try_from(destination.width).map_or(0_i32, |width| width));
    let bottom = destination
        .y
        .saturating_add(i32::try_from(destination.height).map_or(0_i32, |height| height));
    let mut y = destination.y;
    while y < bottom {
        let mut x = destination.x;
        let remaining_y = u32::try_from(bottom.saturating_sub(y)).map_or(0, |value| value);
        let copy_height = remaining_y.min(query.height);
        while x < right {
            let remaining_x = u32::try_from(right.saturating_sub(x)).map_or(0, |value| value);
            let copy_width = remaining_x.min(query.width);
            canvas
                .copy(
                    texture,
                    SdlRect::new(0, 0, copy_width, copy_height),
                    SdlRect::new(x, y, copy_width, copy_height),
                )
                .map_err(runtime_error)?;
            x = x.saturating_add(i32::try_from(copy_width).map_or(0_i32, |value| value));
        }
        y = y.saturating_add(i32::try_from(copy_height).map_or(0_i32, |value| value));
    }
    Ok(())
}
