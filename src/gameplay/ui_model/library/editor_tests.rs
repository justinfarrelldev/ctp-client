//! Local text editing never accesses the filesystem, clipboard or authoritative state.

use super::editor::{Motion, TextEditor};

#[test]
fn unicode_selection_replacement_delete_and_undo_use_character_boundaries() {
    let mut text = "aé🙂z".to_owned();
    let mut editor = TextEditor::default();
    editor.move_caret(&text, Motion::End, false);
    editor.move_caret(&text, Motion::Left, true);
    editor.move_caret(&text, Motion::Left, true);
    assert_eq!(
        editor.selected(&text),
        "🙂z",
        "selection is not a byte-count approximation"
    );
    assert!(
        editor.insert(&mut text, "界\n\t"),
        "replace selection and reject single-line controls"
    );
    assert_eq!(text, "aé界", "Unicode replacement");
    assert!(editor.undo(&mut text), "undo replacement");
    assert_eq!(text, "aé🙂z", "exact original value");
    assert_eq!(editor.selected(&text), "🙂z", "undo restores selection");
    assert!(editor.redo(&mut text), "redo replacement");
    assert!(editor.erase(&mut text, true, false), "backspace one scalar");
    assert_eq!(text, "aé", "no partial UTF-8 character");
    editor.move_caret(&text, Motion::Home, false);
    assert!(editor.erase(&mut text, false, false), "forward delete");
    assert_eq!(text, "é", "delete affects the character after caret");
}

#[test]
fn word_navigation_and_select_all_bound_pastes_and_branching_history() {
    let mut text = "alpha βeta gamma".to_owned();
    let mut editor = TextEditor::default();
    editor.move_caret(&text, Motion::End, false);
    editor.move_caret(&text, Motion::WordLeft, true);
    assert_eq!(
        editor.selected(&text),
        "gamma",
        "Ctrl+Shift+Left selects a word"
    );
    editor.move_caret(&text, Motion::Left, false);
    assert!(
        editor.selected(&text).is_empty(),
        "plain arrow collapses selection"
    );
    editor.move_caret(&text, Motion::WordLeft, false);
    editor.move_caret(&text, Motion::WordRight, true);
    assert_eq!(
        editor.selected(&text),
        "βeta ",
        "word motion handles non-ASCII letters"
    );
    editor.select_all(&text);
    assert!(
        editor.insert(&mut text, &"界".repeat(200_usize)),
        "bounded paste"
    );
    assert_eq!(
        text.chars().count(),
        128_usize,
        "bound in scalars, not bytes"
    );
    assert!(editor.undo(&mut text), "undo paste");
    assert!(editor.insert(&mut text, "new"), "new edit branches history");
    assert!(!editor.redo(&mut text), "obsolete future is discarded");
}

#[test]
fn pointer_metrics_use_measured_prefixes_and_keep_only_the_current_query() {
    let mut metrics = super::editor::metrics::EditMetrics::default();
    metrics.measure("AV🙂", |prefix| match prefix {
        "A" => 6_u32,
        "AV" => 11_u32,
        _ => 19_u32,
    });
    assert_eq!(
        metrics.prefixes,
        vec![
            (0_usize, 0_u32),
            (1_usize, 6_u32),
            (2_usize, 11_u32),
            (6_usize, 19_u32)
        ],
        "actual kerning/prefix widths, not per-byte or per-glyph guesses"
    );
    assert_eq!(
        metrics.hit("AV🙂", 6_usize, 100_u32, 13_i32),
        2_usize,
        "pointer lands on the real nearest caret boundary"
    );
    metrics.measure("i", |_| 2_u32);
    assert_eq!(
        metrics.prefixes.len(),
        2_usize,
        "old queries are not retained as a keystroke log"
    );
}

#[test]
fn invalid_external_offsets_empty_input_and_history_capacity_are_safe() {
    let mut text = "é".to_owned();
    let mut editor = TextEditor {
        caret: usize::MAX,
        anchor: 1_usize,
        ..TextEditor::default()
    };
    assert_eq!(
        editor.selected(&text),
        "é",
        "invalid offsets clamp to valid boundaries"
    );
    assert!(
        editor.insert(&mut text, ""),
        "empty replacement deletes a selection"
    );
    assert!(text.is_empty(), "selected contents removed");
    assert!(
        !editor.erase(&mut text, true, false),
        "empty backspace is harmless"
    );
    assert!(
        !editor.insert(&mut text, "\n"),
        "rejected controls alone do not create history"
    );
    for _ in 0_u32..40_u32 {
        assert!(editor.insert(&mut text, "x"), "bounded edit");
    }
    let mut undo_count = 0_u32;
    while editor.undo(&mut text) {
        undo_count = undo_count.saturating_add(1_u32);
    }
    assert_eq!(undo_count, 32_u32, "history cannot grow without bound");
}
