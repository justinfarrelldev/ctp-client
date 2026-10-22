//! Installed article bindings, shared hyperlink rectangles and safe rule variables.

use alloc::collections::BTreeMap;

use super::{ManagementUi, ManagerIntent, ManagerKind, ManagerView, ManagerWidget, data};
use crate::gameplay::assets::layout::LayoutNode;
use crate::gameplay::assets::library::{LibraryDocument, LibrarySection};
use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::MenuRect;
use crate::gameplay::ui_model::library::layout::{LibraryLayout, document_layout};

/// Collects exact installed words once, with native glyph fallbacks for new values.
pub(super) fn measure(ui: &mut ManagementUi, measure: impl Fn(&str) -> u32, line_height: u32) {
    ui.text_widths.clear();
    ui.line_height = line_height.max(1_u32);
    for character in ' '..='~' {
        let text = character.to_string();
        ui.text_widths.insert(text.clone(), measure(&text));
    }
    for article in ui.library.catalog.articles.values() {
        for document in article.sections.values() {
            for text in document
                .spans
                .iter()
                .filter_map(crate::gameplay::assets::library::LibrarySpan::literal)
            {
                for word in text.split_whitespace() {
                    ui.text_widths
                        .entry(word.to_owned())
                        .or_insert_with(|| measure(word));
                }
            }
        }
    }
}

/// Adds only visible measured words and safe internal hyperlinks to a source box.
/// Returning true means the source hypertext children have been fully replaced.
pub(super) fn append_widgets(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    node: &LayoutNode,
    path: &str,
    rect: MenuRect,
    widgets: &mut Vec<ManagerWidget>,
) -> bool {
    let Some((document, id)) = document(ui, state, view, path) else {
        return false;
    };
    let scroll = view.scroll.get(path).copied().unwrap_or(0_usize);
    let layout = layout(ui, state, document, &id, rect, scroll, node);
    for (index, fragment) in layout.fragments.into_iter().enumerate() {
        let mut fields = BTreeMap::new();
        fields.insert("richtext".to_owned(), "true".to_owned());
        fields.insert("just".to_owned(), "left".to_owned());
        for key in ["fontname", "fontsize", "boldfont", "italicfont"] {
            if let Some(value) = node.field(key) {
                fields.insert(key.to_owned(), value.to_owned());
            }
        }
        let (intent, unavailable) = fragment.target.as_ref().map_or((None, None), |target| {
            if ui.library.catalog.article(target).is_some() {
                (Some(ManagerIntent::Article(target.clone())), None)
            } else {
                (None, Some(format!("Article {target} is not installed")))
            }
        });
        for (channel, value) in [
            (
                "fontcolorred",
                if fragment.target.is_some() {
                    0_u8
                } else {
                    50_u8
                },
            ),
            (
                "fontcolorgreen",
                if fragment.target.is_some() {
                    0_u8
                } else {
                    50_u8
                },
            ),
            (
                "fontcolorblue",
                if fragment.target.is_some() {
                    255_u8
                } else {
                    50_u8
                },
            ),
        ] {
            fields.insert(channel.to_owned(), value.to_string());
        }
        let art = LayoutNode {
            name: node.name.clone(),
            fields,
            bases: Vec::new(),
            children: Vec::new(),
        };
        widgets.push(ManagerWidget {
            id: format!("{path}/word-{index}"),
            label: fragment.text,
            rect: MenuRect {
                x: rect.x.saturating_add(3_i32).saturating_add(fragment.x),
                y: rect.y.saturating_add(3_i32).saturating_add(fragment.y),
                width: fragment.width,
                height: fragment.height,
            },
            node: art,
            intent,
            unavailable,
            selected: false,
        });
    }
    true
}

/// Measures the same current content as painting; absent sections are not invented.
pub(super) fn scroll_limit(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    path: &str,
    rect: MenuRect,
) -> Option<usize> {
    let (document, id) = document(ui, state, view, path)?;
    let node = source_node(ui, view, path)?;
    Some(layout(ui, state, document, &id, rect, 0_usize, node).max_scroll)
}

/// Centralizes font, margins and variable resolution for rendering and scrolling.
fn layout(
    ui: &ManagementUi,
    state: &GameState,
    document: &LibraryDocument,
    id: &str,
    rect: MenuRect,
    scroll: usize,
    node: &LayoutNode,
) -> LibraryLayout {
    let key = super::typography::SourceFont::from_node(node);
    let metrics = ui.font_metrics.get(&key);
    document_layout(
        document,
        rect.width.saturating_sub(6_u32),
        rect.height.saturating_sub(6_u32),
        metrics.map_or(ui.line_height.max(8_u32), |metrics| metrics.line_height),
        |text| {
            metrics.map_or_else(
                || text_width(&ui.text_widths, text),
                |metrics| metrics.width(text),
            )
        },
        |expression| super::variables::resolve(state, id, expression),
        scroll,
    )
}

/// Resolves source properties without deriving them from rendered word fragments.
fn source_node<'ui>(
    ui: &'ui ManagementUi,
    view: &ManagerView,
    path: &str,
) -> Option<&'ui LayoutNode> {
    path.split('.')
        .try_fold(ui.layouts.get(&view.kind)?, |node, part| node.child(part))
}

/// Native visible line count uses exactly the source style used by its article.
pub(super) fn visible_lines(
    ui: &ManagementUi,
    view: &ManagerView,
    path: &str,
    rect: MenuRect,
) -> usize {
    let height = source_node(ui, view, path)
        .and_then(|node| {
            ui.font_metrics
                .get(&super::typography::SourceFont::from_node(node))
        })
        .map_or(ui.line_height.max(8_u32), |metrics| metrics.line_height);
    usize::try_from(
        rect.height
            .saturating_sub(6_u32)
            .div_ceil(height.max(1_u32)),
    )
    .unwrap_or(1_usize)
    .max(1_usize)
}

