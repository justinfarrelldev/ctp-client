//! Canonical string imports use an injected reader; no test writes any files.
use super::strings::load_with_reader;
use std::{io, path::Path};

#[test]
fn imported_addon_and_tooltip_labels_resolve_in_source_order_and_empty_values_stay_paired() {
    let labels = load_with_reader(Path::new("virtual"), |path| {
        let content = match path.file_name().and_then(|name| name.to_str()) {
            Some("Strings.txt") => {
                "import \"add_str.txt\"\nimport \"gl_str.txt\"\nimport \"tips_str.txt\""
            }
            Some("add_str.txt") => {
                "str_ldl_MilitaryAdvisor \"MILITARY ADVISOR\"\nDUPLICATE \"First\"\nEMPTY \"\""
            }
            Some("gl_str.txt") => "DUPLICATE \"Later\"\nUNIT_SETTLER \"Settler\"",
            Some("tips_str.txt") => "STATUS_UNIT \"Show the advisor\"",
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "unexpected virtual asset",
                ));
            }
        };
        Ok(content.as_bytes().to_vec())
    })
    .expect("fileless canonical imports");
    assert_eq!(
        labels.get("str_ldl_MilitaryAdvisor").map(String::as_str),
        Some("MILITARY ADVISOR"),
        "actual addon strings are not raw missing keys"
    );
    assert_eq!(
        labels.get("DUPLICATE").map(String::as_str),
        Some("First"),
        "StringDB retains the first base definition"
    );
    assert_eq!(
        labels.get("EMPTY").map(String::as_str),
        Some(""),
        "empty translations cannot shift following pairs"
    );
    assert_eq!(
        labels.get("STATUS_UNIT").map(String::as_str),
        Some("Show the advisor"),
        "tooltips come from the same import graph"
    );
    assert!(
        !labels.contains_key("MILITARY ADVISOR"),
        "translated text is never an ID"
    );
}
#[test]
fn string_imports_reject_traversal_cycles_and_contextualize_missing_files() {
    for import in [
        "../outside.txt",
        "/etc/passwd",
        "C:\\outside.txt",
        "Strings.txt",
        "missing.txt",
    ] {
        let mut reads = 0_usize;
        let result = load_with_reader(Path::new("virtual"), |path| {
            reads = reads.saturating_add(1);
            if path.file_name().and_then(|name| name.to_str()) == Some("Strings.txt") {
                Ok(format!("import \"{import}\"").into_bytes())
            } else {
                Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "missing imported file",
                ))
            }
        });
        assert!(
            result.is_err(),
            "unsafe/cyclic/missing import is not silently ignored: {import}"
        );
        assert!(
            reads <= 2,
            "traversal and recursion cannot expand the read boundary"
        );
    }
}
#[test]
fn installation_loads_actual_advisor_addon_captions_instead_of_exposing_internal_keys() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
    let book = super::LayoutBook::load(&root).expect("read-only original string manifest");
    assert_eq!(
        book.label("str_ldl_MilitaryAdvisor"),
        "MILITARY ADVISOR",
        "source Strings.txt imports add_str.txt"
    );
    assert_eq!(
        book.label("str_ldl_TradeAdvisor"),
        "TRADE ADVISOR",
        "both docked source titles resolve"
    );
}
