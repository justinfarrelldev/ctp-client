//! Original CTP2 text-field style metadata used by shell controls.

/// RGB color stored as plain bytes for renderer-independent style tests.
pub(in crate::main_menu) type ShellTextFieldColor = (u8, u8, u8);

/// Original CTP2 text-field rendering metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::main_menu) struct ShellTextFieldStyle {
    /// Bevel width read from the LDL block or CTP2 text-field default.
    bevel_width: u32,
    /// Horizontal text inset inside the editable region.
    text_inset_x: i32,
    /// Original Windows edit-control background color.
    background_color: ShellTextFieldColor,
    /// Original Windows edit-control text color.
    text_color: ShellTextFieldColor,
    /// Percent scale that maps the shared shell font to the smaller edit-control font.
    font_scale_percent: u32,
}

impl ShellTextFieldStyle {
    /// Returns the `FIELD` style used by `spnewgame.ldl` on the main setup screen.
    pub(in crate::main_menu) const fn new_game_field() -> Self {
        Self {
            bevel_width: 1,
            text_inset_x: 4,
            background_color: (255, 255, 255),
            text_color: (20, 20, 20),
            font_scale_percent: 80,
        }
    }

    /// Returns the default `ctp2_TextField` style used by popup leader fields.
    pub(in crate::main_menu) const fn popup_default() -> Self {
        Self {
            bevel_width: 2,
            text_inset_x: 4,
            background_color: (255, 255, 255),
            text_color: (20, 20, 20),
            font_scale_percent: 80,
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

    /// Returns the field background color.
    pub(in crate::main_menu) const fn background_color(self) -> ShellTextFieldColor {
        self.background_color
    }

    /// Returns the editable text color.
    pub(in crate::main_menu) const fn text_color(self) -> ShellTextFieldColor {
        self.text_color
    }

    /// Returns the edit-control font scale as a percentage of the shell font.
    pub(in crate::main_menu) const fn font_scale_percent(self) -> u32 {
        self.font_scale_percent
    }
}
