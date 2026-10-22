//! Receipt-safe selections. Removed identities never silently become another row.
use super::{ManagementUi, ManagerKind, ManagerView, data};
use crate::gameplay::model::GameState;

/// Keeps valid current and return-window selections; invalid confirmations stay stale.
pub(super) fn apply(ui: &mut ManagementUi, state: &GameState) {
    if let Some(view) = ui.view.as_mut() {
        view_state(view, state);
    }
    for view in &mut ui.parents {
        view_state(view, state);
    }
    if ui.view.as_ref().is_some_and(|view| {
        view.kind == ManagerKind::DiplomacyDetails && !view.selections.contains_key("contact")
    }) {
        ui.close();
    }
    ui.value_drag = None;
    ui.scroll_drag = None;
}
/// Drops only identities that disappeared, retaining independent content/entry IDs.
fn view_state(view: &mut ManagerView, state: &GameState) {
    if view.kind == ManagerKind::Trade {
        super::trade::reconcile(view, state);
    }
    super::diplomacy::reconcile(view, state);
    let owned = data::owned_city_ids(state);
    if view.city_id.as_ref().is_some_and(|id| !owned.contains(id)) {
        view.city_id = None;
        view.selections.remove("queue");
        view.selections.remove("description");
    }
    if view
        .selections
        .get("city")
        .is_some_and(|id| !owned.contains(id))
    {
        view.selections.remove("city");
    }
    if let Some(viewer) = state.viewer.as_ref() {
        let queue_valid = view.selections.get("queue").is_none_or(|id| {
            view.city_id
                .as_ref()
                .and_then(|city_id| viewer.cities.iter().find(|city| city.id == *city_id))
                .and_then(|city| city.facts["queue"].as_array())
                .is_some_and(|queue| {
                    queue.iter().any(|entry| {
                        entry["entry_id"].as_str().or_else(|| entry["id"].as_str()) == Some(id)
                    })
                })
        });
        if !queue_valid {
            if view.selections.get("description") == view.selections.get("queue") {
                view.selections.remove("description");
            }
            view.selections.remove("queue");
        }
        if view.kind == ManagerKind::Research
            && view.selections.get("advance").is_some_and(|id| {
                viewer
                    .interface
                    .research
                    .as_ref()
                    .is_none_or(|research| !research.available.contains(id))
            })
        {
            view.selections.remove("advance");
        }
        let units = data::owned_units(state);
        if view
            .selections
            .get("unit")
            .is_some_and(|id| !units.iter().any(|unit| unit.id == *id))
        {
            view.selections.remove("unit");
        }
        if view
            .selections
            .get("unit-type")
            .is_some_and(|id| !units.iter().any(|unit| unit.type_id == *id))
        {
            view.selections.remove("unit-type");
        }
    }
}
