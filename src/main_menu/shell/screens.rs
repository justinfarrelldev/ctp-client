//! Original CTP2 shell screen builders.

use alloc::vec::Vec;

use super::builders::asset;
use super::options_state::ShellOptionsState;
use super::pickers;
use super::{GameSetupState, MenuRect, ShellScreen, ShellScreenId};

/// Builds a screen using current setup/option state.
pub(super) fn screen_for(
    id: ShellScreenId,
    setup: &GameSetupState,
    options: &ShellOptionsState,
) -> ShellScreen {
    match id {
        ShellScreenId::Landing => landing_shell_placeholder(),
        ShellScreenId::SinglePlayerHub => super::hub::single_player_hub_screen(),
        ShellScreenId::NewGame => super::new_game::new_game_screen(setup),
        ShellScreenId::Options => super::options::options_screen(),
        ShellScreenId::Gameplay => super::gameplay::gameplay_screen(options),
        ShellScreenId::Graphics => super::options::graphics_screen(options),
        ShellScreenId::Resolution => pickers::resolution_screen(options),
        ShellScreenId::Sound => super::options::sound_screen(options),
        ShellScreenId::Music => super::options::music_screen(options),
        ShellScreenId::MusicTrack => pickers::music_track_screen(options),
        ShellScreenId::Keyboard => super::keyboard::keyboard_screen(options),
        ShellScreenId::NationPicker => pickers::nation_picker_screen(setup),
        ShellScreenId::DifficultyPicker => pickers::difficulty_picker_screen(setup),
        ShellScreenId::MapSizePicker => pickers::map_size_picker_screen(setup),
        ShellScreenId::WorldShapePicker => pickers::world_shape_picker_screen(setup),
        ShellScreenId::PlayersPicker => pickers::players_picker_screen(setup),
        ShellScreenId::RulesPicker => pickers::rules_picker_screen(setup),
        ShellScreenId::CustomMap => super::custom_map::custom_map_screen(setup),
    }
}

/// Builds a placeholder for landing when queried through shell APIs.
const fn landing_shell_placeholder() -> ShellScreen {
    ShellScreen {
        id: ShellScreenId::Landing,
        ldl_name: "InitPlayWindow",
        rect: MenuRect::new(0, 0, 800, 600),
        background: Some(asset("upsg001.rim")),
        pattern: None,
        controls: Vec::new(),
    }
}
