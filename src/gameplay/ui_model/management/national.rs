//! National source revision and effective-accounting boundaries, never a city simulator.
use super::{
    ManagementUi, ManagerCommand, ManagerIntent, ManagerKind, ManagerView, data,
    fidelity::{node, set},
};
use crate::gameplay::{
    GameplayError, asset_error,
    assets::layout::{LayoutBook, LayoutNode},
    model::GameState,
    server::ServerCityProjection,
};
use alloc::collections::{BTreeMap, BTreeSet};
use serde_json::Value;
use std::{fs::File, io::BufReader};
use unciv::ZfsFile;
/// Missing/available reference artwork is tested without changing installed files.
#[cfg(test)]
mod art_tests {
    use super::{node, optional_header_art};
    use crate::gameplay::assets::layout::LayoutBook;
    use alloc::collections::BTreeSet;
    #[test]
    fn optional_reference_images_preserve_real_art_when_present_and_explain_missing_semantics() {
        let book=LayoutBook::parse(&[r#"Root { TabGroup { Tab1 { TabPanel { ResourceList { header { switch9 { string image0 "upic26.tga" } switch10 { string image0 "upic27.tga" } } } } } } }"#]).expect("in-memory source hierarchy");
        let mut missing = book.resolve("Root").expect("resolved fixture");
        optional_header_art(&mut missing, Some(&BTreeSet::new()));
        let conversion = node(
            &mut missing,
            "TabGroup.Tab1.TabPanel.ResourceList.header.switch9",
        )
        .expect("header");
        assert_eq!(
            conversion.field("text"),
            Some("Conv"),
            "honest semantic fallback, not wrong copyrighted art"
        );
        assert_eq!(
            conversion.field("reference-art-unavailable"),
            Some("upic26.rim"),
            "parity blocker is explicit"
        );
        assert!(
            conversion.field("image0").is_none(),
            "renderer never loads absent optional art"
        );
        let mut available = book.resolve("Root").expect("source clone");
        let names = BTreeSet::from(["upic26.rim".to_owned(), "upic27.rim".to_owned()]);
        optional_header_art(&mut available, Some(&names));
        let original = node(
            &mut available,
            "TabGroup.Tab1.TabPanel.ResourceList.header.switch9",
        )
        .expect("original image");
        assert_eq!(
            original.field("image0"),
            Some("upic26.tga"),
            "actual installed reference art is used unchanged when available"
        );
        assert!(
            original.field("reference-art-unavailable").is_none(),
            "no false missing-art claim"
        );
    }
}

/// Original Resources pane.
pub(super) const RESOURCE: &str = "TabGroup.Tab1.TabPanel.ResourceList";
/// Original Status pane, sharing exact city selection.
pub(super) const STATUS: &str = "TabGroup.Tab2.TabPanel.StatusList";
/// Original Specialists pane, not a second resource list.
pub(super) const SPECIALISTS: &str = "TabGroup.Tab3.TabPanel.SpecialistList";

/// Restores the original reference's three-tab/eleven-resource-header revision.
pub(super) fn configure(root: &mut LayoutNode, book: &LayoutBook) -> Result<(), GameplayError> {
    let standard = book.resolve("CTP2_3TAB_DIALOG_WINDOW")?;
    let group = node(root, "TabGroup")
        .ok_or_else(|| asset_error("missing National tab group".to_owned()))?;
    if group.child("Tab3").is_none() {
        let mut specialists = group
            .child("Tab1")
            .cloned()
            .ok_or_else(|| asset_error("missing National resource tab".to_owned()))?;
        "Tab3".clone_into(&mut specialists.name);
        let list = node(&mut specialists, "TabPanel.ResourceList")
            .ok_or_else(|| asset_error("missing National resource list".to_owned()))?;
        "SpecialistList".clone_into(&mut list.name);
        group.children.push(specialists);
    }
    for (tab, key) in [
        ("Tab1", "str_ldl_ND_RESOURCE_TAB"),
        ("Tab2", "str_ldl_ND_STATUS_TAB"),
        ("Tab3", "str_ldl_CityWinSpecialistTab"),
    ] {
        let mut button = standard
            .child("TabGroup")
            .and_then(|standard_group| standard_group.child(tab))
            .and_then(|tab| tab.child("TabButton"))
            .cloned()
            .ok_or_else(|| asset_error("missing original three-tab art".to_owned()))?;
        button.fields.insert("text".to_owned(), key.to_owned());
        let target = node(root, &format!("TabGroup.{tab}.TabButton"))
            .ok_or_else(|| asset_error("missing National tab button".to_owned()))?;
        *target = button;
    }
    resource_headers(root)?;
    set(
        root,
        &format!("{RESOURCE}.header.switch1"),
        &[("text", "Pop.")],
    );
    optional_header_art(root, installed_images(book)?.as_ref());
    specialist_headers(root)?;
    for (path, height) in [(RESOURCE, "238"), (STATUS, "210"), (SPECIALISTS, "238")] {
        set(
            root,
            path,
            &[
                ("xpix", "18"),
                ("ypix", "37"),
                ("widthpix", "458"),
                ("heightpix", height),
            ],
        );
    }
    set(
        root,
        &format!("{STATUS}.header.switch2"),
        &[("widthpix", "60")],
    );
    set(
        root,
        &format!("{STATUS}.header.switch3"),
        &[("widthpix", "120")],
    );
    for (name, template, text) in [
        (
            "BuildQueueButton",
            "CTP2_POSITION_13_BUTTON",
            "str_ldl_ND_BUILD_QUEUE",
        ),
        (
            "CityManagerButton",
            "CTP2_POSITION_23_BUTTON",
            "str_ldl_CityManager",
        ),
        (
            "DisbandButton",
            "CTP2_POSITION_33_BUTTON",
            "str_ldl_ND_DISBAND",
        ),
    ] {
        let mut button = book.resolve(template)?;
        name.clone_into(&mut button.name);
        button.fields.insert("text".to_owned(), text.to_owned());
        if let Some(existing) = root.children.iter_mut().find(|child| child.name == name) {
            if let Some(status) = existing.field("statustext") {
                button
                    .fields
                    .insert("statustext".to_owned(), status.to_owned());
            }
            *existing = button;
        } else {
            root.children.insert(0, button);
        }
    }
    Ok(())
}
/// Clone installed styles only; icon IDs and widths are source revision properties.
fn resource_headers(root: &mut LayoutNode) -> Result<(), GameplayError> {
    let header = node(root, &format!("{RESOURCE}.header"))
        .ok_or_else(|| asset_error("missing National resource headers".to_owned()))?;
    let style = header
        .children
        .last()
        .cloned()
        .ok_or_else(|| asset_error("empty National resource header".to_owned()))?;
    for index in 9_u8..11_u8 {
        if header.child(&format!("switch{index}")).is_none() {
            let mut column = style.clone();
            column.name = format!("switch{index}");
            for field in ["statustext", "tipwindow"] {
                column.fields.remove(field);
            }
            header.children.push(column);
        }
    }
    for (index, column) in header.children.iter_mut().enumerate() {
        column.fields.insert(
            "widthpix".to_owned(),
            if index == 0_usize { "142" } else { "32" }.to_owned(),
        );
    }
    for (index, icon) in [(9_u8, "upic26.tga"), (10_u8, "upic27.tga")] {
        let path = format!("{RESOURCE}.header.switch{index}");
        set(
            root,
            &path,
            &[("image0", icon), ("image1", icon), ("image2", icon)],
        );
    }
    Ok(())
}
/// Read-only archive indexes are inspected once during manager construction.
/// Missing revision-specific images are distinct from corrupt/unreadable archives.
fn installed_images(book: &LayoutBook) -> Result<Option<BTreeSet<String>>, GameplayError> {
    let Some(root) = book.installation_root() else {
        return Ok(None);
    };
    let mut names = BTreeSet::new();
    for archive in [
        crate::gameplay::assets::PICTURE_ARCHIVE,
        "ctp2_data/default/graphics/pictures/pic555.zfs",
    ] {
        let path = root.join(archive);
        if !path
            .try_exists()
            .map_err(|error| asset_error(error.to_string()))?
        {
            continue;
        }
        let file = File::open(&path)
            .map_err(|error| asset_error(format!("{}: {error}", path.display())))?;
        let mut reader = BufReader::new(file);
        let index = ZfsFile::from_stream(&mut reader)
            .map_err(|error| asset_error(format!("{}: {error}", path.display())))?;
        names.extend(
            index
                .files
                .into_iter()
                .map(|entry| entry.name.to_ascii_lowercase()),
        );
    }
    Ok(Some(names))
}
/// Never copy missing newer icons, substitute misleading art, or crash startup.
/// Text preserves the column's meaning; its missing-art marker blocks parity claims.
fn optional_header_art(root: &mut LayoutNode, names: Option<&BTreeSet<String>>) {
    let Some(available) = names else {
        return;
    };
    for (column, image, label, description) in [
        (
            9_u8,
            "upic26.rim",
            "Conv",
            "Conversion gold (original reference icon unavailable in this installation)",
        ),
        (
            10_u8,
            "upic27.rim",
            "Fran",
            "Franchise production loss (original reference icon unavailable in this installation)",
        ),
    ] {
        if available.contains(image) {
            continue;
        }
        if let Some(header) = node(root, &format!("{RESOURCE}.header.switch{column}")) {
            for field in ["image0", "image1", "image2"] {
                header.fields.remove(field);
            }
            for (key, value) in [
                ("text", label),
                ("statustext", description),
                ("reference-art-unavailable", image),
            ] {
                header.fields.insert(key.to_owned(), value.to_owned());
            }
        }
    }
}
/// Specialist counts have their own original header icons and explicit columns.
fn specialist_headers(root: &mut LayoutNode) -> Result<(), GameplayError> {
    let header = node(root, &format!("{SPECIALISTS}.header"))
        .ok_or_else(|| asset_error("missing National specialist header".to_owned()))?;
    let name = header
        .children
        .first()
        .cloned()
        .ok_or_else(|| asset_error("empty National specialist header".to_owned()))?;
    let numeric = header
        .children
        .get(1)
        .cloned()
        .ok_or_else(|| asset_error("missing population header".to_owned()))?;
    let icon = header
        .children
        .get(2)
        .cloned()
        .ok_or_else(|| asset_error("missing source icon header".to_owned()))?;
    header.children.clear();
    for index in 0_u8..10_u8 {
        let mut column = if index == 0 {
            name.clone()
        } else if index == 1 {
            numeric.clone()
        } else {
            icon.clone()
        };
        column.name = format!("switch{index}");
        column.fields.insert(
            "widthpix".to_owned(),
            match index {
                0 => "161",
                1 => "58",
                _ => "30",
            }
            .to_owned(),
        );
        if index >= 2 {
            for field in ["text", "statustext", "tipwindow"] {
                column.fields.remove(field);
            }
        }
        header.children.push(column);
    }
    for (index, image) in (2_u8..10_u8).zip([
        "upic08.tga",
        "upc039.tga",
        "upic10.tga",
        "upic05.tga",
        "upic06.tga",
        "upic09.tga",
        "upic07.tga",
        "upc040.tga",
    ]) {
        set(
            root,
            &format!("{SPECIALISTS}.header.switch{index}"),
            &[("image0", image), ("image1", image), ("image2", image)],
        );
    }
    Ok(())
}
/// Runtime-created list items are cached once; no source reads in a frame.
pub(super) fn templates(
    items: &mut BTreeMap<&'static str, LayoutNode>,
) -> Result<(), GameplayError> {
    let resource = items
        .get_mut("NationalResourceListItem")
        .ok_or_else(|| asset_error("missing National resource item".to_owned()))?;
    let prototype = resource
        .children
        .first()
        .cloned()
        .ok_or_else(|| asset_error("empty National resource item".to_owned()))?;
    resource.children.clear();
    for index in 0_u8..11_u8 {
        let mut column = prototype.clone();
        column.name = format!("column{index}");
        resource.children.push(column);
    }
    let mut specialist = resource.clone();
    "NationalSpecialistListItem".clone_into(&mut specialist.name);
    specialist.children.truncate(10);
    items.insert("NationalSpecialistListItem", specialist);
    Ok(())
}
/// Exact own-only row lookup. Neither nearby ownership nor first-city fallback is used.
fn city<'state>(state: &'state GameState, id: &str) -> Option<&'state ServerCityProjection> {
    data::owned_city_ids(state)
        .iter()
        .any(|owned| owned == id)
        .then_some(())?;
    state
        .viewer
        .as_ref()?
        .cities
        .iter()
        .find(|city| city.id == id)
}
/// Complete v2 scalar/map omission is known zero; malformed/null/legacy is unavailable.
fn scalar(state: &GameState, facts: &Value, key: &str) -> Option<i64> {
    number(state, facts.get(key))
}
/// Current complete v2 projection documents omitted own-city scalar/map fields.
fn complete_v2(state: &GameState) -> bool {
    state
        .viewer
        .as_ref()
        .and_then(|viewer| viewer.interface.contract.as_ref())
        .is_some_and(|contract| contract.version == 2)
}
/// Omitted complete-v2 integers are zero; explicit null/invalid values never are.
fn number(state: &GameState, value: Option<&Value>) -> Option<i64> {
    value.map_or_else(|| complete_v2(state).then_some(0_i64), Value::as_i64)
}
/// Received sort keys never convert unavailable rows into zeroes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum SortKey {
    /// Numeric current resource/count/turn field.
    Number(i64),
    /// Localized names/priorities/queue captions, case-insensitive like the source.
    Text(String),
}
/// Local presentation order retains canonical city selection and no private facts.
pub(super) fn rows(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    path: &str,
) -> Vec<(String, String)> {
    if !matches!(path, RESOURCE | STATUS | SPECIALISTS) {
        return Vec::new();
    }
    let rows: Vec<_> = data::owned_city_ids(state)
        .into_iter()
        .map(|id| {
            let label = city(state, &id).map_or_else(|| "—".to_owned(), data::city_name);
            (id, label)
        })
        .collect();
    let Some((column, descending)) = view
        .selections
        .get(&format!("national-sort:{path}"))
        .and_then(|encoded| encoded.split_once(':'))
        .and_then(|(column, order)| Some((column.parse::<u8>().ok()?, order == "desc")))
    else {
        return rows;
    };
    let mut keyed: Vec<_> = rows
        .into_iter()
        .map(|(id, label)| {
            let raw = value(ui, state, path, &id, &format!("column{column}"));
            let key = if raw == "—" {
                None
            } else if column != 0 && (path != STATUS || column == 4) {
                raw.parse::<i64>().ok().map(SortKey::Number)
            } else {
                Some(SortKey::Text(raw.to_lowercase()))
            };
            (id, label, key)
        })
        .collect();
    keyed.sort_by(|left, right| {
        match (left.2.as_ref(), right.2.as_ref()) {
            (Some(left_key), Some(right_key)) => {
                if descending {
                    right_key.cmp(left_key)
                } else {
                    left_key.cmp(right_key)
                }
            }
            (Some(_), None) => core::cmp::Ordering::Less,
            (None, Some(_)) => core::cmp::Ordering::Greater,
            (None, None) => core::cmp::Ordering::Equal,
        }
        .then_with(|| left.0.cmp(&right.0))
    });
    keyed
        .into_iter()
        .map(|(id, label, _key)| (id, label))
        .collect()
}
/// Exact source header path/column, with independent ascending/descending per list.
fn sort_binding(view: &ManagerView, path: &str) -> Option<ManagerIntent> {
    let (list, suffix) = path.rsplit_once(".header.switch")?;
    let column = suffix.parse::<u8>().ok()?;
    let columns = match list {
        RESOURCE => 11_u8,
        STATUS => 5_u8,
        SPECIALISTS => 10_u8,
        _ => return None,
    };
    if column >= columns {
        return None;
    }
    let key = format!("national-sort:{list}");
    let ascending = format!("{column}:asc");
    let requested = if view.selections.get(&key) == Some(&ascending) {
        format!("{column}:desc")
    } else {
        ascending
    };
    Some(ManagerIntent::Select(key, requested))
}
/// Source rows show exact public facts, not effects of a local `CityWindow::Project`.
pub(super) fn value(
    ui: &ManagementUi,
    state: &GameState,
    path: &str,
    id: &str,
    name: &str,
) -> String {
    let Some(city) = city(state, id) else {
        return "—".to_owned();
    };
    let Some(column) = name
        .strip_prefix("column")
        .and_then(|index| index.parse::<u8>().ok())
    else {
        return String::new();
    };
    if column == 0 {
        return data::city_name(city);
    }
    if (path == RESOURCE || path == SPECIALISTS) && column == 1 {
        return if city.population > 0 || complete_v2(state) {
            city.population.to_string()
        } else {
            "—".to_owned()
        };
    }
    match path {
        RESOURCE => match column {
            2 => scalar(state, &city.facts, "happiness"),
            8 => scalar(state, &city.facts, "crime_percent"),
            _ => None,
        }
        .map_or_else(|| "—".to_owned(), |number| number.to_string()),
        STATUS => status_value(ui, state, city, column),
        SPECIALISTS => specialist(state, city, column)
            .map_or_else(|| "—".to_owned(), |count| count.to_string()),
        _ => "—".to_owned(),
    }
}
/// Source black/red warning palette uses actual canonical riots, not a guessed
/// happiness/food/pollution threshold or a local `CityWindow::Project` calculation.
pub(super) fn style(state: &GameState, id: &str, path: &str, column: &str, node: &mut LayoutNode) {
    let warning = path == RESOURCE
        && matches!(column, "column0" | "column2")
        && city(state, id).is_some_and(|city| city.facts["rioting"].as_bool() == Some(true));
    for (channel, value) in [
        ("fontcolorred", if warning { "255" } else { "0" }),
        ("fontcolorgreen", "0"),
        ("fontcolorblue", "0"),
    ] {
        node.fields.insert(channel.to_owned(), value.to_owned());
    }
}
/// Workers/slaves/combat policing are not inferred from population or visible stacks.
fn specialist(state: &GameState, city: &ServerCityProjection, column: u8) -> Option<i64> {
    let kind = match column {
        4 => "entertainer",
        5 => "farmer",
        6 => "laborer",
        7 => "merchant",
        8 => "scientist",
        _ => return None,
    };
    match city.facts.get("specialists") {
        Some(value) => number(state, value.as_object()?.get(kind)).filter(|count| *count >= 0),
        None => number(state, None),
    }
}
/// Queue content and current forecasts stay separate from production entry identity.
fn status_value(
    ui: &ManagementUi,
    state: &GameState,
    city: &ServerCityProjection,
    column: u8,
) -> String {
    match column {
        1 => city.governor.as_ref().map_or_else(
            || "—".to_owned(),
            |governor| {
                ui.book
                    .label(if governor.enabled {
                        "str_ldl_ND_ON"
                    } else {
                        "str_ldl_ND_OFF"
                    })
                    .to_owned()
            },
        ),
        2 => city.governor.as_ref().map_or_else(
            || "—".to_owned(),
            |governor| {
                if governor.enabled {
                    data::label(ui, &governor.priority)
                } else {
                    String::new()
                }
            },
        ),
        3 => queue_label(ui, state, city),
        4 if empty_queue(state, city) == Some(true) => ui.book.label("str_ldl_ND_NA").to_owned(),
        4 => state.viewer.as_ref().map_or_else(
            || "—".to_owned(),
            |viewer| crate::gameplay::ui_model::observed::production(city, viewer.revision),
        ),
        _ => String::new(),
    }
}
/// An empty complete queue is known N/A; null and missing legacy remain unknown.
fn empty_queue(state: &GameState, city: &ServerCityProjection) -> Option<bool> {
    city.facts.get("queue").map_or_else(
        || complete_v2(state).then_some(true),
        |queue| queue.as_array().map(Vec::is_empty),
    )
}
/// Explicit empty queue differs from absent legacy/null/invalid contents.
fn queue_label(ui: &ManagementUi, state: &GameState, city: &ServerCityProjection) -> String {
    match city.facts.get("queue") {
        Some(value) => value.as_array().map_or_else(
            || "—".to_owned(),
            |queue| {
                if queue.is_empty() {
                    ui.book.label("str_ldl_ND_EMPTY").to_owned()
                } else {
                    queue
                        .first()
                        .and_then(|item| item["id"].as_str())
                        .map_or_else(|| "—".to_owned(), |id| data::label(ui, id))
                }
            },
        ),
        None if complete_v2(state) => ui.book.label("str_ldl_ND_EMPTY").to_owned(),
        None => "—".to_owned(),
    }
}
/// Current single-city National operations reuse the same legal dialogs as City.
pub(super) fn binding(
    state: &GameState,
    view: &ManagerView,
    path: &str,
) -> Option<(Option<ManagerIntent>, Option<String>)> {
    if view.kind != ManagerKind::National {
        return None;
    }
    if let Some(intent) = sort_binding(view, path) {
        return Some((Some(intent), None));
    }
    let selected = view.selections.get("city").and_then(|id| city(state, id));
    if path == "TabGroup.Tab2.TabPanel.GovernorSelect.Toggle" {
        return Some(
            selected
                .and_then(|city| {
                    crate::gameplay::ui_model::operations::OperationDialog::toggle_governor(
                        state, &city.id,
                    )
                })
                .map_or_else(
                    || {
                        (
                            None,
                            Some("Select an owned city with an authorized mayor toggle".to_owned()),
                        )
                    },
                    |command| (Some(ManagerIntent::Submit(command)), None),
                ),
        );
    }
    if matches!(
        path,
        "TabGroup.Tab2.TabPanel.GovernorSelect.Pulldown"
            | "TabGroup.Tab2.TabPanel.GovernorSelect.Pulldown.autobutton"
    ) {
        return Some(
            selected
                .filter(|city| {
                    city.governor
                        .as_ref()
                        .is_some_and(|governor| governor.enabled)
                })
                .and_then(|city| {
                    crate::gameplay::ui_model::operations::OperationDialog::governor(
                        state, &city.id,
                    )
                })
                .map_or_else(
                    || {
                        (
                            None,
                            Some(
                                "Select an owned city with current pinned governor priorities"
                                    .to_owned(),
                            ),
                        )
                    },
                    |dialog| (Some(ManagerIntent::Dialog(Box::new(dialog))), None),
                ),
        );
    }
    if path == "TabGroup.Tab2.TabPanel.RushBuyButton" {
        let command = selected
            .filter(|city| affordable_quote(state, city))
            .and_then(|city| {
                state
                    .viewer
                    .as_ref()?
                    .interface
                    .legal_actions
                    .iter()
                    .find(|action| action.route == "rush-buy" && action.entity_id == city.id)
            })
            .and_then(|action| ManagerCommand::advertised(state, action));
        return Some(command.map_or_else(
            || {
                (
                    None,
                    Some(
                        "Select a currently authorized affordable owned-city rush purchase"
                            .to_owned(),
                    ),
                )
            },
            |command| (Some(ManagerIntent::Submit(command)), None),
        ));
    }
    None
}
/// A displayed current price is an additional UX gate, never authorization.
fn affordable_quote(state: &GameState, city: &ServerCityProjection) -> bool {
    state.viewer.as_ref().is_some_and(|viewer| {
        city.rush_buy_quote.as_ref().is_some_and(|quote| {
            quote.revision == viewer.revision
                && quote.status == "available"
                && quote
                    .cost
                    .zip(viewer.diplomacy.as_ref())
                    .is_some_and(|(cost, nation)| cost >= 0_i64 && cost <= i64::from(nation.gold))
        })
    })
}
/// Source dropdown caption belongs to the explicitly selected city, never a default.
pub(super) fn priority_label(ui: &ManagementUi, state: &GameState, view: &ManagerView) -> String {
    view.selections
        .get("city")
        .and_then(|id| city(state, id))
        .and_then(|city| city.governor.as_ref())
        .map_or_else(
            || "—".to_owned(),
            |governor| {
                if governor.enabled {
                    data::label(ui, &governor.priority)
                } else {
                    String::new()
                }
            },
        )
}
/// Selected-city status controls display only received current quote/priority facts.
pub(super) fn text(state: &GameState, view: &ManagerView, path: &str) -> Option<String> {
    if view.kind != ManagerKind::National {
        return None;
    }
    let selected = view.selections.get("city").and_then(|id| city(state, id));
    match path {
        "TabGroup.Tab2.TabPanel.GovernorSelect.Toggle" => Some(
            selected
                .and_then(|city| city.governor.as_ref())
                .map_or_else(
                    || "—".to_owned(),
                    |governor| {
                        if governor.enabled {
                            "X".to_owned()
                        } else {
                            String::new()
                        }
                    },
                ),
        ),
        "TabGroup.Tab2.TabPanel.RushBuyValue" => {
            Some(selected.zip(state.viewer.as_ref()).map_or_else(
                || "—".to_owned(),
                |(city, viewer)| crate::gameplay::ui_model::observed::quote(city, viewer.revision),
            ))
        }
        _ => None,
    }
}
