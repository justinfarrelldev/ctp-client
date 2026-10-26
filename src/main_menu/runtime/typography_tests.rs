//! Shell typography follows inherited LDL faces, not one bold font everywhere.
use super::text::MenuFontCache;
use super::typography::ShellTypography;
use std::path::Path;

/// Original title and control templates differ in weight even within one screen.
#[test]
fn shell_font_faces_follow_source_inheritance_and_keep_control_identity() {
    let parsed = ShellTypography::parse(&[r#"
        template Standard { string fontname "timesbd.ttf" int fontsize 10 }
        template GreyTitle { string fontname "times.ttf" int fontsize 14 }
        SPNewGameWindow {
            CivilizationLabel:GreyTitle { }
            TribeButton:Standard { }
            Name:Standard { }
        }
        SoundWindow {
            Name:GreyTitle { }
            SFXName:Standard { }
        }
    "#]);
    assert!(parsed.is_ok(), "valid inherited source fonts");
    let typography = parsed.unwrap_or_default();
    assert_eq!(
        typography
            .face("SPNewGameWindow", "CivilizationLabel")
            .map(|face| face.name.as_str()),
        Some("times.ttf"),
        "regular Times title, not bold button font"
    );
    assert_eq!(
        typography
            .face("SPNewGameWindow", "Name")
            .map(|face| face.name.as_str()),
        Some("timesbd.ttf"),
        "same local Name identifier is a different control"
    );
    assert_eq!(
        typography
            .face("SoundWindow", "Name")
            .map(|face| face.name.as_str()),
        Some("times.ttf"),
        "screen-local title identity"
    );
    assert_eq!(
        typography
            .face("SoundWindow", "SFXName")
            .map(|face| face.name.as_str()),
        Some("timesbd.ttf"),
        "ordinary option label remains bold"
    );
    assert!(
        typography.face("SoundWindow", "removed").is_none(),
        "unknown controls use explicit standard fallback"
    );
}

/// Installed data is read-only; the production cache must paint distinct faces.
#[test]
fn shell_font_cache_uses_actual_source_title_and_button_faces() {
    let result = super::with_sdl_test_lock(|| check_source_cache().map_err(super::runtime_error));
    assert!(
        result.is_ok(),
        "production source shell typography: {result:?}"
    );
}

/// Actual font handles and metrics remain owned by the unchanged-size session cache.
fn check_source_cache() -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
    let context = sdl2::ttf::init().map_err(|error| error.to_string())?;
    let mut cache = MenuFontCache::new(Some(&context), &root);
    let fonts = cache.font_for_point_size(10_u16);
    let title = fonts
        .control("SPNewGameWindow", "CivilizationLabel")
        .ok_or_else(|| "missing source title font".to_owned())?;
    let button = fonts
        .control("SPNewGameWindow", "TribeButton")
        .ok_or_else(|| "missing source button font".to_owned())?;
    if core::ptr::eq(title, button) {
        return Err("production title still borrows the bold button face".to_owned());
    }
    let regular_style = title
        .face_style_name()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let bold_style = button
        .face_style_name()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if regular_style.contains("bold") || !bold_style.contains("bold") {
        return Err("actual font weight does not match original title/button templates".to_owned());
    }
    let first_title_width = title
        .size_of("CIVILIZATION")
        .map_err(|error| error.to_string())?
        .0;
    let same_size = cache.font_for_point_size(10_u16);
    let cached_width = same_size
        .control("SPNewGameWindow", "CivilizationLabel")
        .and_then(|font| font.size_of("CIVILIZATION").ok())
        .map(|size| size.0);
    if cached_width != Some(first_title_width) {
        return Err("unchanged menu size lost its source font metrics".to_owned());
    }
    Ok(())
}

/// Duplicate nested names must not borrow an unrelated sibling's font.
#[test]
fn shell_font_faces_keep_synthetic_styles_and_reject_ambiguous_names() {
    let parsed = ShellTypography::parse(&[r#"
        OptionsWindow {
            Styled { string fontname "arial.ttf" bool boldfont true bool italicfont true }
            Left { Name { string fontname "times.ttf" } }
            Right { Name { string fontname "arial.ttf" } }
        }
    "#]);
    assert!(parsed.is_ok(), "valid explicit source styles");
    let typography = parsed.unwrap_or_default();
    assert_eq!(
        typography.face("OptionsWindow", "Styled").map(|face| (
            face.bold,
            face.italic,
            face.name.as_str()
        )),
        Some((true, true, "arial.ttf")),
        "preserve source family, weight and slant"
    );
    assert!(
        typography.face("OptionsWindow", "Name").is_none(),
        "ambiguous sibling is not a source style match"
    );
}
