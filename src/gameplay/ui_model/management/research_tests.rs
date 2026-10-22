//! Source research chooser presentation and receipt-safe related navigation.
use super::{ManagementUi, ManagerIntent, ManagerKind, ManagerWidget, data};
use crate::gameplay::{assets::layout::LayoutBook, model::GameState, server::ServerProjection};
use serde_json::json;
use std::path::Path;

/// The cheapest/first available target is deliberately not the active target.
fn fixture() -> (ManagementUi, GameState) {
    let projection: ServerProjection = serde_json::from_value(json!({
        "revision":7_i64,"turn":1_u32,"contract":{"version":2_u32},
        "research":{"target_id":"ADVANCE_BALLISTICS","goal_id":"ADVANCE_RELIGION","progress":0_i64,"advances":["ADVANCE_AGRICULTURE"],"available":["ADVANCE_IRON_WORKING","ADVANCE_BALLISTICS"],"forecast":{"revision":7_i64,"status":"estimated","turns":3_u32,"assumption":"current allocation"}},
        "rules":{"advances":[{"id":"ADVANCE_AGRICULTURE","cost":242_i64},{"id":"ADVANCE_BALLISTICS","cost":570_i64},{"id":"ADVANCE_IRON_WORKING","cost":120_i64}]},
        "legal_actions":[{"route":"pass","payload":{}}]
    })).expect("received own research and planning authority");
    let book = LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"))
        .expect("read-only source layout");
    (
        ManagementUi::new(book).expect("cached chooser and articles"),
        GameState::from_server_projection(&projection, 1_u8, 1_u64),
    )
}

/// Exact source descriptor, never coordinate/text ambiguity.
fn label<'widgets>(widgets: &'widgets [ManagerWidget], id: &str) -> &'widgets str {
    &widgets
        .iter()
        .find(|widget| widget.id == id)
        .expect("source field")
        .label
}

/// Pure selection through the same descriptor used by native pointer/keyboard input.
fn select(ui: &mut ManagementUi, state: &GameState, id: &str) {
    let intent = ui
        .activate_id(&format!("Background.SciAdvanceList/{id}"), state)
        .expect("explicit available row");
    assert!(
        ui.apply_local(intent),
        "selection is local presentation only"
    );
}

#[test]
fn all_three_original_science_entries_select_exact_current_target_not_cheapest_or_parent_discovery()
{
    for path in [
        "ChangeResearch",
        "CurrentResearch.AdvanceProgress.IconBorder.IconButton",
        "CurrentResearch.AdvanceProgress.IconBorder.IconButton.RadialButton",
    ] {
        let (mut ui, state) = fixture();
        ui.open(ManagerKind::Science, &state, None);
        assert!(
            ui.apply_local(ManagerIntent::Select(
                "advance".to_owned(),
                "ADVANCE_AGRICULTURE".to_owned()
            )),
            "independent parent discovery"
        );
        let before = state.clone();
        let intent = ui
            .activate_id(path, &state)
            .expect("original edit-research callback");
        assert_eq!(
            intent,
            ManagerIntent::Open(ManagerKind::Research),
            "all source entries open original chooser"
        );
        ui.open_related(ManagerKind::Research, &state);
        assert_eq!(
            ui.view
                .as_ref()
                .expect("chooser")
                .selections
                .get("advance")
                .map(String::as_str),
            Some("ADVANCE_BALLISTICS"),
            "exact current target survives cost sorting, never first candidate or parent's known article"
        );
        let fields = ui.widgets(&state);
        assert_eq!(
            label(&fields, "Background.ChangeBox"),
            data::label(&ui, "ADVANCE_BALLISTICS"),
            "original selected advance name"
        );
        assert_eq!(
            label(&fields, "Background.TurnsBox"),
            "3",
            "received current estimate, no locally added turn"
        );
        assert_eq!(
            state, before,
            "opening/selecting current never sends or applies research"
        );
    }
}

