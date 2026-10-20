//! Source font choices/metrics are data, independently testable without SDL or files.

use super::typography::{FontMetrics, SourceFont};
use crate::gameplay::assets::layout::LayoutNode;

#[test]
fn source_points_use_original_96_dpi_and_do_not_turn_regular_headers_into_bold_body_text() {
    let mut node = LayoutNode::default();
    node.fields
        .insert("fontname".to_owned(), "times.ttf".to_owned());
    node.fields.insert("fontsize".to_owned(), "8".to_owned());
    let header = SourceFont::from_node(&node);
    assert_eq!(header.name, "times.ttf", "actual regular source family");
    assert_eq!(header.pixel_size(), 10_u16, "original 96-dpi 8-point font");
    let body = SourceFont::default();
    assert_eq!(body.pixel_size(), 13_u16, "10-point body");
    node.fields.insert("fontsize".to_owned(), "14".to_owned());
    assert_eq!(
        SourceFont::from_node(&node).pixel_size(),
        18_u16,
        "14-point title"
    );
}

#[test]
fn measured_style_words_preserve_real_extents_and_numeric_glyph_fallbacks() {
    let mut metrics = FontMetrics::default();
    metrics.words.insert("known".to_owned(), 19_u32);
    metrics.words.insert("1".to_owned(), 3_u32);
    metrics.words.insert("2".to_owned(), 4_u32);
    assert_eq!(
        metrics.width("known"),
        19_u32,
        "actual measured word extent"
    );
    assert_eq!(
        metrics.width("12"),
        7_u32,
        "new authoritative values use measured glyph advances"
    );
}
