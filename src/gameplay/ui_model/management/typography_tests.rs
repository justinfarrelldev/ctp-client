//! Source font choices/metrics are data, independently testable without SDL or files.

use super::typography::{FontMetrics, SourceFont};
use crate::gameplay::assets::layout::{LayoutBook, LayoutNode};

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

/// Runtime-created map fonts must respect installed table values and safe defaults.
#[test]
fn map_font_table_preserves_literal_variant_and_rejects_invalid_point_sizes() {
    let parsed = LayoutBook::parse(&[r#"
        TiledMapFontStringTable { string string0 "ariali.ttf" string string1 "11" }
    "#]);
    assert!(parsed.is_ok(), "valid injected map table");
    let variant = SourceFont::tiled_map(&parsed.unwrap_or_default());
    assert_eq!(
        variant.name, "ariali.ttf",
        "keep actual installed variant face"
    );
    assert_eq!(
        variant.points, 11_u16,
        "keep actual installed variant points"
    );
    for invalid in ["0", "-1", "bad", "70000"] {
        let source = format!("TiledMapFontStringTable {{ string string1 \"{invalid}\" }}");
        let book = LayoutBook::parse(&[&source]).unwrap_or_default();
        let fallback = SourceFont::tiled_map(&book);
        assert_eq!(
            fallback.name, "arialbd.ttf",
            "missing face uses released map default"
        );
        assert_eq!(
            fallback.points, 9_u16,
            "invalid size uses released map default"
        );
    }
    assert_eq!(
        SourceFont::tiled_map(&LayoutBook::default()).points,
        9_u16,
        "absent table preserves original map default"
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
