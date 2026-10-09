//! Helpers for constructing original CTP2 shell controls.

use alloc::string::String;
use alloc::vec::Vec;

use super::text_field::ShellTextFieldStyle;
use super::{
    MenuAssetReference, MenuRect, SHELL_PICTURES_ARCHIVE, ShellAction, ShellControl,
    ShellControlKind, ShellImageSet, ShellLabelLayout, ShellOptionField, ShellSoundPolicy,
};
use crate::main_menu::ButtonVisualStyle;

/// Creates a shell picture-archive asset reference.
pub(super) const fn asset(entry_name: &'static str) -> MenuAssetReference {
    MenuAssetReference {
        archive_relative_path: SHELL_PICTURES_ARCHIVE,
        entry_name,
    }
}

/// Static metadata for one reusable shell button definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ShellButtonSpec {
    /// Original LDL object name.
    ldl_name: &'static str,
    /// Visible English label text from the original string tables or LDL.
    label_text: &'static str,
    /// Stable accessible name for tests and non-coordinate activation.
    accessible_name: &'static str,
    /// Original LDL rectangle.
    rect: MenuRect,
    /// Original visual button style/template.
    style: ButtonVisualStyle,
    /// Runtime behavior and enabled state.
    behavior: ShellButtonBehavior,
    /// Source reference used to audit the modeled value.
    source: &'static str,
}

impl ShellButtonSpec {
    /// Creates an enabled button spec with the supplied action.
    pub(super) const fn enabled(
        ldl_name: &'static str,
        label_text: &'static str,
        accessible_name: &'static str,
        rect: MenuRect,
        style: ButtonVisualStyle,
        action: ShellAction,
        source: &'static str,
    ) -> Self {
        Self {
            ldl_name,
            label_text,
            accessible_name,
            rect,
            style,
            behavior: ShellButtonBehavior::Enabled(action),
            source,
        }
    }

    /// Creates a disabled button spec that draws with original inactive art.
    pub(super) const fn disabled(
        ldl_name: &'static str,
        label_text: &'static str,
        accessible_name: &'static str,
        rect: MenuRect,
        style: ButtonVisualStyle,
        source: &'static str,
    ) -> Self {
        Self {
            ldl_name,
            label_text,
            accessible_name,
            rect,
            style,
            behavior: ShellButtonBehavior::Disabled,
            source,
        }
    }

    /// Builds a concrete shell control from this reusable spec.
    fn build(&self) -> ShellControl {
        match self.behavior {
            ShellButtonBehavior::Enabled(action) => button(
                self.ldl_name,
                self.label_text,
                self.accessible_name,
                self.rect.x,
                self.rect.y,
                self.rect.width,
                self.rect.height,
                self.style,
                action,
                self.source,
            ),
            ShellButtonBehavior::Disabled => disabled_button(
                self.ldl_name,
                self.label_text,
                self.accessible_name,
                self.rect.x,
                self.rect.y,
                self.rect.width,
                self.rect.height,
                self.style,
                self.source,
            ),
        }
    }
}

/// Enabled/disabled behavior encoded by a reusable button spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShellButtonBehavior {
    /// Active button with an action emitted on activation.
    Enabled(ShellAction),
    /// Inactive button using the original disabled visual state.
    Disabled,
}

/// Appends reusable button specs to an existing control list.
pub(super) fn append_button_specs(controls: &mut Vec<ShellControl>, specs: &[ShellButtonSpec]) {
    controls.extend(specs.iter().map(ShellButtonSpec::build));
}

/// Creates a shell button control.
#[expect(
    clippy::too_many_arguments,
    reason = "Shell builder helpers preserve original LDL control geometry in one call."
)]
pub(super) fn button(
    ldl_name: &'static str,
    label_text: &str,
    accessible_name: &str,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    style: ButtonVisualStyle,
    action: ShellAction,
    source: &'static str,
) -> ShellControl {
    ShellControl {
        ldl_name,
        label: label_text.to_owned(),
        accessible_name: accessible_name.to_owned(),
        rect: MenuRect::new(x, y, width, height),
        kind: ShellControlKind::Button(style.id),
        enabled: true,
        action: Some(action),
        sound: Some(ShellSoundPolicy::original_button()),
        source,
    }
}

/// Creates a shell button that renders in the original disabled state.
#[expect(
    clippy::too_many_arguments,
    reason = "Shell builder helpers preserve original LDL control geometry in one call."
)]
pub(super) fn disabled_button(
    ldl_name: &'static str,
    label_text: &str,
    accessible_name: &str,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    style: ButtonVisualStyle,
    source: &'static str,
) -> ShellControl {
    let mut control = button(
        ldl_name,
        label_text,
        accessible_name,
        x,
        y,
        width,
        height,
        style,
        ShellAction::Disabled,
        source,
    );
    control.enabled = false;
    control
}

