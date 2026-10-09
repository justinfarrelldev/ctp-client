//! Immediate UI sound requests emitted by shell interactions.

use crate::main_menu::MainMenuAction;

/// Immediate UI sound requested by shell interactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::main_menu::runtime) enum ShellRuntimeSound {
    /// Original button execute click (`CCLICK1.WAV`).
    ButtonClick,
    /// Original slider movement sound (`CSLIDEUP.WAV`).
    SliderUp,
    /// Original slider release sound (`CSLIDEDN.WAV`).
    SliderDown,
}

/// Result of applying one shell pointer interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::main_menu::runtime) struct ShellRuntimeInteraction {
    /// Optional terminal menu action requested by the shell.
    pub(in crate::main_menu::runtime) action: Option<MainMenuAction>,
    /// Optional immediate UI sound requested by the shell.
    pub(in crate::main_menu::runtime) sound: Option<ShellRuntimeSound>,
}

impl ShellRuntimeInteraction {
    /// Creates an interaction result with no action or sound.
    pub(super) const fn none() -> Self {
        Self {
            action: None,
            sound: None,
        }
    }

    /// Creates an interaction result with a sound effect.
    pub(super) const fn sound(sound: ShellRuntimeSound) -> Self {
        Self {
            action: None,
            sound: Some(sound),
        }
    }

    /// Creates an interaction result with a terminal action.
    pub(super) const fn action(action: MainMenuAction) -> Self {
        Self {
            action: Some(action),
            sound: None,
        }
    }
}
