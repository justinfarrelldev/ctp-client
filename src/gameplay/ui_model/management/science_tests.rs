//! Original Science Manager fields/selection from injected viewer facts, no writes.
use super::{ManagementUi, ManagerIntent, ManagerKind, ManagerWidget};
use crate::gameplay::{
    assets::{
        layout::LayoutBook,
        library::{LibraryDocument, LibrarySection},
    },
    model::GameState,
    server::ServerProjection,
};
use serde_json::json;
use std::path::Path;
/// Known versus available discoveries remain distinct; zero spending is received.
fn fixture() -> (ManagementUi, GameState) {
    let projection:ServerProjection=serde_json::from_value(json!({"revision":7_i64,"turn":1_u32,"research":{"target_id":"ADVANCE_BALLISTICS","progress":0_i64,"advances":["ADVANCE_AGRICULTURE"],"available":["ADVANCE_BALLISTICS","ADVANCE_IRON_WORKING"]},"rules":{"advances":[{"id":"ADVANCE_AGRICULTURE","cost":20_i64},{"id":"ADVANCE_BALLISTICS","cost":570_i64},{"id":"ADVANCE_IRON_WORKING","cost":120_i64}]}})).expect("current own research facts");
    let state = GameState::from_server_projection(&projection, 1_u8, 42_u64);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
    let mut ui = ManagementUi::new(LayoutBook::load(&root).expect("read-only installed layouts"))
        .expect("source templates/library");
    ui.open(ManagerKind::Science, &state, None);
    (ui, state)
}
/// Exact shared descriptor used by native labels and accessible interactions.
fn label<'widgets>(widgets: &'widgets [ManagerWidget], id: &str) -> &'widgets str {
    &widgets
        .iter()
        .find(|widget| widget.id == id)
        .expect("source control")
        .label
}
#[test]
fn original_science_constructor_moves_unused_advice_button_outside_window_instead_of_leaving_a_stray_visible_control()
 {
    let (ui, state) = fixture();
    let root = ui
        .layouts
        .get(&ManagerKind::Science)
        .expect("source science");
    let button = root
        .child("AdvisorButton")
        .expect("installed optional button");
    assert_eq!(
        button.integer("xpix"),
        Some(100_i32),
        "original constructor Move(-X,-Y) uses absolute positions, not Offset"
    );
    assert_eq!(
        button.integer("ypix"),
        Some(-354_i32),
        "inherited nav-button Y is moved above clipped parent"
    );
    assert!(
        !ui.widgets(&state)
            .iter()
            .any(|widget| widget.id == "AdvisorButton" || widget.id.starts_with("AdvisorButton.")),
        "off-window source advisor does not paint, focus or expose a fake disabled Advice control"
    );
}
#[test]
fn active_science_uses_same_original_radial_and_turn_value_with_received_zero_cost_and_current_forecast_only()
 {
    let (ui, mut state) = fixture();
    let fields = ui.widgets(&state);
    assert_eq!(
        label(&fields, "CurrentResearch.Spent.Value"),
        "0",
        "received zero spending"
    );
    assert_eq!(
        label(&fields, "CurrentResearch.TotalCost.Value"),
        "570",
        "received authoritative cost, not installed/local simulation"
    );
    for id in [
        "CurrentResearch.Turn.Value",
        "CurrentResearch.AdvanceProgress.IconBorder.IconButton.RadialButton",
    ] {
        assert_eq!(
            label(&fields, id),
            "-",
            "original active unavailable turn is hyphen in both source controls"
        );
    }
    let research = state
        .viewer
        .as_mut()
        .expect("viewer")
        .interface
        .research
        .as_mut()
        .expect("research");
    research.forecast = Some(
        serde_json::from_value(
            json!({"revision":7_i64,"status":"estimated","turns":3_u32,"rate":20_i64,"assumption":"current effective rate"}),
        )
        .expect("current forecast"),
    );
    let forecast_widgets = ui.widgets(&state);
    for id in [
        "CurrentResearch.Turn.Value",
        "CurrentResearch.AdvanceProgress.IconBorder.IconButton.RadialButton",
    ] {
        assert_eq!(
            label(&forecast_widgets, id),
            "3",
            "use current supplied turns, never add original engine's internal off-by-one convention to a server estimate"
        );
    }
    state
        .viewer
        .as_mut()
        .expect("viewer")
        .interface
        .research
        .as_mut()
        .expect("research")
        .forecast
        .as_mut()
        .expect("forecast")
        .revision = 6_i64;
    let stale_widgets = ui.widgets(&state);
    assert_eq!(
        label(&stale_widgets, "CurrentResearch.Turn.Value"),
        "-",
        "stale estimate unavailable"
    );
}
#[test]
fn known_no_target_or_already_discovered_target_clears_original_current_controls_and_icon_but_unprojected_research_is_unavailable()
 {
    let (ui, mut state) = fixture();
    for completed in [false, true] {
        let research = state
            .viewer
            .as_mut()
            .expect("viewer")
            .interface
            .research
            .as_mut()
            .expect("research");
        if completed {
            research.target_id = "ADVANCE_BALLISTICS".to_owned();
            research.advances.push("ADVANCE_BALLISTICS".to_owned());
        } else {
            research.target_id.clear();
        }
        let fields = ui.widgets(&state);
        for id in [
            "CurrentResearch.AdvanceProgress.Title",
            "CurrentResearch.Spent.Value",
            "CurrentResearch.TotalCost.Value",
            "CurrentResearch.Turn.Value",
            "CurrentResearch.AdvanceProgress.IconBorder.IconButton.RadialButton",
        ] {
            assert_eq!(
                label(&fields, id),
                "",
                "source ClearScience clears known inactive state"
            );
        }
        assert!(
            super::articles::image(
                &ui,
                &state,
                ui.view.as_ref().expect("science"),
                "CurrentResearch.AdvanceProgress.IconBorder.IconButton"
            )
            .is_none(),
            "ClearScience never retains a completed target portrait"
        );
    }
    state.viewer.as_mut().expect("viewer").interface.research = None;
    let fields = ui.widgets(&state);
    assert_eq!(
        label(&fields, "CurrentResearch.Spent.Value"),
        "—",
        "unprojected is not known zero/empty current state"
    );
    state.viewer = None;
    let unavailable = ui.widgets(&state);
    assert_eq!(
        label(&unavailable, "CurrentResearch.Spent.Value"),
        "—",
        "missing viewer cannot expose a source-template placeholder as an authoritative value"
    );
}
#[test]
fn discovery_selection_changes_only_source_description_and_links_not_current_research_portrait_or_target()
 {
    let (mut ui, state) = fixture();
    for (id, text) in [
        ("ADVANCE_BALLISTICS", "Current description"),
        (
            "ADVANCE_AGRICULTURE",
            "Selected description <L:DATABASE_UNITS,UNIT_SETTLER>Settler<e>",
        ),
    ] {
        ui.library
            .catalog
            .articles
            .get_mut(id)
            .expect("installed article")
            .sections
            .insert(LibrarySection::Statistics, LibraryDocument::parse(text));
    }
    let view = ui.view.as_mut().expect("science");
    view.selections
        .insert("advance".to_owned(), "ADVANCE_AGRICULTURE".to_owned());
    let fields = ui.widgets(&state);
    let words = fields
        .iter()
        .filter(|widget| widget.id.starts_with("CurrentResearch.Description/word-"))
        .map(|widget| widget.label.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        words.contains("Selected") && !words.contains("Current"),
        "source AdvanceListCallback updates description for exact selected known advance: {words}"
    );
    assert!(
        fields.iter().any(
            |widget| widget.id.starts_with("CurrentResearch.Description/")
                && widget.intent == Some(ManagerIntent::Article("UNIT_SETTLER".to_owned()))
        ),
        "description keeps exact shared library link"
    );
    assert_eq!(
        label(&fields, "CurrentResearch.AdvanceProgress.Title"),
        "Ballistics",
        "selection does not become active research"
    );
    assert_eq!(
        state
            .viewer
            .as_ref()
            .expect("viewer")
            .interface
            .research
            .as_ref()
            .expect("research")
            .target_id,
        "ADVANCE_BALLISTICS",
        "no local research command/execution"
    );
    ui.view
        .as_mut()
        .expect("science")
        .selections
        .insert("advance".to_owned(), "ADVANCE_IRON_WORKING".to_owned());
    let fallback_widgets = ui.widgets(&state);
    assert!(
        fallback_widgets.iter().any(|widget| widget
            .id
            .starts_with("CurrentResearch.Description/word-")
            && widget.label == "Current"),
        "unavailable/foreign/injected selection cannot silently replace known-list description"
    );
}