/// Creates a standard `c3_PopupWindow::AddClose` button.
pub(super) fn popup_close_button(screen_width: u32, screen_height: u32) -> ShellControl {
    let x = i32::try_from(screen_width.saturating_sub(87_u32)).map_or(0_i32, |value| value);
    let y = i32::try_from(screen_height.saturating_sub(41_u32)).map_or(0_i32, |value| value);
    popup_button(
        "c3_PopupClose",
        "Close",
        "Close",
        x,
        y,
        ShellAction::Back,
        "c3_popupwindow.cpp:AddClose",
    )
}

/// Creates a standard `c3_PopupWindow::AddOk` button.
pub(super) fn popup_ok_button(
    label_text: &str,
    accessible_name: &str,
    screen_width: u32,
    screen_height: u32,
    action: ShellAction,
    source: &'static str,
) -> ShellControl {
    let x = i32::try_from(screen_width.saturating_sub(87_u32)).map_or(0_i32, |value| value);
    let y = i32::try_from(screen_height.saturating_sub(41_u32)).map_or(0_i32, |value| value);
    popup_button(
        "c3_PopupOk",
        label_text,
        accessible_name,
        x,
        y,
        action,
        source,
    )
}

/// Creates a reusable small popup button at an already computed position.
fn popup_button(
    ldl_name: &'static str,
    label_text: &str,
    accessible_name: &str,
    x: i32,
    y: i32,
    action: ShellAction,
    source: &'static str,
) -> ShellControl {
    button(
        ldl_name,
        label_text,
        accessible_name,
        x,
        y,
        70,
        24,
        ButtonVisualStyle::ctp2_small(),
        action,
        source,
    )
}

/// Creates a patterned list-button control.
#[expect(
    clippy::too_many_arguments,
    reason = "Shell builder helpers preserve original LDL control geometry in one call."
)]
pub(super) fn list_button(
    ldl_name: &'static str,
    label_text: &str,
    accessible_name: &str,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    action: ShellAction,
    source: &'static str,
) -> ShellControl {
    let mut control = button(
        ldl_name,
        label_text,
        accessible_name,
        x,
        y,
        width,
        height,
        ButtonVisualStyle::blue_large(),
        action,
        source,
    );
    control.kind = ShellControlKind::ListItem {
        images: None,
        selected: false,
    };
    control
}

/// Creates original CTP2 scrollable listbox chrome.
#[expect(
    clippy::too_many_arguments,
    reason = "Listbox chrome preserves original LDL geometry plus runtime scroll metadata."
)]
pub(super) fn listbox_chrome(
    ldl_name: &'static str,
    accessible_name: &str,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    scroll_offset: usize,
    item_count: usize,
    visible_rows: usize,
    source: &'static str,
) -> ShellControl {
    ShellControl {
        ldl_name,
        label: String::new(),
        accessible_name: accessible_name.to_owned(),
        rect: MenuRect::new(x, y, width, height),
        kind: ShellControlKind::ListBoxChrome {
            scroll_offset,
            item_count,
            visible_rows,
        },
        enabled: true,
        action: None,
        sound: None,
        source,
    }
}

/// Creates a picker list item.
#[expect(
    clippy::too_many_arguments,
    reason = "Shell builder helpers preserve original LDL control geometry in one call."
)]
pub(super) fn list_item(
    ldl_name: &'static str,
    label_text: &str,
    accessible_name: &str,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    selected: bool,
    action: ShellAction,
    source: &'static str,
) -> ShellControl {
    let mut control = list_button(
        ldl_name,
        label_text,
        accessible_name,
        x,
        y,
        width,
        height,
        action,
        source,
    );
    control.kind = ShellControlKind::ListItem {
        images: None,
        selected,
    };
    control
}

/// Creates a static centered label.
#[expect(
    clippy::too_many_arguments,
    reason = "Shell builder helpers preserve original LDL control geometry in one call."
)]
pub(super) fn label(
    ldl_name: &'static str,
    label_text: &str,
    accessible_name: &str,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    source: &'static str,
) -> ShellControl {
    label_with_layout(
        ldl_name,
        label_text,
        accessible_name,
        x,
        y,
        width,
        height,
        ShellLabelLayout::centered(),
        source,
    )
}

/// Creates a centered label with a visual vertical offset.
#[expect(
    clippy::too_many_arguments,
    reason = "Shell builder helpers preserve original LDL control geometry in one call."
)]
pub(super) fn raised_label(
    ldl_name: &'static str,
    label_text: &str,
    accessible_name: &str,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    offset_y: i32,
    source: &'static str,
) -> ShellControl {
    label_with_layout(
        ldl_name,
        label_text,
        accessible_name,
        x,
        y,
        width,
        height,
        ShellLabelLayout::centered_with_offset_y(offset_y),
        source,
    )
}

