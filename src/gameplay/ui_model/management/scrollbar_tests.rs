//! Source ranger geometry/actions are shared by every list and article viewport.

use super::{ManagementUi, ManagerKind};
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::library::LibraryCategory;
use std::path::Path;

#[test]
fn original_index_ranger_exists_when_empty_and_tracks_bounded_real_content() {
    let state = GameState::new(8_u16, 8_u16, 1_u8, 1_u64);
    let mut ui = ManagementUi::new(
        LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"))
            .expect("read-only source"),
    )
    .expect("source controls");
    ui.open(ManagerKind::Library, &state, None);
    let empty = ui.widgets(&state);
    assert!(
        empty
            .iter()
            .any(|widget| widget.id == "IndexSheet/ranger/track"),
        "alwaysranger keeps the original empty-list scrollbar"
    );
    assert!(
        empty
            .iter()
            .find(|widget| widget.id == "IndexSheet/up")
            .expect("up arrow")
            .intent
            .is_none(),
        "beginning arrow disabled"
    );
    ui.library.set_category(LibraryCategory::Units);
    let populated = ui.widgets(&state);
    let thumb = populated
        .iter()
        .find(|widget| widget.id == "IndexSheet/ranger/thumb")
        .expect("source thumb");
    assert_eq!(
        thumb.node.field("pattern"),
        Some("upbt14.tga"),
        "installed source thumb artwork"
    );
    assert!(
        thumb.intent.is_some(),
        "real overflow has a draggable/accessible value control"
    );
    assert!(
        ui.scroll_to("IndexSheet", usize::MAX, &state),
        "absolute accessible scroll clamps"
    );
    let bottom = ui.widgets(&state);
    assert!(
        bottom
            .iter()
            .find(|widget| widget.id == "IndexSheet/down")
            .expect("down")
            .intent
            .is_none(),
        "end arrow disabled"
    );
    assert!(
        bottom
            .iter()
            .find(|widget| widget.id == "IndexSheet/up")
            .expect("up")
            .intent
            .is_some(),
        "returning remains possible"
    );
}
