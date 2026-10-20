//! Original library text is read-only; markup/variable handling is tested in memory.

use super::library::{LibraryCatalog, LibrarySection, LibrarySpan};
use std::path::Path;

#[test]
fn source_articles_keep_sections_links_unicode_and_unevaluated_variables() {
    let catalog=LibraryCatalog::parse("[ADVANCE_TEST_PREREQ]\nRequires <L:DATABASE_ADVANCES,ADVANCE_OTHER>Other Advance<e>\nCost: {AdvanceDB(Advance[0]).Cost}\n[END]\n[ADVANCE_TEST_GAMEPLAY]\nA Unicode city’s history.\n[END]").expect("synthetic source article");
    let article = catalog.article("ADVANCE_TEST").expect("article identity");
    let prerequisite = article
        .sections
        .get(&LibrarySection::Prerequisites)
        .expect("prerequisite section");
    assert!(
        prerequisite.spans.iter().any(|span| *span
            == LibrarySpan::Link {
                target: "ADVANCE_OTHER".to_owned(),
                label: "Other Advance".to_owned()
            }),
        "hyperlink target and visible label retained separately"
    );
    assert!(
        prerequisite
            .spans
            .iter()
            .any(|span| *span == LibrarySpan::Variable("AdvanceDB(Advance[0]).Cost".to_owned())),
        "source expressions are data, never executable client code"
    );
    assert!(
        article.sections[&LibrarySection::Gameplay]
            .plain_text(|_| "—".to_owned())
            .contains("city’s"),
        "UTF-8 Unicode remains intact"
    );
    assert_eq!(
        prerequisite.plain_text(|_| "570".to_owned()),
        "Requires Other Advance\nCost: 570",
        "explicit resolver controls displayed values"
    );
}

#[test]
fn malformed_sections_and_markup_fail_safely_without_any_script_execution() {
    assert!(
        LibraryCatalog::parse("[UNIT_A_GAMEPLAY]\nmissing end").is_err(),
        "unterminated section rejected"
    );
    assert!(
        LibraryCatalog::parse("[END]").is_err(),
        "unmatched end rejected"
    );
    let catalog = LibraryCatalog::parse(
        "[UNIT_A_GAMEPLAY]\n<L:missing>unfinished {arbitrary_script()}\n[END]",
    )
    .expect("malformed inline markup is literal text");
    let text = catalog.article("UNIT_A").expect("article").sections[&LibrarySection::Gameplay]
        .plain_text(|_| "unknown".to_owned());
    assert!(
        text.contains("unfinished"),
        "broken link must not drop the rest of the article"
    );
    assert!(!text.is_empty(), "malformed markup does not hang or panic");
}

#[test]
fn installed_articles_cover_categories_and_retail_reference_topics_without_copying_text() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
    let catalog = LibraryCatalog::load(&root).expect("read installed library");
    assert!(
        catalog.articles.len() > 300_usize,
        "full installed reference, not a single hardcoded paragraph"
    );
    assert_eq!(
        catalog.art["ADVANCE_BALLISTICS"].icon.as_deref(),
        Some("upap012l.rim"),
        "active all-category uniticon, not the obsolete advanceicon table"
    );
    assert!(
        catalog
            .document(
                catalog.art["UNIT_SETTLER"]
                    .summary
                    .as_deref()
                    .expect("source summary tag")
            )
            .is_some(),
        "production summary resolves its actual source section"
    );
    for id in [
        "ADVANCE_BALLISTICS",
        "UNIT_SETTLER",
        "IMPROVE_GRANARY",
        "CONCEPT_CITY",
    ] {
        assert!(
            catalog.article(id).is_some(),
            "installed reference topic {id}"
        );
    }
}
