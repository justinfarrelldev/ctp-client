//! Original tall diplomacy table and viewer-safe relation columns.
use super::{ManagerKind, ManagerView, fidelity::set};
use crate::gameplay::{
    GameplayError,
    assets::layout::{LayoutBook, LayoutNode},
    model::GameState,
};
use alloc::collections::BTreeSet;
use serde_json::Value;
/// Source shared intelligence list; canonical contact ID is its row identity.
pub(super) const LIST: &str = "DiplomacyTabs.Intelligence.TabPanel.List";
/// Current public diplomacywindow.ldl reference revision, not an invented scale.
pub(super) fn configure(root: &mut LayoutNode, book: &LayoutBook) -> Result<(), GameplayError> {
    set(root, "", &[("widthpix", "601"), ("heightpix", "558")]);
    set(
        root,
        "Background",
        &[("widthpix", "601"), ("heightpix", "558")],
    );
    set(
        root,
        "DiplomacyTabs",
        &[("xpix", "38"), ("widthpix", "580")],
    );
    set(
        root,
        LIST,
        &[
            ("xpix", "21"),
            ("ypix", "20"),
            ("widthpix", "470"),
            ("heightpix", "396"),
        ],
    );
    for (name, x) in [
        ("DetailsButton", "14"),
        ("MessageButton", "118"),
        ("WarButton", "222"),
        ("EmbargoButton", "326"),
    ] {
        set(
            root,
            &format!("DiplomacyTabs.Intelligence.TabPanel.{name}"),
            &[("xpix", x), ("ypix", "432")],
        );
    }
    set(root, "CloseButton", &[("xpix", "468"), ("ypix", "485")]);
    optional_frame_art(root, super::reference_art::pictures(book)?.as_ref());
    Ok(())
}
/// The newer reference uses `dw_lb`/`dw_rb`, absent from retail archives. Keep
/// installed original column art tiled as an explicit usability fallback; never
/// pretend its extension is the missing reference art or request absent images.
fn optional_frame_art(root: &mut LayoutNode, available: Option<&BTreeSet<String>>) {
    let Some(names) = available else {
        return;
    };
    let Some(background) = super::fidelity::node(root, "Background") else {
        return;
    };
    let mut missing = Vec::new();
    background.fields.remove("reference-art-unavailable");
    for (index, name) in [(3_u8, "dw_lb"), (5_u8, "dw_rb")] {
        if names.contains(&format!("{name}.rim")) {
            background
                .fields
                .insert(format!("image0{index}"), format!("{name}.tga"));
            background.fields.remove(&format!("imagestretchy0{index}"));
        } else {
            missing.push(format!("{name}.rim"));
            background
                .fields
                .insert(format!("imagestretchy0{index}"), "true".to_owned());
        }
    }
    if !missing.is_empty() {
        background
            .fields
            .insert("reference-art-unavailable".to_owned(), missing.join(","));
    }
}
/// Optional reference extensions are injected without mutating installed assets.
#[cfg(test)]
mod art_tests;
/// Only projected current contacts are intelligence rows; visibility is not contact.
fn relation<'state>(state: &'state GameState, id: &str) -> Option<&'state Value> {
    state.viewer.as_ref()?.diplomacy.as_ref()?.facts["relations"]
        .as_array()?
        .iter()
        .find(|relation| relation["target_player_id"].as_str() == Some(id))
}
/// Server semantic labels remain text, not guessed original strength/regard icons.
fn caption(text: Option<&str>) -> String {
    let Some(value) = text.filter(|value| !value.is_empty()) else {
        return "—".to_owned();
    };
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return "—".to_owned();
    };
    let mut displayed = first.to_uppercase().collect::<String>();
    displayed.push_str(characters.as_str());
    displayed
}
/// Row values disclose only received relation facts. ID is explicitly an ID,
/// never a substituted civilization/city name or inferred flag/foreign rank.
pub(super) fn value(state: &GameState, id: &str, name: &str) -> String {
    let Some(facts) = relation(state, id) else {
        return "—".to_owned();
    };
    match name {
        "Nation" => format!("ID {id}"),
        "Regard" => caption(facts["regard"].as_str()),
        "Treaties" => {
            let treaty = facts["treaty"].as_str();
            let mut text = caption(treaty);
            if facts["trade"].as_bool() == Some(true) && !matches!(treaty, Some("trade" | "union"))
            {
                if text == "—" {
                    "Trade".clone_into(&mut text);
                } else {
                    text.push_str(", Trade");
                }
            }
            text
        }
        "Flag" | "Strength" | "Embassy" => "—".to_owned(),
        _ => String::new(),
    }
}
/// Removed contacts lose selection on receipt; no neighboring nation is substituted.
pub(super) fn reconcile(view: &mut ManagerView, state: &GameState) {
    if view.kind == ManagerKind::Diplomacy
        && view
            .selections
            .get("contact")
            .is_some_and(|id| relation(state, id).is_none())
    {
        view.selections.remove("contact");
    }
}
