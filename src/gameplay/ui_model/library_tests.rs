//! Fileless library navigation, filtering and safe rich-text geometry.

use super::library::layout::layout_document;
use super::library::{LibraryBrowser, LibraryCategory};
use crate::gameplay::assets::library::{LibraryCatalog, LibraryDocument};

/// Synthetic articles, not copied retail passages or a simulated server.
fn browser() -> LibraryBrowser {
    LibraryBrowser::new(LibraryCatalog::parse("[ADVANCE_ALPHA_GAMEPLAY]\nSee <L:DATABASE_UNITS,UNIT_ARCHER>an Archer<e>.\n[END]\n[UNIT_ARCHER_GAMEPLAY]\nA test unit.\n[END]\n[CONCEPT_CITY_GAMEPLAY]\nA test city.\n[END]\n[TERRAIN_PLAINS_GOOD_ONE_GAMEPLAY]\nA test commodity.\n[END]\n[TERRAIN_PLAINS_GAMEPLAY]\nA test terrain.\n[END]").expect("synthetic library"))
}

#[test]
fn categories_search_links_and_history_are_distinct_from_game_state() {
    let mut browser = browser();
    assert!(
        browser.entries(str::to_owned).is_empty(),
        "opening an article does not implicitly populate the original index"
    );
    assert!(browser.navigate("ADVANCE_ALPHA"), "open exact topic");
    assert!(
        browser.navigate("UNIT_ARCHER"),
        "follow installed internal topic"
    );
    assert_eq!(
        browser.category,
        LibraryCategory::Units,
        "link follows its target category"
    );
    assert!(browser.go_back(), "web-style back");
    assert_eq!(
        browser.current.as_deref(),
        Some("ADVANCE_ALPHA"),
        "previous article"
    );
    assert!(browser.go_forward(), "web-style next");
    assert!(browser.navigate("CONCEPT_CITY"), "new topic");
    assert!(
        !browser.go_forward(),
        "new navigation discards obsolete forward history"
    );
    assert!(
        !browser.navigate("../../private/file"),
        "unknown targets never become paths"
    );
    browser.set_category(LibraryCategory::Units);
    browser.set_query("arch");
    browser.execute_search(str::to_owned);
    assert_eq!(
        browser.entries(str::to_owned).len(),
        2_usize,
        "original search covers labels and linked article text across all categories"
    );
    browser.set_query("absent");
    browser.execute_search(str::to_owned);
    assert!(
        browser.entries(str::to_owned).is_empty(),
        "empty results do not invent an article"
    );
    browser.set_category(LibraryCategory::Goods);
    assert_eq!(
        browser.entries(str::to_owned)[0_usize].0,
        "TERRAIN_PLAINS_GOOD_ONE",
        "original goods use terrain-good IDs"
    );
    browser.set_category(LibraryCategory::Terrain);
    assert_eq!(
        browser.entries(str::to_owned).len(),
        1_usize,
        "commodity topics are not ordinary terrain"
    );
}

#[test]
fn excessive_scroll_clamps_to_last_full_page_without_re_evaluating_variables() {
    let document = LibraryDocument::parse("first\n{known}\nlast");
    let mut calls = 0_u32;
    let fragments = layout_document(
        &document,
        80_u32,
        20_u32,
        10_u32,
        |word| u32::try_from(word.len()).unwrap_or(u32::MAX),
        |_| {
            calls = calls.saturating_add(1_u32);
            "middle".to_owned()
        },
        usize::MAX,
    );
    assert_eq!(
        fragments
            .iter()
            .map(|fragment| fragment.text.as_str())
            .collect::<Vec<_>>(),
        vec!["middle", "last"],
        "overscroll cannot hide every article word"
    );
    assert_eq!(
        calls, 1_u32,
        "retrying a clamped layout does not evaluate a variable twice"
    );
}

#[test]
fn rich_layout_preserves_actual_measured_words_wrapping_links_and_variable_values() {
    let doc = LibraryDocument::parse(
        "Cost: {Cost}\n<L:DATABASE_CONCEPTS,CONCEPT_CITY>land cities<e> are known.",
    );
    let spans = layout_document(
        &doc,
        60_u32,
        80_u32,
        8_u32,
        |text| {
            u32::try_from(text.chars().count())
                .unwrap_or(u32::MAX)
                .saturating_mul(6_u32)
        },
        |_| "570".to_owned(),
        0_usize,
    );
    assert!(
        spans.iter().any(|span| span.text == "570"),
        "explicit variable value"
    );
    assert!(
        spans
            .iter()
            .any(|span| span.text == "land" && span.target.as_deref() == Some("CONCEPT_CITY")),
        "link target survives wrapping"
    );
    assert!(
        spans
            .iter()
            .any(|span| span.text == "cities" && span.y > 8_i32),
        "multiword link wraps to a later line"
    );
    assert!(
        spans.iter().all(|span| u32::try_from(span.x)
            .unwrap_or(u32::MAX)
            .saturating_add(span.width)
            <= 60_u32),
        "all clickable glyph extents stay inside the viewport"
    );
    let scrolled = layout_document(
        &doc,
        60_u32,
        8_u32,
        8_u32,
        |text| {
            u32::try_from(text.chars().count())
                .unwrap_or(u32::MAX)
                .saturating_mul(6_u32)
        },
        |_| "570".to_owned(),
        1_usize,
    );
    assert!(
        scrolled.iter().all(|span| span.y == 0_i32),
        "scrolling clips whole source text lines"
    );
}
