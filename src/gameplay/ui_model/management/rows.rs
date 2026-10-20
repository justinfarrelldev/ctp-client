//! Cached source item-template columns and viewer-safe values.

use super::{ManagementUi, ManagerKind, ManagerView, ManagerWidget, data, tree};
use crate::gameplay::GameplayError;
use crate::gameplay::assets::layout::{LayoutBook, LayoutNode};
use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::MenuRect;
use alloc::collections::BTreeMap;

/// Resolves source runtime-created items once; no frame-time asset reads.
pub(super) fn templates(
    book: &LayoutBook,
) -> Result<BTreeMap<&'static str, LayoutNode>, GameplayError> {
    [
        "eq_UnitItem",
        "eq_BuildingWonderItem",
        "eq_QueueListItem",
        "ScienceAdvanceListItem",
        "SciListItem",
        "cw_CityItem",
    ]
    .into_iter()
    .map(|name| book.resolve(name).map(|node| (name, node)))
    .collect()
}

/// Immutable source/value context for one exact selectable row identity.
pub(super) struct Row<'view> {
    /// Session-owned templates and localized labels.
    pub ui: &'view ManagementUi,
    /// Viewer projection, never canonical hidden state.
    pub state: &'view GameState,
    /// Current city/manager selections.
    pub view: &'view ManagerView,
    /// Source list's column header metadata.
    pub list: &'view LayoutNode,
    /// Stable list path.
    pub path: &'view str,
    /// Exact canonical content identity.
    pub id: &'view str,
    /// Complete content row, excluding frame borders.
    pub bounds: MenuRect,
}

impl Row<'_> {
    /// Returns false only for managers whose item templates remain separate work.
    pub(super) fn append(&self, widgets: &mut Vec<ManagerWidget>) -> bool {
        let Some(name) = template_name(self.view.kind, self.path) else {
            return false;
        };
        let Some(template) = self.ui.row_templates.get(name) else {
            return false;
        };
        if self.view.kind == ManagerKind::Science {
            self.science(template, widgets);
        } else {
            for child in template.children.iter().rev() {
                self.visit(child, &child.name, self.bounds, widgets);
            }
        }
        true
    }

    /// Source declared nested control coordinates, not space-padded text tables.
    fn visit(
        &self,
        node: &LayoutNode,
        path: &str,
        parent: MenuRect,
        widgets: &mut Vec<ManagerWidget>,
    ) {
        let rect = tree::child_rect(node, parent);
        let label = if node.children.is_empty() {
            self.value(&node.name)
        } else {
            String::new()
        };
        widgets.push(self.widget(node, path, rect, label));
        for child in node.children.iter().rev() {
            self.visit(child, &format!("{path}.{}", child.name), rect, widgets);
        }
    }

    /// Numeric fields use verified runtime semantics; unknown scales/rates stay unknown.
    fn value(&self, name: &str) -> String {
        if super::selectors::city_list(self.view.kind, self.path) {
            return self
                .state
                .viewer
                .as_ref()
                .and_then(|viewer| viewer.cities.iter().find(|city| city.id == self.id))
                .map_or_else(|| self.id.to_owned(), data::city_name);
        }
        if matches!(name, "Item" | "Label" | "Name") {
            return data::label(self.ui, self.id);
        }
        let field = match name {
            "Attack" => "attack",
            "Defense" => "defense",
            "Armor" => "armor",
            "Ranged" => "ranged",
            "Damage" => "firepower",
            _ => return "—".to_owned(),
        };
        let definition = self
            .state
            .viewer
            .as_ref()
            .and_then(|viewer| viewer.interface.rules.as_ref())
            .and_then(|rules| rules["units"].as_array())
            .and_then(|units| {
                units
                    .iter()
                    .find(|unit| unit["id"].as_str() == Some(self.id))
            });
        let value = definition.and_then(|unit| match unit.get(field) {
            Some(value) => value.as_i64(),
            None if matches!(field, "ranged" | "firepower") => Some(0_i64), // Verified Go omitempty scalar fields.
            None => None,
        });
        value.map_or_else(|| "—".to_owned(), |number| number.to_string())
    }

    /// Preserves local discovery colors without inferring foreign advances from embassies.
    fn science(&self, template: &LayoutNode, widgets: &mut Vec<ManagerWidget>) {
        let headers = self.list.child("header");
        let mut x = self.bounds.x;
        for (index, column) in template.children.iter().enumerate() {
            let width = headers
                .and_then(|header| header.children.get(index))
                .and_then(|header| header.integer("widthpix"))
                .or_else(|| column.integer("widthpix"))
                .and_then(|width| u32::try_from(width).ok())
                .unwrap_or(0_u32);
            let rect = MenuRect {
                x,
                y: self.bounds.y,
                width,
                height: self.bounds.height,
            };
            let label = if index == 0_usize {
                data::label(self.ui, self.id)
            } else {
                String::new()
            };
            widgets.push(self.widget(column, &column.name, rect, label));
            if index == 1_usize {
                let mut mark =
                    knowledge_marker(format!("{}/{}/known/self", self.path, self.id), rect);
                clip(&mut mark.node, self.bounds);
                widgets.push(mark);
            }
            x = x.saturating_add(i32::try_from(width).unwrap_or(i32::MAX));
        }
    }

    /// Every child clips to the content pane and cannot steal its parent's action.
    fn widget(
        &self,
        node: &LayoutNode,
        path: &str,
        rect: MenuRect,
        label: String,
    ) -> ManagerWidget {
        let mut flat = LayoutNode {
            name: node.name.clone(),
            fields: node.fields.clone(),
            ..LayoutNode::default()
        };
        clip(&mut flat, self.bounds);
        ManagerWidget {
            id: format!("{}/{}/cell/{path}", self.path, self.id),
            label,
            rect,
            node: flat,
            intent: None,
            unavailable: None,
            selected: false,
        }
    }
}

