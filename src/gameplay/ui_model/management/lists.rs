//! Source runtime list frame/pane geometry, independent of native rendering.
//!
//! `ctp2_ListBox::DoneInstantiatingThis` grows/moves the frame and offsets its
//! pane back to the declared origin. Borders are presentation, not extra rows.

use super::ManagerWidget;
use crate::gameplay::assets::layout::LayoutNode;
use crate::gameplay::ui_model::MenuRect;

/// True only for the source list control, not text boxes or arbitrary groups.
pub(super) fn is_list(node: &LayoutNode) -> bool {
    node.field("objecttype")
        .is_some_and(|kind| kind.eq_ignore_ascii_case("ctp2_ListBox"))
}

/// Expands the frame using checked/saturating source border dimensions.
pub(super) fn frame(node: &LayoutNode, content: MenuRect) -> MenuRect {
    let horizontal = node.integer("borderwidth").unwrap_or(0_i32).max(0_i32);
    let vertical = node.integer("borderheight").unwrap_or(0_i32).max(0_i32);
    MenuRect {
        x: content.x.saturating_sub(horizontal),
        y: content.y.saturating_sub(vertical),
        width: content.width.saturating_add(
            u32::try_from(horizontal)
                .unwrap_or(0_u32)
                .saturating_mul(2_u32),
        ),
        height: content.height.saturating_add(
            u32::try_from(vertical)
                .unwrap_or(0_u32)
                .saturating_mul(2_u32),
        ),
    }
}

/// Keeps original content bounds exact even if malformed expanded extents saturate.
pub(super) fn stamp(node: &mut LayoutNode, content: MenuRect) {
    for (name, value) in [
        ("list-content-x", content.x.to_string()),
        ("list-content-y", content.y.to_string()),
        ("list-content-width", content.width.to_string()),
        ("list-content-height", content.height.to_string()),
    ] {
        node.fields.insert(name.to_owned(), value);
    }
}

/// Scroll accounting uses the declared pane, never the painted border rectangle.
pub(super) fn content(widget: &ManagerWidget) -> MenuRect {
    let field = |name| widget.node.field(name);
    match (
        field("list-content-x").and_then(|value| value.parse::<i32>().ok()),
        field("list-content-y").and_then(|value| value.parse::<i32>().ok()),
        field("list-content-width").and_then(|value| value.parse::<u32>().ok()),
        field("list-content-height").and_then(|value| value.parse::<u32>().ok()),
    ) {
        (Some(x), Some(y), Some(width), Some(height)) => MenuRect {
            x,
            y,
            width,
            height,
        },
        _ => widget.rect,
    }
}
