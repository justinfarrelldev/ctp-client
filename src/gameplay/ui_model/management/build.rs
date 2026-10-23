//! Original Build queue-head purchase controls, backed only by received quotes.
use super::{ManagementUi, ManagerCommand, ManagerIntent, ManagerKind, ManagerView, data};
use crate::gameplay::{model::GameState, server::{ServerCityProjection, v2::Quote}};
use serde_json::json;

/*
 * Original released-source provenance for this control group:
 * - ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::{UpdateButtons,RushBuy}
 *   requires a selected queue index0, a valid GetOvertimeCost, adequate treasury
 *   and not AlreadyBoughtFront. Future/no selection uses ---; an unaffordable
 *   selected head still displays its real price. RushBuy repeats head checks.
 * - The same file's constructor wires RushBuyCallback, while ListCallback and
 *   QueueListCallback call UpdateButtons after exact choice/entry selection.
 * - ctp2/ctp2_data/english/uidata/layouts/editqueue.ldl::BuildEditorWindow:
 *   RushBuyButton (385,118,width104), RushBuyCost (495,118,102x22) and
 *   QueueGroup.QueueList supply styles/relative bounds/selection controls.
 * Client integration replaces private CityData pricing/mutation with a current
 * owner-only quote and exact advertised city command. An entry ID, not its
 * possibly repeated content ID, proves explicit head selection. Missing price
 * is unavailable (—), not a free purchase; source not-applicable uses ---.
 * Enqueue/receipt cannot spend or complete production locally. No scaling edits.
 */

/// Exact retained queue-head identity; a repeated content row cannot become head.
fn selected_head<'state>(state: &'state GameState, view: &ManagerView) -> Option<(&'state ServerCityProjection, &'state str)> {
    let selected = view.selections.get("queue")?;
    let city = data::city(state, view)?;
    let items = city.facts["queue"].as_array()?;
    let head = items.first()?;
    let key = |entry: &'state serde_json::Value| entry["entry_id"].as_str().or_else(|| entry["id"].as_str());
    if key(head) != Some(selected.as_str()) || items.iter().filter(|entry| key(entry) == Some(selected.as_str())).count() != 1_usize {
        return None;
    }
    Some((city, head["id"].as_str().filter(|id| !id.is_empty())?))
}

/// Revision binds price to the same complete city/head facts, never a recalculation.
fn quote<'city>(state: &GameState, city: &'city ServerCityProjection) -> Option<&'city Quote> {
    let revision = state.viewer.as_ref()?.revision;
    city.rush_buy_quote.as_ref().filter(|quote| quote.revision == revision)
}

/// Source price box follows selection even when the city cannot afford the head.
pub(super) fn text(state: &GameState, view: &ManagerView, path: &str) -> Option<String> {
    if view.kind != ManagerKind::Build || path != "RushBuyCost" {
        return None;
    }
    let Some((city, _content)) = selected_head(state, view) else {
        return Some("---".to_owned());
    };
    let Some(quote) = quote(state, city) else {
        return Some("—".to_owned());
    };
    Some(match quote.status.as_str() {
        "available" | "insufficient-gold" => quote.cost.filter(|cost| *cost >= 0_i64)
            .map_or_else(|| "—".to_owned(), |cost| cost.to_string()),
        "completed" | "empty-queue" | "not-rushable" => "---".to_owned(),
        _ => "—".to_owned(),
    })
}