/// Source ColorBox/ColorHeader callbacks inset one pixel; self is the client's blue player.
pub(super) fn knowledge_marker(id: String, bounds: MenuRect) -> ManagerWidget {
    let mut node = LayoutNode::default();
    for (name, value) in [
        ("primitive", "fill"),
        ("fontcolorred", "0"),
        ("fontcolorgreen", "0"),
        ("fontcolorblue", "255"),
    ] {
        node.fields.insert(name.to_owned(), value.to_owned());
    }
    ManagerWidget {
        id,
        label: String::new(),
        rect: MenuRect {
            x: bounds.x.saturating_add(1_i32),
            y: bounds.y.saturating_add(1_i32),
            width: bounds.width.saturating_sub(2_u32),
            height: bounds.height.saturating_sub(2_u32),
        },
        node,
        intent: None,
        unavailable: None,
        selected: false,
    }
}

/// Parent pane clipping is shared by glyphs, patterns and color blocks.
fn clip(node: &mut LayoutNode, bounds: MenuRect) {
    for (name, value) in [
        ("clip-x", bounds.x.to_string()),
        ("clip-y", bounds.y.to_string()),
        ("clip-width", bounds.width.to_string()),
        ("clip-height", bounds.height.to_string()),
    ] {
        node.fields.insert(name.to_owned(), value);
    }
}

/// Source item class shared by instantiation and scroll-capacity measurement.
fn template_name(kind: ManagerKind, path: &str) -> Option<&'static str> {
    if super::selectors::city_list(kind, path) {
        return Some("cw_CityItem");
    }
    match kind {
        ManagerKind::Build if path == "QueueGroup.QueueList" => Some("eq_QueueListItem"),
        ManagerKind::Build if path == "ItemsBox.UnitsList" => Some("eq_UnitItem"),
        ManagerKind::Build => Some("eq_BuildingWonderItem"),
        ManagerKind::Science => Some("ScienceAdvanceListItem"),
        ManagerKind::Research => Some("SciListItem"),
        _ => None,
    }
}

/// Uses actual installed item height, including the source runtime child-sized item.
pub(super) fn height(ui: &ManagementUi, kind: ManagerKind, path: &str) -> Option<i32> {
    let template = ui.row_templates.get(template_name(kind, path)?)?;
    template
        .integer("heightpix")
        .or_else(|| {
            template
                .children
                .iter()
                .filter_map(|child| child.integer("heightpix"))
                .max()
        })
        .filter(|height| *height > 0_i32)
}

/// Discovery table stays distinct from the available-target picker; both sort by cost.
pub(super) fn advances(ui: &ManagementUi, state: &GameState, known: bool) -> Vec<(String, String)> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Vec::new();
    };
    let Some(research) = viewer.interface.research.as_ref() else {
        return Vec::new();
    };
    let ids = if known {
        &research.advances
    } else {
        &research.available
    };
    let mut result: Vec<_> = ids
        .iter()
        .map(|id| (id.clone(), data::label(ui, id)))
        .collect();
    result.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    result.dedup_by(|left, right| left.0 == right.0);
    result.sort_by_cached_key(|row| {
        let cost = viewer
            .interface
            .rules
            .as_ref()
            .and_then(|rules| rules["advances"].as_array())
            .and_then(|advances| {
                advances
                    .iter()
                    .find(|advance| advance["id"].as_str() == Some(row.0.as_str()))
            })
            .and_then(|advance| advance["cost"].as_i64());
        (
            cost.is_none(),
            cost.unwrap_or(i64::MAX),
            row.1.to_lowercase(),
        )
    });
    result
}
