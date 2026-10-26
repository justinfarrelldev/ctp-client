//! Pure font-policy regressions: injected probes, no installed-font requirement.
use super::{find_with, gtk::FAMILY_CSS, opted_in_root};
use std::path::{Path, PathBuf};

/// A game's bundled fonts do not inherit Microsoft's Windows system-font grant.
#[test]
fn font_policy_requires_explicit_game_font_license_acknowledgement() {
    let root = Some(Path::new("selected-game"));
    for setting in [
        None,
        Some(std::ffi::OsStr::new("0")),
        Some(std::ffi::OsStr::new("true")),
    ] {
        assert!(
            opted_in_root(root, setting).is_none(),
            "unverified game fonts disabled by default"
        );
    }
    assert_eq!(
        opted_in_root(root, Some(std::ffi::OsStr::new("1"))),
        root,
        "explicit user licensing acknowledgement retains optional original installation support"
    );
}

/// Environment-specific Windows drives and user font directories are injectable.
#[test]
fn font_search_supports_non_c_windows_and_user_installed_open_fonts() {
    let roots = [
        PathBuf::from("D:/Windows/Fonts"),
        PathBuf::from("user/fonts"),
    ];
    for (expected, exact) in [
        ("D:/Windows/Fonts/timesbd.ttf", true),
        ("user/fonts/LiberationSerif-Bold.ttf", false),
    ] {
        assert_eq!(
            find_with(None, "timesbd.ttf", false, false, &roots, |path| path
                == Path::new(expected)),
            Some((PathBuf::from(expected), exact)),
            "use configured/user font paths without a fixed C drive"
        );
    }
}

/// Explicit style flags use the matching open face, including combined styles.
#[test]
fn font_substitutes_combine_named_and_explicit_style_and_prefer_liberation() {
    for (source, bold, italic, expected) in [
        ("times.ttf", true, false, "LiberationSerif-Bold.ttf"),
        ("timesbd.ttf", false, true, "LiberationSerif-BoldItalic.ttf"),
        ("ariali.ttf", true, false, "LiberationSans-BoldItalic.ttf"),
        ("arial.ttf", false, false, "LiberationSans-Regular.ttf"),
    ] {
        let expected_path = Path::new("/usr/share/fonts/truetype/liberation2").join(expected);
        assert_eq!(
            find_with(None, source, bold, italic, &[], |path| {
                path == expected_path || path.starts_with("/usr/share/fonts/truetype/dejavu")
            }),
            Some((expected_path, false)),
            "prefer the metric-compatible family with original requested style"
        );
    }
}

/// A missing Times face must not become Arial merely because Arial exists.
#[test]
fn font_search_never_crosses_source_families_or_claims_unknown_alias_exactness() {
    assert!(
        find_with(None, "times.ttf", false, false, &[], |path| path
            == Path::new("C:/Windows/Fonts/arialbd.ttf"))
        .is_none(),
        "do not turn serif text into unrelated bold sans"
    );
    let alias = PathBuf::from("/Library/Fonts/Arial.ttf");
    assert_eq!(
        find_with(None, "missing.ttf", false, false, &[], |path| path == alias),
        Some((alias, false)),
        "unknown original face cannot certify a desktop alias as exact"
    );
}

/// Native UI has no retail counterpart; sizes/weights remain accessibility-controlled.
#[test]
fn font_policy_covers_all_native_descendants_without_overriding_accessible_sizes() {
    assert!(
        FAMILY_CSS.starts_with("* {"),
        "cover entries/buttons/dialog bodies, not just the launcher root"
    );
    for family in ["Times New Roman", "Liberation Serif", "DejaVu Serif"] {
        assert!(
            FAMILY_CSS.contains(family),
            "common exact/open fallback: {family}"
        );
    }
    for forbidden in ["font-size", "font-weight", "font-style", "url("] {
        assert!(
            !FAMILY_CSS.contains(forbidden),
            "no sizing override or bundled font URL: {forbidden}"
        );
    }
}
