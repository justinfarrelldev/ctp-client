//! Original SciAdvanceScreen callbacks, never local research/goal simulation.
use super::{ManagementUi, ManagerCommand, ManagerIntent, ManagerKind, ManagerView, data};
use crate::gameplay::{
    GameplayError,
    assets::layout::{LayoutBook, LayoutNode},
    model::GameState,
};
use serde_json::json;

/// Cache the original constructor's StringTable.string0 instruction on its box.
pub(super) fn configure(root: &mut LayoutNode, book: &LayoutBook) -> Result<(), GameplayError> {
    let Some(message) = super::fidelity::node(root, "Background.Message") else {
        return Ok(());
    };
    let table = book.resolve("SciAdvanceString")?;
    if let Some(key) = table.field("string0") {
        message.fields.insert("text".to_owned(), key.to_owned());
    }
    Ok(())
}

/// Source loadList selects the exact researching identity after populating options.
/// Unlike its force-select fallback, no absent current target becomes a first row.
pub(super) fn initial(state: &GameState) -> Option<&str> {
    super::science::current_target(state).filter(|id| available(state, id))
}

/// Contradictory known-only/empty candidates are not valid chooser identities.
fn available(state: &GameState, id: &str) -> bool {
    !id.is_empty()
        && state
            .viewer
            .as_ref()
            .and_then(|viewer| viewer.interface.research.as_ref())
            .is_some_and(|research| {
                research.available.iter().any(|candidate| candidate == id)
                    && !research.advances.iter().any(|known| known == id)
            })
}

/// Validate even before receipt reconciliation; installed art isn't authorization.
pub(super) fn selected<'view>(view: &'view ManagerView, state: &GameState) -> Option<&'view str> {
    view.selections
        .get("advance")
        .map(String::as_str)
        .filter(|id| available(state, id))
}

/// Source change/turn/goal boxes are distinct from active Science-manager fields.
pub(super) fn text(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    path: &str,
) -> Option<String> {
    match path {
        "Background.ChangeBox" => {
            Some(selected(view, state).map_or_else(String::new, |id| data::label(ui, id)))
        }
        "Background.TurnsBox" => Some(
            selected(view, state)
                .and_then(|id| super::science::turns(state, id))
                .map_or_else(|| "-".to_owned(), |turns| turns.to_string()),
        ),
        "Background.ResearchGoal" => {
            let prefix = ui.book.label("str_ldl_ResearchGoal");
            Some(
                state
                    .viewer
                    .as_ref()
                    .and_then(|viewer| viewer.interface.research.as_ref())
                    .map_or_else(
                        || format!("{prefix} —"),
                        |research| {
                            if research.goal_id.is_empty() {
                                prefix.to_owned()
                            } else {
                                format!("{prefix} {}", data::label(ui, &research.goal_id))
                            }
                        },
                    ),
            )
        }
        _ => None,
    }
}

/// Original `GoalCallback` only opens Library; OK confirms one exact current choice.
pub(super) fn binding(
    state: &GameState,
    view: &ManagerView,
    path: &str,
) -> Option<(Option<ManagerIntent>, Option<String>)> {
    if view.kind != ManagerKind::Research {
        return None;
    }
    Some(match path {
        "Background.GoalButton" => (Some(ManagerIntent::Open(ManagerKind::Library)), None),
        "Background.BackButton" => {
            let Some(advance) = selected(view, state) else {
                return Some((
                    None,
                    Some("Select a currently available advance first".to_owned()),
                ));
            };
            let Some(viewer) = state.viewer.as_ref() else {
                return Some((
                    None,
                    Some("No current authoritative research snapshot".to_owned()),
                ));
            };
            let command = ManagerCommand {
                revision: viewer.revision,
                route: "set-research".to_owned(),
                payload: json!({"advance_id":advance}),
            };
            if command.valid_for(state) {
                (Some(ManagerIntent::Submit(command)), None)
            } else {
                (
                    None,
                    Some(
                        "The selected advance is not authorized in this viewer revision/phase"
                            .to_owned(),
                    ),
                )
            }
        }
        _ => return None,
    })
}

/// Drop removed identities in active and parent views, never select a new neighbor.
pub(super) fn reconcile(view: &mut ManagerView, state: &GameState) {
    let valid = match view.kind {
        ManagerKind::Research => selected(view, state).is_some(),
        ManagerKind::Science => view.selections.get("advance").is_some_and(|id| {
            state
                .viewer
                .as_ref()
                .and_then(|viewer| viewer.interface.research.as_ref())
                .is_some_and(|research| research.advances.contains(id))
        }),
        _ => return,
    };
    if !valid && view.selections.remove("advance").is_some() {
        clear_description_scroll(view);
    }
}

/// Source `SetHyperText` starts the new selected document at its first line.
pub(super) fn clear_description_scroll(view: &mut ManagerView) {
    let path = match view.kind {
        ManagerKind::Science => "CurrentResearch.Description",
        ManagerKind::Research => "Background.GlStats",
        _ => return,
    };
    view.scroll.remove(path);
}
