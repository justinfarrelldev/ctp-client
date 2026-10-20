//! Shared source-layout traversal for painting and semantic/pointer activation.

#![expect(
    clippy::default_numeric_fallback,
    reason = "Original LDL fixed geometry is signed i32; dimensions are checked before conversion to u32"
)]

use super::{ManagementUi, ManagerIntent, ManagerKind, ManagerView, ManagerWidget, bindings, data};
use crate::gameplay::assets::layout::LayoutNode;
use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::MenuRect;

/// Produces only controls visible in the current source panel/mode.
pub(super) fn widgets(ui: &ManagementUi, state: &GameState) -> Vec<ManagerWidget> {
    let Some(view) = ui.view.as_ref() else {
        return Vec::new();
    };
    let Some(root) = ui.layouts.get(&view.kind) else {
        return Vec::new();
    };
    let rect = root_rect(root);
    let mut walker = Walker {
        ui,
        state,
        view,
        root_rect: rect,
        widgets: Vec::new(),
    };
    walker.walk(root, "", rect);
    if matches!(view.kind, ManagerKind::Graphics | ManagerKind::Sound) {
        // c3_PopupWindow::AddClose constructs this control at runtime, not in LDL.
        walker.widgets.push(ManagerWidget {
            id: "PopupClose".to_owned(),
            label: "Close".to_owned(),
            rect: MenuRect {
                x: rect
                    .x
                    .saturating_add(i32::try_from(rect.width).unwrap_or(0))
                    .saturating_sub(86),
                y: rect
                    .y
                    .saturating_add(i32::try_from(rect.height).unwrap_or(0))
                    .saturating_sub(39),
                width: 64,
                height: 20,
            },
            node: LayoutNode::default(),
            intent: Some(ManagerIntent::Close),
            unavailable: None,
            selected: false,
        });
    }
    walker.widgets
}

/// Context kept separate from recursive arguments and from SDL ownership.
struct Walker<'view> {
    /// Installed source and localization.
    ui: &'view ManagementUi,
    /// Current viewer snapshot, read-only.
    state: &'view GameState,
    /// Independent local selections.
    view: &'view ManagerView,
    /// Title bars are resized by original `ctp2_Window` at runtime.
    root_rect: MenuRect,
    /// Flattened draw-order controls.
    widgets: Vec<ManagerWidget>,
}

