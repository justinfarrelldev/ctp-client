//! Original CTP2 text-field style metadata used by shell controls.

/// Original CTP2 text-field rendering metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::main_menu) struct ShellTextFieldStyle {
    /// Bevel width read from the LDL block or CTP2 text-field default.
    bevel_width: u32,
    /// Horizontal text inset inside the editable region.
    text_inset_x: i32,
}

impl ShellTextFieldStyle {
    /// Returns the `FIELD` style used by `spnewgame.ldl` on the main setup screen.
    pub(in crate::main_menu) const fn new_game_field() -> Self {
        Self {
            bevel_width: 1,
            text_inset_x: 4,
        }
    }

    /// Returns the default `ctp2_TextField` style used by popup leader fields.
    pub(in crate::main_menu) const fn popup_default() -> Self {
        Self {
            bevel_width: 2,
            text_inset_x: 4,
        }
    }

    /// Returns the original bevel width.
    pub(in crate::main_menu) const fn bevel_width(self) -> u32 {
        self.bevel_width
    }

    /// Returns the horizontal text inset for left-aligned field text.
    pub(in crate::main_menu) const fn text_inset_x(self) -> i32 {
        self.text_inset_x
    }
}
