//! Original one-hop advance chart with viewer-safe relations and bounded pages.
//!
//! `Chart::Update`'s geometry/order is retained, but overflow beyond four neighbors
//! is paginated rather than discarded. Neighbors recenter only this local chart;
//! the formerly empty center callback opens its installed article explicitly.

use alloc::collections::BTreeSet;
use serde_json::Value;

use super::{ManagementUi, ManagerIntent, ManagerKind, ManagerView, ManagerWidget, data};
use crate::gameplay::assets::layout::LayoutNode;
use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::MenuRect;

/// Original chart capacity, not an authority limit on actual prerequisites.
const PAGE_SIZE: usize = 4_usize;
/// Stable active panel path shared by drawing, pointers and accessible controls.
const PANEL: &str = "Tabs.TechTreeTab.TabPanel";

/// An unavailable chart must not become a selectable blank panel.
pub(super) fn unavailable(ui: &ManagementUi, state: &GameState) -> Option<String> {
    let current = ui.library.current.as_deref();
    if current.is_none_or(|id| !id.starts_with("ADVANCE_")) {
        return Some("The source dependency chart applies only to advances".to_owned());
    }
    let present =
        definitions(state).is_some_and(|rows| rows.iter().any(|row| row["id"].as_str() == current));
    if !present {
        return Some("Current server advance definitions are unavailable; the retail tree is not substituted".to_owned());
    }
    if ui.chart.child("button").is_none() {
        return Some("Original chart button template is unavailable".to_owned());
    }
    None
}

/// Injects the runtime-created source chart only in its actual active panel.
pub(super) fn append_widgets(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    path: &str,
    parent: MenuRect,
    widgets: &mut Vec<ManagerWidget>,
) -> bool {
    if view.kind != ManagerKind::Library || path != PANEL {
        return false;
    }
    let Some(rows) = definitions(state) else {
        return true;
    };
    let Some(button) = ui.chart.child("button") else {
        return true;
    };
    let current = ui.library.current.as_deref().unwrap_or("");
    let focus = view
        .selections
        .get("chart-focus")
        .map_or(current, String::as_str);
    let selected = rows
        .iter()
        .find(|row| row["id"].as_str() == Some(focus))
        .or_else(|| rows.iter().find(|row| row["id"].as_str() == Some(current)));
    let Some(selected) = selected else {
        return true;
    };
    let Some(center) = selected["id"].as_str() else {
        return true;
    };
    let prerequisites = unique(
        selected["prerequisites"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter(|id| *id != center),
    );
    let successors = unique(
        rows.iter()
            .filter(|row| {
                row["prerequisites"]
                    .as_array()
                    .is_some_and(|ids| ids.iter().any(|id| id.as_str() == Some(center)))
            })
            .filter_map(|row| row["id"].as_str())
            .filter(|id| *id != center),
    );
    let rect = MenuRect {
        x: parent
            .x
            .saturating_add(ui.chart.integer("xpix").unwrap_or(0_i32)),
        y: parent
            .y
            .saturating_add(ui.chart.integer("ypix").unwrap_or(0_i32)),
        width: dimension(&ui.chart, "widthpix"),
        height: dimension(&ui.chart, "heightpix"),
    };
    let width = dimension(button, "widthpix");
    let height = dimension(button, "heightpix");
    let mut chart = Chart {
        ui,
        state,
        view,
        button,
        rect,
        width,
        height,
        widgets,
        lines: Vec::new(),
    };
    chart.column(&prerequisites, "prerequisites", 0_i32);
    chart.column(&[center], "center", 1_i32);
    chart.column(&successors, "successors", 2_i32);
    chart.widgets.append(&mut chart.lines);
    true
}

/// Restores article-local chart navigation after changing the actual document.
pub(super) fn reset(view: &mut ManagerView) {
    for key in [
        "chart-focus",
        "chart-prerequisites-page",
        "chart-successors-page",
    ] {
        view.selections.remove(key);
    }
}

/// Runtime definitions are public rule metadata, never installed retail rules.
fn definitions(state: &GameState) -> Option<&Vec<Value>> {
    state.viewer.as_ref()?.interface.rules.as_ref()?["advances"].as_array()
}

/// Keeps server order while suppressing malformed duplicate/self references.
fn unique<'id>(ids: impl Iterator<Item = &'id str>) -> Vec<&'id str> {
    let mut seen = BTreeSet::new();
    ids.filter(|id| seen.insert(*id)).collect()
}

/// Original integer dimensions are bounded before use as SDL-compatible extents.
fn dimension(node: &LayoutNode, key: &str) -> u32 {
    node.integer(key)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(0_u32)
}