/*
 * Source-backed regression decisions:
 * ctp2_source/ctp2_code/ui/interface/ScienceManagementDialog.cpp::CompareAdvance
 * sorts the name header by cost; ctp2_source/ctp2_code/ui/aui_common/aui_header.cpp::
 * aui_HeaderSwitchAction::Execute provides ascending/descending ON states1/2.
 * science.ldl::AdvanceList.header retains all eight civilization controls, but
 * original UpdateAdvanceItem/ColorHeaderActionCallback require actual permitted
 * player/embassy facts. Missing server knowledge cannot become a fake sort key.
 * The malformed/zero fixture costs below exercise client input safety, not a
 * copied rules database or a claim that negative costs exist in released CTP2.
 */
/// Cost values intentionally disagree with alphabetical names and include unavailable facts.
fn sorted_fixture() -> (ManagementUi, GameState) {
    let (mut ui, mut state) = fixture();
    let viewer = state.viewer.as_mut().expect("viewer");
    viewer.interface.research.as_mut().expect("research").advances = ["ADVANCE_RELIGION", "ADVANCE_AGRICULTURE", "ADVANCE_TOOLMAKING", "ADVANCE_MISSING", "ADVANCE_NULL", "ADVANCE_NEGATIVE", "ADVANCE_ZERO"].into_iter().map(str::to_owned).collect();
    viewer.interface.rules = Some(json!({"advances":[{"id":"ADVANCE_TOOLMAKING","cost":196_i64},{"id":"ADVANCE_AGRICULTURE","cost":242_i64},{"id":"ADVANCE_RELIGION","cost":334_i64},{"id":"ADVANCE_NULL","cost":null},{"id":"ADVANCE_NEGATIVE","cost":-1_i64},{"id":"ADVANCE_ZERO","cost":0_i64},{"id":"ADVANCE_BALLISTICS","cost":570_i64}]}));
    ui.open(ManagerKind::Science, &state, None);
    (ui, state)
}