/// A selected head/quote does not grant authority to choose a different city/order.
pub(super) fn binding(ui: &ManagementUi, state: &GameState, view: &ManagerView, path: &str) -> Option<(Option<ManagerIntent>, Option<String>)> {
    if view.kind != ManagerKind::Build { return None; }
    if let Some(reason) = editing_reason(state, view, path) { return Some((None, Some(reason))); }
    if path == "QueueGroup.ClearButton" {
        let dialog = clear_request(state, view).and_then(|command| super::confirmation::dialog(ui, state, command));
        return Some(dialog.map_or_else(
            || disabled("A current safe editable queue/funding state is required before source Clear confirmation"),
            |dialog| (Some(ManagerIntent::Dialog(Box::new(dialog))), None),
        ));
    }
    if path != "RushBuyButton" { return None; }
    let Some((city, content)) = selected_head(state, view) else {
        return Some(disabled("Select the exact current owned-city queue-head entry to rush"));
    };
    let Some(viewer) = state.viewer.as_ref() else {
        return Some(disabled("No current authoritative city snapshot"));
    };
    let affordable = quote(state, city).is_some_and(|quote| {
        quote.status == "available" && quote.cost.zip(viewer.diplomacy.as_ref())
            .is_some_and(|(cost, nation)| cost >= 0_i64 && cost <= i64::from(nation.gold))
    });
    if !affordable {
        return Some(disabled("A current available head quote and sufficient known treasury are required"));
    }
    let payload = json!({"city_id":city.id});
    let mut actions = viewer.interface.legal_actions.iter().filter(|action| {
        action.route == "rush-buy" && action.entity_id == city.id && action.payload == payload
            && (action.target_id.is_empty() || action.target_id == content)
    });
    let action = actions.next();
    let command = actions.next().is_none().then_some(action).flatten()
        .and_then(|action| ManagerCommand::advertised(state, action));
    Some(command.map_or_else(
        || disabled("No single exact current authorization for this selected owned-city head"),
        |command| (Some(ManagerIntent::Submit(command)), None),
    ))
}

/*
 * Source: ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::UpdateButtons,
 * {InsertItem,Add,InsertInQueue},{RemoveItem,Remove},{ItemUp,Up},{ItemDown,Down}
 * disable removing/demoting/inserting before bought index0 and index1->0, not
 * future-only edits. editqueue.ldl::{ItemsBox.InsertButton,QueueGroup.RemoveButton,
 * QueueGroup.UpButton,QueueGroup.DownButton} retains its source styling/positions.
 * CityData.cpp::InsertBuildItem and bldque.cpp::{InsertBefore,RemoveNodeByIndex,
 * RemoveHead,MoveNodeUp,MoveNodeDown} mutate the original queue without an ordinary
 * Remove Query; here existing atomic requests preserve exact entry identities.
 * Current quote funding conservatively replaces unavailable AlreadyBoughtFront;
 * unknown/stale/queued funding cannot unlock a head. Safe future-only operations
 * remain available. Return an explicit binding reason BEFORE any legacy fallback,
 * never cancel a duplicate content item or another city. No local queue execution.
 */
/// Explains an unsafe current source edit; None leaves existing safe validation.
fn editing_reason(state: &GameState, view: &ManagerView, path: &str) -> Option<String> {
    if !matches!(path, "ItemsBox.InsertButton" | "QueueGroup.RemoveButton" | "QueueGroup.UpButton" | "QueueGroup.DownButton") { return None; }
    let Some(city) = data::city(state, view) else { return Some("Select an exact current owned city".to_owned()); };
    let Some(items) = city.facts["queue"].as_array() else { return Some("Select a known current queue before editing".to_owned()); };
    let selected = view.selections.get("queue");
    let index = items.iter().position(|item| item["entry_id"].as_str().is_some_and(|id| selected.is_some_and(|entry| entry == id)));
    let Some(index) = index else { return Some("Select an exact current queue entry before editing".to_owned()); };
    if path == "QueueGroup.UpButton" && index == 0_usize { return Some("The first queue entry cannot move up".to_owned()); }
    if path == "QueueGroup.DownButton" && index.saturating_add(1_usize) >= items.len() { return Some("The last queue entry cannot move down".to_owned()); }
    let touches_head = if path == "QueueGroup.UpButton" { index == 1_usize } else { index == 0_usize };
    if !touches_head { return None; }
    let queued = state.viewer.as_ref().is_some_and(|viewer| viewer.interface.planning_status == "queued");
    if !queued && quote(state, city).is_some_and(|current| matches!(current.status.as_str(), "available" | "insufficient-gold") && current.cost.is_some_and(|cost| cost >= 0_i64)) { return None; }
    Some("A funded or unprojected/stale/queued funding state protects this exact queue head; edit only future entries".to_owned())
}

