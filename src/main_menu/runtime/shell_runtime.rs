//! SDL rendering and interaction for CTP2 shell screens below the landing page.

use sdl2::rect::Rect as SdlRect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

#[cfg(not(test))]
use crate::config::AppConfig;
use crate::config::ctp2_options::Ctp2Options;
use crate::main_menu::shell::setup::CustomMapField;
use crate::main_menu::shell::{
    self, ShellAction, ShellControlKind, ShellFlowOutcome, ShellOptionField, ShellScreen,
    ShellScreenId, SinglePlayerShellModel,
};
use crate::main_menu::{MainMenuAction, MainMenuError, MainMenuModel, MenuRect};

use super::frame::to_sdl_rect;
use super::text::draw_centered_text;
use super::{
    BUTTON_CHROMA_KEY, MenuTextures, SHELL_ART_ARCHIVE, create_texture_from_rgba,
    load_rim_rgba_entry, parse_resolution_label, runtime_error,
};

/// Individual shell control drawing helpers.
mod controls;
/// Original listbox chrome drawing helpers.
pub(in crate::main_menu::runtime) mod listbox;
/// Shell screen rendering helpers.
mod render;
/// Immediate shell UI sound types.
pub(super) mod sound;
use sound::{ShellRuntimeInteraction, ShellRuntimeSound};

/// Runtime state for the navigable single-player shell.
pub(super) struct ShellRuntimeState {
    /// Testable shell model.
    model: SinglePlayerShellModel,
    /// Runtime effects queued by shell interactions.
    pending_effects: ShellRuntimeEffects,
    /// Numeric slider currently captured by pointer dragging.
    active_slider: Option<ActiveSlider>,
    /// Scrollable option list currently captured by pointer dragging.
    active_listbox_scroll: Option<ShellOptionField>,
}