/// Read the exact visible row identities, not labels, cell children or array indices.
fn discovery_ids(ui: &ManagementUi, state: &GameState) -> Vec<String> {
    ui.widgets(state).into_iter().filter_map(|widget| widget.id.strip_prefix("AdvanceList/").filter(|id| !id.contains('/')).map(str::to_owned)).collect()
}

#[test]
fn science_header_switch_uses_original_cost_comparator_and_preserves_exact_selected_discovery_on_return() {
    let (mut ui, state) = sorted_fixture();
    let before = state.clone();
    let header = ui.activate_id("AdvanceList.header.switch0", &state).expect("original aui_HeaderSwitchAction is actionable");
    assert!(ui.apply_local(header), "first ON/state1 sorts ascending by source cost, not name");
    assert_eq!(discovery_ids(&ui, &state), ["ADVANCE_ZERO", "ADVANCE_TOOLMAKING", "ADVANCE_AGRICULTURE", "ADVANCE_RELIGION", "ADVANCE_MISSING", "ADVANCE_NEGATIVE", "ADVANCE_NULL"], "source name header actually compares costs; explicit0 valid, missing/null/negative unavailable last");
    let selected = ui.activate_id("AdvanceList/ADVANCE_AGRICULTURE", &state).expect("exact discovery");
    assert!(ui.apply_local(selected), "known-only local article selection");
    ui.view.as_mut().expect("science").scroll.insert("CurrentResearch.Description".to_owned(), 2_usize);
    let descending = ui.activate_id("AdvanceList.header.switch0", &state).expect("second ON/state2");
    assert!(ui.apply_local(descending), "source descending callback");
    assert_eq!(discovery_ids(&ui, &state), ["ADVANCE_RELIGION", "ADVANCE_AGRICULTURE", "ADVANCE_TOOLMAKING", "ADVANCE_ZERO", "ADVANCE_MISSING", "ADVANCE_NEGATIVE", "ADVANCE_NULL"], "descending never moves unknown costs ahead or reverses an unrelated chooser");
    let selected_view = ui.view.clone();
    ui.open_related(ManagerKind::Library, &state);
    ui.close();
    assert_eq!(ui.view, selected_view, "sort order, exact advance and its current description scroll survive return");
    assert_eq!(ui.view.as_ref().expect("science").selections.get("advance").map(String::as_str), Some("ADVANCE_AGRICULTURE"), "selected identity stays stable as its row moves");
    assert_eq!(state, before, "header never changes current target/goal/knowledge or submits");
}

#[test]
fn science_header_default_cost_order_never_treats_missing_null_or_negative_as_cheaper_than_known_zero() {
    let (ui, state) = sorted_fixture();
    assert_eq!(discovery_ids(&ui, &state), ["ADVANCE_ZERO", "ADVANCE_TOOLMAKING", "ADVANCE_AGRICULTURE", "ADVANCE_RELIGION", "ADVANCE_MISSING", "ADVANCE_NEGATIVE", "ADVANCE_NULL"], "source initial ascending cost order with unavailable-last policy");
    let research = state.viewer.as_ref().expect("viewer").interface.research.as_ref().expect("research");
    assert_eq!(research.target_id, "ADVANCE_BALLISTICS", "current research isn't a discovery selection or sorting target");
}

#[test]
fn science_header_cannot_sort_fabricated_nation_slot_or_foreign_knowledge_sets() {
    let (ui, state) = sorted_fixture();
    let fields = ui.widgets(&state);
    for column in 1_u8..=8_u8 {
        let id = format!("AdvanceList.header.switch{column}");
        let header = fields.iter().find(|widget| widget.id == id).expect("every original civilization header remains present");
        assert!(header.intent.is_none(), "unknown original nation slot/embassy sets aren't false/empty");
        assert!(header.unavailable.as_ref().is_some_and(|reason| reason.contains("knowledge")), "unprojected knowledge sort must explain why unavailable instead of silent no-op");
    }
}
