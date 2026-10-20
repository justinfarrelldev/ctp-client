//! Empire navigation uses its own source panels, never a substituted manager.

use super::{ManagementUi, ManagerIntent, ManagerKind};
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::model::GameState;
use crate::gameplay::server::ServerProjection;
use serde_json::json;
use std::path::Path;

/// Fileless public-shaped facts; source layout is read-only, no live server exists.
fn fixture() -> (ManagementUi, GameState) {
    let projection: ServerProjection = serde_json::from_value(
        json!({"revision":3_i64,"turn":2_u32,"diplomacy":{"government_id":"GOVERNMENT_TYRANNY"}}),
    )
    .expect("viewer");
    let state = GameState::from_server_projection(&projection, 1_u8, 1_u64);
    let mut ui = ManagementUi::new(
        LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2")).expect("source"),
    )
    .expect("layout");
    ui.open(ManagerKind::Empire, &state, None);
    (ui, state)
}

/// Distinct projected government identities, with only one authorized alternative.
fn governments() -> (ManagementUi, GameState) {
    let (mut ui, _empty) = fixture();
    let projection:ServerProjection=serde_json::from_value(json!({"revision":3_i64,"turn":2_u32,"diplomacy":{"government_id":"GOVERNMENT_TYRANNY"},"rules":{"governments":[{"id":"GOVERNMENT_TYRANNY","too_many_cities_threshold":12_i64},{"id":"GOVERNMENT_REPUBLIC","too_many_cities_threshold":20_i64},{"id":"GOVERNMENT_SOCIALISM","too_many_cities_threshold":24_i64}]},"legal_actions":[{"route":"set-government","entity_id":"human","target_id":"GOVERNMENT_REPUBLIC","fields":["government_id"],"payload":{"government_id":"GOVERNMENT_REPUBLIC"}}]})).expect("government public schema");
    let state = GameState::from_server_projection(&projection, 1_u8, 1_u64);
    ui.open(ManagerKind::Empire, &state, None);
    assert!(
        ui.apply_local(ManagerIntent::Select(
            "tab:TabGroup".to_owned(),
            "Tab2".to_owned()
        )),
        "government tab"
    );
    (ui, state)
}

#[test]
fn government_comparison_uses_explicit_eligible_identity_and_exact_enactment_payload() {
    let (mut ui, state) = governments();
    let path = "TabGroup.Tab2.TabPanel.CompareGovernment";
    assert!(
        ui.activate_id("TabGroup.Tab2.TabPanel.EnactGovernment", &state)
            .is_none(),
        "never enact an implicit first target"
    );
    let open = ui
        .activate_id(&format!("{path}.autobutton"), &state)
        .expect("source government dropdown");
    assert!(ui.apply_local(open), "explicit open");
    assert_eq!(
        ui.dropdown_choices(&state)
            .iter()
            .map(|choice| choice.0.as_str())
            .collect::<Vec<_>>(),
        vec!["GOVERNMENT_REPUBLIC"],
        "not current or unadvertised governments"
    );
    let row = ui
        .activate_id(
            &format!("{path}.window.listbox/GOVERNMENT_REPUBLIC"),
            &state,
        )
        .expect("source popup row");
    assert_eq!(
        row,
        ManagerIntent::Select("government".to_owned(), "GOVERNMENT_REPUBLIC".to_owned()),
        "stable government identity, never city selection"
    );
    assert!(ui.apply_local(row), "presentation comparison");
    let widgets = ui.widgets(&state);
    assert_eq!(
        widgets
            .iter()
            .find(|widget| widget.id == "TabGroup.Tab2.TabPanel.Cities.CurrentValue")
            .map(|widget| widget.label.as_str()),
        Some("12"),
        "current source-valued threshold"
    );
    assert_eq!(
        widgets
            .iter()
            .find(|widget| widget.id == "TabGroup.Tab2.TabPanel.Cities.CompareValue")
            .map(|widget| widget.label.as_str()),
        Some("20"),
        "explicit selected comparison threshold"
    );
    assert_eq!(
        widgets
            .iter()
            .find(|widget| widget.id == "TabGroup.Tab2.TabPanel.Growth.CompareValue")
            .map(|widget| widget.label.as_str()),
        Some("—"),
        "server does not publish source ranks; never substitute percent bonuses"
    );
    let enact = ui
        .activate_id("TabGroup.Tab2.TabPanel.EnactGovernment", &state)
        .expect("eligible enactment");
    assert!(
        matches!(enact,ManagerIntent::Submit(command) if command.revision==3_i64&&command.route=="set-government"&&command.payload==json!({"government_id":"GOVERNMENT_REPUBLIC"})),
        "exact advertised target and displayed revision"
    );
}

