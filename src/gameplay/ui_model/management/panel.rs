//! Source lower-panel body composition without commands or invented game facts.
//!
//! Source controls are cached with the managers; descriptors share native image,
//! font and clipping code. Missing viewer economy/turn estimates remain unknown.

use super::{
    ManagementUi, ManagerIntent, ManagerKind, ManagerView, ManagerWidget, data, lists, scrollbar,
    selectors, tree,
};
use crate::gameplay::assets::layout::LayoutNode;
use crate::gameplay::model::{CityId, GameState};
use crate::gameplay::server::ServerCityProjection;
use crate::gameplay::ui_model::{ControlPanelTab, InGameUiModel, MenuRect, TileCategory};
use alloc::collections::BTreeMap;

/// Panel coordinates are from `ControlTabPanel`/`CONTROL_PANEL_TAB_BACKGROUND`.
const BODY: MenuRect = MenuRect {
    x: 388_i32,
    y: 461_i32,
    width: 355_u32,
    height: 117_u32,
};

/// Selected original body only; unit single/stack composition remains specialized.
pub(super) fn widgets(
    management: &ManagementUi,
    state: &GameState,
    ui: &InGameUiModel,
) -> Vec<ManagerWidget> {
    let name = match ui.active_tab {
        ControlPanelTab::Domestic => "DomesticTab",
        ControlPanelTab::City => "CityTab",
        ControlPanelTab::Messages => "MessageTab",
        ControlPanelTab::TileImprovements => "TilesTab",
        ControlPanelTab::Unit => return Vec::new(),
    };
    let Some(body) = management
        .panel
        .child("ControlPanel")
        .and_then(|panel| panel.child("ControlTabPanel"))
        .and_then(|tabs| tabs.child(name))
        .and_then(|tab| tab.child("TabPanel"))
    else {
        return Vec::new();
    };
    let mut painter = Body {
        management,
        state,
        ui,
        city: selected_city(state, ui),
        widgets: Vec::new(),
    };
    for child in body.children.iter().rev() {
        painter.walk(child, &child.name, BODY);
    }
    if ui.active_tab == ControlPanelTab::TileImprovements {
        if let Some(source) = body.child("TerraformLandButton") {
            let mut ocean = source.clone();
            "TerraformOceanButton".clone_into(&mut ocean.name);
            ocean.fields.insert("ypix".to_owned(), "97".to_owned());
            ocean
                .fields
                .insert("text".to_owned(), "Terraform Ocean".to_owned());
            painter.walk(&ocean, &ocean.name, BODY);
        }
    }
    painter.widgets
}

/// Only fresh owned city identities may populate the City tab.
fn selected_city<'state>(
    state: &'state GameState,
    ui: &InGameUiModel,
) -> Option<&'state ServerCityProjection> {
    let owned = data::owned_city_ids(state);
    let selected = ui.selected_city.map_or_else(
        || owned.first(),
        |id| {
            owned
                .iter()
                .find(|owned| CityId::from_server_id(owned) == id)
        },
    )?;
    state
        .viewer
        .as_ref()?
        .cities
        .iter()
        .find(|city| city.id == *selected)
}

/// Read-only context and output, separated from recursive geometry arguments.
struct Body<'view> {
    /// Session-cached original source controls and article art.
    management: &'view ManagementUi,
    /// Current authoritative receipt.
    state: &'view GameState,
    /// Local panel/city selection only.
    ui: &'view InGameUiModel,
    /// Fresh owned city, never a remembered/foreign caption.
    city: Option<&'view ServerCityProjection>,
    /// Tail-to-head source draw order.
    widgets: Vec<ManagerWidget>,
}