impl Walker<'_> {
    /// Visits an active node and its children in inherited source order.
    fn walk(&mut self, node: &LayoutNode, path: &str, parent: MenuRect) {
        if hidden(self.view, path) || matches!(node.name.as_str(), "rangery" | "rangerx") {
            return;
        }
        let mut rect = if path.is_empty() {
            parent
        } else {
            child_rect(node, parent)
        };
        if node.name == "TitleBar" {
            rect.x = self.root_rect.x;
            rect.y = self.root_rect.y;
            rect.width = self.root_rect.width;
        }
        // Clone only the flattened descriptor, not every descendant at every
        // level (which makes a large manager traversal quadratic in its tree).
        let flat = LayoutNode {
            name: node.name.clone(),
            bases: Vec::new(),
            fields: node.fields.clone(),
            children: Vec::new(),
        };
        let (intent, unavailable) = bindings::binding(self.ui, self.state, self.view, path, node);
        let title = node.name == "TitleBar";
        let label = if title {
            self.ui
                .layouts
                .get(&self.view.kind)
                .and_then(|root| root.field("title"))
                .map(|key| self.ui.book.label(key).to_owned())
        } else {
            data::text(self.ui, self.state, self.view, path)
        }
        .unwrap_or_else(|| source_label(self.ui, node));
        let selected = path
            .strip_prefix("ItemsBox.")
            .and_then(|name| name.strip_suffix("Button"))
            .is_some_and(|category| {
                self.view
                    .selections
                    .get("build-category")
                    .is_some_and(|selected| selected == category)
            });
        self.widgets.push(ManagerWidget {
            id: path.to_owned(),
            label,
            rect,
            node: flat,
            selected,
            intent,
            unavailable,
        });
        if node
            .field("objecttype")
            .is_some_and(|kind| kind.eq_ignore_ascii_case("ctp2_ListBox"))
        {
            self.list(node, path, rect);
            return;
        }
        let tabs: Vec<_> = node
            .children
            .iter()
            .filter(|child| {
                child
                    .field("objecttype")
                    .is_some_and(|kind| kind.eq_ignore_ascii_case("ctp2_Tab"))
            })
            .collect();
        let tab_key = format!("tab:{path}");
        let active = self
            .view
            .selections
            .get(&tab_key)
            .map(String::as_str)
            .or_else(|| tabs.first().map(|tab| tab.name.as_str()));
        for child in &node.children {
            let child_path = join(path, &child.name);
            if tabs.iter().any(|tab| tab.name == child.name) {
                for part in &child.children {
                    if part.name == "TabButton" {
                        let start = self.widgets.len();
                        self.walk(part, &join(&child_path, &part.name), rect);
                        for widget in &mut self.widgets[start..] {
                            widget.intent =
                                Some(ManagerIntent::Select(tab_key.clone(), child.name.clone()));
                            widget.unavailable = None;
                            widget.selected = active == Some(child.name.as_str());
                        }
                    } else if active == Some(child.name.as_str()) {
                        self.walk(part, &join(&child_path, &part.name), rect);
                    }
                }
            } else {
                self.walk(child, &child_path, rect);
            }
        }
    }

    /// Source header columns, clipped dynamic rows and original list-local ranger.
    fn list(&mut self, node: &LayoutNode, path: &str, rect: MenuRect) {
        if let Some(header) = node.child("header") {
            let mut x = rect.x;
            for column in &header.children {
                let height = u32::try_from(column.integer("heightpix").unwrap_or(20)).unwrap_or(20);
                let width = u32::try_from(column.integer("widthpix").unwrap_or(0)).unwrap_or(0);
                let parent = MenuRect {
                    x,
                    y: rect.y.saturating_sub(i32::try_from(height).unwrap_or(0)),
                    width,
                    height,
                };
                let mut copy = column.clone();
                copy.fields.insert("xpix".to_owned(), "0".to_owned());
                self.walk(
                    &copy,
                    &join(path, &format!("header.{}", column.name)),
                    parent,
                );
                x = x.saturating_add(i32::try_from(width).unwrap_or(0));
            }
        }
        let rows = data::rows(self.ui, self.state, self.view, path);
        let height = if self.view.kind == ManagerKind::Build && path == "QueueGroup.QueueList" {
            16_i32
        } else if self.view.kind == ManagerKind::Science {
            14
        } else {
            18
        };
        let visible = usize::try_from(rect.height)
            .unwrap_or(0)
            .checked_div(usize::try_from(height).unwrap_or(1))
            .unwrap_or(0)
            .max(1);
        let offset = self
            .view
            .scroll
            .get(path)
            .copied()
            .unwrap_or(0)
            .min(rows.len().saturating_sub(visible));
        let key = selection_key(self.view.kind, path);
        for (index, row) in rows.iter().skip(offset).take(visible).enumerate() {
            let id = &row.0;
            let label = &row.1;
            let selected = self
                .view
                .selections
                .get(key)
                .is_some_and(|value| value == id);
            self.widgets.push(ManagerWidget {
                id: format!("{path}/{id}"),
                label: label.clone(),
                rect: MenuRect {
                    x: rect.x.saturating_add(3),
                    y: rect
                        .y
                        .saturating_add(i32::try_from(index).unwrap_or(0).saturating_mul(height))
                        .saturating_add(3),
                    width: rect.width.saturating_sub(6),
                    height: u32::try_from(height).unwrap_or(0),
                },
                node: LayoutNode::default(),
                intent: Some(ManagerIntent::Select(key.to_owned(), id.clone())),
                unavailable: None,
                selected,
            });
        }
        if rows.len() > visible {
            for (suffix, label, y, delta) in [
                ("up", "↑", rect.y, -1_i32),
                (
                    "down",
                    "↓",
                    rect.y
                        .saturating_add(i32::try_from(rect.height).unwrap_or(0))
                        .saturating_sub(15),
                    1,
                ),
            ] {
                self.widgets.push(ManagerWidget {
                    id: format!("{path}/{suffix}"),
                    label: label.to_owned(),
                    rect: MenuRect {
                        x: rect
                            .x
                            .saturating_add(i32::try_from(rect.width).unwrap_or(0)),
                        y,
                        width: 15,
                        height: 15,
                    },
                    node: LayoutNode::default(),
                    intent: Some(ManagerIntent::Scroll(path.to_owned(), delta)),
                    unavailable: None,
                    selected: false,
                });
            }
        }
    }
}

/// Unbound source number placeholders are unknown, but declared slider minima
/// remain real zero labels. Dynamic authoritative overrides are handled before
/// this fallback and must never be replaced merely because their value is zero.
fn source_label(ui: &ManagementUi, node: &LayoutNode) -> String {
    match node.field("text") {
        Some("str_ldl_0") if !node.name.ends_with("Min") => "—".to_owned(),
        Some("str_ldl_CityManager") => "City Manager".to_owned(),
        Some(key) => ui.book.label(key).to_owned(),
        None => String::new(),
    }
}