/// Source article identity and section for each actual C++ hypertext callback.
fn document<'catalog>(
    ui: &'catalog ManagementUi,
    state: &GameState,
    view: &ManagerView,
    path: &str,
) -> Option<(&'catalog LibraryDocument, String)> {
    let (id, section) = match view.kind {
        ManagerKind::Library => {
            let id = ui.library.current.as_ref()?.clone();
            let section = match path {
                "RequirementsText" => LibrarySection::Prerequisites,
                "VariablesText" => LibrarySection::Statistics,
                "Tabs.Gameplay.TabPanel.GameplayText" => LibrarySection::Gameplay,
                "Tabs.Historical.TabPanel.HistoricalText" => LibrarySection::Historical,
                _ => return None,
            };
            (id, section)
        }
        ManagerKind::Build if path == "ItemDescription" => {
            let id = selected_description(view, state)?.to_owned();
            if let Some(document) = ui
                .library
                .catalog
                .art
                .get(&id)
                .and_then(|art| art.summary.as_deref())
                .and_then(|tag| ui.library.catalog.document(tag))
            {
                return Some((document, id));
            }
            (id, LibrarySection::Summary)
        }
        ManagerKind::Science if path == "CurrentResearch.Description" => (
            super::science::description_target(view, state)?.to_owned(),
            LibrarySection::Statistics,
        ),
        ManagerKind::Research if path == "Background.GlStats" => (
            super::research::selected(view, state)?.to_owned(),
            LibrarySection::Statistics,
        ),
        _ => return None,
    };
    Some((ui.library.catalog.article(&id)?.sections.get(&section)?, id))
}

/// Last interacted list controls the description without discarding either selection.
pub(super) fn selected_description<'view>(
    view: &'view ManagerView,
    state: &'view GameState,
) -> Option<&'view str> {
    let selected = view
        .selections
        .get("description")
        .or_else(|| view.selections.get("buildable"))
        .or_else(|| view.selections.get("queue"))
        .map(String::as_str)?;
    Some(super::queue::content(state, view, selected).unwrap_or(selected))
}

/// Visible article title/index/search text, not client-authored game facts.
pub(super) fn text(ui: &ManagementUi, view: &ManagerView, path: &str) -> Option<String> {
    if view.kind != ManagerKind::Library {
        return None;
    }
    match path {
        "Title" => Some({
            let title = ui.book.label("str_ldl_CAPS_GREAT_LIBRARY");
            if title == "str_ldl_CAPS_GREAT_LIBRARY" {
                ui.book
                    .label("str_ldl_Great_Library")
                    .to_uppercase()
                    .chars()
                    .map(|character| character.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            } else {
                title.to_owned()
            }
        }),
        "ItemLabel" => Some(
            ui.library
                .current
                .as_deref()
                .map_or_else(|| "Contents".to_owned(), |id| data::label(ui, id)),
        ),
        "Category" => Some(
            if ui.library.search_results.is_some() {
                "Search Results"
            } else {
                let key = ui.library.category.label_key();
                let label = ui.book.label(key);
                if label == key {
                    ui.library.category.title()
                } else {
                    label
                }
            }
            .to_owned(),
        ),
        "SearchWord" => Some(ui.library.query.clone()),
        "SortByAgeButton" => Some(
            {
                let label = ui.book.label("str_ldl_SORT_BY_AGE");
                if label == "str_ldl_SORT_BY_AGE" {
                    "Sort by Age"
                } else {
                    label
                }
            }
            .to_owned(),
        ),
        _ => None,
    }
}

/// Preserves source missing-section rules and requires actual public graph data.
pub(super) fn tab_unavailable(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    name: &str,
) -> Option<String> {
    if view.kind != ManagerKind::Library {
        return None;
    }
    if name == "TechTreeTab" {
        return super::chart::unavailable(ui, state);
    }
    let section = match name {
        "Historical" => LibrarySection::Historical,
        "Gameplay" => LibrarySection::Gameplay,
        _ => return None,
    };
    let present = ui
        .library
        .current
        .as_ref()
        .and_then(|id| ui.library.catalog.article(id))
        .and_then(|article| article.sections.get(&section))
        .is_some_and(|document| !document.spans.is_empty());
    (!present).then(|| "No installed article section is available".to_owned())
}

/// Original picture choice for science/build and library still-frame controls.
pub(super) fn image(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    path: &str,
) -> Option<String> {
    let (id, still) = match view.kind {
        ManagerKind::Library if path == "TechStillShot" => (ui.library.current.as_deref()?, true),
        ManagerKind::Science if path == "CurrentResearch.AdvanceProgress.IconBorder.IconButton" => {
            (super::science::current_target(state)?, false)
        }
        ManagerKind::Build if path == "ItemImage.IconBorder.IconButton" => {
            (selected_description(view, state)?, true)
        }
        _ => return None,
    };
    let art = ui.library.catalog.art.get(id)?;
    if still {
        art.first_frame.clone().or_else(|| art.icon.clone())
    } else {
        art.icon.clone()
    }
}

/// Native word measurements are preferred; new numeric values use glyph advances.
fn text_width(metrics: &BTreeMap<String, u32>, text: &str) -> u32 {
    metrics.get(text).copied().unwrap_or_else(|| {
        text.chars()
            .map(|character| {
                metrics
                    .get(&character.to_string())
                    .copied()
                    .unwrap_or(6_u32)
            })
            .fold(0_u32, u32::saturating_add)
    })
}
