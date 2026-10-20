//! Community sort-by-age/index controls use projected age metadata, never retail guesses.

use super::{ManagementUi, ManagerKind};
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::assets::library::LibraryCatalog;
use crate::gameplay::model::GameState;
use serde_json::json;
use std::path::Path;

/// Two artificial topics deliberately have reverse alphabetic/chronological order.
fn fixture() -> (ManagementUi, GameState) {
    let mut state = GameState::new(8_u16, 8_u16, 1_u8, 1_u64);
    state.viewer=Some(serde_json::from_value(json!({"revision":1_i64,"turn":1_u32,"rules":{
        "ages":[{"id":"AGE_EARLY","index":1_i64},{"id":"AGE_LATE","index":9_i64}],
        "advances":[{"id":"ADVANCE_ALPHA","age_id":"AGE_LATE"},{"id":"ADVANCE_BETA","age_id":"AGE_EARLY"}]
    }})).expect("public rules"));
    let mut ui = ManagementUi::new(
        LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"))
            .expect("installed layouts"),
    )
    .expect("manager controls");
    ui.library.catalog = LibraryCatalog::parse(
        "[ADVANCE_ALPHA_GAMEPLAY]\nalpha\n[END]\n[ADVANCE_BETA_GAMEPLAY]\nbeta\n[END]",
    )
    .expect("synthetic catalog");
    ui.open(ManagerKind::Library, &state, None);
    ui.library
        .set_category(crate::gameplay::ui_model::library::LibraryCategory::Advances);
    (ui, state)
}

#[test]
fn sort_by_age_uses_real_control_headers_and_exact_topic_identity() {
    let (mut ui, state) = fixture();
    let intent = ui
        .activate_id("SortByAgeButton", &state)
        .expect("source community sort control");
    assert!(ui.apply_local(intent), "view-only sort toggle");
    let widgets = ui.widgets(&state);
    let topics: Vec<_> = widgets
        .iter()
        .filter(|widget| widget.id.starts_with("IndexSheet/ADVANCE_"))
        .map(|widget| widget.id.as_str())
        .collect();
    assert_eq!(
        topics,
        vec!["IndexSheet/ADVANCE_BETA", "IndexSheet/ADVANCE_ALPHA"],
        "chronological metadata overrides alpha order"
    );
    let header = widgets
        .iter()
        .find(|widget| widget.id == "IndexSheet/@age/AGE_EARLY")
        .expect("age header");
    assert!(
        header.intent.is_none(),
        "header never navigates to a fake topic"
    );
    let button = widgets
        .iter()
        .find(|widget| widget.id == "SortByAgeButton")
        .expect("sort button");
    assert!(button.selected, "source toggle state is painted");
    assert_eq!(
        button.rect,
        crate::gameplay::ui_model::MenuRect {
            x: 107_i32,
            y: 549_i32,
            width: 140_u32,
            height: 20_u32
        },
        "community control matches reference placement"
    );
}

#[test]
fn search_and_unsupported_categories_disable_age_sort_without_inventing_age_zero() {
    let (mut ui, mut state) = fixture();
    ui.library.set_query("alpha");
    ui.search_library();
    assert!(
        ui.activate_id("SortByAgeButton", &state).is_none(),
        "source search results are not age sorted"
    );
    ui.library
        .set_category(crate::gameplay::ui_model::library::LibraryCategory::Terrain);
    assert!(
        ui.activate_id("SortByAgeButton", &state).is_none(),
        "terrain has no tech-age sort"
    );
    ui.library
        .set_category(crate::gameplay::ui_model::library::LibraryCategory::Advances);
    state.viewer.as_mut().expect("viewer").interface.rules = None;
    assert!(
        ui.activate_id("SortByAgeButton", &state).is_none(),
        "missing age data is not fabricated from installed content"
    );
}