#[test]
fn explicit_other_target_updates_stats_name_and_current_only_forecast_without_changing_research() {
    let (mut ui, mut state) = fixture();
    ui.open(ManagerKind::Research, &state, None);
    let before = state.clone();
    let initial = ui.widgets(&state);
    let words = |fields: &[ManagerWidget]| {
        fields
            .iter()
            .filter(|widget| widget.id.starts_with("Background.GlStats/word-"))
            .map(|widget| widget.label.clone())
            .collect::<Vec<_>>()
    };
    select(&mut ui, &state, "ADVANCE_IRON_WORKING");
    let changed = ui.widgets(&state);
    assert_eq!(
        label(&changed, "Background.ChangeBox"),
        data::label(&ui, "ADVANCE_IRON_WORKING"),
        "selected current available identity owns ChangeBox"
    );
    assert_eq!(
        label(&changed, "Background.TurnsBox"),
        "-",
        "active Ballistics estimate is not an Iron Working counterfactual"
    );
    assert!(
        !words(&changed).is_empty(),
        "actual installed selected-advance statistics"
    );
    assert_ne!(
        words(&changed),
        words(&initial),
        "selection updates source stats/hyperlinks"
    );
    assert_eq!(
        state, before,
        "selection doesn't spend research, change goal or execute an order"
    );
    select(&mut ui, &state, "ADVANCE_BALLISTICS");
    for (revision, status, turns, expected) in [
        (6_i64, "estimated", 3_u32, "-"),
        (7_i64, "unavailable", 0_u32, "-"),
        (7_i64, "completed", 0_u32, "0"),
    ] {
        let forecast = state
            .viewer
            .as_mut()
            .expect("viewer")
            .interface
            .research
            .as_mut()
            .expect("research")
            .forecast
            .as_mut()
            .expect("forecast");
        forecast.revision = revision;
        forecast.status = status.to_owned();
        forecast.turns = Some(turns);
        assert_eq!(
            label(&ui.widgets(&state), "Background.TurnsBox"),
            expected,
            "only current received supported forecast; explicit zero stays zero"
        );
    }
}

#[test]
fn unavailable_empty_known_or_missing_current_requires_explicit_choice_and_clear_source_fields() {
    let (mut ui, original) = fixture();
    for mode in 0_u8..5_u8 {
        let mut state = original.clone();
        match mode {
            0_u8 => state
                .viewer
                .as_mut()
                .expect("viewer")
                .interface
                .research
                .as_mut()
                .expect("research")
                .available
                .retain(|id| id != "ADVANCE_BALLISTICS"),
            1_u8 => state
                .viewer
                .as_mut()
                .expect("viewer")
                .interface
                .research
                .as_mut()
                .expect("research")
                .target_id
                .clear(),
            2_u8 => state
                .viewer
                .as_mut()
                .expect("viewer")
                .interface
                .research
                .as_mut()
                .expect("research")
                .advances
                .push("ADVANCE_BALLISTICS".to_owned()),
            3_u8 => state.viewer.as_mut().expect("viewer").interface.research = None,
            _ => state.viewer = None,
        }
        ui.open(ManagerKind::Research, &state, None);
        assert!(
            !ui.view
                .as_ref()
                .expect("chooser")
                .selections
                .contains_key("advance"),
            "no first-candidate default when exact active target is unavailable"
        );
        let fields = ui.widgets(&state);
        assert_eq!(
            label(&fields, "Background.ChangeBox"),
            "",
            "original no-selection name is empty"
        );
        assert_eq!(
            label(&fields, "Background.TurnsBox"),
            "-",
            "no-selection estimate is hyphen, not source zero placeholder"
        );
        assert!(
            ui.activate_id("Background.BackButton", &state).is_none(),
            "no arbitrary neighbor confirmation"
        );
        assert!(
            !fields
                .iter()
                .any(|widget| widget.id.starts_with("Background.GlStats/word-")),
            "no stale/unknown selected-advance document"
        );
    }
}

