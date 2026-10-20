//! Source dropdown lifecycle and exact current owned-city selection.

use super::{ManagementUi, ManagerIntent, ManagerKind, ManagerView, ManagerWidget, data};
use crate::gameplay::assets::layout::LayoutNode;
use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::MenuRect;

/// Only source city pickers may acquire these presentation-only city callbacks.
fn city_path(kind: ManagerKind, path: &str) -> bool {
    match kind {
        ManagerKind::Build => path.starts_with("SingleCityChooser."),
        ManagerKind::City => path.starts_with("CityList."),
        _ => false,
    }
}

/// Exact source city dropdown roots; arbitrary path-like keys are not choosers.
pub(super) fn valid_path(kind: ManagerKind, path: &str) -> bool {
    city_path(kind, path) && matches!(path, "SingleCityChooser.Pulldown" | "CityList.Pulldown")
}

/// Identifies detached popup rows rather than treating them as buildable entries.
pub(super) fn city_list(kind: ManagerKind, path: &str) -> bool {
    city_path(kind, path) && path.ends_with(".Pulldown.window.listbox")
}

/// Fresh owned choices; visibility or a shared label cannot authorize a foreign city.
pub(super) fn choices(state: &GameState) -> Vec<(String, String)> {
    data::owned_city_ids(state)
        .into_iter()
        .map(|id| {
            let label = state
                .viewer
                .as_ref()
                .and_then(|viewer| viewer.cities.iter().find(|city| city.id == id))
                .map_or_else(|| id.clone(), data::city_name);
            (id, label)
        })
        .collect()
}

/// Returns None for unrelated controls; unavailable city controls retain a reason.
pub(super) fn binding(
    state: &GameState,
    view: &ManagerView,
    path: &str,
) -> Option<(Option<ManagerIntent>, Option<String>)> {
    if !city_path(view.kind, path) {
        return None;
    }
    let owned = choices(state);
    if path.ends_with(".Previous") || path.ends_with(".Next") {
        if owned.len() < 2_usize {
            return Some((
                None,
                Some("At least two owned cities are required to cycle".to_owned()),
            ));
        }
        let Some(current) = owned
            .iter()
            .position(|entry| Some(entry.0.as_str()) == view.city_id.as_deref())
        else {
            return Some((None, Some("Choose a currently owned city first".to_owned())));
        };
        let index = if path.ends_with(".Next") {
            current
                .saturating_add(1_usize)
                .checked_rem(owned.len())
                .unwrap_or(0_usize)
        } else if current == 0_usize {
            owned.len().saturating_sub(1_usize)
        } else {
            current.saturating_sub(1_usize)
        };
        return Some((
            Some(ManagerIntent::Select(
                "city".to_owned(),
                owned[index].0.clone(),
            )),
            None,
        ));
    }
    let selector_path = path.strip_suffix(".autobutton").unwrap_or(path);
    if selector_path.ends_with(".Pulldown") {
        if owned.is_empty() {
            return Some((
                None,
                Some("No currently owned city is available".to_owned()),
            ));
        }
        return Some((
            Some(ManagerIntent::Select(
                "dropdown".to_owned(),
                if view
                    .selections
                    .get("dropdown")
                    .is_some_and(|open| open == selector_path)
                {
                    String::new()
                } else {
                    selector_path.to_owned()
                },
            )),
            None,
        ));
    }
    None
}

/// Reports actual popup dismissal instead of claiming an absent popup changed.
pub(super) fn dismiss(view: &mut ManagerView) -> bool {
    let changed = view.selections.remove("dropdown").is_some();
    view.selections.remove("dropdown-highlight");
    changed
}

/// Local city changes invalidate dependent presentation selections, never game facts.
pub(super) fn change_city(view: &mut ManagerView, id: &str) {
    if view.city_id.as_deref() != Some(id) {
        for dependent in ["queue", "buildable", "description"] {
            view.selections.remove(dependent);
        }
        view.scroll.clear();
    }
    view.city_id = Some(id.to_owned());
    view.selections.remove("dropdown");
    view.selections.remove("dropdown-highlight");
}

