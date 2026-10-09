//! Gameplay options screen builder.

use super::MenuRect;
use super::builders::{image_switch_control, label, popup_close_button, slider};
use super::options_state::ShellOptionsState;
use super::{ShellAction, ShellImageSet, ShellOptionField, ShellScreen, ShellScreenId};

/// Builds the gameplay options popup.
#[expect(
    clippy::too_many_lines,
    reason = "The gameplay builder is a direct LDL control manifest for validation."
)]
pub(super) fn gameplay_screen(options: &ShellOptionsState) -> ShellScreen {
    ShellScreen {
        id: ShellScreenId::Gameplay,
        ldl_name: "GamePlayOptionsWindow",
        rect: MenuRect::new(120, 175, 560, 250),
        background: None,
        pattern: Some(super::builders::asset("uptg20e.rim")),
        controls: vec![
            label(
                "Name",
                "GAMEPLAY OPTIONS",
                "Gameplay Options Title",
                140,
                20,
                280,
                25,
                "gameplayoptions.ldl:31",
            ),
            gameplay_switch(
                "TutorialButton",
                "Tutorial Advice",
                ShellOptionField::TutorialAdvice,
                30,
                50,
                options,
                "gameplayoptions.ldl:57",
            ),
            gameplay_switch(
                "AutoCycleTurnButton",
                "Auto Cycle Turns",
                ShellOptionField::AutoCycleTurns,
                30,
                80,
                options,
                "gameplayoptions.ldl:65",
            ),
            gameplay_switch(
                "AutoCycleUnitsButton",
                "Auto Cycle Units",
                ShellOptionField::AutoCycleUnits,
                30,
                110,
                options,
                "gameplayoptions.ldl:73",
            ),
            gameplay_switch(
                "AutoCenterButton",
                "Auto Center",
                ShellOptionField::AutoCenter,
                30,
                140,
                options,
                "gameplayoptions.ldl:81",
            ),
            gameplay_switch(
                "AutoTabSelectButton",
                "Auto Tab Select",
                ShellOptionField::AutoTabSelect,
                30,
                170,
                options,
                "gameplayoptions.ldl:89",
            ),
            gameplay_switch(
                "AutoSaveButton",
                "Autosave",
                ShellOptionField::AutoSave,
                280,
                50,
                options,
                "gameplayoptions.ldl:101",
            ),
            gameplay_switch(
                "BattleViewButton",
                "Battle View",
                ShellOptionField::BattleView,
                280,
                80,
                options,
                "gameplayoptions.ldl:109",
            ),
            gameplay_switch(
                "BattleViewAlwaysButton",
                "Show All Battles",
                ShellOptionField::ShowAllBattles,
                280,
                110,
                options,
                "gameplayoptions.ldl:117",
            ),
            gameplay_switch(
                "EnemyMovesButton",
                "Enemy Moves",
                ShellOptionField::EnemyMoves,
                280,
                140,
                options,
                "gameplayoptions.ldl:125",
            ),
            gameplay_switch(
                "LeftHandedMouseButton",
                "Left Handed Mouse",
                ShellOptionField::LeftHandedMouse,
                280,
                170,
                options,
                "gameplayoptions.ldl:133",
            ),
            label(
                "MouseSpeedName",
                "Mouse Speed",
                "Mouse Speed Label",
                93,
                206,
                102,
                20,
                "gameplayoptions.ldl:166",
            ),
            slider(
                "MouseSpeedSlider",
                "Mouse Speed",
                "Mouse Speed",
                196,
                195,
                160,
                38,
                0,
                15,
                options.numeric(ShellOptionField::MouseSpeed).unwrap_or(5),
                ShellOptionField::MouseSpeed,
                "gameplayoptions.ldl:143",
            ),
            popup_close_button(560, 250),
        ],
    }
}

/// Creates one gameplay checkbox with original art and current label state.
fn gameplay_switch(
    ldl_name: &'static str,
    label_stem: &'static str,
    field: ShellOptionField,
    x: i32,
    y: i32,
    options: &ShellOptionsState,
    source: &'static str,
) -> super::ShellControl {
    let enabled = options.bool_option(field);
    let label_text = gameplay_toggle_label(label_stem, enabled);
    image_switch_control(
        ldl_name,
        &label_text,
        label_stem,
        x,
        y,
        230,
        24,
        enabled,
        ShellImageSet::graphics_checkbox(),
        ShellAction::ToggleOption(field),
        source,
    )
}

/// Returns the visible on/off label for a gameplay toggle.
fn gameplay_toggle_label(label_stem: &str, enabled: bool) -> String {
    let suffix = if enabled { "On" } else { "Off" };
    format!("{label_stem} {suffix}")
}
