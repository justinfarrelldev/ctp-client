//! Atomic C3 option-slider artwork; presentation never grants command authority.

use super::ManagerWidget;
use crate::gameplay::assets::layout::LayoutNode;
use crate::gameplay::ui_model::MenuRect;

/// Original options construct `C3Slider` at runtime even without `objecttype` in LDL.
pub(super) fn append(
    node: &LayoutNode,
    path: &str,
    rect: MenuRect,
    widgets: &mut Vec<ManagerWidget>,
) -> bool {
    if node.field("type") != Some("slider") {
        return false;
    }
    let Some(thumb) = node.child("thumb") else {
        return false;
    };
    let mut track = LayoutNode::default();
    track
        .fields
        .insert("c3-slider-track".to_owned(), "true".to_owned());
    track.fields.insert(
        "pattern".to_owned(),
        node.field("pattern").unwrap_or("uptg20e.tga").to_owned(),
    );
    for name in ["ticks", "maxxval", "minthumbsize"] {
        if let Some(value) = node.field(name) {
            track.fields.insert(name.to_owned(), value.to_owned());
        }
    }
    widgets.push(widget(format!("{path}/track"), rect, track));
    let span = u32::try_from(node.integer("maxxval").unwrap_or(0_i32)).unwrap_or(0_u32);
    let value = u32::try_from(node.integer("xval").unwrap_or(0_i32))
        .unwrap_or(0_u32)
        .min(span);
    let width = u32::try_from(thumb.integer("widthpix").unwrap_or(9_i32))
        .unwrap_or(0_u32)
        .min(rect.width);
    let offset = rect
        .width
        .saturating_sub(width)
        .saturating_mul(value)
        .saturating_add(span.div_euclid(2_u32))
        .checked_div(span)
        .unwrap_or(0_u32);
    let mut style = thumb.clone();
    style.children.clear();
    widgets.push(widget(
        format!("{path}/thumb"),
        MenuRect {
            x: rect
                .x
                .saturating_add(i32::try_from(offset).unwrap_or(0_i32)),
            width,
            ..rect
        },
        style,
    ));
    true
}

/// Noninteractive native artwork cannot silently act as an implemented option.
const fn widget(id: String, rect: MenuRect, node: LayoutNode) -> ManagerWidget {
    ManagerWidget {
        id,
        label: String::new(),
        rect,
        node,
        intent: None,
        unavailable: None,
        selected: false,
    }
}
