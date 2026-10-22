//! Original docked advisors. Strategic planner/private rankings are not simulated.
use super::{ManagementUi, ManagerIntent, ManagerKind, ManagerView, data};
use crate::gameplay::{
    GameplayError,
    assets::layout::{LayoutBook, LayoutNode},
    model::GameState,
    ui_model::MenuRect,
};
use alloc::collections::BTreeMap;

/// Resolved once, including source artwork and geometry; never frame-time file I/O.
pub(super) fn templates(
    book: &LayoutBook,
) -> Result<BTreeMap<ManagerKind, LayoutNode>, GameplayError> {
    [
        (ManagerKind::Unit, "UnitManagerAdviceWindow"),
        (ManagerKind::Trade, "TradeAdvice"),
    ]
    .into_iter()
    .map(|(kind, name)| {
        let mut root = book.resolve(name)?;
        root.fields.insert("xpix".to_owned(), "0".to_owned());
        root.fields.insert("ypix".to_owned(), "0".to_owned());
        Ok((kind, root))
    })
    .collect()
}
/// Source constructors position advice at main.X-advice.Width,main.Y and show it.
pub(super) fn window<'ui>(
    ui: &'ui ManagementUi,
    view: &ManagerView,
    main: MenuRect,
) -> Option<(&'ui LayoutNode, MenuRect)> {
    if view
        .selections
        .get("advisor-hidden")
        .is_some_and(|hidden| hidden == "true")
    {
        return None;
    }
    let root = ui.advisor_layouts.get(&view.kind)?;
    let width = u32::try_from(root.integer("widthpix")?).ok()?;
    let height = u32::try_from(root.integer("heightpix")?).ok()?;
    Some((
        root,
        MenuRect {
            x: main.x.saturating_sub(i32::try_from(width).ok()?),
            y: main.y,
            width,
            height,
        },
    ))
}
/// Source advice/upkeep callbacks alter presentation only, never authority.
pub(super) fn binding(view: &ManagerView, path: &str) -> Option<ManagerIntent> {
    let key = match (view.kind, path) {
        (ManagerKind::Unit, "AdviceButton") | (ManagerKind::Trade, "ShowAdviceButton") => {
            "advisor-hidden"
        }
        (ManagerKind::Unit, "Advisor.UpkeepButton") => "upkeep-total",
        _ => return None,
    };
    Some(ManagerIntent::Select(
        key.to_owned(),
        view.selections
            .get(key)
            .is_none_or(|value| value != "true")
            .to_string(),
    ))
}
/// Exact public canonical types, not guessed military categories or foreign counts.
pub(super) fn counts(state: &GameState) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for unit in data::owned_units(state) {
        let count = counts.entry(unit.type_id.clone()).or_insert(0_usize);
        *count = count.saturating_add(1);
    }
    counts
}
/// Stable own-only list identities with readable installed type labels.
pub(super) fn rows(ui: &ManagementUi, state: &GameState) -> Vec<(String, String)> {
    counts(state)
        .into_keys()
        .map(|id| {
            let name = data::label(ui, &id);
            (id, name)
        })
        .collect()
}

/// Source category-item columns use exact type grouping when category is unprojected.
pub(super) fn row_value(ui: &ManagementUi, state: &GameState, id: &str, name: &str) -> String {
    let counts = counts(state);
    let count = counts.get(id).copied();
    match name {
        "Type" => data::label(ui, id),
        "Total" => count.map_or_else(|| "—".to_owned(), |value| value.to_string()),
        "Percent" => count
            .and_then(|value| {
                value.checked_mul(100)?.checked_div(
                    counts
                        .values()
                        .try_fold(0_usize, |total, item| total.checked_add(*item))?,
                )
            })
            .map_or_else(|| "—".to_owned(), |value| format!("{value}%")),
        _ => String::new(),
    }
}
/// Dynamic advisor fields never fall back to source's static placeholder zeroes.
pub(super) fn text(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    path: &str,
) -> Option<String> {
    if !path.starts_with("Advisor.") {
        return None;
    }
    let value = match path {
        "Advisor.Advice" => {
            return Some("Strategic advice is unavailable in this viewer snapshot.".to_owned());
        }
        "Advisor.Available" => transport(state, "trade_transport_points")
            .zip(transport(state, "used_trade_transport_points"))
            .and_then(|(total, used)| total.checked_sub(used))
            .filter(|value| *value >= 0),
        "Advisor.InUse" => transport(state, "used_trade_transport_points"),
        "Advisor.Routes" => Some(i64::try_from(outgoing(state).len()).ok()?),
        "Advisor.Profit" => outgoing(state)
            .iter()
            .try_fold(0_i64, |sum, route| sum.checked_add(route["gold"].as_i64()?)),
        "Advisor.UpkeepButton" => {
            let total = upkeep(state);
            let absolute = view
                .selections
                .get("upkeep-total")
                .is_some_and(|value| value == "true");
            let key = if absolute {
                "str_ldl_UpkeepTotalFormat"
            } else {
                "str_ldl_UpkeepPercentFormat"
            };
            let value = total.map(|value| value.to_string());
            return Some(
                ui.book
                    .label(key)
                    .replace("%d", value.as_deref().unwrap_or("—"))
                    .replace("%%", "%"),
            );
        }
        _ => return None,
    };
    Some(value.map_or_else(|| "—".to_owned(), |value| value.to_string()))
}
/// Only routes originating in a fresh owned city enter outbound advisor totals.
fn outgoing(state: &GameState) -> Vec<&serde_json::Value> {
    let own = data::owned_city_ids(state);
    state.viewer.as_ref().map_or_else(Vec::new, |viewer| {
        viewer
            .interface
            .trade_routes
            .iter()
            .filter(|route| {
                route["source_city_id"]
                    .as_str()
                    .is_some_and(|id| own.iter().any(|city| city == id))
            })
            .collect()
    })
}
/// The v2 Go integer DTO has documented omitempty zero; legacy absence is unknown.
fn transport(state: &GameState, key: &str) -> Option<i64> {
    let viewer = state.viewer.as_ref()?;
    let fields = &viewer.diplomacy.as_ref()?.facts;
    fields.get(key).map_or_else(
        || {
            viewer
                .interface
                .contract
                .as_ref()
                .filter(|contract| contract.version == 2)
                .map(|_contract| 0_i64)
        },
        |value| value.as_i64().filter(|value| *value >= 0),
    )
}
/// Zero shield hunger/exemption proves zero support. Nonzero applied readiness
/// costs require server accounting, not gold upkeep or a duplicated economy model.
fn upkeep(state: &GameState) -> Option<i64> {
    let version = state
        .viewer
        .as_ref()?
        .interface
        .contract
        .as_ref()
        .map(|contract| contract.version);
    data::owned_units(state)
        .iter()
        .all(|unit| {
            let hunger = unit.facts.get("shield_hunger").map_or_else(
                || (version == Some(2)).then_some(0_i64),
                serde_json::Value::as_i64,
            );
            hunger == Some(0)
                || unit.facts["needs_no_support"].as_bool() == Some(true)
                || unit.facts["is_professional"].as_bool() == Some(true)
        })
        .then_some(0)
}
