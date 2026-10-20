//! Native source policy slider/spinner descriptors, shared with accessible input.

use super::{Spec, command, control, spec};
use crate::gameplay::assets::layout::LayoutNode;
use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::MenuRect;
use crate::gameplay::ui_model::management::{ManagementUi, ManagerIntent, ManagerWidget};

/// Recognized source controls consume their atomic children and supply real values.
pub(super) fn append(
    ui: &ManagementUi,
    state: &GameState,
    node: &LayoutNode,
    path: &str,
    rect: MenuRect,
    widgets: &mut Vec<ManagerWidget>,
) -> bool {
    if control(path).is_none() {
        return false;
    }
    let Some(settings) = spec(state, path) else {
        widgets.push(widget(
            format!("{path}/unavailable"),
            "—".to_owned(),
            rect,
            LayoutNode::default(),
            None,
            Some("Current viewer policy facts are unavailable".to_owned()),
        ));
        return true;
    };
    if node
        .field("objecttype")
        .is_some_and(|kind| kind.eq_ignore_ascii_case("C3Slider"))
    {
        slider(ui, state, node, path, rect, settings, widgets);
    } else {
        spinner(state, node, path, rect, settings, widgets);
    }
    true
}

/// Integer source positions use the actual thumb width and full travel distance.
fn position(rect: MenuRect, width: u32, value: i32, settings: Spec) -> u32 {
    let span = u32::try_from(settings.maximum.saturating_sub(settings.minimum)).unwrap_or(0_u32);
    let value = u32::try_from(value.saturating_sub(settings.minimum))
        .unwrap_or(0_u32)
        .min(span);
    if span == 0_u32 {
        return 0_u32;
    }
    rect.width
        .saturating_sub(width)
        .saturating_mul(value)
        .saturating_add(span.div_euclid(2_u32))
        .div_euclid(span)
}

/// Hit boundaries are halfway between actual source thumb centers, not equal boxes.
fn step_rect(rect: MenuRect, width: u32, value: i32, settings: Spec) -> MenuRect {
    let center = position(rect, width, value, settings).saturating_add(width.div_euclid(2_u32));
    let left = if value == settings.minimum {
        0_u32
    } else {
        position(rect, width, value.saturating_sub(1_i32), settings)
            .saturating_add(width.div_euclid(2_u32))
            .saturating_add(center)
            .div_euclid(2_u32)
            .saturating_add(1_u32)
    };
    let right = if value == settings.maximum {
        rect.width
    } else {
        position(rect, width, value.saturating_add(1_i32), settings)
            .saturating_add(width.div_euclid(2_u32))
            .saturating_add(center)
            .div_euclid(2_u32)
            .saturating_add(1_u32)
    };
    MenuRect {
        x: rect
            .x
            .saturating_add(i32::try_from(left).unwrap_or(i32::MAX)),
        y: rect.y,
        width: right.saturating_sub(left),
        height: rect.height,
    }
}

/// Source sliders have five quantized levels and a native 9x20 thumb, not labels.
fn slider(
    ui: &ManagementUi,
    state: &GameState,
    node: &LayoutNode,
    path: &str,
    rect: MenuRect,
    settings: Spec,
    widgets: &mut Vec<ManagerWidget>,
) {
    let mut track = LayoutNode::default();
    track
        .fields
        .insert("c3-slider-track".to_owned(), "true".to_owned());
    track.fields.insert(
        "pattern".to_owned(),
        node.field("pattern").unwrap_or("uptg20e.tga").to_owned(),
    );
    widgets.push(widget(
        format!("{path}/track"),
        String::new(),
        rect,
        track,
        None,
        None,
    ));
    let Some(source) = node.child("thumb") else {
        return;
    };
    let width = source
        .integer("widthpix")
        .and_then(|width| u32::try_from(width).ok())
        .unwrap_or(9_u32)
        .min(rect.width);
    let height = source
        .integer("heightpix")
        .and_then(|height| u32::try_from(height).ok())
        .unwrap_or(rect.height)
        .min(rect.height);
    for value in settings.minimum..=settings.maximum {
        let intent = command(state, path, value)
            .map(|request| ManagerIntent::SetValue(path.to_owned(), value, request.revision));
        let mut style = LayoutNode::default();
        style
            .fields
            .insert("aria-label".to_owned(), format!("Set {path} to {value}"));
        widgets.push(widget(
            format!("{path}/value/{value}"),
            String::new(),
            step_rect(rect, width, value, settings),
            style,
            intent,
            None,
        ));
    }
    let value = ui
        .value_drag
        .as_ref()
        .filter(|drag| {
            drag.path == path
                && state
                    .viewer
                    .as_ref()
                    .is_some_and(|viewer| viewer.revision == drag.revision)
        })
        .map_or(settings.current, |drag| drag.value);
    if !(settings.minimum..=settings.maximum).contains(&value) {
        return;
    }
    let thumb = thumb_style(source, path, value, settings);
    let enabled = command(state, path, settings.minimum).is_some()
        || command(state, path, settings.maximum).is_some();
    widgets.push(widget(
        format!("{path}/thumb"),
        String::new(),
        MenuRect {
            x: rect.x.saturating_add(
                i32::try_from(position(rect, width, value, settings)).unwrap_or(i32::MAX),
            ),
            y: rect.y,
            width,
            height,
        },
        thumb,
        enabled.then(|| ManagerIntent::BeginValue(path.to_owned(), settings.revision)),
        (!enabled).then(|| "No current authority to change this policy".to_owned()),
    ));
}

