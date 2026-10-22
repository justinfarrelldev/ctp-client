//! Optional frame art is injected without modifying installed resources.
use super::{BTreeSet, LayoutBook, optional_frame_art};
#[test]
fn reference_extensions_prefer_actual_art_and_mark_tiled_retail_fallbacks() {
    let book = LayoutBook::parse(&[
        r#"Root { Background { string image03 "uptg23d.tga" string image05 "uptg23f.tga" } }"#,
    ])
    .expect("in-memory old source");
    let mut missing = book.resolve("Root").expect("source");
    optional_frame_art(&mut missing, Some(&BTreeSet::new()));
    let fallback = &missing.children[0];
    assert_eq!(
        fallback.field("reference-art-unavailable"),
        Some("dw_lb.rim,dw_rb.rim"),
        "exact absent reference art blocks acceptance"
    );
    assert_eq!(
        (fallback.field("image03"), fallback.field("imagestretchy03")),
        (Some("uptg23d.tga"), Some("true")),
        "reuse installed original columns, not invented or copied art"
    );
    let mut present = book.resolve("Root").expect("source");
    optional_frame_art(
        &mut present,
        Some(&BTreeSet::from([
            "dw_lb.rim".to_owned(),
            "dw_rb.rim".to_owned(),
        ])),
    );
    let original = &present.children[0];
    assert_eq!(
        (original.field("image03"), original.field("image05")),
        (Some("dw_lb.tga"), Some("dw_rb.tga")),
        "available exact original reference pieces are preferred"
    );
    assert!(
        original.field("reference-art-unavailable").is_none(),
        "no false missing claim"
    );
}
