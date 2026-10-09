//! Original CTP2 text-field style metadata used by shell controls.

use crate::main_menu::MenuRect;

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
    /// Native edit-control text height used by the original resize path.
    native_text_height: u32,
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
            native_text_height: 16,
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
            native_text_height: 16,
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

    /// Returns the native edit-control rectangle after the original height resize.
    pub(in crate::main_menu) const fn edit_rect(self, ldl_rect: MenuRect) -> MenuRect {
        MenuRect::new(
            ldl_rect.x,
            ldl_rect.y,
            ldl_rect.width,
            self.edit_height(ldl_rect.height),
        )
    }

    /// Returns the complete visible text-field rectangle including the original bevel.
    pub(in crate::main_menu) const fn bevel_rect(self, ldl_rect: MenuRect) -> MenuRect {
        let edit_rect = self.edit_rect(ldl_rect);
        MenuRect::new(
            edit_rect.x.saturating_sub(2_i32),
            edit_rect.y.saturating_sub(2_i32),
            edit_rect.width.saturating_add(4_u32),
            edit_rect.height.saturating_add(4_u32),
        )
    }

    /// Returns the interactive rectangle accepted by the original shell field.
    pub(in crate::main_menu) fn interaction_rect(self, ldl_rect: MenuRect) -> MenuRect {
        union_rect(self.bevel_rect(ldl_rect), ldl_rect)
    }

    /// Returns the height produced by `aui_TextField::Resize` for one-line fields.
    const fn edit_height(self, requested_height: u32) -> u32 {
        if self.native_text_height == 0_u32 {
            return requested_height;
        }
        let remainder = match requested_height.checked_rem(self.native_text_height) {
            Some(value) => value,
            None => 0_u32,
        };
        let resized = requested_height.saturating_sub(remainder);
        if resized > 0_u32 {
            resized
        } else {
            self.native_text_height
        }
    }
}

/// Returns the smallest rectangle containing both input rectangles.
fn union_rect(first: MenuRect, second: MenuRect) -> MenuRect {
    let left = first.x.min(second.x);
    let top = first.y.min(second.y);
    let right = rect_right(first).max(rect_right(second));
    let bottom = rect_bottom(first).max(rect_bottom(second));
    MenuRect::new(
        left,
        top,
        u32::try_from(right.saturating_sub(left)).map_or(u32::MAX, |value| value),
        u32::try_from(bottom.saturating_sub(top)).map_or(u32::MAX, |value| value),
    )
}

/// Returns a rectangle's right edge.
fn rect_right(rect: MenuRect) -> i32 {
    rect.x
        .saturating_add(i32::try_from(rect.width).map_or(i32::MAX, |value| value))
}

/// Returns a rectangle's bottom edge.
fn rect_bottom(rect: MenuRect) -> i32 {
    rect.y
        .saturating_add(i32::try_from(rect.height).map_or(i32::MAX, |value| value))
}
