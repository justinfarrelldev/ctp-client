//! Original list frame/pane/ranger lifecycle, entirely read-only and in memory.

use super::{ManagementUi, ManagerIntent, ManagerKind, index, lists};
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::assets::layout::LayoutNode;
use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::{MenuRect, library::LibraryCategory};
use std::path::Path;

/// Installed source metadata is read-only; mutable views/geometry stay in memory.
fn fixture() -> (ManagementUi, GameState) {
    let book = LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"))
        .expect("installed source");
    let mut ui = ManagementUi::new(book).expect("resolved controls");
    let state = GameState::new(8_u16, 8_u16, 1_u8, 1_u64);
    ui.open(ManagerKind::Library, &state, None);
    (ui, state)
}

#[test]
fn list_runtime_frame_expands_around_declared_content_and_moves_its_ranger() {
    let (mut ui, state) = fixture();
    ui.apply_local(ManagerIntent::LibraryCategory(LibraryCategory::Advances));
    let widgets = ui.widgets(&state);
    let frame = widgets
        .iter()
        .find(|widget| widget.id == "IndexSheet")
        .expect("index frame");
    assert_eq!(
        frame.rect,
        MenuRect {
            x: 98_i32,
            y: 129_i32,
            width: 159_u32,
            height: 398_u32
        },
        "ctp2_ListBox::DoneInstantiatingThis expands each border by five without moving content"
    );
    let up = widgets
        .iter()
        .find(|widget| widget.id == "IndexSheet/up")
        .expect("ranger");
    assert_eq!(
        (up.rect.x, up.rect.y),
        (257_i32, 129_i32),
        "aui_ListBox::RepositionRangers uses the resized outer width/height"
    );
    let row = widgets
        .iter()
        .find(|widget| widget.id.starts_with("IndexSheet/ADVANCE_"))
        .expect("topic row");
    assert_eq!(
        (row.rect.x, row.rect.y),
        (103_i32, 134_i32),
        "the content pane stays at its declared origin, not inside an extra guessed inset"
    );
    assert_eq!(
        row.rect.width, 149_u32,
        "row selection is bounded to its content pane"
    );
    assert!(
        state.viewer.is_none(),
        "runtime geometry cannot invent authoritative state"
    );
}

#[test]
fn malformed_borders_and_empty_panes_do_not_create_clickable_phantom_rows() {
    let content = MenuRect {
        x: i32::MIN,
        y: i32::MAX,
        width: u32::MAX,
        height: 0_u32,
    };
    let mut node = LayoutNode::default();
    node.fields
        .insert("borderwidth".to_owned(), "-10".to_owned());
    node.fields
        .insert("borderheight".to_owned(), "-5".to_owned());
    assert_eq!(
        lists::frame(&node, content),
        content,
        "negative border metadata cannot invert geometry"
    );
    node.fields
        .insert("borderwidth".to_owned(), i32::MAX.to_string());
    assert_eq!(
        lists::frame(&node, content).width,
        u32::MAX,
        "oversized frame extents saturate without overflow"
    );
    for height in [0_i32, 1_i32, 19_i32] {
        let (mut ui, state) = fixture();
        ui.layouts
            .get_mut(&ManagerKind::Library)
            .expect("library")
            .children
            .iter_mut()
            .find(|child| child.name == "IndexSheet")
            .expect("index")
            .fields
            .insert("heightpix".to_owned(), height.to_string());
        ui.apply_local(ManagerIntent::LibraryCategory(LibraryCategory::Advances));
        assert!(
            !ui.widgets(&state)
                .iter()
                .any(|widget| widget.id.starts_with("IndexSheet/ADVANCE_")),
            "no full row fits in height {height}; borders must not create phantom actions"
        );
    }
}

#[test]
fn scroll_capacity_uses_content_not_expanded_border_and_clips_partial_last_rows() {
    let (mut ui, state) = fixture();
    let root = ui.layouts.get_mut(&ManagerKind::Library).expect("library");
    root.children
        .iter_mut()
        .find(|node| node.name == "IndexSheet")
        .expect("index")
        .fields
        .insert("heightpix".to_owned(), "31".to_owned());
    ui.apply_local(ManagerIntent::LibraryCategory(LibraryCategory::Advances));
    let count = index::rows(&ui, &state).len();
    ui.scroll_to("IndexSheet", usize::MAX, &state);
    assert_eq!(
        ui.view
            .as_ref()
            .expect("view")
            .scroll
            .get("IndexSheet")
            .copied(),
        Some(count.saturating_sub(1_usize)),
        "borders are not extra content capacity"
    );
    let rows: Vec<_> = ui
        .widgets(&state)
        .into_iter()
        .filter(|widget| widget.id.starts_with("IndexSheet/ADVANCE_"))
        .collect();
    assert_eq!(
        rows.len(),
        1_usize,
        "do not preserve source overfill caused by counting expanded borders as pane height"
    );
    assert_eq!(rows[0].rect.height, 20_u32, "complete source row height");
}