impl Body<'_> {
    /// Flattens one visible source subtree, respecting dropdown/list lifecycles.
    fn walk(&mut self, source: &LayoutNode, path: &str, parent: MenuRect) {
        if self.hidden(path) {
            return;
        }
        let mut style = source.clone();
        style.children.clear();
        if source.field("text").is_some() && source.field("heightpix").is_none() {
            style.fields.insert("heightpix".to_owned(), "16".to_owned());
            style
                .fields
                .insert("vertcenter".to_owned(), "false".to_owned());
        }
        let mut rect = tree::child_rect(&style, parent);
        if selectors::is_dropdown(source) {
            rect.height = 29_u32;
            self.dropdown(source, path, rect);
            return;
        }
        if let Some(art) = self.art(path) {
            style.fields.insert("image40".to_owned(), art);
            style
                .fields
                .insert("imageblttype40".to_owned(), "stretch".to_owned());
        }
        let label = self.caption(path).unwrap_or_else(|| {
            source.field("text").map_or_else(String::new, |key| {
                self.management.book.label(key).to_owned()
            })
        });
        let frame = if lists::is_list(source) {
            lists::frame(source, rect)
        } else {
            rect
        };
        if path == "tiLandButtonBank.b11" && self.ui.tile_category == TileCategory::Land {
            // Static original palette art is not a declaration of affordability.
            style
                .fields
                .insert("image40".to_owned(), "upti01.tga".to_owned());
            style
                .fields
                .insert("imageblttype40".to_owned(), "stretch".to_owned());
        }
        let category = [
            TileCategory::Land,
            TileCategory::Ocean,
            TileCategory::Special,
            TileCategory::TerraformLand,
            TileCategory::TerraformOcean,
        ]
        .into_iter()
        .find(|category| category.control().0 == path);
        self.widgets.push(ManagerWidget {
            id: path.to_owned(),
            label,
            rect: frame,
            node: style,
            intent: category
                .map(|category| {
                    ManagerIntent::Select(
                        "tile-category".to_owned(),
                        category.control().0.to_owned(),
                    )
                })
                .or_else(|| city_intent(self.state, self.city, path)),
            unavailable: None,
            selected: category == Some(self.ui.tile_category)
                || (path.ends_with("GovernorSelect.Toggle")
                    && self
                        .city
                        .and_then(|city| city.governor.as_ref())
                        .is_some_and(|governor| governor.enabled)),
        });
        if lists::is_list(source) {
            let view = ManagerView {
                kind: ManagerKind::City,
                city_id: None,
                selections: BTreeMap::new(),
                scroll: BTreeMap::from([(path.to_owned(), self.ui.messages.start)]),
            };
            let start = self.widgets.len();
            scrollbar::append(
                self.management,
                self.state,
                &view,
                source,
                path,
                rect,
                &mut self.widgets,
            );
            if self.ui.active_tab != ControlPanelTab::Messages {
                for widget in &mut self.widgets[start..] {
                    widget.intent = None;
                }
            }
            return;
        }
        for child in source.children.iter().rev() {
            self.walk(child, &format!("{path}.{}", child.name), rect);
        }
    }

    /// Exactly one tile bank is visible, not four overpainted sets of controls.
    fn hidden(&self, path: &str) -> bool {
        self.ui.active_tab == ControlPanelTab::TileImprovements
            && [
                "tiOceanButtonBank",
                "tiSpecialButtonBank",
                "tfLandButtonBank",
            ]
            .iter()
            .any(|bank| path.starts_with(bank))
    }

    /// The dropdown paints its five-part button and one caption, never its popup.
    fn dropdown(&mut self, source: &LayoutNode, path: &str, rect: MenuRect) {
        if let Some(button) = source.child("autobutton") {
            let mut style = button.clone();
            style.children.clear();
            self.widgets.push(ManagerWidget {
                id: format!("{path}.autobutton"),
                label: String::new(),
                rect,
                node: style,
                intent: city_intent(self.state, self.city, path),
                unavailable: None,
                selected: false,
            });
        }
        if matches!(path, "CitySelect.Pulldown" | "GovernorSelect.Pulldown") {
            if let Some(city) = self.city {
                let mut style = LayoutNode::default();
                style
                    .fields
                    .insert("fontname".to_owned(), "timesbd.ttf".to_owned());
                self.widgets.push(ManagerWidget {
                    id: format!("{path}/selected"),
                    label: if path == "GovernorSelect.Pulldown" {
                        city.governor.as_ref().map_or_else(
                            || "—".to_owned(),
                            |governor| data::label(self.management, &governor.priority),
                        )
                    } else {
                        data::city_name(city)
                    },
                    rect: MenuRect {
                        x: rect.x.saturating_add(2_i32),
                        y: rect.y.saturating_add(2_i32),
                        width: rect.width.saturating_sub(18_u32),
                        height: 20_u32,
                    },
                    node: style,
                    intent: None,
                    unavailable: None,
                    selected: false,
                });
            }
        }
    }

    /// Current production/research topic uses installed picture associations only.
    fn topic(&self, path: &str) -> Option<&str> {
        if path.starts_with("AdvanceProgress") {
            return self
                .state
                .viewer
                .as_ref()?
                .interface
                .research
                .as_ref()
                .map(|research| research.target_id.as_str())
                .filter(|id| !id.is_empty());
        }
        if path.starts_with("BuildProgress") {
            return self.city?.facts["queue"].as_array()?.first()?["id"].as_str();
        }
        None
    }

    /// First-frame portraits, not tiny unit buttons stretched into a large picture.
    fn art(&self, path: &str) -> Option<String> {
        if !path.ends_with(".IconBorder.IconButton") {
            return None;
        }
        let art = self.management.library.catalog.art.get(self.topic(path)?)?;
        art.first_frame.clone().or_else(|| art.icon.clone())
    }

    /// Source labels are static; values come only from currently viewer-owned data.
    fn caption(&self, path: &str) -> Option<String> {
        if path.ends_with(".Title") {
            return Some(
                self.topic(path)
                    .map_or_else(String::new, |id| data::label(self.management, id)),
            );
        }
        if path.ends_with(".RadialButton") {
            return Some(self.state.viewer.as_ref().map_or_else(
                || "—".to_owned(),
                |viewer| {
                    if path.starts_with("BuildProgress") {
                        self.city.map_or_else(
                            || "—".to_owned(),
                            |city| {
                                crate::gameplay::ui_model::observed::production(
                                    city,
                                    viewer.revision,
                                )
                            },
                        )
                    } else {
                        viewer
                            .interface
                            .research
                            .as_ref()
                            .and_then(|research| research.forecast.as_ref())
                            .map_or_else(
                                || "—".to_owned(),
                                |forecast| forecast.label(viewer.revision),
                            )
                    }
                },
            ));
        }
        if self.ui.active_tab == ControlPanelTab::Domestic {
            return domestic(self.management, self.state, path);
        }
        if path == "RushBuyCost" {
            return Some(self.city.zip(self.state.viewer.as_ref()).map_or_else(
                || "—".to_owned(),
                |(city, viewer)| crate::gameplay::ui_model::observed::quote(city, viewer.revision),
            ));
        }
        None
    }
}