/// Native source style plus the same range metadata consumed by keyboard/drag input.
fn thumb_style(source: &LayoutNode, path: &str, value: i32, settings: Spec) -> LayoutNode {
    let mut thumb = source.clone();
    thumb.children.clear();
    for (name, setting) in [
        ("value-minimum", settings.minimum.to_string()),
        ("value-maximum", settings.maximum.to_string()),
        ("value-current", value.to_string()),
        ("value-step", settings.step.to_string()),
        ("value-owner", path.to_owned()),
    ] {
        thumb.fields.insert(name.to_owned(), setting);
    }
    thumb
}

/// Original horizontal spinner places arrows at the ends and resizes its display.
fn spinner(
    state: &GameState,
    node: &LayoutNode,
    path: &str,
    rect: MenuRect,
    settings: Spec,
    widgets: &mut Vec<ManagerWidget>,
) {
    let left = node
        .child("autodecx")
        .and_then(|arrow| arrow.integer("widthpix"))
        .and_then(|width| u32::try_from(width).ok())
        .unwrap_or(24_u32)
        .min(rect.width);
    let right = node
        .child("autoincx")
        .and_then(|arrow| arrow.integer("widthpix"))
        .and_then(|width| u32::try_from(width).ok())
        .unwrap_or(24_u32)
        .min(rect.width.saturating_sub(left));
    if let Some(display) = node.child("autodisplay") {
        let mut style = display.clone();
        style.children.clear();
        widgets.push(widget(
            format!("{path}/display"),
            settings.current.to_string(),
            MenuRect {
                x: rect
                    .x
                    .saturating_add(i32::try_from(left).unwrap_or(i32::MAX)),
                y: rect.y,
                width: rect.width.saturating_sub(left).saturating_sub(right),
                height: rect.height,
            },
            style,
            None,
            None,
        ));
    }
    for (suffix, source, width, x, value) in [
        (
            "decrease",
            "autodecx",
            left,
            rect.x,
            settings
                .current
                .saturating_sub(settings.step)
                .max(settings.minimum),
        ),
        (
            "increase",
            "autoincx",
            right,
            rect.x.saturating_add(
                i32::try_from(rect.width.saturating_sub(right)).unwrap_or(i32::MAX),
            ),
            settings
                .current
                .saturating_add(settings.step)
                .min(settings.maximum),
        ),
    ] {
        let Some(arrow) = node.child(source) else {
            continue;
        };
        let mut style = arrow.clone();
        style.children.clear();
        let intent = command(state, path, value)
            .map(|request| ManagerIntent::SetValue(path.to_owned(), value, request.revision));
        let unavailable = intent
            .is_none()
            .then(|| "Current policy boundary or unavailable authority".to_owned());
        widgets.push(widget(
            format!("{path}/{suffix}"),
            String::new(),
            MenuRect {
                x,
                y: rect.y,
                width,
                height: rect.height,
            },
            style,
            intent,
            unavailable,
        ));
    }
}

/// All geometry/identities are shared with native pointers and semantic activation.
const fn widget(
    id: String,
    label: String,
    rect: MenuRect,
    node: LayoutNode,
    intent: Option<ManagerIntent>,
    unavailable: Option<String>,
) -> ManagerWidget {
    ManagerWidget {
        id,
        label,
        rect,
        node,
        intent,
        unavailable,
        selected: false,
    }
}