/// Creates a static label rendered from the left/top LDL anchor.
#[expect(
    clippy::too_many_arguments,
    reason = "Shell builder helpers preserve original LDL control geometry in one call."
)]
pub(super) fn left_label(
    ldl_name: &'static str,
    label_text: &str,
    accessible_name: &str,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    source: &'static str,
) -> ShellControl {
    label_with_layout(
        ldl_name,
        label_text,
        accessible_name,
        x,
        y,
        width,
        height,
        ShellLabelLayout::left_top(),
        source,
    )
}

/// Creates a static label using explicit text-layout metadata.
#[expect(
    clippy::too_many_arguments,
    reason = "Shell builder helpers preserve original LDL control geometry in one call."
)]
fn label_with_layout(
    ldl_name: &'static str,
    label_text: &str,
    accessible_name: &str,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    layout: ShellLabelLayout,
    source: &'static str,
) -> ShellControl {
    ShellControl {
        ldl_name,
        label: label_text.to_owned(),
        accessible_name: accessible_name.to_owned(),
        rect: MenuRect::new(x, y, width, height),
        kind: ShellControlKind::Label { layout },
        enabled: true,
        action: None,
        sound: None,
        source,
    }
}

/// Creates a text field with the main setup screen `FIELD` styling.
#[expect(
    clippy::too_many_arguments,
    reason = "Shell builder helpers preserve original LDL control geometry in one call."
)]
pub(super) fn field(
    ldl_name: &'static str,
    label_text: &str,
    accessible_name: &str,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    focused: bool,
    source: &'static str,
) -> ShellControl {
    field_with_style(
        ldl_name,
        label_text,
        accessible_name,
        x,
        y,
        width,
        height,
        ShellTextFieldStyle::new_game_field(),
        focused,
        source,
    )
}

/// Creates a text field with explicit original CTP2 style metadata.
#[expect(
    clippy::too_many_arguments,
    reason = "Shell builder helpers preserve original LDL control geometry in one call."
)]
pub(super) fn field_with_style(
    ldl_name: &'static str,
    label_text: &str,
    accessible_name: &str,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    style: ShellTextFieldStyle,
    focused: bool,
    source: &'static str,
) -> ShellControl {
    let mut control = label(
        ldl_name,
        label_text,
        accessible_name,
        x,
        y,
        width,
        height,
        source,
    );
    control.kind = ShellControlKind::TextField { style, focused };
    control.action = Some(ShellAction::FocusLeaderName);
    control.sound = Some(ShellSoundPolicy::original_button());
    control
}

/// Creates a slider control.
#[expect(
    clippy::too_many_arguments,
    reason = "Shell builder helpers preserve original LDL control geometry in one call."
)]
pub(super) fn slider(
    ldl_name: &'static str,
    label_text: &str,
    accessible_name: &str,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    min: i32,
    max: i32,
    value: i32,
    field: ShellOptionField,
    source: &'static str,
) -> ShellControl {
    slider_with_action(
        ldl_name,
        label_text,
        accessible_name,
        x,
        y,
        width,
        height,
        min,
        max,
        value,
        ShellAction::OpenOption(field),
        source,
    )
}

/// Creates a slider control with an explicit shell action.
#[expect(
    clippy::too_many_arguments,
    reason = "Shell builder helpers preserve original LDL control geometry in one call."
)]
pub(super) fn slider_with_action(
    ldl_name: &'static str,
    label_text: &str,
    accessible_name: &str,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    min: i32,
    max: i32,
    value: i32,
    action: ShellAction,
    source: &'static str,
) -> ShellControl {
    ShellControl {
        ldl_name,
        label: label_text.to_owned(),
        accessible_name: accessible_name.to_owned(),
        rect: MenuRect::new(x, y, width, height),
        kind: ShellControlKind::Slider { min, max, value },
        enabled: true,
        action: Some(action),
        sound: Some(ShellSoundPolicy::original_button()),
        source,
    }
}

/// Creates a switch/radio control with an original image family.
#[expect(
    clippy::too_many_arguments,
    reason = "Shell builder helpers preserve original LDL control geometry in one call."
)]
pub(super) fn image_switch_control(
    ldl_name: &'static str,
    label_text: &str,
    accessible_name: &str,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    selected: bool,
    images: ShellImageSet,
    action: ShellAction,
    source: &'static str,
) -> ShellControl {
    ShellControl {
        ldl_name,
        label: label_text.to_owned(),
        accessible_name: accessible_name.to_owned(),
        rect: MenuRect::new(x, y, width, height),
        kind: ShellControlKind::Switch {
            enabled: selected,
            images: Some(images),
        },
        enabled: !matches!(action, ShellAction::Disabled),
        action: Some(action),
        sound: Some(ShellSoundPolicy::original_button()),
        source,
    }
}