/// Captured slider target while the pointer is dragging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveSlider {
    /// Persisted shell option slider.
    Option(ShellOptionField),
    /// Non-persisted custom-map setup slider.
    CustomMap(CustomMapField),
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
            active_listbox_scroll: None,
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

    /// Returns the immediate sound for a click target before action handling.
    pub(super) fn click_sound_at(&self, point_x: i32, point_y: i32) -> Option<ShellRuntimeSound> {
        let screen = current_screen(&self.model);
        let control = screen.find_at_point(point_x, point_y)?;
        if !control.enabled() || !control.has_sound() {
            return None;
        }
        match control.kind() {
            ShellControlKind::Slider { .. } => None,
            ShellControlKind::Button(_)
            | ShellControlKind::Label { .. }
            | ShellControlKind::TextField { .. }
            | ShellControlKind::Switch { .. }
            | ShellControlKind::ListBoxChrome { .. }
            | ShellControlKind::ListItem { .. } => Some(ShellRuntimeSound::ButtonClick),
        }
    }

    /// Handles a click in the current shell screen.
    pub(super) fn click_shell(&mut self, point_x: i32, point_y: i32) -> ShellRuntimeInteraction {
        let screen = current_screen(&self.model);
        if self.handle_listbox_scroll_press(&screen, point_x, point_y) {
            return ShellRuntimeInteraction::none();
        }
        let Some(control) = screen.find_at_point(point_x, point_y) else {
            return ShellRuntimeInteraction::none();
        };
        let Some(action) = control.action() else {
            return ShellRuntimeInteraction::none();
        };
        if let Some(target) = slider_target(action, control.kind()) {
            self.active_slider = Some(target);
            let rect = screen_rect(&screen, control.rect());
            if self.update_slider_value(target, &rect, control.kind(), point_x) {
                return ShellRuntimeInteraction::sound(ShellRuntimeSound::SliderUp);
            }
            return ShellRuntimeInteraction::none();
        }
        self.active_slider = None;
        self.active_listbox_scroll = None;
        match self.model.apply_action(action) {
            ShellFlowOutcome::Continue => {
                self.record_action_effect(action);
                ShellRuntimeInteraction::none()
            }
            ShellFlowOutcome::LaunchGame => {
                ShellRuntimeInteraction::action(MainMenuAction::LaunchSinglePlayer)
            }
            ShellFlowOutcome::Quit => ShellRuntimeInteraction::action(MainMenuAction::Quit),
        }
    }

    /// Handles mouse-wheel scrolling for the active scrollable listbox.
    pub(super) fn scroll_shell(&mut self, delta_y: i32) {
        let Some(field) = scrollable_option_field(self.model.current()) else {
            return;
        };
        self.model
            .options_mut()
            .scroll_option_list(field, delta_y.saturating_neg());
    }

    /// Handles pointer dragging for a captured slider.
    pub(super) fn drag_shell(&mut self, point_x: i32, point_y: i32) -> Option<ShellRuntimeSound> {
        if self.update_active_listbox_scroll(point_y) {
            return None;
        }
        let target = self.active_slider?;
        let screen = current_screen(&self.model);
        let Some(control) = screen.controls().iter().find(|control| {
            control
                .action()
                .is_some_and(|action| slider_target(action, control.kind()) == Some(target))
        }) else {
            self.active_slider = None;
            return None;
        };
        let rect = screen_rect(&screen, control.rect());
        self.update_slider_value(target, &rect, control.kind(), point_x)
            .then_some(ShellRuntimeSound::SliderUp)
    }

    /// Releases any captured pointer control.
    pub(super) fn release_pointer(&mut self) -> Option<ShellRuntimeSound> {
        self.active_listbox_scroll = None;
        self.active_slider
            .take()
            .map(|_field| ShellRuntimeSound::SliderDown)
    }

    /// Applies text input to the shell model.
    pub(super) fn handle_text_input(&mut self, text: &str) {
        self.model.handle_text_input(text);
    }

    /// Applies a text backspace to the shell model.
    pub(super) fn handle_backspace(&mut self) {
        self.model.handle_backspace();
    }

    /// Returns whether a text-entry field owns keyboard focus.
    pub(super) const fn text_input_focused(&self) -> bool {
        self.model.text_input_focused()
    }

    /// Returns the current leader name for runtime tests.
    #[cfg(test)]
    pub(super) fn leader_name_for_test(&self) -> &str {
        self.model.leader_name()
    }

    /// Returns a numeric option for runtime tests.
    #[cfg(test)]
    pub(super) const fn numeric_option_for_test(&self, field: ShellOptionField) -> Option<i32> {
        self.model.options().numeric(field)
    }

    /// Returns a custom-map setup value for runtime tests.
    #[cfg(test)]
    pub(super) const fn custom_map_value_for_test(&self, field: CustomMapField) -> i32 {
        self.model.custom_map_value(field)
    }

    /// Handles a press on a listbox scrollbar and captures thumb dragging.
    fn handle_listbox_scroll_press(
        &mut self,
        screen: &ShellScreen,
        point_x: i32,
        point_y: i32,
    ) -> bool {
        let Some(field) = scrollable_option_field(self.model.current()) else {
            return false;
        };
        let Some((rect, item_count, visible_rows)) = listbox_metrics_at(screen, point_x, point_y)
        else {
            return false;
        };
        self.active_slider = None;
        if let Some(delta) = listbox::scrollbar_delta_at(&rect, point_x, point_y) {
            let outcome = self
                .model
                .apply_action(ShellAction::ScrollOptionList(field, delta));
            debug_assert!(
                matches!(outcome, ShellFlowOutcome::Continue),
                "scrollbar arrow clicks should not leave the shell"
            );
            self.active_listbox_scroll = None;
        } else {
            self.active_listbox_scroll = Some(field);
            self.update_listbox_scroll_offset(field, &rect, item_count, visible_rows, point_y);
        }
        true
    }

    /// Updates a captured listbox scrollbar from pointer motion.
    fn update_active_listbox_scroll(&mut self, point_y: i32) -> bool {
        let Some(field) = self.active_listbox_scroll else {
            return false;
        };
        let screen = current_screen(&self.model);
        let Some((rect, item_count, visible_rows)) = listbox_metrics_for_field(&screen, field)
        else {
            self.active_listbox_scroll = None;
            return false;
        };
        self.update_listbox_scroll_offset(field, &rect, item_count, visible_rows, point_y);
        true
    }

    /// Updates listbox scroll offset from a pointer y coordinate.
    fn update_listbox_scroll_offset(
        &mut self,
        field: ShellOptionField,
        rect: &MenuRect,
        item_count: usize,
        visible_rows: usize,
        point_y: i32,
    ) {
        let offset = listbox::scrollbar_drag_offset_at(rect, point_y, item_count, visible_rows);
        self.model
            .options_mut()
            .set_option_list_scroll_offset(field, offset);
    }

    /// Updates a slider value from a pointer X coordinate.
    fn update_slider_value(
        &mut self,
        target: ActiveSlider,
        rect: &MenuRect,
        kind: ShellControlKind,
        point_x: i32,
    ) -> bool {
        let before = self.slider_value(target);
        let value = slider_value_at(rect, kind, point_x);
        let updated = self.set_slider_value(target, value);
        debug_assert!(updated, "slider controls must target numeric state");
        let after = self.slider_value(target);
        if updated && before != after {
            if let Some(effective_value) = after {
                if let ActiveSlider::Option(field) = target {
                    self.record_numeric_effect(field, effective_value);
                    self.record_persisted_options();
                }
            }
            true
        } else {
            false
        }
    }

    /// Returns the current value for a slider target.
    const fn slider_value(&self, target: ActiveSlider) -> Option<i32> {
        match target {
            ActiveSlider::Option(field) => self.model.options().numeric(field),
            ActiveSlider::CustomMap(field) => Some(self.model.custom_map_value(field)),
        }
    }

    /// Sets one slider target value.
    fn set_slider_value(&mut self, target: ActiveSlider, value: i32) -> bool {
        match target {
            ActiveSlider::Option(field) => self.model.options_mut().set_numeric(field, value),
            ActiveSlider::CustomMap(field) => self.model.set_custom_map_value(field, value),
        }
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
            | ShellAction::AdjustCustomMap(_)
            | ShellAction::SelectGameSetup(_, _)
            | ShellAction::SelectGameSetupInPlace(_, _)
            | ShellAction::SelectOption(_, _)
            | ShellAction::ScrollOptionList(_, _)
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
        render::draw_shell_screen(
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

    /// Draws sharp shell text over the final physically scaled frame.
    pub(super) fn draw_text_overlay(
        &self,
        canvas: &mut Canvas<Window>,
        textures: &MenuTextures<'_>,
        mouse_x: i32,
        mouse_y: i32,
        font: Option<&sdl2::ttf::Font<'_, '_>>,
    ) -> Result<(), MainMenuError> {
        let screen = current_screen(&self.model);
        render::draw_shell_text_overlay(canvas, textures, &screen, mouse_x, mouse_y, font)
    }
}

/// Builds the current shell screen from runtime state.
fn current_screen(state: &SinglePlayerShellModel) -> ShellScreen {
    shell::screen_for(state.current(), state.setup(), state.options())
}

/// Returns listbox metrics when a point is inside one scrollbar.
fn listbox_metrics_at(
    screen: &ShellScreen,
    point_x: i32,
    point_y: i32,
) -> Option<(MenuRect, usize, usize)> {
    screen.controls().iter().find_map(|control| {
        let ShellControlKind::ListBoxChrome {
            item_count,
            visible_rows,
            ..
        } = control.kind()
        else {
            return None;
        };
        let rect = screen_rect(screen, control.rect());
        listbox::scrollbar_contains_point(&rect, point_x, point_y).then_some((
            rect,
            item_count,
            visible_rows,
        ))
    })
}

/// Returns listbox metrics for one scrollable option field.
fn listbox_metrics_for_field(
    screen: &ShellScreen,
    field: ShellOptionField,
) -> Option<(MenuRect, usize, usize)> {
    if scrollable_option_field(screen.id()) != Some(field) {
        return None;
    }
    screen.controls().iter().find_map(|control| {
        let ShellControlKind::ListBoxChrome {
            item_count,
            visible_rows,
            ..
        } = control.kind()
        else {
            return None;
        };
        Some((
            screen_rect(screen, control.rect()),
            item_count,
            visible_rows,
        ))
    })
}

/// Returns the option field controlled by the current scrollable picker.
const fn scrollable_option_field(screen: ShellScreenId) -> Option<ShellOptionField> {
    match screen {
        ShellScreenId::Resolution => Some(ShellOptionField::Resolution),
        ShellScreenId::MusicTrack => Some(ShellOptionField::MusicTrack),
        ShellScreenId::Landing
        | ShellScreenId::SinglePlayerHub
        | ShellScreenId::NewGame
        | ShellScreenId::Options
        | ShellScreenId::Gameplay
        | ShellScreenId::Graphics
        | ShellScreenId::Sound
        | ShellScreenId::Music
        | ShellScreenId::Keyboard
        | ShellScreenId::NationPicker
        | ShellScreenId::DifficultyPicker
        | ShellScreenId::MapSizePicker
        | ShellScreenId::WorldShapePicker
        | ShellScreenId::PlayersPicker
        | ShellScreenId::RulesPicker
        | ShellScreenId::CustomMap => None,
    }
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

/// Returns modeled control drawing order for tests.
#[cfg(test)]
pub(super) fn control_draw_order_for_test(screen: &ShellScreen) -> alloc::vec::Vec<&'static str> {
    render::control_draw_order_for_test(screen)
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

/// Returns the state target controlled by a slider action.
const fn slider_target(action: ShellAction, kind: ShellControlKind) -> Option<ActiveSlider> {
    match (action, kind) {
        (ShellAction::OpenOption(field), ShellControlKind::Slider { .. }) => {
            Some(ActiveSlider::Option(field))
        }
        (ShellAction::AdjustCustomMap(field), ShellControlKind::Slider { .. }) => {
            Some(ActiveSlider::CustomMap(field))
        }
        (
            ShellAction::Navigate(_)
            | ShellAction::OpenPicker(_)
            | ShellAction::SelectGameSetup(_, _)
            | ShellAction::SelectGameSetupInPlace(_, _)
            | ShellAction::SelectOption(_, _)
            | ShellAction::ScrollOptionList(_, _)
            | ShellAction::ToggleOption(_)
            | ShellAction::SelectKeyboardCategory(_)
            | ShellAction::ResetKeyboardDefaults
            | ShellAction::FocusLeaderName
            | ShellAction::LaunchGame
            | ShellAction::Back
            | ShellAction::Quit
            | ShellAction::Disabled
            | ShellAction::OpenOption(_)
            | ShellAction::AdjustCustomMap(_),
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