/// Separates source lists that must not steal one another's selection.
fn selection_key(kind: ManagerKind, path: &str) -> &'static str {
    match kind {
        ManagerKind::Build if path == "QueueGroup.QueueList" => "queue",
        ManagerKind::Build => "buildable",
        ManagerKind::Science | ManagerKind::Research => "advance",
        ManagerKind::National => "city",
        ManagerKind::Unit | ManagerKind::Army => "unit",
        ManagerKind::Trade => "trade",
        ManagerKind::Diplomacy => "contact",
        ManagerKind::Library => "article",
        ManagerKind::City
        | ManagerKind::Empire
        | ManagerKind::Statistics
        | ManagerKind::Graphics
        | ManagerKind::Sound => "row",
    }
}

/// Hides mutually exclusive source modes rather than painting all lists at once.
fn hidden(view: &ManagerView, path: &str) -> bool {
    if matches!(view.kind, ManagerKind::Graphics | ManagerKind::Sound) && path == "AcceptButton" {
        // The original callbacks instantiate AddClose, never this obsolete block.
        return true;
    }
    if view.kind == ManagerKind::Build {
        if [
            "LoadBox",
            "MultiGroup",
            "MultiCityList",
            "CustomModeButtons",
        ]
        .iter()
        .any(|name| path == *name)
        {
            return true;
        }
        let category = view
            .selections
            .get("build-category")
            .map_or("Units", String::as_str);
        return match path {
            "ItemsBox.UnitsList" => category != "Units",
            "ItemsBox.BuildingList" => category != "Buildings",
            "ItemsBox.WonderList" => category != "Wonders",
            _ => false,
        };
    }
    false
}

/// Original main-window centering differs from centered C3 popup positioning.
fn root_rect(node: &LayoutNode) -> MenuRect {
    let (x, y, width, height) = if node
        .field("centeredwindow")
        .is_some_and(|value| value.eq_ignore_ascii_case("true"))
    {
        node.centered_rect(800, 600)
    } else {
        let width = node.integer("widthpix").unwrap_or(0);
        let height = node.integer("heightpix").unwrap_or(0);
        let x = if node.field("xanchor") == Some("center") {
            800_i32.saturating_sub(width).div_euclid(2)
        } else {
            node.integer("xpix").unwrap_or(0)
        };
        let y = if node.field("yanchor") == Some("center") {
            600_i32.saturating_sub(height).div_euclid(2)
        } else {
            node.integer("ypix").unwrap_or(0)
        };
        (x, y, width, height)
    };
    MenuRect {
        x,
        y,
        width: u32::try_from(width).unwrap_or(0),
        height: u32::try_from(height).unwrap_or(0),
    }
}

/// AUI anchors are parent-relative; missing group extents do not alter children.
fn child_rect(node: &LayoutNode, parent: MenuRect) -> MenuRect {
    let width = dimension(
        node,
        "width",
        parent.width,
        node.integer("widthpix").is_none().then_some(parent.width),
    );
    let button = node
        .field("objecttype")
        .is_some_and(|kind| kind.to_ascii_lowercase().contains("button"));
    let height = dimension(
        node,
        "height",
        parent.height,
        Some(if button { 20 } else { parent.height }),
    );
    let x = anchor(
        node.field("xanchor"),
        parent.width,
        width,
        node.integer("xpix").unwrap_or(0),
    );
    let y = anchor(
        node.field("yanchor"),
        parent.height,
        height,
        node.integer("ypix").unwrap_or(0),
    );
    MenuRect {
        x: parent.x.saturating_add(x),
        y: parent.y.saturating_add(y),
        width,
        height,
    }
}

/// Percentage dimensions use checked integer arithmetic and retain explicit zeros.
fn dimension(node: &LayoutNode, name: &str, parent: u32, fallback: Option<u32>) -> u32 {
    node.integer(&format!("{name}pix"))
        .and_then(|value| u32::try_from(value).ok())
        .or_else(|| {
            node.integer(&format!("{name}pct"))
                .and_then(|value| u32::try_from(value).ok())
                .map(|percent| parent.saturating_mul(percent).div_euclid(100))
        })
        .unwrap_or_else(|| fallback.unwrap_or(0))
}

/// Original right/bottom offsets are inward; center offsets remain signed.
fn anchor(anchor: Option<&str>, parent: u32, size: u32, offset: i32) -> i32 {
    let remaining = i32::try_from(parent.saturating_sub(size)).unwrap_or(0);
    match anchor {
        Some("center") => remaining.div_euclid(2).saturating_add(offset),
        Some("right" | "bottom") => remaining.saturating_sub(offset),
        _ => offset,
    }
}

/// Stable scoped path without a leading dot.
fn join(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_owned()
    } else {
        format!("{parent}.{name}")
    }
}
