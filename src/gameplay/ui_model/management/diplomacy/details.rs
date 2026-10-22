//! Original selected-contact intelligence panes without private foreign globals.
use super::{ManagerKind, ManagerView};
use crate::gameplay::{
    model::GameState,
    ui_model::{
        MenuRect,
        management::{ManagementUi, ManagerWidget},
    },
};
/// Only a current projected contact may open intelligence details.
pub(in crate::gameplay::ui_model::management) fn known(state: &GameState, id: &str) -> bool {
    !id.is_empty() && super::relation(state, id).is_some()
}
/// Namespace-safe view-only navigation retains the exact parent selection.
pub(in crate::gameplay::ui_model::management) fn open(
    ui: &mut ManagementUi,
    state: &GameState,
    id: &str,
) -> bool {
    if !known(state, id) {
        return false;
    }
    ui.open_related(ManagerKind::DiplomacyDetails, state);
    if let Some(view) = ui.view.as_mut() {
        view.city_id = None;
        view.selections.insert("contact".to_owned(), id.to_owned());
    }
    true
}
/// Takes just the cached original caption prefix, not a SLIC evaluator or a
/// foreign value substitution. Private placeholders and printf tails never show.
fn prefix(ui: &ManagementUi, key: &str) -> String {
    ui.book
        .label(key)
        .split(['{', '%'])
        .next()
        .unwrap_or_default()
        .trim_end()
        .to_owned()
}
/// Source labels retain their semantics/positions; only actual bilateral facts
/// are populated. No embassy/rank/leader/city-count/foreign-advance inference.
pub(in crate::gameplay::ui_model::management) fn text(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    path: &str,
) -> Option<String> {
    if view.kind != ManagerKind::DiplomacyDetails {
        return None;
    }
    let id = view.selections.get("contact").filter(|id| known(state, id));
    if path == "IntelReportLabel" {
        return Some(format!(
            "{} {}",
            prefix(ui, "str_ldl_DipDetails_IntelReport"),
            id.map_or_else(|| "—".to_owned(), |id| format!("ID {id}"))
        ));
    }
    if path == "EmbassyMsgLabel" {
        return Some(id.map_or_else(
            || "Contact intelligence unavailable".to_owned(),
            |id| {
                format!(
                    "Regard: {}. Treaty: {}. Foreign intelligence unavailable.",
                    super::value(state, id, "Regard"),
                    super::value(state, id, "Treaties")
                )
            },
        ));
    }
    if path.ends_with("PriveledgedLabel") {
        return Some("Foreign intelligence unavailable".to_owned());
    }
    if path == "TabGroup.Tab3.TabPanel.KnownToGovLabel" {
        return Some(format!(
            "{} {}",
            prefix(ui, "str_ldl_DipDetails_KnownToGov"),
            id.map_or_else(|| "—".to_owned(), |id| format!("ID {id}"))
        ));
    }
    let key = match path {
        "LeaderNameLabel" => "str_ldl_DipDetails_LeaderName",
        "LeaderPersonalityLabel" => "str_ldl_DipDetails_LeaderPersonality",
        "TabGroup.Tab2.TabPanel.EmpireSizeLabel" => "str_ldl_DipDetails_EmpireSize",
        "TabGroup.Tab2.TabPanel.PopulationLabel" => "str_ldl_DipDetails_Population",
        "TabGroup.Tab2.TabPanel.PollutionLabel" => "str_ldl_DipDetails_Pollution",
        "TabGroup.Tab2.TabPanel.CapitalLabel" => "str_ldl_DipDetails_Capital",
        "TabGroup.Tab2.TabPanel.LargestCityLabel" => "str_ldl_DipDetails_LargestCity",
        "TabGroup.Tab2.TabPanel.GovTypeLabel" => "str_ldl_DipDetails_GovType",
        "TabGroup.Tab2.TabPanel.MilStateLabel" => "str_ldl_DipDetails_MilitaryState",
        "TabGroup.Tab3.TabPanel.NowResearchLabel" => "str_ldl_DipDetails_NowResearching",
        _ => return None,
    };
    Some(format!("{} —", prefix(ui, key)))
}
/// Lists concern the selected foreign nation's contacts/wonders or advance set
/// differences. The viewer's own relations/advances cannot fill these missing
/// contracts. Explicit unavailable text is not a fake selectable target row.
pub(in crate::gameplay::ui_model::management) fn unavailable(
    ui: &ManagementUi,
    view: &ManagerView,
    path: &str,
    rect: MenuRect,
    widgets: &mut Vec<ManagerWidget>,
) {
    if view.kind != ManagerKind::DiplomacyDetails {
        return;
    }
    let label = match path {
        "TabGroup.Tab1.TabPanel.List" => "Foreign relations unavailable",
        "TabGroup.Tab2.TabPanel.WonderList" => "Foreign wonders unavailable",
        "TabGroup.Tab3.TabPanel.ForAdvancesList" => "Foreign advances unavailable",
        "TabGroup.Tab3.TabPanel.YourAdvancesList" => "Advance comparison unavailable",
        _ => return,
    };
    let mut node = ui
        .row_templates
        .get("ForAdvancesItem")
        .and_then(|item| item.children.first())
        .cloned()
        .unwrap_or_default();
    node.children.clear();
    node.fields.insert("wordwrap".to_owned(), "true".to_owned());
    node.fields.insert("just".to_owned(), "left".to_owned());
    widgets.push(ManagerWidget {id:format!("{path}/unavailable"),label:label.to_owned(),rect:MenuRect {height:rect.height.min(48_u32),..rect},node,intent:None,unavailable:Some("The current viewer contract does not project this foreign intelligence; unknown does not mean an empty set".to_owned()),selected:false});
}
