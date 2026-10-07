//! Helpers for constructing original CTP2 shell controls.

use alloc::string::String;

use super::{
    MenuAssetReference, MenuRect, SHELL_PICTURES_ARCHIVE, ShellAction, ShellControl,
    ShellControlKind, ShellImageSet, ShellOptionField, ShellSoundPolicy,
};
use crate::main_menu::ButtonVisualStyle;

/// Creates a shell picture-archive asset reference.
pub(super) const fn asset(entry_name: &'static str) -> MenuAssetReference {
    MenuAssetReference {
        archive_relative_path: SHELL_PICTURES_ARCHIVE,
        entry_name,
    }
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
        action: Some(action),
        sound: Some(ShellSoundPolicy::original_button()),
        source,
    }
}

/// Creates a standard `c3_PopupWindow::AddClose` button.
pub(super) fn popup_close_button(screen_width: u32, screen_height: u32) -> ShellControl {
    let x = i32::try_from(screen_width.saturating_sub(87_u32)).map_or(0_i32, |value| value);
    let y = i32::try_from(screen_height.saturating_sub(41_u32)).map_or(0_i32, |value| value);
    button(
        "c3_PopupClose",
        "Close",
        "Close",
        x,
        y,
        70,
        24,
        ButtonVisualStyle::ctp2_small(),
        ShellAction::Back,
        "c3_popupwindow.cpp:AddClose",
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

/// Creates original CTP2 listbox chrome.
pub(super) fn listbox_chrome(
    ldl_name: &'static str,
    accessible_name: &str,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    source: &'static str,
) -> ShellControl {
    ShellControl {
        ldl_name,
        label: String::new(),
        accessible_name: accessible_name.to_owned(),
        rect: MenuRect::new(x, y, width, height),
        kind: ShellControlKind::ListBoxChrome,
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
        selected: false,
    };
    control
}

/// Creates a static label.
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
    ShellControl {
        ldl_name,
        label: label_text.to_owned(),
        accessible_name: accessible_name.to_owned(),
        rect: MenuRect::new(x, y, width, height),
        kind: ShellControlKind::Label,
        action: None,
        sound: None,
        source,
    }
}

/// Creates a text field.
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
    control.kind = ShellControlKind::TextField;
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
    ShellControl {
        ldl_name,
        label: label_text.to_owned(),
        accessible_name: accessible_name.to_owned(),
        rect: MenuRect::new(x, y, width, height),
        kind: ShellControlKind::Slider { min, max, value },
        action: Some(ShellAction::OpenOption(field)),
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
        action: Some(action),
        sound: Some(ShellSoundPolicy::original_button()),
        source,
    }
}
