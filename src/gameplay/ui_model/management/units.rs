//! Source unit statistics/tactical columns and explicit, current disband choices.
use super::{ManagementUi, ManagerView, ManagerWidget, advisors, data};
use crate::gameplay::assets::layout::LayoutNode;
use crate::gameplay::{
    model::GameState,
    server::ServerUnitProjection,
    ui_model::{MenuRect, operations::OperationDialog},
};

/// Current received readiness and total, never static LDL placeholders.
pub(super) fn text(ui: &ManagementUi, state: &GameState, path: &str) -> Option<String> {
    match path {
        "TotalUnits" => Some(data::owned_units(state).len().to_string()),
        "State" => Some(
            state
                .viewer
                .as_ref()
                .and_then(|viewer| viewer.diplomacy.as_ref())
                .and_then(|diplomacy| diplomacy.facts["readiness"].as_str())
                .and_then(|value| match value {
                    "peace" => Some("str_ldl_READINESS_LEVEL_PEACE"),
                    "alert" => Some("str_ldl_READINESS_LEVEL_ALERT"),
                    "war" => Some("str_ldl_READINESS_LEVEL_WAR"),
                    _ => None,
                })
                .map_or_else(|| "—".to_owned(), |key| ui.book.label(key).to_owned()),
        ),
        _ => None,
    }
}
/// Source small icon is copied; only the documented missing-small-icon fallback stretches.
pub(super) fn icon(
    ui: &ManagementUi,
    state: &GameState,
    path: &str,
    id: &str,
) -> Option<(String, bool)> {
    let id = if path.contains("Tactical") {
        actor(state, id)?.type_id.as_str()
    } else {
        id
    };
    let art = ui.library.catalog.art.get(id)?;
    art.small_icon
        .as_ref()
        .map(|name| (name.clone(), false))
        .or_else(|| art.icon.as_ref().map(|name| (name.clone(), true)))
}
/// Own row contents use exact actor/type identities and public pinned records only.
pub(super) fn value(
    ui: &ManagementUi,
    state: &GameState,
    path: &str,
    id: &str,
    name: &str,
) -> String {
    if path.contains("Tactical") {
        return tactical(ui, state, id, name);
    }
    match name {
        "UnitName" => data::label(ui, id),
        "Count" => advisors::counts(state)
            .get(id)
            .map_or_else(|| "—".to_owned(), alloc::string::ToString::to_string),
        "UnitType" => definition(state, id)
            .and_then(|unit| unit["role"].as_str())
            .filter(|role| !role.is_empty())
            .map_or_else(|| data::label(ui, id), |role| data::label(ui, role)),
        "UnitIcon" => String::new(),
        _ => {
            let field = match name {
                "Support" => "shield_hunger",
                "A" => "attack",
                "R" => "ranged",
                "D" => "defense",
                "M" => "movement",
                _ => return String::new(),
            };
            let known = definition(state, id).and_then(|unit| {
                unit.get(field).map_or_else(
                    || (known_v2(state) && field != "movement").then_some(0_i64),
                    serde_json::Value::as_i64,
                )
            });
            known.map_or_else(|| "—".to_owned(), |value| value.to_string())
        }
    }
}
/// Tactical facts never replace an unprojected army name with an invented one.
fn tactical(ui: &ManagementUi, state: &GameState, id: &str, name: &str) -> String {
    let Some(unit) = actor(state, id) else {
        return "—".to_owned();
    };
    match name {
        "UnitName" => data::label(ui, &unit.type_id),
        "Health" => {
            if unit.facts["health"].as_u64().is_some()
                && unit.facts["max_health"]
                    .as_u64()
                    .is_some_and(|maximum| maximum > 0)
            {
                String::new()
            } else {
                "—".to_owned()
            }
        }
        "Army" => unit.facts["army_name"].as_str().unwrap_or("—").to_owned(),
        "Order" => {
            let moving = state.viewer.as_ref().is_some_and(|viewer| {
                viewer
                    .interface
                    .movement_orders
                    .iter()
                    .any(|order| order.unit_id == unit.id || order.unit_ids.contains(&unit.id))
            });
            let key = if unit.facts["asleep"].as_bool() == Some(true) {
                "UNIT_ORDER_SLEEP"
            } else if unit.facts["fortified"].as_bool() == Some(true) {
                "UNIT_ORDER_ENTRENCH"
            } else if moving {
                "UNIT_ORDER_MOVE"
            } else {
                "UNIT_ORDER_NONE"
            };
            ui.book.label(key).to_owned()
        }
        "Location" => {
            let city = state.viewer.as_ref().and_then(|viewer| {
                viewer
                    .cities
                    .iter()
                    .find(|city| city.tile_id == unit.tile_id && !city.stale)
            });
            if let Some(city) = city {
                return data::city_name(city);
            }
            ui.book
                .label(if unit.facts["in_space"].as_bool() == Some(true) {
                    "UNIT_LOCATION_IN_SPACE"
                } else {
                    "UNIT_LOCATION_IN_FIELD"
                })
                .to_owned()
        }
        _ => String::new(),
    }
}
/// Original black inset with fractional red/yellow/green health; unknown is not full.
pub(super) fn health(
    state: &GameState,
    id: &str,
    path: &str,
    bounds: MenuRect,
    widgets: &mut Vec<ManagerWidget>,
) {
    let source = actor(state, id).filter(|unit| unit.facts["health"].as_u64().is_some());
    let Some((filled, color)) =
        crate::gameplay::ui_model::stack::health_bar(source, bounds.width.saturating_sub(6))
    else {
        return;
    };
    let backdrop = MenuRect {
        x: bounds.x.saturating_add(2),
        y: bounds.y.saturating_add(2),
        width: bounds.width.saturating_sub(4),
        height: bounds.height.saturating_sub(4),
    };
    widgets.push(block(
        format!("{path}/{id}/health/background"),
        backdrop,
        (0, 0, 0),
    ));
    widgets.push(block(
        format!("{path}/{id}/health/value"),
        MenuRect {
            x: bounds.x.saturating_add(3),
            y: bounds.y.saturating_add(3),
            width: filled,
            height: bounds.height.saturating_sub(6),
        },
        color,
    ));
}
/// Primitive native pixels share the original column clip and do not own input.
fn block(id: String, rect: MenuRect, color: (u8, u8, u8)) -> ManagerWidget {
    let mut node = LayoutNode::default();
    for (key, value) in [
        ("primitive", "fill".to_owned()),
        ("fontcolorred", color.0.to_string()),
        ("fontcolorgreen", color.1.to_string()),
        ("fontcolorblue", color.2.to_string()),
    ] {
        node.fields.insert(key.to_owned(), value);
    }
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
/// Source confirmation carries only exact currently authorized own identities.
pub(super) fn disband(state: &GameState, view: &ManagerView) -> Option<OperationDialog> {
    let tactical = view
        .selections
        .get("tab:Tabs")
        .is_some_and(|tab| tab == "Tactical");
    let selected = view
        .selections
        .get(if tactical { "unit" } else { "unit-type" })?;
    let own = data::owned_units(state);
    let viewer = state.viewer.as_ref()?;
    let actions = viewer
        .interface
        .legal_actions
        .iter()
        .filter(|action| {
            action.route == "disband"
                && own.iter().any(|unit| {
                    unit.id == action.entity_id
                        && (unit.id == *selected || (!tactical && unit.type_id == *selected))
                })
        })
        .cloned();
    OperationDialog::actions(state, "Disband selected unit?", actions)
}
/// Finds a living owned actor; visible foreign metadata cannot enter these rows.
fn actor<'state>(state: &'state GameState, id: &str) -> Option<&'state ServerUnitProjection> {
    let source = state
        .viewer
        .as_ref()?
        .units
        .iter()
        .find(|unit| unit.id == id)?;
    state
        .units
        .iter()
        .any(|unit| {
            unit.alive
                && unit.owner == crate::gameplay::model::PlayerId::Human
                && unit.id == crate::gameplay::model::UnitId::from_server_id(id)
        })
        .then_some(source)
}
/// Full public pinned unit definition, never an installed gameplay-rule fallback.
fn definition<'state>(state: &'state GameState, id: &str) -> Option<&'state serde_json::Value> {
    state.viewer.as_ref()?.interface.rules.as_ref()?["units"]
        .as_array()?
        .iter()
        .find(|unit| unit["id"].as_str() == Some(id))
}
/// Versioned Go scalar omissions mean zero only in the explicit v2 DTO.
fn known_v2(state: &GameState) -> bool {
    state
        .viewer
        .as_ref()
        .and_then(|viewer| viewer.interface.contract.as_ref())
        .is_some_and(|contract| contract.version == 2)
}