/// Read-only chart context keeps source geometry and authoritative metadata distinct.
struct Chart<'view, 'output> {
    /// Installed source and article availability.
    ui: &'view ManagementUi,
    /// Current immutable viewer snapshot.
    state: &'view GameState,
    /// Local focus and independent column pages.
    view: &'view ManagerView,
    /// Resolved original button style, not copied artwork.
    button: &'view LayoutNode,
    /// Source chart viewport inside the original tab panel.
    rect: MenuRect,
    /// Source button width.
    width: u32,
    /// Source button height.
    height: u32,
    /// Shared draw-order/interaction output.
    widgets: &'output mut Vec<ManagerWidget>,
    /// Connectors are painted after all buttons and borders, matching `Chart::Draw`.
    lines: Vec<ManagerWidget>,
}

impl Chart<'_, '_> {
    /// Emits a centered source column and optional overflow navigation controls.
    fn column(&mut self, ids: &[&str], relation: &str, column: i32) {
        let max_page = ids.len().saturating_sub(1_usize).div_euclid(PAGE_SIZE);
        let key = format!("chart-{relation}-page");
        let page = self
            .view
            .selections
            .get(&key)
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0_usize)
            .min(max_page);
        let visible: Vec<_> = ids
            .iter()
            .skip(page.saturating_mul(PAGE_SIZE))
            .take(PAGE_SIZE)
            .copied()
            .collect();
        let gap = self
            .rect
            .width
            .saturating_sub(self.width.saturating_mul(3_u32))
            .div_euclid(4_u32);
        let vertical = self
            .rect
            .height
            .saturating_sub(self.height.saturating_mul(4_u32))
            .div_euclid(5_u32);
        let count = u32::try_from(visible.len()).unwrap_or(u32::MAX);
        let occupied = count
            .saturating_mul(self.height)
            .saturating_add(count.saturating_sub(1_u32).saturating_mul(vertical));
        let start = self.rect.height.saturating_sub(occupied).div_euclid(2_u32);
        let x =
            self.rect
                .x
                .saturating_add(i32::try_from(gap).unwrap_or(0_i32))
                .saturating_add(column.saturating_mul(
                    i32::try_from(self.width.saturating_add(gap)).unwrap_or(0_i32),
                ));
        for (index, id) in visible.iter().enumerate() {
            let offset = start.saturating_add(
                u32::try_from(index)
                    .unwrap_or(u32::MAX)
                    .saturating_mul(self.height.saturating_add(vertical)),
            );
            let rect = MenuRect {
                x,
                y: self
                    .rect
                    .y
                    .saturating_add(i32::try_from(offset).unwrap_or(0_i32)),
                width: self.width,
                height: self.height,
            };
            self.node(id, relation, rect);
            if column != 1_i32 {
                self.connector(rect, column, gap);
            }
        }
        if max_page > 0_usize {
            self.pages(relation, &key, page, max_page, x);
        }
    }

    /// A neighbor only recenters; the center opens actual article content.
    fn node(&mut self, id: &str, relation: &str, rect: MenuRect) {
        let known = definitions(self.state)
            .is_some_and(|rows| rows.iter().any(|row| row["id"].as_str() == Some(id)));
        let installed = self.ui.library.catalog.article(id).is_some();
        let (intent, unavailable) = if relation == "center" {
            if installed {
                (Some(ManagerIntent::ChartArticle(id.to_owned())), None)
            } else {
                (
                    None,
                    Some("No installed article for this server-defined advance".to_owned()),
                )
            }
        } else if known {
            (
                Some(ManagerIntent::Select(
                    "chart-focus".to_owned(),
                    id.to_owned(),
                )),
                None,
            )
        } else {
            (
                None,
                Some(
                    "The prerequisite definition is absent from the current viewer rules"
                        .to_owned(),
                ),
            )
        };
        let mut node = self.button.clone();
        node.children.clear();
        self.widgets.push(ManagerWidget {
            id: format!("{PANEL}/{relation}/{id}"),
            label: data::label(self.ui, id),
            rect,
            node,
            intent,
            unavailable,
            selected: false,
        });
        let border = MenuRect {
            x: rect.x.saturating_sub(1_i32),
            y: rect.y.saturating_sub(1_i32),
            width: rect.width.saturating_add(2_u32),
            height: rect.height.saturating_add(2_u32),
        };
        self.primitive(
            format!("{PANEL}/border/{relation}/{id}"),
            border,
            "frame",
            self.color(id),
            None,
        );
    }

    /// Research-state borders come from viewer knowledge, not local prerequisites.
    fn color(&self, id: &str) -> [u8; 3] {
        if definitions(self.state)
            .is_none_or(|rows| !rows.iter().any(|row| row["id"].as_str() == Some(id)))
        {
            return [126_u8; 3_usize];
        }
        let Some(research) = self
            .state
            .viewer
            .as_ref()
            .and_then(|viewer| viewer.interface.research.as_ref())
        else {
            return [126_u8; 3_usize];
        };
        if research.advances.iter().any(|known| known == id) {
            [0_u8, 255_u8, 0_u8]
        } else if research.available.iter().any(|available| available == id) {
            [255_u8, 255_u8, 0_u8]
        } else {
            [255_u8, 0_u8, 0_u8]
        }
    }

    /// Mirrors `Chart::Draw`'s horizontal stubs and diagonal joins in integer pixels.
    fn connector(&mut self, node: MenuRect, column: i32, gap: u32) {
        let center_x = self.rect.x.saturating_add(
            i32::try_from(gap.saturating_mul(2_u32).saturating_add(self.width)).unwrap_or(0_i32),
        );
        let center_y = self
            .rect
            .y
            .saturating_add(i32::try_from(self.rect.height.div_euclid(2_u32)).unwrap_or(0_i32));
        let half = i32::try_from(gap.div_euclid(2_u32)).unwrap_or(0_i32);
        let y = node
            .y
            .saturating_add(i32::try_from(node.height.div_euclid(2_u32)).unwrap_or(0_i32));
        if column == 0_i32 {
            let x = node
                .x
                .saturating_add(i32::try_from(node.width).unwrap_or(0_i32));
            self.line((x, y), (x.saturating_add(5_i32), y));
            self.line(
                (x.saturating_add(5_i32), y),
                (center_x.saturating_sub(half), center_y),
            );
            self.line(
                (center_x.saturating_sub(half), center_y),
                (center_x, center_y),
            );
        } else {
            let x = center_x.saturating_add(i32::try_from(self.width).unwrap_or(0_i32));
            self.line((x, center_y), (x.saturating_add(half), center_y));
            self.line(
                (x.saturating_add(half), center_y),
                (node.x.saturating_sub(5_i32), y),
            );
            self.line((node.x.saturating_sub(5_i32), y), (node.x, y));
        }
    }

    /// Fully bounded segment descriptors are also safe under the renderer's clips.
    fn line(&mut self, start: (i32, i32), end: (i32, i32)) {
        let rect = MenuRect {
            x: start.0.min(end.0),
            y: start.1.min(end.1),
            width: start.0.abs_diff(end.0).saturating_add(1_u32),
            height: start.1.abs_diff(end.1).saturating_add(1_u32),
        };
        self.primitive(
            format!(
                "{PANEL}/edge/{}",
                self.widgets.len().saturating_add(self.lines.len())
            ),
            rect,
            "line",
            [0_u8; 3_usize],
            Some((start, end)),
        );
    }

    /// Draw-only primitives cannot intercept any of the adjacent semantic buttons.
    fn primitive(
        &mut self,
        id: String,
        rect: MenuRect,
        kind: &str,
        color: [u8; 3],
        line: Option<((i32, i32), (i32, i32))>,
    ) {
        let mut node = LayoutNode::default();
        node.fields.insert("primitive".to_owned(), kind.to_owned());
        for (channel, value) in ["fontcolorred", "fontcolorgreen", "fontcolorblue"]
            .into_iter()
            .zip(color)
        {
            node.fields.insert(channel.to_owned(), value.to_string());
        }
        if let Some((start, end)) = line {
            for (field, value) in [
                ("x1", start.0),
                ("y1", start.1),
                ("x2", end.0),
                ("y2", end.1),
            ] {
                node.fields.insert(field.to_owned(), value.to_string());
            }
        }
        let widget = ManagerWidget {
            id,
            label: String::new(),
            rect,
            node,
            intent: None,
            unavailable: None,
            selected: false,
        };
        if kind == "line" {
            self.lines.push(widget);
        } else {
            self.widgets.push(widget);
        }
    }

    /// More-than-four server neighbors remain reachable, unlike the original cutoff.
    fn pages(&mut self, relation: &str, key: &str, page: usize, maximum: usize, x: i32) {
        for (suffix, label, target, enabled, offset) in [
            (
                "previous",
                "‹",
                page.saturating_sub(1_usize),
                page > 0_usize,
                0_i32,
            ),
            (
                "next",
                "›",
                page.saturating_add(1_usize).min(maximum),
                page < maximum,
                64_i32,
            ),
        ] {
            let mut node = self.button.clone();
            node.children.clear();
            let rect = MenuRect {
                x: x.saturating_add(offset),
                y: self
                    .rect
                    .y
                    .saturating_add(i32::try_from(self.rect.height).unwrap_or(0_i32))
                    .saturating_sub(18_i32),
                width: 60_u32,
                height: 16_u32,
            };
            self.widgets.push(ManagerWidget {
                id: format!("{PANEL}/{relation}/{suffix}"),
                label: label.to_owned(),
                rect,
                node,
                intent: enabled.then(|| ManagerIntent::Select(key.to_owned(), target.to_string())),
                unavailable: (!enabled)
                    .then(|| "No further neighbor page in this direction".to_owned()),
                selected: false,
            });
        }
    }
}