/*
 * Source: ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::ClearButton and
 * ClearMessageCallback ask QueryClearQueue, clearing all unless AlreadyBoughtFront
 * requires ClearAllButHead. MessageBoxDialog.{cpp,h} owns original OK/Cancel.
 * Client quote.completed proves a funded head, not why it was funded: conservatively
 * retain that exact entry. Current available/insufficient quotes prove unfinished
 * work. Missing/stale/other/queued funding cannot be treated as unbought. Commands
 * capture the current revision once; no private paid flag is invented or replayed.
 */
/// Captures a safe source clear; actual state changes remain entirely server-owned.
fn clear_request(state: &GameState, view: &ManagerView) -> Option<ManagerCommand> {
    let viewer = state.viewer.as_ref()?;
    if viewer.interface.planning_status == "queued" { return None; }
    let city = data::city(state, view)?;
    let current = quote(state, city)?;
    match current.status.as_str() {
        "available" | "insufficient-gold" if current.cost.is_some_and(|cost| cost >= 0_i64) => super::queue::request(state, view, "QueueGroup.ClearButton"),
        "completed" => {
            let items = city.facts["queue"].as_array()?;
            if items.len() <= 1_usize { return None; }
            let head = items.first()?["entry_id"].as_str()?;
            super::queue::replacement(state, &city.id, &[json!({"entry_id":head})])
        }
        _ => None,
    }
}

/*
 * Source: ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::{ToggleUnits,
 * ToggleBuildings,ToggleWonders,SelectChoiceList} hides/deselects the other left
 * lists, preserving the independent queue. editqueue.ldl::BuildEditorWindow.
 * ItemsBox.{UnitsButton,BuildingsButton,WondersButton,UnitsList,BuildingList,
 * WonderList} supplies unchanged source styles/positions. ShowSelectedInfo and
 * citywindow.cpp::CityWindow::SetItemDescription resolve content before Library.
 * Client's existing last-interacted/reference pane interpretation is retained,
 * not claimed identical to released ShowSelectedInfo's queue-first priority.
 * Explicit role disambiguates opaque entry IDs from identical content IDs;
 * category navigation cannot retain a hidden executable choice. No local rules.
 */
/// Applies one source Build presentation choice without changing game facts.
pub(super) fn select(view: &mut ManagerView, key: &str, value: &str) -> Option<bool> {
    if view.kind != ManagerKind::Build { return None; }
    match key {
        "build-category" => {
            if !matches!(value, "Units" | "Buildings" | "Wonders") { return Some(false); }
            let previous = view.selections.get(key).map_or("Units", String::as_str);
            if previous != value {
                let removed = view.selections.remove("buildable");
                let source = view.selections.get("description-source").map(String::as_str);
                if source == Some("buildable") || (source.is_none() && removed.is_some() && view.selections.get("description") == removed.as_ref()) {
                    view.selections.remove("description");
                    view.selections.remove("description-source");
                }
            }
            view.selections.insert(key.to_owned(), value.to_owned());
        }
        "buildable" | "queue" => {
            view.selections.insert(key.to_owned(), value.to_owned());
            view.selections.insert("description".to_owned(), value.to_owned());
            view.selections.insert("description-source".to_owned(), key.to_owned());
        }
        _ => return None,
    }
    Some(true)
}

/*
 * Source: EditQueue.cpp::ShowSelectedInfo -> citywindow.cpp::SetItemDescription
 * -> aui_HyperTextBox::SetHyperText/RepositionRanger (original ui/interface and
 * ui/aui_common paths under ctp2_source/ctp2_code/). aui_Ranger::SetMaximum only
 * clamps excess; it does NOT reset every replacement to zero. ItemDescription
 * in editqueue.ldl remains246x65; native measured-line units/current scaling are
 * unchanged. Clamp stored state on input/receipt/return, not only painted text,
 * so a shorter document cannot later resurrect an obsolete larger offset.
 */
/// Refreshes only the current Build hypertext ranger, preserving other offsets.
pub(super) fn refresh_description(ui: &mut ManagementUi, state: &GameState) -> bool {
    ui.view.as_ref().is_some_and(|view| view.kind == ManagerKind::Build && view.scroll.contains_key("ItemDescription"))
        && ui.scroll("ItemDescription", 0_i32, state)
}

/// Unavailable controls explain the missing proof instead of becoming silent no-ops.
fn disabled(reason: &str) -> (Option<ManagerIntent>, Option<String>) {
    (None, Some(reason.to_owned()))
}
