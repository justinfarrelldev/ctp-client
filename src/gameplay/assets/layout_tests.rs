//! Read-only and in-memory contracts for original LDL manager layouts.

use super::layout::LayoutBook;
use std::path::Path;

#[test]
fn inherited_child_overrides_keep_siblings_and_original_draw_order() {
    let book = LayoutBook::parse(&[r#"
        template Font { int fontsize 10 string text "a # quoted { brace }" }
        template Base:Font { int widthpix 576 int heightpix 384
            Background { int widthpix 576 string image00 "uptg20a.tga" }
            Close { int xpix 455 int ypix 354 int widthpix 80 int heightpix 20 }
        }
        Window:Base { int widthpix 800 Background { int widthpix 800 }
            Extra:Font { string text "Close" }
        } $
    "#])
    .expect("valid in-memory layout");
    let window = book.resolve("Window").expect("resolved inheritance");
    assert_eq!(window.integer("widthpix"), Some(800_i32), "local override");
    assert_eq!(
        window.integer("fontsize"),
        Some(10_i32),
        "transitive inheritance"
    );
    assert_eq!(
        window.children[0].name, "Background",
        "background remains first"
    );
    assert_eq!(
        window
            .child("Background")
            .expect("background")
            .integer("widthpix"),
        Some(800_i32),
        "recursive child override"
    );
    assert_eq!(
        window
            .child("Background")
            .expect("background")
            .field("image00"),
        Some("uptg20a.tga"),
        "unchanged child fields survive"
    );
    assert!(
        window.child("Close").is_some(),
        "inherited sibling survives"
    );
    assert_eq!(
        window.child("Extra").expect("extra").field("text"),
        Some("Close"),
        "quoted hash/braces never affect grammar"
    );
}

#[test]
fn ldl_first_template_wins_and_parent_child_attributes_precede_child_fallback_templates() {
    let book = LayoutBook::parse(&[r#"
        template Standard { int fontsize 10 string fontname "timesbd.ttf" }
        template Gold { int fontsize 16 bool shadowfont true }
        template Parent { Child { int fontsize 12 } }
        Window:Standard:Gold:Parent { Child:Gold { string text "local" } }
    "#])
    .expect("source precedence fixture");
    let window = book.resolve("Window").expect("resolved");
    assert_eq!(
        window.integer("fontsize"),
        Some(10_i32),
        "ldl_datablock::CopyAttributesFrom only fills absent attributes, so first template wins"
    );
    assert_eq!(
        window.field("shadowfont"),
        Some("true"),
        "later templates still fill missing fields"
    );
    assert_eq!(
        window.child("Child").expect("child").integer("fontsize"),
        Some(12_i32),
        "parent child attributes fill before that child's fallback templates"
    );
}

#[test]
fn malformed_or_recursive_layouts_fail_without_panicking() {
    for input in [
        "Broken { int xpix }",
        "Broken {",
        "} Unexpected",
        "Broken { string text \"unterminated",
    ] {
        assert!(
            LayoutBook::parse(&[input]).is_err(),
            "reject malformed layout {input}"
        );
    }
    let recursive =
        LayoutBook::parse(&["A:B {} B:A {} Missing:Absent {}"]).expect("syntactically valid");
    assert!(
        recursive.resolve("A").is_err(),
        "inheritance cycle rejected"
    );
    assert!(
        recursive.resolve("Missing").is_err(),
        "missing parent rejected"
    );
    assert!(
        recursive.resolve("Absent").is_err(),
        "missing root rejected"
    );
}

#[test]
fn installed_manager_geometry_and_chrome_are_original_not_guessed() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
    let book = LayoutBook::load(&root).expect("read installed layouts");
    for (name, art) in [
        ("BuildEditorWindow", "uptg25a.tga"),
        ("ScienceManagementDialog", "uptg24a.tga"),
        ("DomesticDialog", "uptg22a.tga"),
        ("UnitManager", "uptg21a.tga"),
        ("TradeManager", "uptg27a.tga"),
    ] {
        let node = book.resolve(name).expect("retail dialog");
        assert_eq!(
            node.child("Background")
                .expect("background")
                .field("image00"),
            Some(art),
            "exact {name} corner art"
        );
        assert_eq!(
            node.integer("heightpix"),
            Some(384_i32),
            "retail dialog height"
        );
    }
    assert_eq!(
        book.label("UNIT_HOPLITE"),
        "Hoplite",
        "manager labels use installed content localization too"
    );
    assert_eq!(book.label("GOLD"), "GOLD", "uppercase translated text is not another dictionary key");
    assert_eq!(book.label("10"), "10", "numeric translated text must not alias the following key");
    let science = book.resolve("ScienceManagementDialog").expect("science");
    let spent = science
        .child("CurrentResearch")
        .expect("research group")
        .child("Spent")
        .expect("spent")
        .child("Value")
        .expect("value");
    assert_eq!(
        spent.integer("heightpix"),
        Some(14_i32),
        "AUI derives unspecified text-display height from the original image, not the entire parent group"
    );
    let close = science.child("CloseButton").expect("Close");
    assert_eq!(close.integer("xpix"), Some(455_i32), "source close X");
    assert_eq!(close.integer("ypix"), Some(354_i32), "source close Y");
    assert_eq!(
        science.centered_rect(800, 600),
        (224_i32, 108_i32, 576_i32, 384_i32),
        "ctp2_Window standard minimum X"
    );
    let build = book.resolve("BuildEditorWindow").expect("build");
    assert_eq!(
        build.centered_rect(800, 600),
        (0_i32, 108_i32, 800_i32, 384_i32),
        "wide dialog clamps to frame"
    );
    assert_eq!(
        book.label("str_ldl_BuildManagerTitle"),
        "B U I L D   M A N A G E R",
        "installed localization"
    );
}
