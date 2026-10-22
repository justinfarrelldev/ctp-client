//! Original installation resources precede platform font directories.
use crate::gameplay::ui_model::management::typography::SourceFont;
use std::path::{Path, PathBuf};
/// Original `civ3_main` adds selected-language fonts, then default fonts, then
/// Windows fonts. Discovery only reads regular files in these bounded locations.
pub(in crate::gameplay::runtime) fn find(
    root: Option<&Path>,
    key: &SourceFont,
) -> Option<(PathBuf, bool)> {
    find_with(root, key, |path| {
        path.metadata().is_ok_and(|metadata| metadata.is_file())
    })
}
/// Injected probes make source search order/path safety testable without files.
pub(super) fn find_with(
    root: Option<&Path>,
    key: &SourceFont,
    mut present: impl FnMut(&Path) -> bool,
) -> Option<(PathBuf, bool)> {
    let name = key.name.to_ascii_lowercase();
    if !safe_name(&name) {
        return None;
    }
    if let Some(installation) = root {
        for directory in [
            "ctp2_data/english/uidata/fonts",
            "ctp2_data/default/uidata/fonts",
        ] {
            let path = installation.join(directory).join(&name);
            if present(&path) {
                return Some((path, true));
            }
        }
    }
    for directory in [
        "C:/Windows/Fonts",
        "/usr/share/fonts/truetype/msttcorefonts",
    ] {
        let path = Path::new(directory).join(&name);
        if present(&path) {
            return Some((path, true));
        }
    }
    let serif = name.contains("times") || name.contains("serif");
    let bold = key.bold || name.contains("bd") || name.contains("bold") || name.contains("bi.");
    let italic = key.italic || name.contains("italic") || name.ends_with("i.ttf");
    let family = if serif {
        "LiberationSerif"
    } else {
        "LiberationSans"
    };
    let style = match (bold, italic) {
        (true, true) => "BoldItalic",
        (true, false) => "Bold",
        (false, true) => "Italic",
        (false, false) => "Regular",
    };
    for directory in [
        "/usr/share/fonts/truetype/liberation2",
        "/usr/share/fonts/truetype/liberation",
    ] {
        let path = Path::new(directory).join(format!("{family}-{style}.ttf"));
        if present(&path) {
            return Some((path, false));
        }
    }
    for candidate in [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/truetype/freefont/FreeSansBold.ttf",
        "C:/Windows/Fonts/arialbd.ttf",
    ] {
        let path = Path::new(candidate);
        if present(path) {
            return Some((path.to_path_buf(), false));
        }
    }
    None
}
/// A source font cannot smuggle absolute/drive/parent paths across platforms.
fn safe_name(name: &str) -> bool {
    name.len() <= 128_usize
        && !name.starts_with('.')
        && Path::new(name)
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("ttf"))
        && name.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        })
}
