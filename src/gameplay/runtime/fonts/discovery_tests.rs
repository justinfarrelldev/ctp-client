//! Font lookup uses injected presence probes, never file writes or host fonts.
use super::discovery::find_with;
use crate::gameplay::ui_model::management::typography::SourceFont;
use std::path::{Path, PathBuf};
#[test]
fn selected_installation_language_and_default_exact_faces_precede_every_system_fallback() {
    for name in [
        "times.ttf",
        "timesbd.ttf",
        "timesi.ttf",
        "timesbi.ttf",
        "arial.ttf",
    ] {
        let key = SourceFont {
            name: name.to_owned(),
            ..SourceFont::default()
        };
        let mut probes = Vec::new();
        let result = find_with(Some(Path::new("selected")), &key, |path| {
            probes.push(path.to_path_buf());
            true
        });
        assert_eq!(
            result,
            Some((
                Path::new("selected/ctp2_data/english/uidata/fonts").join(name),
                true
            )),
            "selected language exact font precedes another installation/system"
        );
        assert_eq!(
            probes.len(),
            1_usize,
            "do not probe unrelated directories after an exact match"
        );
        let expected = Path::new("selected/ctp2_data/default/uidata/fonts").join(name);
        let default_result = find_with(Some(Path::new("selected")), &key, |path| path == expected);
        assert_eq!(
            default_result,
            Some((expected, true)),
            "default installation resource is the next exact face"
        );
    }
}
#[test]
fn missing_installation_faces_keep_an_explicit_style_specific_platform_fallback() {
    let key = SourceFont {
        name: "timesbi.ttf".to_owned(),
        ..SourceFont::default()
    };
    let expected =
        PathBuf::from("/usr/share/fonts/truetype/liberation2/LiberationSerif-BoldItalic.ttf");
    assert_eq!(
        find_with(Some(Path::new("no-fonts")), &key, |path| path == expected),
        Some((expected, false)),
        "lookalike is not an exact named face"
    );
    assert!(
        find_with(None, &key, |_path| false).is_none(),
        "no invented file/face if all candidates are absent"
    );
}
#[test]
fn source_font_names_are_safe_basenames_on_both_platforms_before_any_probe() {
    for name in [
        "../times.ttf",
        "/private/times.ttf",
        "C:\\Windows\\Fonts\\times.ttf",
        "nested/times.ttf",
        "nested\\times.ttf",
        "times.ttf\0",
        ".ttf",
        "",
    ] {
        let mut probes = 0_usize;
        let key = SourceFont {
            name: name.to_owned(),
            ..SourceFont::default()
        };
        assert!(
            find_with(Some(Path::new("selected")), &key, |_path| {
                probes = probes.saturating_add(1_usize);
                true
            })
            .is_none(),
            "unsafe source font cannot select external path: {name:?}"
        );
        assert_eq!(probes, 0_usize, "reject names before filesystem discovery");
    }
}
