//! Single-player hub screen builder.

use super::MenuRect;
use super::builders::{asset, button, label};
use super::{ShellAction, ShellScreen, ShellScreenId};
use crate::main_menu::ButtonVisualStyle;

/// Builds the original single-player hub screen.
#[expect(
    clippy::too_many_lines,
    reason = "The hub builder is a direct LDL control manifest for validation."
)]
pub(super) fn single_player_hub_screen() -> ShellScreen {
    ShellScreen {
        id: ShellScreenId::SinglePlayerHub,
        ldl_name: "SPWindow",
        rect: MenuRect::new(0, 0, 800, 600),
        background: Some(asset("upsg002.rim")),
        pattern: None,
        controls: vec![
            button(
                "NewGameButton",
                "NEW GAME",
                "New Game",
                554,
                230,
                180,
                32,
                ButtonVisualStyle::blue_large(),
                ShellAction::Navigate(ShellScreenId::NewGame),
                "spnewgame.ldl:181",
            ),
            button(
                "LoadGameButton",
                "LOAD GAME",
                "Load Game",
                554,
                300,
                180,
                32,
                ButtonVisualStyle::blue_large(),
                ShellAction::Disabled,
                "spnewgame.ldl:174",
            ),
            button(
                "ScenarioButton",
                "SCENARIO",
                "Scenario",
                46,
                134,
                180,
                32,
                ButtonVisualStyle::blue_large(),
                ShellAction::Disabled,
                "spnewgame.ldl:188",
            ),
            button(
                "MapEditorButton",
                "MAP EDITOR",
                "Map Editor",
                46,
                113,
                180,
                32,
                ButtonVisualStyle::blue_large(),
                ShellAction::Disabled,
                "spnewgame.ldl:167",
            ),
            button(
                "OptionsButton",
                "OPTIONS",
                "Options",
                66,
                260,
                180,
                32,
                ButtonVisualStyle::blue_large(),
                ShellAction::Navigate(ShellScreenId::Options),
                "spnewgame.ldl:195",
            ),
            button(
                "TutorialButton",
                "TUTORIAL",
                "Play Tutorial",
                850,
                230,
                180,
                32,
                ButtonVisualStyle::blue_large(),
                ShellAction::Disabled,
                "spnewgame.ldl:160",
            ),
            button(
                "ReturnButton",
                "BACK",
                "Back",
                36,
                558,
                140,
                32,
                ButtonVisualStyle::stone_large(),
                ShellAction::Back,
                "spnewgame.ldl:209",
            ),
            button(
                "QuitButton",
                "QUIT",
                "Quit",
                624,
                558,
                140,
                32,
                ButtonVisualStyle::stone_large(),
                ShellAction::Quit,
                "spnewgame.ldl:202",
            ),
            label(
                "InstaStartButton",
                "INSTANT START",
                "Instant Start",
                760,
                173,
                165,
                28,
                "spnewgame.ldl:216",
            ),
        ],
    }
}