/// Source city panel controls use the selected viewer city and exact legal payload.
fn city_intent(
    state: &GameState,
    city: Option<&ServerCityProjection>,
    path: &str,
) -> Option<ManagerIntent> {
    let city = city?;
    if path == "GovernorSelect.Toggle" {
        return crate::gameplay::ui_model::operations::OperationDialog::toggle_governor(
            state, &city.id,
        )
        .map(ManagerIntent::Submit);
    }
    if path.starts_with("GovernorSelect") {
        return crate::gameplay::ui_model::operations::OperationDialog::governor(state, &city.id)
            .map(|dialog| ManagerIntent::Dialog(Box::new(dialog)));
    }
    if path == "CitySelect.Pulldown" {
        return Some(ManagerIntent::ForCity(ManagerKind::City, city.id.clone()));
    }
    if path.contains("IconButton") {
        return Some(ManagerIntent::ForCity(ManagerKind::Build, city.id.clone()));
    }
    if path == "RushBuyButton" {
        let action = state
            .viewer
            .as_ref()?
            .interface
            .legal_actions
            .iter()
            .find(|action| action.route == "rush-buy" && action.entity_id == city.id)?;
        return super::ManagerCommand::advertised(state, action).map(ManagerIntent::Submit);
    }
    None
}

/// Never use visible foreign population or invented research/pollution forecasts.
fn domestic(ui: &ManagementUi, state: &GameState, path: &str) -> Option<String> {
    if !path.ends_with("Value") {
        return None;
    }
    let owned = data::owned_city_ids(state);
    let facts = state
        .viewer
        .as_ref()
        .and_then(|viewer| viewer.diplomacy.as_ref())
        .map(|diplomacy| &diplomacy.facts);
    Some(match path {
        "CitiesValue" => owned.len().to_string(),
        "PopulationValue" => state.viewer.as_ref().map_or_else(
            || "—".to_owned(),
            |viewer| {
                viewer
                    .cities
                    .iter()
                    .filter(|city| owned.contains(&city.id))
                    .fold(0_u64, |total, city| {
                        total.saturating_add(u64::from(city.population))
                    })
                    .to_string()
            },
        ),
        "GovernmentValue" => facts
            .and_then(|facts| facts["government_id"].as_str())
            .map_or_else(|| "—".to_owned(), |id| data::label(ui, id)),
        "ScienceValue" | "PublicWorksValue" => facts
            .and_then(|facts| {
                facts[if path == "ScienceValue" {
                    "science_percent"
                } else {
                    "public_works_percent"
                }]
                .as_i64()
            })
            .map_or_else(|| "—".to_owned(), |value| format!("{value}%")),
        _ => "—".to_owned(),
    })
}
