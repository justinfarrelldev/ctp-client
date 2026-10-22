//! Source footer descriptions are local, semantic and never overwrite receipts.
use super::{
    InGameUiModel,
    management::{ManagementUi, ManagerKind},
};
use crate::gameplay::{
    assets::layout::LayoutBook,
    model::{GameState, NetworkStatus},
    server::ServerProjection,
};
use serde_json::json;
use std::path::Path;
/// Only read installed layouts; all pointer/state changes remain in memory.
fn fixture(kind: ManagerKind) -> (InGameUiModel, GameState) {
    let viewer:ServerProjection=serde_json::from_value(json!({"revision":1_i64,"turn":1_u32,"contract":{"version":2_u32},"tiles":[],"units":[],"cities":[]})).expect("complete empty viewer");
    let mut state = GameState::from_server_projection(&viewer, 1, 1);
    state.popups.clear();
    let mut ui = InGameUiModel {
        management: ManagementUi::new(
            LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"))
                .expect("read-only installation"),
        )
        .expect("cached layouts"),
        ..InGameUiModel::default()
    };
    ui.management.open(kind, &state, None);
    (ui, state)
}
/// Pointer is derived from an exact accessible identity, never a hardcoded click.
fn hover(ui: &mut InGameUiModel, state: &GameState, id: &str) {
    let rect = ui
        .management
        .widgets(state)
        .into_iter()
        .find(|widget| widget.id == id)
        .expect("semantic source control")
        .rect;
    ui.pointer = Some((
        rect.x
            .saturating_add(i32::try_from(rect.width.div_euclid(2)).expect("source width")),
        rect.y
            .saturating_add(i32::try_from(rect.height.div_euclid(2)).expect("source height")),
    ));
}
#[test]
fn original_advice_footer_uses_imported_status_text_and_clears_without_erasing_operational_state() {
    let (mut ui, mut state) = fixture(ManagerKind::Unit);
    ui.status_text = "Order queued. End Turn to close planning.".to_owned();
    hover(&mut ui, &state, "AdviceButton");
    assert_eq!(
        ui.footer_status(&state),
        ui.management
            .book
            .label("STATUSBAR_UNITMAN_MAIN_SHOWADV_BUTTON"),
        "native hover matches semantic source metadata"
    );
    assert_eq!(
        ui.source_status_for_id("AdviceButton", &state).as_deref(),
        Some(
            ui.management
                .book
                .label("STATUSBAR_UNITMAN_MAIN_SHOWADV_BUTTON")
        ),
        "automation reads the same caption by exact ID"
    );
    ui.pointer = None;
    assert_eq!(
        ui.footer_status(&state),
        ui.status_text,
        "MouseMoveAway releases only local description"
    );
    hover(&mut ui, &state, "AdviceButton");
    state.network = NetworkStatus::Pending;
    assert_eq!(
        ui.footer_status(&state),
        ui.status_text,
        "pending operation status is never covered by a tooltip"
    );
    state.network = NetworkStatus::Resynchronizing;
    assert_eq!(
        ui.footer_status(&state),
        ui.status_text,
        "recovery remains visible"
    );
}
#[test]
fn unavailable_quote_thumb_explains_actual_contract_gap_instead_of_an_unrelated_parent_tip() {
    let (mut ui, state) = fixture(ManagerKind::Trade);
    let id = "TradeTabs.Market.TabPanel.Filters.CitiesSlider/thumb";
    hover(&mut ui, &state, id);
    assert!(
        ui.footer_status(&state).contains("quotes"),
        "visible disabled source slider has an actionable explanation"
    );
    assert!(
        ui.source_status_for_id(id, &state)
            .is_some_and(|reason| reason.contains("quotes")),
        "same semantic disabled reason"
    );
}
#[test]
fn keyboard_focus_has_the_same_description_without_losing_pointer_or_game_state() {
    let (mut ui, state) = fixture(ManagerKind::Unit);
    ui.pointer = Some((0_i32, 0_i32));
    ui.management.focused = Some("AdviceButton".to_owned());
    ui.keyboard_status = true;
    assert_eq!(
        ui.footer_status(&state),
        ui.management
            .book
            .label("STATUSBAR_UNITMAN_MAIN_SHOWADV_BUTTON"),
        "explicit keyboard modality uses exact focused control"
    );
    ui.keyboard_status = false;
    assert_eq!(
        ui.footer_status(&state),
        "Unexplored",
        "pointer modality does not leave a stale keyboard tooltip"
    );
}
#[test]
fn missing_internal_caption_keys_and_covered_managers_are_not_displayed() {
    let (mut ui, mut state) = fixture(ManagerKind::Unit);
    ui.status_text = "Authoritative result".to_owned();
    let root = ui
        .management
        .layouts
        .get_mut(&ManagerKind::Unit)
        .expect("cached root");
    root.children
        .iter_mut()
        .find(|node| node.name == "AdviceButton")
        .expect("control")
        .fields
        .insert(
            "statustext".to_owned(),
            "STATUSBAR_UNKNOWN_IMPORT".to_owned(),
        );
    hover(&mut ui, &state, "AdviceButton");
    assert_eq!(
        ui.footer_status(&state),
        "Authoritative result",
        "never expose an untranslated internal key"
    );
    ui.management.focused = Some("CloseButton".to_owned());
    ui.keyboard_status = true;
    ui.messages.selected = Some("covered".to_owned());
    assert_eq!(
        ui.footer_status(&state),
        "Authoritative result",
        "a message modal owns the interaction layer"
    );
    ui.messages.selected = None;
    state.popups.push(crate::gameplay::model::GameplayPopup {
        title: "Notice".to_owned(),
        body: "Confirmed notice".to_owned(),
        kind: crate::gameplay::model::PopupKind::Info,
    });
    assert_eq!(
        ui.footer_status(&state),
        "Authoritative result",
        "same for game popups"
    );
}
