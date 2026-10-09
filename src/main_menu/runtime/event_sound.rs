//! Immediate menu sound routing for SDL events.

use sdl2::event::Event;

use crate::main_menu::MainMenuModel;

use super::audio::SdlMenuMusicBackend;
use super::shell_runtime;
use super::viewport::output_point_to_menu_point;

/// Plays one immediate UI sound effect.
pub(super) fn play_menu_sound(
    backend: &SdlMenuMusicBackend,
    sound: shell_runtime::sound::ShellRuntimeSound,
) {
    match sound {
        shell_runtime::sound::ShellRuntimeSound::ButtonClick => backend.play_button_click(),
        shell_runtime::sound::ShellRuntimeSound::SliderUp => backend.play_slider_up(),
        shell_runtime::sound::ShellRuntimeSound::SliderDown => backend.play_slider_down(),
    }
}

/// Returns a sound that should begin before a mouse action mutates shell state.
pub(super) fn sound_before_menu_event(
    event: &Event,
    model: &MainMenuModel,
    shell_state: &shell_runtime::ShellRuntimeState,
    output_size: (u32, u32),
) -> Option<shell_runtime::sound::ShellRuntimeSound> {
    let Event::MouseButtonDown { x, y, .. } = event.clone() else {
        return None;
    };
    let (menu_x, menu_y) = output_point_to_menu_point(output_size.0, output_size.1, x, y)?;
    if shell_state.is_landing() {
        model
            .activate_at_point(menu_x, menu_y)
            .is_some()
            .then_some(shell_runtime::sound::ShellRuntimeSound::ButtonClick)
    } else {
        shell_state.click_sound_at(menu_x, menu_y)
    }
}