#[test]
fn original_instruction_goal_name_and_goal_button_open_library_then_restore_exact_chooser_and_parent()
 {
    let (mut ui, state) = fixture();
    ui.open(ManagerKind::Science, &state, None);
    assert!(
        ui.apply_local(ManagerIntent::Select(
            "advance".to_owned(),
            "ADVANCE_AGRICULTURE".to_owned()
        )),
        "selected discovery"
    );
    let parent = ui.view.clone();
    ui.open_related(ManagerKind::Research, &state);
    select(&mut ui, &state, "ADVANCE_IRON_WORKING");
    ui.view
        .as_mut()
        .expect("chooser")
        .scroll
        .insert("Background.GlStats".to_owned(), 2_usize);
    let chooser = ui.view.clone();
    let fields = ui.widgets(&state);
    assert_eq!(
        label(&fields, "Background.Message"),
        ui.book.label("str_tbl_ldl_Select_a_new"),
        "original SciAdvanceString.string0, not client-written instructions"
    );
    assert_eq!(
        label(&fields, "Background.ResearchGoal"),
        format!(
            "{} {}",
            ui.book.label("str_ldl_ResearchGoal"),
            data::label(&ui, "ADVANCE_RELIGION")
        ),
        "persistent goal is distinct from selected/current advance"
    );
    let intent = ui
        .activate_id("Background.GoalButton", &state)
        .expect("original GoalCallback opens Library, not an inert button");
    assert_eq!(
        intent,
        ManagerIntent::Open(ManagerKind::Library),
        "view-only source callback, no automatic goal submission"
    );
    ui.open_related(ManagerKind::Library, &state);
    ui.close();
    assert_eq!(
        ui.view, chooser,
        "Library close preserves exact chooser choice and scroll"
    );
    assert_eq!(
        ui.activate_id("Background.CancelButton", &state),
        Some(ManagerIntent::Close),
        "cancel is presentation close, never restore/replay a previous research command"
    );
    ui.close();
    assert_eq!(
        ui.view, parent,
        "cancel restores original manager known selection"
    );
}

#[test]
fn removed_or_injected_unavailable_selection_never_leaks_stats_or_retargets_after_reconciliation() {
    let (mut ui, original) = fixture();
    for mode in 0_u8..4_u8 {
        let mut state = original.clone();
        ui.open(ManagerKind::Research, &state, None);
        select(&mut ui, &state, "ADVANCE_IRON_WORKING");
        match mode {
            0_u8 => state
                .viewer
                .as_mut()
                .expect("viewer")
                .interface
                .research
                .as_mut()
                .expect("research")
                .available
                .retain(|id| id != "ADVANCE_IRON_WORKING"),
            1_u8 => state
                .viewer
                .as_mut()
                .expect("viewer")
                .interface
                .research
                .as_mut()
                .expect("research")
                .advances
                .push("ADVANCE_IRON_WORKING".to_owned()),
            2_u8 => state.viewer.as_mut().expect("viewer").interface.research = None,
            _ => state.viewer = None,
        }
        let fields = ui.widgets(&state);
        assert_eq!(
            label(&fields, "Background.ChangeBox"),
            "",
            "validate selected identity even before reconciliation callback"
        );
        assert!(
            !fields
                .iter()
                .any(|widget| widget.id.starts_with("Background.GlStats/word-")),
            "installed article alone is not an available research choice"
        );
        assert!(
            ui.activate_id("Background.BackButton", &state).is_none(),
            "removed/known-only choice cannot send an order"
        );
        ui.reconcile(&state);
        assert!(
            !ui.view
                .as_ref()
                .expect("chooser")
                .selections
                .contains_key("advance"),
            "drop exact invalid identity, including absent viewer; don't silently replace with active/first target"
        );
    }
    let (mut known_ui, state) = fixture();
    known_ui.open(ManagerKind::Research, &state, None);
    assert!(
        known_ui.apply_local(ManagerIntent::Select(
            "advance".to_owned(),
            "ADVANCE_AGRICULTURE".to_owned()
        )),
        "injected source selection"
    );
    assert!(
        !known_ui
            .widgets(&state)
            .iter()
            .any(|widget| widget.id.starts_with("Background.GlStats/word-")),
        "known-only parent discovery cannot masquerade as a chooser candidate"
    );
}