/// Detached popup windows are not descendants painted while a dropdown is closed.
pub(super) fn is_dropdown(node: &LayoutNode) -> bool {
    node.field("objecttype")
        .is_some_and(|kind| kind.eq_ignore_ascii_case("ctp2_DropDown"))
}

/// Caption appears once, after the resized source button, as `DrawSelectedItem` does.
pub(super) fn caption(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    path: &str,
    rect: MenuRect,
) -> Option<ManagerWidget> {
    if !city_path(view.kind, path) {
        return None;
    }
    let city = data::city(state, view)?;
    let item = ui.row_templates.get("cw_CityItem")?;
    let label = item.children.first()?;
    let height = label
        .integer("heightpix")
        .and_then(|height| u32::try_from(height).ok())
        .unwrap_or(22_u32)
        .min(rect.height);
    let mut node = label.clone();
    node.children.clear();
    Some(ManagerWidget {
        id: format!("{path}/selected"),
        label: data::city_name(city),
        rect: MenuRect {
            x: rect.x.saturating_add(2_i32),
            y: rect.y.saturating_add(2_i32),
            width: rect.width.saturating_sub(4_u32),
            height,
        },
        node,
        intent: None,
        unavailable: None,
        selected: false,
    })
}

/// Fresh popup node/position, detached from its button's source child traversal.
pub(super) fn popup(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    widgets: &[ManagerWidget],
) -> Option<(LayoutNode, String, MenuRect)> {
    let path = view.selections.get("dropdown")?;
    if !city_path(view.kind, path) || !path.ends_with(".Pulldown") {
        return None;
    }
    let owner = widgets.iter().find(|widget| widget.id == *path)?;
    let mut source = ui.layouts.get(&view.kind)?;
    for part in path.split('.') {
        source = source.child(part)?;
    }
    let mut window = source.child("window")?.clone();
    let rows = choices(state).len();
    let row_height = ui
        .row_templates
        .get("cw_CityItem")
        .and_then(|item| item.integer("heightpix"))
        .and_then(|height| u32::try_from(height).ok())
        .unwrap_or(22_u32);
    let natural = u32::try_from(rows)
        .unwrap_or(u32::MAX)
        .saturating_mul(row_height);
    let height = natural.min(350_u32);
    if height == 0_u32 {
        return None;
    }
    let width = owner.rect.width;
    for (key, value) in [
        ("xpix", 0_i32),
        ("ypix", 0_i32),
        ("widthpix", i32::try_from(width).ok()?),
        ("heightpix", i32::try_from(height).ok()?),
    ] {
        window.fields.insert(key.to_owned(), value.to_string());
    }
    let list = window
        .children
        .iter_mut()
        .find(|child| child.name == "listbox")?;
    // The original dropdown constructs its ListBox directly, without the LDL
    // DoneInstantiatingThis border-expansion pass used by ordinary manager lists.
    for (key, value) in [
        ("xpix", 0_i32),
        ("ypix", 0_i32),
        (
            "widthpix",
            i32::try_from(width.saturating_sub(if natural > height { 15_u32 } else { 0_u32 }))
                .ok()?,
        ),
        ("heightpix", i32::try_from(height).ok()?),
        ("borderwidth", 0_i32),
        ("borderheight", 0_i32),
    ] {
        list.fields.insert(key.to_owned(), value.to_string());
    }
    list.fields
        .insert("alwaysranger".to_owned(), (natural > height).to_string());
    let y = owner
        .rect
        .y
        .saturating_add(i32::try_from(owner.rect.height).ok()?);
    Some((
        window,
        format!("{path}.window"),
        MenuRect {
            x: owner.rect.x,
            y: y.min(600_i32.saturating_sub(i32::try_from(height).ok()?))
                .max(0_i32),
            width,
            height,
        },
    ))
}
