//! Pure exact source serialization/portable name boundaries; no actual file effects.
use super::super::{Item, Template, codec};

#[test]
fn build_save_codec_roundtrips_exact_categories_duplicate_unicode_ids_and_correct_cap_inf_not_original_swap()
 {
    let source = Template {
        items: vec![
            Item::Unit("UNIT_SETTLER".to_owned()),
            Item::Unit("UNIT_SETTLER".to_owned()),
            Item::Building("IMPROVE_図書館".to_owned()),
            Item::Wonder("WONDER_ONE".to_owned()),
            Item::Capitalization,
            Item::Infrastructure,
        ],
    };
    let text = codec::encode(&source).expect("whole source grammar");
    assert_eq!(
        text, "U UNIT_SETTLER\nU UNIT_SETTLER\nB IMPROVE_図書館\nW WONDER_ONE\nCAP\nINF\n",
        "CAP means capitalization and INF infrastructure, not original swapped Save cases"
    );
    assert_eq!(
        codec::parse(&text),
        Ok(source),
        "complete precise roundtrip"
    );
    assert_eq!(
        codec::encode(&Template::default()),
        Ok(String::new()),
        "explicit empty template is a real empty file"
    );
}
#[test]
fn build_save_codec_rejects_invalid_ids_control_or_space_whole_oversize_and_never_truncates() {
    for id in [
        String::new(),
        "x y".to_owned(),
        "x\ny".to_owned(),
        "x\u{0000}y".to_owned(),
        "x".repeat(257_usize),
    ] {
        assert!(
            codec::encode(&Template {
                items: vec![Item::Unit("VALID".to_owned()), Item::Building(id)]
            })
            .is_err(),
            "bad trailing record never returns partial valid prefix"
        );
    }
    assert!(
        codec::encode(&Template {
            items: vec![Item::Capitalization; 257_usize]
        })
        .is_err(),
        "complete item cap"
    );
    assert!(
        codec::encode(&Template {
            items: vec![Item::Unit("x".repeat(256_usize)); 256_usize]
        })
        .is_err(),
        "individually valid records can exceed whole byte cap"
    );
    assert!(
        codec::encode(&Template {
            items: vec![Item::Unit("x".repeat(256_usize))]
        })
        .is_ok(),
        "explicit max ID remains valid"
    );
}
#[test]
fn build_save_name_bounds_are_utf8_safe_portable_and_literal_not_paths_devices_or_printf() {
    for name in [
        "../escape",
        "a/b",
        "a\\b",
        "NUL.txt",
        "COM1",
        "foo:",
        "a\n",
        "name.",
        "name ",
        ".hidden",
        "",
    ] {
        assert!(!super::name_valid(name), "unsafe basename rejected");
    }
    assert!(
        super::name_valid(&"a".repeat(255_usize)),
        "source255-byte name bound"
    );
    assert!(
        !super::name_valid(&"a".repeat(256_usize)),
        "no overlong platform name"
    );
    assert!(
        super::name_valid(&"界".repeat(85_usize)),
        "exact255 UTF-8 bytes accepted"
    );
    assert!(
        !super::name_valid(&"界".repeat(86_usize)),
        "never byte-truncate Unicode"
    );
    assert!(
        super::name_valid("日本 %s 100%.queue"),
        "literal percent and Unicode no substitution or script evaluation"
    );
}
