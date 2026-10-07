//! Original CTP2 shell screen builders.

use alloc::vec::Vec;

use super::builders::asset;
use super::pickers;
use super::{
    GameSetupField, GameSetupState, MenuRect, ShellOptionsState, ShellScreen, ShellScreenId,
};

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
        ShellScreenId::Graphics => super::options::graphics_screen(options),
        ShellScreenId::Resolution => pickers::resolution_screen(),
        ShellScreenId::Sound => super::options::sound_screen(options),
        ShellScreenId::Music => super::options::music_screen(options),
        ShellScreenId::MusicTrack => pickers::music_track_screen(),
        ShellScreenId::NationPicker => pickers::picker_screen(
            ShellScreenId::NationPicker,
            "SPNewGameTribeScreen",
            "NATION",
            GameSetupField::Nation,
        ),
        ShellScreenId::DifficultyPicker => pickers::picker_screen(
            ShellScreenId::DifficultyPicker,
            "SPNewGameDiffScreen",
            "DIFFICULTY",
            GameSetupField::Difficulty,
        ),
        ShellScreenId::MapSizePicker => pickers::picker_screen(
            ShellScreenId::MapSizePicker,
            "SPNewGameMapSizeScreen",
            "MAP SIZE",
            GameSetupField::MapSize,
        ),
        ShellScreenId::WorldShapePicker => pickers::picker_screen(
            ShellScreenId::WorldShapePicker,
            "SPNewGameMapShapeScreen",
            "WORLD SHAPE",
            GameSetupField::WorldShape,
        ),
        ShellScreenId::WorldTypePicker => pickers::picker_screen(
            ShellScreenId::WorldTypePicker,
            "SPNewGameMapScreen",
            "WORLD TYPE",
            GameSetupField::WorldType,
        ),
        ShellScreenId::PlayersPicker => pickers::picker_screen(
            ShellScreenId::PlayersPicker,
            "SPNewGamePlayersScreen",
            "PLAYERS",
            GameSetupField::Players,
        ),
        ShellScreenId::RulesPicker => pickers::picker_screen(
            ShellScreenId::RulesPicker,
            "SPNewGameRulesScreen",
            "RULES",
            GameSetupField::Rules,
        ),
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
