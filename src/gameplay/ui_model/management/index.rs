//! Community Library sort/index controls backed only by current public age metadata.

use super::{ManagementUi, data};
use crate::gameplay::GameplayError;
use crate::gameplay::assets::layout::{LayoutBook, LayoutNode};
use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::library::LibraryCategory;
use alloc::collections::BTreeMap;
use serde_json::Value;

/// Applies reference-version metadata from the public community greatlibrary.ldl;
/// no installed layout/art is modified or embedded. Source coordinates differ
/// from the earlier retail layout in the read-only installation.
pub(super) fn configure(root: &mut LayoutNode, book: &LayoutBook) -> Result<(), GameplayError> {
    for (name, values) in [
        ("SearchWord", vec![("widthpix", 136_i32)]),
        (
            "IndexSheet",
            vec![
                ("xpix", 103_i32),
                ("ypix", 113_i32),
                ("widthpix", 149_i32),
                ("heightpix", 388_i32),
            ],
        ),
        ("OkButton", vec![("xpix", 630_i32), ("ypix", 528_i32)]),
        (
            "SetGoalButton",
            vec![("xpix", 480_i32), ("ypix", 528_i32), ("heightpix", 24_i32)],
        ),
    ] {
        if let Some(node) = root.children.iter_mut().find(|node| node.name == name) {
            for (key, value) in values {
                node.fields.insert(key.to_owned(), value.to_string());
            }
        }
    }
    // The reference community GREAT_BOX adds CTP2_HEADER_FONT before the
    // ordinary hypertext template; the earlier installed layout does not.
    let header = book.resolve("CTP2_HEADER_FONT")?;
    for name in ["RequirementsText", "VariablesText"] {
        if let Some(node) = root.children.iter_mut().find(|node| node.name == name) {
            node.fields.extend(header.fields.clone());
        }
    }
    let mut button = book.resolve("CTP2_GENERIC_SIZABLE_TEXT_BUTTON")?;
    "SortByAgeButton".clone_into(&mut button.name);
    for (key, value) in [
        ("xpix", "107"),
        ("ypix", "528"),
        ("widthpix", "140"),
        ("heightpix", "20"),
        ("togglebutton", "true"),
        ("numberoflayers", "5"),
        ("layerupon4", "true"),
        ("image00", "upbt16aX.tga"),
        ("image10", "upbt16aU.tga"),
        ("image20", "upbt16aD.tga"),
        ("image30", "upbt16aH.tga"),
        ("image40", "upbt16aD.tga"),
        ("image41", "upbt01bD.tga"),
        ("image42", "upbt01cD.tga"),
        ("text", "str_ldl_SORT_BY_AGE"),
    ] {
        button.fields.insert(key.to_owned(), value.to_owned());
    }
    if let Some(existing) = root
        .children
        .iter_mut()
        .find(|node| node.name == button.name)
    {
        *existing = button;
    } else {
        root.children.push(button);
    }
    Ok(())
}

/// Matches the source-supported categories and never age-sorts search results.
pub(super) fn can_sort(ui: &ManagementUi, state: &GameState) -> bool {
    if ui.library.search_results.is_some()
        || !matches!(
            ui.library.category,
            LibraryCategory::Advances
                | LibraryCategory::Units
                | LibraryCategory::Buildings
                | LibraryCategory::Wonders
                | LibraryCategory::Governments
                | LibraryCategory::TileImprovements
        )
    {
        return false;
    }
    rules(state)
        .and_then(|rules| rules["ages"].as_array())
        .is_some_and(|ages| {
            ages.iter()
                .any(|age| age["id"].as_str().is_some() && age["index"].as_i64().is_some())
        })
}

/// Alphabetical reference topics, or grouped ages only when the contract is present.
pub(super) fn rows(ui: &ManagementUi, state: &GameState) -> Vec<(String, String)> {
    let entries = ui.library.entries(|id| data::label(ui, id));
    if !ui.library.sort_by_age || !can_sort(ui, state) {
        return entries;
    }
    let Some(rules) = rules(state) else {
        return entries;
    };
    let ages: BTreeMap<_, _> = rules["ages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|age| Some((age["id"].as_str()?, age["index"].as_i64()?)))
        .collect();
    let mut sorted: Vec<_> = entries
        .into_iter()
        .map(|entry| {
            let age = topic_age(rules, &ages, &entry.0);
            (entry, age)
        })
        .collect();
    sorted.sort_by_cached_key(|entry| {
        (
            entry.1.is_none(),
            entry.1.map_or(i64::MAX, |age| age.1),
            entry.1.map_or("", |age| age.0).to_owned(),
            entry.0.1.to_lowercase(),
        )
    });
    let mut result = Vec::new();
    let mut previous = None;
    for (entry, age) in sorted {
        let age_id = age.map_or("unknown", |age| age.0);
        if previous != Some(age_id) {
            if previous.is_some() {
                result.push((format!("@age-spacer/{}", result.len()), String::new()));
            }
            result.push((
                format!("@age/{age_id}"),
                if age.is_some() {
                    data::label(ui, age_id)
                } else {
                    "Age unavailable".to_owned()
                },
            ));
            previous = Some(age_id);
        }
        result.push(entry);
    }
    result
}

/// Exact viewer metadata, never numeric age data read from the installed game.
fn rules(state: &GameState) -> Option<&Value> {
    state.viewer.as_ref()?.interface.rules.as_ref()
}

/// Direct advance ages or the latest required enabling age for another category.
fn topic_age<'rules>(
    rules: &'rules Value,
    ages: &BTreeMap<&'rules str, i64>,
    id: &str,
) -> Option<(&'rules str, i64)> {
    if id.starts_with("ADVANCE_") {
        return advance_age(rules, ages, id);
    }
    let (category, single) = match LibraryCategory::for_topic(id)? {
        LibraryCategory::Units => ("units", None),
        LibraryCategory::Buildings | LibraryCategory::Wonders => ("production", None),
        LibraryCategory::Governments => ("governments", Some("enable_advance_id")),
        LibraryCategory::TileImprovements => ("terrain_improvements", Some("required_advance_id")),
        _ => return None,
    };
    let definition = rules[category]
        .as_array()?
        .iter()
        .find(|definition| definition["id"].as_str() == Some(id))?;
    let prerequisites: Vec<_> = single.map_or_else(
        || {
            definition["prerequisites"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect()
        },
        |field| {
            definition[field]
                .as_str()
                .filter(|advance_id| !advance_id.is_empty())
                .into_iter()
                .collect()
        },
    );
    if prerequisites.is_empty() {
        return ages
            .iter()
            .min_by_key(|age| *age.1)
            .map(|age| (*age.0, *age.1));
    }
    prerequisites
        .into_iter()
        .map(|advance_id| advance_age(rules, ages, advance_id))
        .collect::<Option<Vec<_>>>()?
        .into_iter()
        .max_by_key(|age| age.1)
}

/// Resolves only an explicitly published advance/age association.
fn advance_age<'rules>(
    rules: &'rules Value,
    ages: &BTreeMap<&'rules str, i64>,
    id: &str,
) -> Option<(&'rules str, i64)> {
    let definition = rules["advances"]
        .as_array()?
        .iter()
        .find(|definition| definition["id"].as_str() == Some(id))?;
    let age = definition["age_id"].as_str()?;
    Some((age, *ages.get(age)?))
}
