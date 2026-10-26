//! Cached source item-template columns and viewer-safe values.

use super::{ManagementUi, ManagerKind, ManagerView, ManagerWidget, data, tree};
use crate::gameplay::GameplayError;
use crate::gameplay::assets::layout::{LayoutBook, LayoutNode};
use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::MenuRect;
use alloc::collections::BTreeMap;
use core::cmp::Ordering;

/// Resolves source runtime-created items once; no frame-time asset reads.
pub(super) fn templates(
    book: &LayoutBook,
) -> Result<BTreeMap<&'static str, LayoutNode>, GameplayError> {
    let mut templates = [
        "eq_UnitItem",
        "eq_BuildingWonderItem",
        "eq_QueueListItem",
        "eq_LoadListItem",
        "eq_ContentsItem",
        "ScienceAdvanceListItem",
        "SciListItem",
        "cw_CityItem",
        "GovernmentListItem",
        "UnitManagerAdviceItem",
        "UnitStatsItem",
        "UnitTacticalItem",
        "CreateRouteItem",
        "TradeSummaryItem",
        "NationalResourceListItem",
        "NationalStatusListItem",
        "IntelligenceListItem",
        "DiplomacyDetailsListItem",
        "ForAdvancesItem",
        "YourAdvancesItem",
        "DipDetWondersItem",
        "DipWizProposalItem",
    ]
    .into_iter()
    .map(|name| book.resolve(name).map(|node| (name, node)))
    .collect::<Result<_, _>>()?;
    super::national::templates(&mut templates)?;
    Ok(templates)
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
        if matches!(
            self.view.kind,
            ManagerKind::Unit | ManagerKind::Trade | ManagerKind::National | ManagerKind::Diplomacy
        ) {
            self.columns(template, widgets);
        } else if self.view.kind == ManagerKind::Science {
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
        let mut rect = tree::child_rect(node, parent);
        /* Released EditQueue::UpdateQueueList -> CityWindow::PopulateQueueList
         * creates editqueue.ldl::eq_QueueListItem.StaticThing.Turns. Its wrapper
         * x4 plus Turns x117/width36 exceeds the declared153px row by4. AUI's
         * window-level text clip lets it spill; our required pane clip hid the
         * rightmost digits. Fix that source overflow by narrowing only this
         * right-aligned field to the pane; keep row/x/font/scaling unchanged.
         */
        if self.path == "QueueGroup.QueueList" && node.name == "Turns" {
            let right = self.bounds.x.saturating_add(i32::try_from(self.bounds.width).unwrap_or(i32::MAX));
            rect.width = rect.width.min(u32::try_from(right.saturating_sub(rect.x)).unwrap_or(0_u32));
        }
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
        /* Released EditQueue.cpp::AddChoiceItem(-1)/UpdateQueueList explicitly
         * blanks Custom Time/Turns in editqueue.ldl::eq_UnitItem/eq_BuildingWonderItem/
         * eq_QueueListItem. No city ETA is applicable to an independent template.
         */
        if let Some(text)=super::build::description::time(self.view,name){return text;}
        if self.view.kind == ManagerKind::Build {
            if let Some(text) = super::build::estimates::text(self.state, self.view, self.path, self.id, name) { return text; }
        }
        /* Original EditQueue.cpp::{UpdateFileLists,DisplayQueueContents} creates
         * eq_LoadListItem/eq_ContentsItem (editqueue.ldl). Their lowercase label
         * is actual filename/preview content, never a guessed library identity.
         */
        if matches!(self.path, "LoadBox.QueuesList" | "LoadBox.Contents") {
            return super::saved::rows(self.ui, self.view, self.path).and_then(|rows| rows.into_iter().find(|row| row.0 == self.id)).map_or_else(String::new, |row| row.1);
        }
        if self.path == "Advisor.List" {
            return super::advisors::row_value(self.ui, self.state, self.id, name);
        }
        if self.view.kind == ManagerKind::DiplomacyReview {
            return super::diplomacy::review::row_label(self.ui);
        }
        if self.view.kind == ManagerKind::Diplomacy {
            return super::diplomacy::value(self.state, self.id, name);
        }
        if self.view.kind == ManagerKind::National {
            return super::national::value(self.ui, self.state, self.path, self.id, name);
        }
        if self.view.kind == ManagerKind::Trade {
            return super::trade::value(self.ui, self.state, self.path, self.id, name);
        }
        if self.view.kind == ManagerKind::Unit {
            return super::units::value(self.ui, self.state, self.path, self.id, name);
        }
        if super::selectors::city_list(self.view.kind, self.path) {
            return self
                .state
                .viewer
                .as_ref()
                .and_then(|viewer| viewer.cities.iter().find(|city| city.id == self.id))
                .map_or_else(|| self.id.to_owned(), data::city_name);
        }
        if matches!(name, "Item" | "Label" | "Name") {
            return data::label(
                self.ui,
                super::queue::content(self.state, self.view, self.id).unwrap_or(self.id),
            );
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

    /// Original listbox item columns are arranged using corresponding header widths.
    fn columns(&self, template: &LayoutNode, widgets: &mut Vec<ManagerWidget>) {
        let mut x = self.bounds.x;
        for (index, column) in template.children.iter().enumerate() {
            let width = self
                .list
                .child("header")
                .and_then(|header| header.children.get(index))
                .and_then(|header| header.integer("widthpix"))
                .or_else(|| column.integer("widthpix"))
                .and_then(|width| u32::try_from(width).ok())
                .unwrap_or(0);
            let rect = MenuRect {
                x,
                y: self.bounds.y,
                width,
                height: self.bounds.height,
            };
            let mut widget = self.widget(column, &column.name, rect, self.value(&column.name));
            if self.view.kind == ManagerKind::Unit && column.name == "UnitIcon" {
                widget.rect.x = widget.rect.x.saturating_add(2);
                widget.rect.y = widget.rect.y.saturating_add(1);
                widget.rect.width = widget.rect.width.saturating_sub(2);
                widget.rect.height = widget.rect.height.saturating_sub(1);
                if let Some((name, stretch)) =
                    super::units::icon(self.ui, self.state, self.path, self.id)
                {
                    widget.node.fields.insert("image00".to_owned(), name);
                    widget.node.fields.insert(
                        "imageblttype".to_owned(),
                        if stretch { "stretch" } else { "copy" }.to_owned(),
                    );
                }
            }
            if self.view.kind == ManagerKind::Trade {
                if let Some(image) =
                    super::trade::image(self.ui, self.state, self.path, self.id, &column.name)
                {
                    widget.node.fields.insert("image00".to_owned(), image);
                }
            }
            if self.view.kind == ManagerKind::National {
                super::national::style(
                    self.state,
                    self.id,
                    self.path,
                    &column.name,
                    &mut widget.node,
                );
            }
            widgets.push(widget);
            if self.view.kind == ManagerKind::Unit && column.name == "Health" {
                let start = widgets.len();
                super::units::health(self.state, self.id, self.path, rect, widgets);
                for part in &mut widgets[start..] {
                    clip(&mut part.node, self.bounds);
                }
            }
            x = x.saturating_add(i32::try_from(width).unwrap_or(i32::MAX));
        }
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

/*
 * Original released-source style provenance:
 * ctp2_source/ctp2_code/ui/interface/ScienceManagementDialog.cpp::
 * ColorBoxActionCallback and ColorHeaderActionCallback inset the actual control
 * rectangle by one pixel and fill GetPlayerColor(cookie). Row bounds come from
 * science.ldl::{ADVANCE_LIST_COLUMN,ScienceAdvanceListItem,AdvanceList.header}.
 * The fixed blue/self-first-column below is an explicitly limited client
 * fallback: the server supplies no original slot/color/embassy knowledge map.
 * It is not source-exact nation identity, foreign knowledge, or frame acceptance.
 */
/// Source one-pixel insets with an explicit limited self-color fallback.
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
    if super::selectors::list_path(kind, path) {
        return super::selectors::template(kind, path);
    }
    match kind {
        ManagerKind::Build if path == "LoadBox.QueuesList" => Some("eq_LoadListItem"),
        ManagerKind::Build if path == "LoadBox.Contents" => Some("eq_ContentsItem"),
        ManagerKind::Build if path == "QueueGroup.QueueList" => Some("eq_QueueListItem"),
        ManagerKind::Build if path == "ItemsBox.UnitsList" => Some("eq_UnitItem"),
        ManagerKind::Build => Some("eq_BuildingWonderItem"),
        ManagerKind::Science => Some("ScienceAdvanceListItem"),
        ManagerKind::Research => Some("SciListItem"),
        ManagerKind::Unit if path == "Advisor.List" => Some("UnitManagerAdviceItem"),
        ManagerKind::Unit if path.contains("Tactical") => Some("UnitTacticalItem"),
        ManagerKind::Unit => Some("UnitStatsItem"),
        ManagerKind::Diplomacy if path == super::diplomacy::LIST => Some("IntelligenceListItem"),
        ManagerKind::DiplomacyReview => Some("DipWizProposalItem"),
        ManagerKind::National if path == super::national::RESOURCE => {
            Some("NationalResourceListItem")
        }
        ManagerKind::National if path == super::national::STATUS => Some("NationalStatusListItem"),
        ManagerKind::National if path == super::national::SPECIALISTS => {
            Some("NationalSpecialistListItem")
        }
        ManagerKind::Trade if path == super::trade::MARKET => Some("CreateRouteItem"),
        ManagerKind::Trade if matches!(path, super::trade::SUMMARY | super::trade::IMPORT) => {
            Some("TradeSummaryItem")
        }
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

/*
 * Original released-source behavior/element provenance:
 * - ctp2_source/ctp2_code/ui/interface/ScienceManagementDialog.cpp::
 *   UpdateAdvanceList/CreateAdvanceItem/UpdateAdvanceItem build discovered own
 *   or embassy-permitted rows; CompareAdvance column0 compares source costs.
 * - ctp2_source/ctp2_code/ui/interface/sci_advancescreen.cpp::
 *   sci_advancescreen_loadList/ScienceSortCallback instead use CanResearch's
 *   candidate IDs and their costs. These lists must never be conflated.
 * - ctp2/ctp2_data/english/uidata/layouts/science.ldl::{ScienceAdvanceListItem,
 *   SciListItem,AdvanceList.header} define their distinct columns/row geometry;
 *   geometry is read from cached templates, not guessed by the sort operation.
 * Client integration: only projected own discovery/available IDs and costs;
 * no foreign knowledge is inferred. Missing/null/negative costs remain unknown
 * last in either direction, while explicit zero remains a valid cost. Cache
 * keys once rather than the original quadratic bubble sort/private engine.
 */
/// Discovery table stays distinct from the available-target picker; both sort by cost.
pub(super) fn advances(ui: &ManagementUi, state: &GameState, known: bool, descending: bool) -> Vec<(String, String)> {
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
    // Cache projected costs and folded titles once, never in the comparison loop.
    let mut keyed: Vec<_> = result.into_iter().map(|row| {
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
            .and_then(|advance| advance["cost"].as_i64())
            .filter(|cost| *cost >= 0_i64);
        let title = row.1.to_lowercase();
        (row, cost, title)
    }).collect();
    keyed.sort_by(|left, right| {
        match (left.1, right.1) {
            (Some(left_cost), Some(right_cost)) => {
                if descending { right_cost.cmp(&left_cost) } else { left_cost.cmp(&right_cost) }
            }
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }.then_with(|| left.2.cmp(&right.2)).then_with(|| left.0.0.cmp(&right.0.0))
    });
    keyed.into_iter().map(|(row, _cost, _title)| row).collect()
}