#[test]
fn selecting_new_advance_resets_only_its_source_description_scroll() {
    let (mut ui, state) = fixture();
    for (kind, description, id) in [
        (
            ManagerKind::Science,
            "CurrentResearch.Description",
            "ADVANCE_AGRICULTURE",
        ),
        (
            ManagerKind::Research,
            "Background.GlStats",
            "ADVANCE_IRON_WORKING",
        ),
    ] {
        ui.open(kind, &state, None);
        let view = ui.view.as_mut().expect("source view");
        view.scroll.insert(description.to_owned(), 9_usize);
        view.scroll.insert("unrelated-list".to_owned(), 2_usize);
        assert!(
            ui.apply_local(ManagerIntent::Select("advance".to_owned(), id.to_owned())),
            "source AdvanceListCallback updates hypertext"
        );
        assert!(
            !ui.view
                .as_ref()
                .expect("view")
                .scroll
                .contains_key(description),
            "SetHyperText resets selected document position"
        );
        assert_eq!(
            ui.view.as_ref().expect("view").scroll.get("unrelated-list"),
            Some(&2_usize),
            "independent list scroll remains exact"
        );
    }
}

#[test]
fn removed_research_and_known_discovery_drop_only_exact_parent_selections_and_old_document_scroll()
{
    let (mut ui, mut state) = fixture();
    ui.open(ManagerKind::Science, &state, None);
    assert!(
        ui.apply_local(ManagerIntent::Select(
            "advance".to_owned(),
            "ADVANCE_AGRICULTURE".to_owned()
        )),
        "parent discovery selected"
    );
    ui.view
        .as_mut()
        .expect("science")
        .scroll
        .insert("CurrentResearch.Description".to_owned(), 9_usize);
    ui.open_related(ManagerKind::Research, &state);
    select(&mut ui, &state, "ADVANCE_IRON_WORKING");
    ui.view
        .as_mut()
        .expect("chooser")
        .scroll
        .insert("Background.GlStats".to_owned(), 9_usize);
    ui.open_related(ManagerKind::Library, &state);
    let research = state
        .viewer
        .as_mut()
        .expect("viewer")
        .interface
        .research
        .as_mut()
        .expect("research");
    research.advances.clear();
    research.available.retain(|id| id != "ADVANCE_IRON_WORKING");
    ui.reconcile(&state);
    ui.close();
    let chooser = ui.view.as_ref().expect("restored chooser");
    assert_eq!(
        chooser.kind,
        ManagerKind::Research,
        "one related window at a time"
    );
    assert!(
        !chooser.selections.contains_key("advance"),
        "removed parent choice must not become current/first available"
    );
    assert!(
        !chooser.scroll.contains_key("Background.GlStats"),
        "discard old selected document scroll with removed identity"
    );
    ui.close();
    let science = ui.view.as_ref().expect("restored science");
    assert_eq!(science.kind, ManagerKind::Science, "original parent");
    assert!(
        !science.selections.contains_key("advance"),
        "lost known discovery doesn't become another known row"
    );
    assert!(
        !science.scroll.contains_key("CurrentResearch.Description"),
        "current-target fallback must not inherit vanished discovery scroll"
    );
}

#[test]
fn explicit_empty_future_goal_differs_from_unprojected_goal_without_reusing_selected_or_current_target()
 {
    let (mut ui, mut state) = fixture();
    ui.open(ManagerKind::Research, &state, None);
    let prefix = ui.book.label("str_ldl_ResearchGoal").to_owned();
    state
        .viewer
        .as_mut()
        .expect("viewer")
        .interface
        .research
        .as_mut()
        .expect("research")
        .goal_id
        .clear();
    assert_eq!(
        label(&ui.widgets(&state), "Background.ResearchGoal"),
        prefix,
        "known no future goal is original label only"
    );
    state.viewer.as_mut().expect("viewer").interface.research = None;
    assert_eq!(
        label(&ui.widgets(&state), "Background.ResearchGoal"),
        format!("{prefix} —"),
        "unprojected goal isn't a known empty goal or active target"
    );
    state.viewer = None;
    assert_eq!(
        label(&ui.widgets(&state), "Background.ResearchGoal"),
        format!("{prefix} —"),
        "missing viewer never leaks an old or substituted goal"
    );
}
