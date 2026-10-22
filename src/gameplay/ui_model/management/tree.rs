//! Shared source-layout traversal for painting and semantic/pointer activation.

#![expect(
    clippy::default_numeric_fallback,
    reason = "Original LDL fixed geometry is signed i32; dimensions are checked before conversion to u32"
)]

use super::{
    ManagementUi, ManagerIntent, ManagerKind, ManagerView, ManagerWidget, articles, bindings,
    chart, data, lists, policy, rows, scrollbar, selectors,
};
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
    if let Some((advice, advice_rect)) = super::advisors::window(ui, view, rect) {
        let mut advisor = Walker {
            ui,
            state,
            view,
            root_rect: advice_rect,
            widgets: Vec::new(),
        };
        advisor.walk(advice, "Advisor", advice_rect);
        walker.widgets.extend(advisor.widgets);
    }
    if let Some((window, path, popup_rect)) = selectors::popup(ui, state, view, &walker.widgets) {
        walker.walk(&window, &path, popup_rect);
    }
    if matches!(view.kind, ManagerKind::Graphics | ManagerKind::Sound) {
        // c3_PopupWindow::AddClose constructs this control at runtime, not in LDL.
        walker.widgets.push(ManagerWidget {
            id: "PopupClose".to_owned(),
            label: "Close".to_owned(),
            rect: MenuRect {
                x: rect
                    .x
                    .saturating_add(i32::try_from(rect.width).unwrap_or(0))
                    .saturating_sub(87),
                y: rect
                    .y
                    .saturating_add(i32::try_from(rect.height).unwrap_or(0))
                    .saturating_sub(41),
                width: 70,
                height: 24,
            },
            node: ui.book.resolve("c3_PopupClose").unwrap_or_default(),
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
        let rect = self.position(node, path, parent);
        let widget_index = self.widgets.len();
        if lists::is_list(node) {
            let mut frame = self.descriptor(node, path, lists::frame(node, rect));
            lists::stamp(&mut frame.node, rect);
            self.widgets.push(frame);
            self.list(node, path, rect);
            return;
        }
        self.widgets.push(self.descriptor(node, path, rect));
        if self.atomic(node, path, rect, widget_index) {
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
        let active = self.active_tab(&tabs, &tab_key);
        // aui_Region::DrawChildren paints from tail to head; hit testing uses
        // the reverse flattened order, keeping foreground controls consistent.
        for child in node.children.iter().rev() {
            let child_path = join(path, &child.name);
            if tabs.iter().any(|tab| tab.name == child.name) {
                for part in child.children.iter().rev() {
                    if part.name == "TabButton" {
                        let start = self.widgets.len();
                        self.walk(part, &join(&child_path, &part.name), rect);
                        for widget in &mut self.widgets[start..] {
                            widget.unavailable = articles::tab_unavailable(
                                self.ui,
                                self.state,
                                self.view,
                                &child.name,
                            );
                            widget.intent = widget.unavailable.is_none().then(|| {
                                ManagerIntent::Select(tab_key.clone(), child.name.clone())
                            });
                            widget.selected = active.as_deref() == Some(child.name.as_str());
                            let right = tabs.iter().position(|tab| tab.name == child.name)
                                > tabs
                                    .iter()
                                    .position(|tab| Some(tab.name.as_str()) == active.as_deref());
                            widget.node.fields.insert(
                                "tabside".to_owned(),
                                if right { "right" } else { "left" }.to_owned(),
                            );
                        }
                    } else if active.as_deref() == Some(child.name.as_str()) {
                        self.walk(part, &join(&child_path, &part.name), rect);
                    }
                }
            } else {
                self.walk(child, &child_path, rect);
            }
        }
    }

    /// Runtime-created atomic content owns its source descendants and clipping.
    fn atomic(&mut self, node: &LayoutNode, path: &str, rect: MenuRect, index: usize) -> bool {
        if matches!(self.view.kind, ManagerKind::Graphics | ManagerKind::Sound)
            && super::option_art::append(node, path, rect, &mut self.widgets)
        {
            self.widgets[index].label.clear();
            return true;
        }
        if self.view.kind == ManagerKind::Empire
            && policy::append(self.ui, self.state, node, path, rect, &mut self.widgets)
        {
            self.widgets[index].label.clear();
            return true;
        }
        if selectors::is_dropdown(node) {
            self.widgets[index].label.clear();
            self.dropdown(node, path, rect);
            return true;
        }
        if chart::append_widgets(
            self.ui,
            self.state,
            self.view,
            path,
            rect,
            &mut self.widgets,
        ) {
            return true;
        }
        if articles::append_widgets(
            self.ui,
            self.state,
            self.view,
            node,
            path,
            rect,
            &mut self.widgets,
        ) {
            self.widgets[index].label.clear();
            scrollbar::append(
                self.ui,
                self.state,
                self.view,
                node,
                path,
                rect,
                &mut self.widgets,
            );
            return true;
        }
        false
    }

    /// Dropdown creates its button/selected item, but not an always-visible popup.
    fn dropdown(&mut self, node: &LayoutNode, path: &str, rect: MenuRect) {
        if let Some(button) = node.child("autobutton") {
            let mut resized = button.clone();
            resized
                .fields
                .insert("widthpix".to_owned(), rect.width.to_string());
            resized.fields.insert("xpix".to_owned(), "0".to_owned());
            resized.fields.insert("ypix".to_owned(), "0".to_owned());
            let start = self.widgets.len();
            self.walk(&resized, &join(path, "autobutton"), rect);
            if let Some(widget) = self.widgets.get_mut(start) {
                widget.selected = self
                    .view
                    .selections
                    .get("dropdown")
                    .is_some_and(|open| open == path);
            }
        }
        if let Some(caption) = selectors::caption(self.ui, self.state, self.view, path, rect) {
            self.widgets.push(caption);
        }
    }

    /// Original window code resizes title bars without moving ordinary descendants.
    fn position(&self, node: &LayoutNode, path: &str, parent: MenuRect) -> MenuRect {
        let mut rect = if path.is_empty() {
            parent
        } else {
            child_rect(node, parent)
        };
        if let (Some(x), Some(y)) = (node.integer("reference-x"), node.integer("reference-y")) {
            rect.x = self.root_rect.x.saturating_add(35_i32).saturating_add(x);
            rect.y = self.root_rect.y.saturating_add(55_i32).saturating_add(y);
        }
        if node.name == "TitleBar" {
            rect.x = self.root_rect.x;
            rect.y = self.root_rect.y;
            rect.width = self.root_rect.width;
        }
        rect
    }

    /// A selected tab whose content disappeared falls back to an available panel.
    fn active_tab(&self, tabs: &[&LayoutNode], key: &str) -> Option<String> {
        self.view
            .selections
            .get(key)
            .filter(|name| {
                tabs.iter().any(|tab| tab.name == **name)
                    && articles::tab_unavailable(self.ui, self.state, self.view, name).is_none()
            })
            .cloned()
            .or_else(|| {
                tabs.iter()
                    .find(|tab| {
                        articles::tab_unavailable(self.ui, self.state, self.view, &tab.name)
                            .is_none()
                    })
                    .map(|tab| tab.name.clone())
            })
    }

    /// Clones only flattened properties, not every descendant at every level.
    fn descriptor(&self, node: &LayoutNode, path: &str, rect: MenuRect) -> ManagerWidget {
        let mut flat = LayoutNode {
            name: node.name.clone(),
            bases: Vec::new(),
            fields: node.fields.clone(),
            children: Vec::new(),
        };
        if let Some(image) = articles::image(self.ui, self.state, self.view, path) {
            flat.fields.insert(
                if self.view.kind == ManagerKind::Library {
                    "image00"
                } else {
                    "image40"
                }
                .to_owned(),
                image,
            );
        }
        if self.view.kind == ManagerKind::Library && path == "Title" {
            flat.fields
                .insert("overflow-title".to_owned(), "true".to_owned());
        }
        if self.ui.focused.as_deref() == Some(path) {
            flat.fields
                .insert("textfocused".to_owned(), "true".to_owned());
        }
        if self.view.kind == ManagerKind::Library && path == "SearchWord" {
            let selection = self.ui.editor.range(&self.ui.library.query);
            for (key, value) in [
                (
                    "edit-caret",
                    self.ui.editor.caret_offset(&self.ui.library.query),
                ),
                ("selection-start", selection.start),
                ("selection-end", selection.end),
            ] {
                flat.fields.insert(key.to_owned(), value.to_string());
            }
            flat.fields
                .insert("composition".to_owned(), self.ui.editor.composition.clone());
            flat.fields.insert(
                "composition-caret".to_owned(),
                self.ui.editor.composition_cursor.to_string(),
            );
        }
        let (intent, unavailable) = bindings::binding(self.ui, self.state, self.view, path, node);
        let label = if node.name == "TitleBar" {
            self.ui
                .layouts
                .get(&self.view.kind)
                .and_then(|root| root.field("title"))
                .map(|key| self.ui.book.label(key).to_owned())
        } else {
            data::text(self.ui, self.state, self.view, path)
        }
        .unwrap_or_else(|| source_label(self.ui, node));
        let selected = (path == "GovernorBox.Toggle"
            && data::city(self.state, self.view)
                .and_then(|city| city.governor.as_ref())
                .is_some_and(|governor| governor.enabled))
            || (self.view.kind == ManagerKind::Library
                && path == "SortByAgeButton"
                && self.ui.library.sort_by_age
                && super::index::can_sort(self.ui, self.state))
            || path
                .strip_prefix("ItemsBox.")
                .and_then(|name| name.strip_suffix("Button"))
                .is_some_and(|category| {
                    self.view
                        .selections
                        .get("build-category")
                        .is_some_and(|selected| selected == category)
                });
        ManagerWidget {
            id: path.to_owned(),
            label,
            rect,
            node: flat,
            intent,
            unavailable,
            selected,
        }
    }

    /// Source header columns are laid out above, not inside, the list rows.
    fn list_header(&mut self, node: &LayoutNode, path: &str, rect: MenuRect) {
        if let Some(header) = node.child("header") {
            let mut x = rect.x;
            for (column_index, column) in header.children.iter().enumerate() {
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
                if self.view.kind == ManagerKind::Science
                    && column_index == 1_usize
                    && self
                        .state
                        .viewer
                        .as_ref()
                        .and_then(|viewer| viewer.interface.research.as_ref())
                        .is_some()
                {
                    self.widgets.push(rows::knowledge_marker(
                        format!("{path}/header/known/self"),
                        parent,
                    ));
                }
                x = x.saturating_add(i32::try_from(width).unwrap_or(0));
            }
        }
    }

    /// Clipped dynamic rows and original list-local ranger controls.
    fn list(&mut self, node: &LayoutNode, path: &str, rect: MenuRect) {
        self.list_header(node, path, rect);
        let rows = data::rows(self.ui, self.state, self.view, path);
        let height = row_height(self.ui, self.view.kind, path);
        let visible = visible_rows(self.ui, self.view.kind, path, rect);
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
            let selected = if selectors::list_path(self.view.kind, path) {
                selectors::selected(self.view) == Some(id.as_str())
            } else if self.view.kind == ManagerKind::Library {
                self.ui.library.current.as_ref() == Some(id)
            } else {
                self.view
                    .selections
                    .get(key)
                    .is_some_and(|value| value == id)
            };
            let header = self.view.kind == ManagerKind::Library && id.starts_with('@');
            let mut row_style = LayoutNode::default();
            if header {
                if let Ok(style) = self.ui.book.resolve("CTP_GREY_TITLE_FONT") {
                    row_style.fields = style.fields;
                }
                row_style
                    .fields
                    .insert("underlinefont".to_owned(), "true".to_owned());
            } else if self.view.kind != ManagerKind::Library {
                row_style
                    .fields
                    .insert("just".to_owned(), "left".to_owned());
                // Preserve the legacy flattened row's label inset explicitly;
                // measured rich-text words never acquire renderer-only padding.
                row_style
                    .fields
                    .insert("text-inset-left".to_owned(), "3".to_owned());
            }
            let row_index = self.widgets.len();
            self.widgets.push(ManagerWidget {
                id: format!("{path}/{id}"),
                label: label.clone(),
                rect: MenuRect {
                    x: rect.x,
                    y: rect
                        .y
                        .saturating_add(i32::try_from(index).unwrap_or(0).saturating_mul(height)),
                    width: rect.width,
                    height: u32::try_from(height).unwrap_or(0),
                },
                node: row_style,
                intent: (!header).then(|| {
                    if self.view.kind == ManagerKind::Library {
                        ManagerIntent::Article(id.clone())
                    } else {
                        ManagerIntent::Select(key.to_owned(), id.clone())
                    }
                }),
                unavailable: None,
                selected,
            });
            let bounds = self.widgets[row_index].rect;
            if (rows::Row {
                ui: self.ui,
                state: self.state,
                view: self.view,
                list: node,
                path,
                id,
                bounds,
            })
            .append(&mut self.widgets)
            {
                self.widgets[row_index]
                    .node
                    .fields
                    .insert("row-container".to_owned(), "true".to_owned());
            }
        }
        scrollbar::append(
            self.ui,
            self.state,
            self.view,
            node,
            path,
            rect,
            &mut self.widgets,
        );
    }
}

/// Uses the same row capacity for scroll accounting and native row descriptors.
pub(super) fn visible_rows(
    ui: &ManagementUi,
    kind: ManagerKind,
    path: &str,
    rect: MenuRect,
) -> usize {
    usize::try_from(rect.height)
        .unwrap_or(0_usize)
        .checked_div(usize::try_from(row_height(ui, kind, path)).unwrap_or(1_usize))
        .unwrap_or(0_usize)
}

/// Source-specific row spacing; not a guessed number of entries per page.
fn row_height(ui: &ManagementUi, kind: ManagerKind, path: &str) -> i32 {
    if let Some(height) = rows::height(ui, kind, path) {
        return height;
    }
    if kind == ManagerKind::Build && path == "QueueGroup.QueueList" {
        16_i32
    } else if kind == ManagerKind::Library {
        20_i32
    } else if kind == ManagerKind::Science {
        14_i32
    } else if kind == ManagerKind::Research {
        24_i32
    } else {
        18_i32
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
    if path == "Advisor.List" {
        return "advisor-category";
    }
    if selectors::list_path(kind, path) {
        return if kind == ManagerKind::Empire {
            "government"
        } else {
            "city"
        };
    }
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
    if view.kind == ManagerKind::Library && path == "SearchLabel" {
        return true;
    }
    if matches!(view.kind, ManagerKind::Graphics | ManagerKind::Sound)
        && matches!(path, "AcceptButton" | "Background")
    {
        // The original callbacks instantiate AddClose, never this obsolete block.
        return true;
    }
    if view.kind == ManagerKind::Empire {
        if path == "DetailsButton"
            && view
                .selections
                .get("tab:TabGroup")
                .is_some_and(|tab| tab != "Tab1")
        {
            return true;
        }
        let details = view
            .selections
            .get("empire-details")
            .is_some_and(|value| value == "true");
        if path.ends_with(".InformationDetails") {
            return !details;
        }
        if path.ends_with(".InformationSummary") {
            return details;
        }
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
pub(super) fn child_rect(node: &LayoutNode, parent: MenuRect) -> MenuRect {
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
