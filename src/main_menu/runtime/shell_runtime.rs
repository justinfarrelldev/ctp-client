//! SDL rendering and interaction for CTP2 shell screens below the landing page.

use sdl2::pixels::Color;
use sdl2::rect::Rect as SdlRect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

#[cfg(not(test))]
use crate::config::AppConfig;
use crate::config::ctp2_options::Ctp2Options;
use crate::main_menu::shell::{
    self, ShellAction, ShellControl, ShellControlKind, ShellFlowOutcome, ShellImageSet,
    ShellOptionField, ShellScreen, ShellScreenId, SinglePlayerShellModel,
};
use crate::main_menu::{
    MainMenuAction, MainMenuError, MainMenuModel, MenuAssetReference, MenuRect,
};

use super::{
    BUTTON_CHROMA_KEY, MenuTextures, SHELL_ART_ARCHIVE, button_art, create_texture_from_rgba,
    draw_centered_text, draw_fallback_background, draw_fallback_button, load_rim_rgba,
    load_rim_rgba_entry, parse_resolution_label, runtime_error, textures_for_style, to_sdl_rect,
};

/// Runtime state for the navigable single-player shell.
pub(super) struct ShellRuntimeState {
    /// Testable shell model.
    model: SinglePlayerShellModel,
    /// Runtime effects queued by shell interactions.
    pending_effects: ShellRuntimeEffects,
    /// Numeric slider currently captured by pointer dragging.
    active_slider: Option<ShellOptionField>,
}

/// SDL/audio effects requested by shell controls.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct ShellRuntimeEffects {
    /// Requested window resolution.
    pub(super) resolution: Option<(u32, u32)>,
    /// Requested one-based music track number.
    pub(super) music_track: Option<usize>,
    /// Requested music enabled state.
    pub(super) music_enabled: Option<bool>,
    /// Requested music volume on the original 0-10 scale.
    pub(super) music_volume: Option<i32>,
    /// Requested SFX volume on the original 0-10 scale.
    pub(super) sfx_volume: Option<i32>,
    /// Requested voice volume on the original 0-10 scale.
    pub(super) voice_volume: Option<i32>,
    /// Requested mouse speed on the original 0-15 scale.
    pub(super) mouse_speed: Option<i32>,
    /// Requested music auto-repeat state.
    pub(super) auto_repeat: Option<bool>,
    /// Requested music random-order state.
    pub(super) random_order: Option<bool>,
    /// Updated options that should be persisted.
    pub(super) persisted_options: Option<Ctp2Options>,
}