#[test]
fn government_headers_are_not_replaced_by_identity_and_missing_ranks_are_not_guessed() {
    let (mut ui,state)=governments();
    let widgets=ui.widgets(&state);
    assert_eq!(widgets.iter().find(|widget|widget.id=="TabGroup.Tab2.TabPanel.CurrentGovernmentLabel").map(|widget|widget.label.as_str()),Some(ui.book.label("str_ldl_DD_GOV_CURRENT")),"a label containing CurrentGovernment is not the dynamic value control");
    assert!(!ui.select_government("GOVERNMENT_SOCIALISM",&state),"a rule definition without current capability cannot select an enactment target");
    assert!(ui.select_government("GOVERNMENT_REPUBLIC",&state),"explicit current capability");
    let compare=ui.widgets(&state);
    assert_eq!(compare.iter().find(|widget|widget.id=="TabGroup.Tab2.TabPanel.Cities.Comparison").map(|widget|widget.label.as_str()),Some("<"),"repair source blank comparison-character placeholders for known numeric values");
}

#[test]
fn government_receipts_invalidate_cached_enactment_and_removed_comparison_targets() {
    let (mut ui,mut state)=governments();
    assert!(ui.select_government("GOVERNMENT_REPUBLIC",&state),"current eligible selection");
    let command = ui.activate_id("TabGroup.Tab2.TabPanel.EnactGovernment", &state).and_then(ManagerIntent::into_command).expect("enactment must be an exact advertised command");
    state.viewer.as_mut().expect("viewer").revision=4_i64;
    assert!(!command.valid_for(&state),"no stale cached enactment");
    state.viewer.as_mut().expect("viewer").interface.legal_actions.clear();
    assert!(ui.activate_id("TabGroup.Tab2.TabPanel.EnactGovernment",&state).is_none(),"removed eligibility disables enactment");
    assert!(!ui.widgets(&state).iter().any(|widget|widget.id.ends_with("CompareGovernment/selected")),"no silently substituted first caption");
}

#[test]
fn empire_details_alternates_source_groups_without_opening_national_or_mutating_game() {
    let (mut ui, state) = fixture();
    let initial = ui.widgets(&state);
    assert!(
        initial
            .iter()
            .any(|widget| widget.id.ends_with(".InformationSummary")),
        "original default summary"
    );
    assert!(
        !initial
            .iter()
            .any(|widget| widget.id.contains(".InformationDetails")),
        "source constructor hides details, not overpaints both layouts"
    );
    let intent = ui
        .activate_id("DetailsButton", &state)
        .expect("source callback");
    assert_eq!(
        intent,
        ManagerIntent::Select("empire-details".to_owned(), "true".to_owned()),
        "Details is DisplayDetails, not National Manager navigation"
    );
    assert!(ui.apply_local(intent), "local display mode");
    let details = ui.widgets(&state);
    assert!(
        details
            .iter()
            .any(|widget| widget.id.ends_with(".InformationDetails")),
        "details visible"
    );
    assert!(
        !details
            .iter()
            .any(|widget| widget.id.contains(".InformationSummary")),
        "summary hidden"
    );
    assert_eq!(
        ui.view.as_ref().expect("Empire remains open").kind,
        ManagerKind::Empire,
        "never substitute a different screen"
    );
    let reverse = ui
        .activate_id("DetailsButton", &state)
        .expect("reverse source callback");
    assert!(ui.apply_local(reverse), "summary restored");
    assert!(
        !ui.widgets(&state)
            .iter()
            .any(|widget| widget.id.contains(".InformationDetails")),
        "same button restores summary"
    );
}

#[test]
fn empire_details_button_hides_on_government_and_restores_the_exact_presentation_mode() {
    let (mut ui, state) = fixture();
    let details = ui.activate_id("DetailsButton", &state).expect("details");
    assert!(ui.apply_local(details), "details selected");
    assert!(
        ui.apply_local(ManagerIntent::Select(
            "tab:TabGroup".to_owned(),
            "Tab2".to_owned()
        )),
        "government tab"
    );
    assert!(
        !ui.widgets(&state)
            .iter()
            .any(|widget| widget.id == "DetailsButton"),
        "CauseAndEffectTabActionCallback hides Details outside domestic policy"
    );
    assert!(
        ui.apply_local(ManagerIntent::Select(
            "tab:TabGroup".to_owned(),
            "Tab1".to_owned()
        )),
        "domestic tab"
    );
    let restored = ui.widgets(&state);
    assert!(
        restored.iter().any(|widget| widget.id == "DetailsButton")
            && restored
                .iter()
                .any(|widget| widget.id.ends_with(".InformationDetails")),
        "tab re-entry restores saved mode and button"
    );
}