impl ShellRuntimeState {
    /// Creates shell runtime state at the landing page.
    pub(super) fn new() -> Self {
        Self {
            model: SinglePlayerShellModel::with_persisted_options(&load_persisted_options()),
            pending_effects: ShellRuntimeEffects::default(),
            active_slider: None,
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

    /// Returns whether a click target has the original button sound policy.
    pub(super) fn click_has_sound(&self, point_x: i32, point_y: i32) -> bool {
        let screen = current_screen(&self.model);
        screen
            .find_at_point(point_x, point_y)
            .is_some_and(|control| control.enabled() && control.has_sound())
    }

    /// Handles a click in the current shell screen.
    pub(super) fn click_shell(&mut self, point_x: i32, point_y: i32) -> Option<MainMenuAction> {
        let screen = current_screen(&self.model);
        let control = screen.find_at_point(point_x, point_y)?;
        let action = control.action()?;
        if let Some(field) = slider_field(action, control.kind()) {
            self.active_slider = Some(field);
            let rect = screen_rect(&screen, control.rect());
            self.update_slider_value(field, &rect, control.kind(), point_x);
            return None;
        }
        self.active_slider = None;
        match self.model.apply_action(action) {
            ShellFlowOutcome::Continue => {
                self.record_action_effect(action);
                None
            }
            ShellFlowOutcome::LaunchGame => Some(MainMenuAction::LaunchSinglePlayer),
            ShellFlowOutcome::Quit => Some(MainMenuAction::Quit),
        }
    }

    /// Handles pointer dragging for a captured slider.
    pub(super) fn drag_shell(&mut self, point_x: i32, _point_y: i32) {
        let Some(field) = self.active_slider else {
            return;
        };
        let screen = current_screen(&self.model);
        let Some(control) = screen.controls().iter().find(|control| {
            control
                .action()
                .is_some_and(|action| slider_field(action, control.kind()) == Some(field))
        }) else {
            self.active_slider = None;
            return;
        };
        let rect = screen_rect(&screen, control.rect());
        self.update_slider_value(field, &rect, control.kind(), point_x);
    }

    /// Releases any captured pointer control.
    pub(super) const fn release_pointer(&mut self) {
        self.active_slider = None;
    }

    /// Applies text input to the shell model.
    pub(super) fn handle_text_input(&mut self, text: &str) {
        self.model.handle_text_input(text);
    }

    /// Applies a text backspace to the shell model.
    pub(super) fn handle_backspace(&mut self) {
        self.model.handle_backspace();
    }

    /// Returns a numeric option for runtime tests.
    #[cfg(test)]
    pub(super) const fn numeric_option_for_test(&self, field: ShellOptionField) -> Option<i32> {
        self.model.options().numeric(field)
    }

    /// Updates a slider value from a pointer X coordinate.
    fn update_slider_value(
        &mut self,
        field: ShellOptionField,
        rect: &MenuRect,
        kind: ShellControlKind,
        point_x: i32,
    ) {
        let value = slider_value_at(rect, kind, point_x);
        let updated = self.model.options_mut().set_numeric(field, value);
        debug_assert!(updated, "slider controls must target numeric options");
        self.record_numeric_effect(field, value);
        self.record_persisted_options();
    }

    /// Takes and clears queued runtime effects.
    pub(super) fn take_effects(&mut self) -> ShellRuntimeEffects {
        core::mem::take(&mut self.pending_effects)
    }

    /// Records a numeric option effect.
    const fn record_numeric_effect(&mut self, field: ShellOptionField, value: i32) {
        match field {
            ShellOptionField::SfxVolume => self.pending_effects.sfx_volume = Some(value),
            ShellOptionField::VoiceVolume => self.pending_effects.voice_volume = Some(value),
            ShellOptionField::MusicVolume => self.pending_effects.music_volume = Some(value),
            ShellOptionField::MouseSpeed => self.pending_effects.mouse_speed = Some(value),
            ShellOptionField::UnitSpeed
            | ShellOptionField::UnitAnimation
            | ShellOptionField::AnimatedTradeRoutes
            | ShellOptionField::WonderMovies
            | ShellOptionField::CityNames
            | ShellOptionField::PoliticalBorders
            | ShellOptionField::TradeRoutes
            | ShellOptionField::CityInfluence
            | ShellOptionField::Grid
            | ShellOptionField::Resolution
            | ShellOptionField::MusicEnabled
            | ShellOptionField::AutoRepeat
            | ShellOptionField::RandomOrder
            | ShellOptionField::MusicTrack
            | ShellOptionField::TutorialAdvice
            | ShellOptionField::AutoCycleTurns
            | ShellOptionField::AutoCycleUnits
            | ShellOptionField::BattleView
            | ShellOptionField::EnemyMoves
            | ShellOptionField::AutoCenter
            | ShellOptionField::AutoTabSelect
            | ShellOptionField::ShowAllBattles
            | ShellOptionField::AutoSave
            | ShellOptionField::LeftHandedMouse => {}
        }
    }

    /// Records a non-numeric option effect after the model has been updated.
    fn record_action_effect(&mut self, action: ShellAction) {
        match action {
            ShellAction::SelectOption(ShellOptionField::Resolution, _) => {
                self.pending_effects.resolution =
                    parse_resolution_label(self.model.options().resolution());
                self.record_persisted_options();
            }
            ShellAction::SelectOption(ShellOptionField::MusicTrack, _) => {
                self.pending_effects.music_track = Some(self.model.options().music_track());
                self.record_persisted_options();
            }
            ShellAction::ToggleOption(ShellOptionField::MusicEnabled) => {
                self.pending_effects.music_enabled = Some(self.model.options().music_enabled());
                self.record_persisted_options();
            }
            ShellAction::ToggleOption(ShellOptionField::AutoRepeat) => {
                self.pending_effects.auto_repeat = Some(self.model.options().auto_repeat());
                self.record_persisted_options();
            }
            ShellAction::ToggleOption(ShellOptionField::RandomOrder) => {
                self.pending_effects.random_order = Some(self.model.options().random_order());
                self.record_persisted_options();
            }
            ShellAction::ToggleOption(_)
            | ShellAction::SelectKeyboardCategory(_)
            | ShellAction::ResetKeyboardDefaults => self.record_persisted_options(),
            ShellAction::Navigate(_)
            | ShellAction::OpenPicker(_)
            | ShellAction::OpenOption(_)
            | ShellAction::SelectGameSetup(_, _)
            | ShellAction::SelectGameSetupInPlace(_, _)
            | ShellAction::SelectOption(_, _)
            | ShellAction::FocusLeaderName
            | ShellAction::LaunchGame
            | ShellAction::Back
            | ShellAction::Quit
            | ShellAction::Disabled => {}
        }
    }

    /// Queues the current options for persistence.
    fn record_persisted_options(&mut self) {
        self.pending_effects.persisted_options = Some(self.model.options().to_persisted());
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

/// Loads persisted options for production runtime startup.
fn load_persisted_options() -> Ctp2Options {
    #[cfg(test)]
    {
        Ctp2Options::default()
    }
    #[cfg(not(test))]
    {
        AppConfig::load().map_or_else(
            |error| {
                log::warn!("Could not load CTP2 options; using defaults: {error}");
                Ctp2Options::default()
            },
            |config| config.ctp2_options,
        )
    }
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
        draw_shell_control(
            canvas,
            texture_creator,
            textures,
            model,
            screen,
            control,
            mouse_x,
            mouse_y,
            font,
        )?;
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
#[expect(
    clippy::too_many_arguments,
    reason = "Control rendering needs SDL texture resources and current mouse state."
)]
fn draw_shell_control(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    textures: &MenuTextures<'_>,
    model: &MainMenuModel,
    screen: &ShellScreen,
    control: &ShellControl,
    mouse_x: i32,
    mouse_y: i32,
    font: Option<&sdl2::ttf::Font<'_, '_>>,
) -> Result<(), MainMenuError> {
    let rect = screen_rect(screen, control.rect());
    match control.kind() {
        ShellControlKind::Button(style) => {
            let enabled = control.enabled();
            let hovered = enabled && rect.contains_point(mouse_x, mouse_y);
            if enabled {
                if let Some(button_textures) = textures_for_style(textures, style) {
                    button_art::draw_styled_button(canvas, &rect, button_textures, hovered)?;
                    let text_rect =
                        button_art::styled_button_text_rect(&rect, button_textures, hovered);
                    draw_centered_text(canvas, font, &text_rect, control.label())
                } else {
                    draw_fallback_button(canvas, &rect, style, hovered)?;
                    draw_centered_text(canvas, font, &rect, control.label())
                }
            } else if let Some(button_textures) = textures_for_style(textures, style) {
                button_art::draw_disabled_styled_button(canvas, &rect, button_textures)?;
                let text_rect = button_art::disabled_button_text_rect(&rect, button_textures);
                draw_centered_text(canvas, font, &text_rect, control.label())
            } else {
                draw_disabled_button(canvas, font, &rect, control.label())
            }
        }
        ShellControlKind::Label => draw_centered_text(canvas, font, &rect, control.label()),
        ShellControlKind::TextField => draw_text_field(canvas, font, &rect, control.label()),
        ShellControlKind::ListBoxChrome => draw_listbox_chrome(canvas, &rect),
        ShellControlKind::Slider { min, max, value } => draw_slider(
            canvas,
            texture_creator,
            model,
            font,
            &rect,
            control.label(),
            min,
            max,
            value,
        ),
        ShellControlKind::Switch { enabled, images } => draw_switch(
            canvas,
            texture_creator,
            model,
            font,
            &rect,
            control.label(),
            enabled,
            images,
        ),
        ShellControlKind::ListItem { images, selected } => draw_list_item(
            canvas,
            texture_creator,
            model,
            font,
            &rect,
            control,
            images,
            selected,
        ),
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
            | ShellAction::SelectGameSetupInPlace(_, _)
            | ShellAction::SelectOption(_, _)
            | ShellAction::ToggleOption(_)
            | ShellAction::SelectKeyboardCategory(_)
            | ShellAction::ResetKeyboardDefaults
            | ShellAction::FocusLeaderName
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

/// Draws a disabled button fallback.
fn draw_disabled_button(
    canvas: &mut Canvas<Window>,
    font: Option<&sdl2::ttf::Font<'_, '_>>,
    rect: &MenuRect,
    label: &str,
) -> Result<(), MainMenuError> {
    canvas.set_draw_color(Color::RGB(82, 78, 70));
    canvas.fill_rect(to_sdl_rect(rect)).map_err(runtime_error)?;
    canvas.set_draw_color(Color::RGB(35, 32, 28));
    canvas.draw_rect(to_sdl_rect(rect)).map_err(runtime_error)?;
    draw_centered_text(canvas, font, rect, label)
}

/// Draws CTP2 listbox chrome with a patterned/beveled fallback.
fn draw_listbox_chrome(canvas: &mut Canvas<Window>, rect: &MenuRect) -> Result<(), MainMenuError> {
    canvas.set_draw_color(Color::RGB(150, 137, 103));
    canvas.fill_rect(to_sdl_rect(rect)).map_err(runtime_error)?;
    canvas.set_draw_color(Color::RGB(50, 42, 31));
    canvas.draw_rect(to_sdl_rect(rect)).map_err(runtime_error)
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
#[expect(
    clippy::too_many_arguments,
    reason = "Slider rendering combines geometry, state, original art, and SDL resources."
)]
fn draw_slider(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    model: &MainMenuModel,
    _font: Option<&sdl2::ttf::Font<'_, '_>>,
    rect: &MenuRect,
    _label: &str,
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
    let thumb_rect = MenuRect::new(rect.x.saturating_add(offset), rect.y, 9, rect.height);
    if let Ok(texture) = create_texture_for_ldl_asset(texture_creator, model, "upba2513.tga") {
        canvas
            .copy(&texture, None, to_sdl_rect(&thumb_rect))
            .map_err(runtime_error)?;
    } else {
        canvas.set_draw_color(Color::RGB(230, 215, 175));
        canvas
            .fill_rect(to_sdl_rect(&thumb_rect))
            .map_err(runtime_error)?;
    }
    Ok(())
}

/// Draws a switch with its label.
#[expect(
    clippy::too_many_arguments,
    reason = "Switch rendering combines state, original art, and SDL resources."
)]
fn draw_switch(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    model: &MainMenuModel,
    font: Option<&sdl2::ttf::Font<'_, '_>>,
    rect: &MenuRect,
    label: &str,
    enabled: bool,
    images: Option<ShellImageSet>,
) -> Result<(), MainMenuError> {
    if let Some(image_set) = images {
        draw_shell_image(canvas, texture_creator, model, rect, image_set, enabled)?;
        return draw_centered_text(canvas, font, rect, label);
    }

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

/// Draws a patterned dropdown/list item.
#[expect(
    clippy::too_many_arguments,
    reason = "List rendering combines optional original art and SDL resources."
)]
fn draw_list_item(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    model: &MainMenuModel,
    font: Option<&sdl2::ttf::Font<'_, '_>>,
    rect: &MenuRect,
    control: &ShellControl,
    images: Option<ShellImageSet>,
    selected: bool,
) -> Result<(), MainMenuError> {
    if let Some(image_set) = images {
        draw_shell_image(canvas, texture_creator, model, rect, image_set, selected)?;
        return draw_centered_text(canvas, font, rect, control.label());
    }

    canvas.set_draw_color(Color::RGB(165, 150, 112));
    canvas.fill_rect(to_sdl_rect(rect)).map_err(runtime_error)?;
    canvas.set_draw_color(Color::RGB(70, 60, 45));
    canvas.draw_rect(to_sdl_rect(rect)).map_err(runtime_error)?;
    draw_centered_text(canvas, font, rect, control.label())
}

/// Draws one original single-image shell control.
fn draw_shell_image(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    model: &MainMenuModel,
    rect: &MenuRect,
    image_set: ShellImageSet,
    selected: bool,
) -> Result<(), MainMenuError> {
    let image_name = image_set.image_for_state(selected, true);
    if let Ok(texture) = create_texture_for_ldl_asset(texture_creator, model, image_name) {
        canvas
            .copy(&texture, None, to_sdl_rect(rect))
            .map_err(runtime_error)
    } else {
        canvas.set_draw_color(if selected {
            Color::RGB(190, 180, 130)
        } else {
            Color::RGB(95, 88, 70)
        });
        canvas.fill_rect(to_sdl_rect(rect)).map_err(runtime_error)
    }
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

/// Creates a texture for one LDL image name from the original shell archive.
fn create_texture_for_ldl_asset<'texture>(
    texture_creator: &'texture TextureCreator<WindowContext>,
    model: &MainMenuModel,
    ldl_asset_name: &str,
) -> Result<Texture<'texture>, MainMenuError> {
    let entry_name = ldl_asset_entry_name(ldl_asset_name);
    let image = load_rim_rgba_entry(
        &model.installation_root,
        SHELL_ART_ARCHIVE,
        &entry_name,
        Some(BUTTON_CHROMA_KEY),
    )?;
    create_texture_from_rgba(texture_creator, &image)
}

/// Converts an LDL shell image name to the packed RIM entry name.
fn ldl_asset_entry_name(ldl_asset_name: &str) -> String {
    let lower_name = ldl_asset_name.to_ascii_lowercase();
    lower_name
        .strip_suffix(".tga")
        .map_or(lower_name.clone(), |stem| {
            let mut entry = String::with_capacity(stem.len().saturating_add(4_usize));
            entry.push_str(stem);
            entry.push_str(".rim");
            entry
        })
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

#[cfg(test)]
#[path = "shell_runtime_tests.rs"]
mod shell_runtime_tests;
