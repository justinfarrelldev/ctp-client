//! Shared, read-only font policy for SDL gameplay/shell and client-only GTK UI.
//!
//! No font bytes are embedded, extracted, converted or redistributed. See
//! `FONT-LICENSES.md` for installed-font permissions and open alternatives.

use std::path::{Path, PathBuf};

/// Native launcher/settings/dialog typography, separate from original SDL UI.
pub mod gtk;
/// Fileless discovery, cross-platform and licensing-policy regressions.
#[cfg(test)]
mod tests;

/// Source family and named weight/slant; explicit AUI style flags are additive.
struct Face {
    /// Whether the requested face is Times-compatible rather than Arial-compatible.
    serif: bool,
    /// Named or explicitly requested bold weight.
    bold: bool,
    /// Named or explicitly requested italic slant.
    italic: bool,
}

impl Face {
    /// Recognizes the actual shipped Times/Arial filenames, including bold italic.
    fn from_name(name: &str, bold: bool, italic: bool) -> Self {
        Self {
            serif: name.contains("times") || name.contains("serif"),
            bold: bold || name.contains("bd") || name.contains("bold") || name.contains("bi."),
            italic: italic || name.contains("italic") || name.ends_with("i.ttf"),
        }
    }

    /// Standard installed macOS names differ from Windows' short filenames.
    fn desktop_name(&self) -> String {
        let family = if self.serif {
            "Times New Roman"
        } else {
            "Arial"
        };
        let style = match (self.bold, self.italic) {
            (true, true) => " Bold Italic",
            (true, false) => " Bold",
            (false, true) => " Italic",
            (false, false) => "",
        };
        format!("{family}{style}.ttf")
    }

    /// Liberation 2 is OFL-1.1 and metric-compatible, not identical glyph artwork.
    fn liberation_name(&self) -> String {
        let family = if self.serif { "Serif" } else { "Sans" };
        let style = match (self.bold, self.italic) {
            (true, true) => "BoldItalic",
            (true, false) => "Bold",
            (false, true) => "Italic",
            (false, false) => "Regular",
        };
        format!("Liberation{family}-{style}.ttf")
    }

    /// Emergency substitutes must retain family/weight/slant, not always Sans Bold.
    fn dejavu_name(&self) -> String {
        let family = if self.serif { "Serif" } else { "Sans" };
        let style = match (self.bold, self.italic, self.serif) {
            (true, true, true) => "-BoldItalic",
            (true, true, false) => "-BoldOblique",
            (true, false, _) => "-Bold",
            (false, true, true) => "-Italic",
            (false, true, false) => "-Oblique",
            (false, false, _) => "",
        };
        format!("DejaVu{family}{style}.ttf")
    }

    /// Widely packaged GNU `FreeFont` is a final same-family alternative.
    fn freefont_name(&self) -> String {
        let family = if self.serif { "Serif" } else { "Sans" };
        let style = match (self.bold, self.italic, self.serif) {
            (true, true, true) => "BoldItalic",
            (true, true, false) => "BoldOblique",
            (true, false, _) => "Bold",
            (false, true, true) => "Italic",
            (false, true, false) => "Oblique",
            (false, false, _) => "",
        };
        format!("Free{family}{style}.ttf")
    }
}

/* Original source: ctp2_source/ctp2_code/ctp/civ3_main.cpp::ui_Initialize
 * (AddBitmapFontSearchPath calls), ui/aui_common/aui_bitmapfont.cpp::InitCommon,
 * Load and ui/aui_common/aui_textbase.cpp::InitCommonLdl/TextReloadFont load
 * installed TTF faces, never a universal bold sans. Installed English layouts
 * ctp_template.ldl::{CTP2_STANDARD_FONT,CTP2_HEADER_FONT,CTP_GREY_TITLE_FONT}
 * establish timesbd.ttf/times.ttf; controlpanel.ldl::ICON_VALUE_PAIR uses Arial.
 * images/ctp2_references/{build-manager-multiple-items-added-granary-selected,
 * science-manager-opened,great-library-opened}.png corroborate serif controls.
 * Selected-installation precedence is retained only with explicit license opt-in.
 * Microsoft's FAQ covers Windows system fonts, NOT fonts supplied by other apps;
 * it does not prove a bundled game's font license permits a replacement client.
 * Without opt-in, use system-installed fonts/open alternatives, not game-only
 * files. Platform substitutes below are a client licensing policy, NOT retail
 * source behavior or a claim of
 * pixel equality. All sizes, positions, callbacks and scaling remain unchanged.
 */
/// Finds a lawfully installed face or an explicitly nonexact common substitute.
pub fn find(root: Option<&Path>, name: &str, bold: bool, italic: bool) -> Option<(PathBuf, bool)> {
    let mut directories = Vec::new();
    if let Some(windows) = std::env::var_os("WINDIR") {
        directories.push(PathBuf::from(windows).join("Fonts"));
    }
    if let Some(user_fonts) = dirs::font_dir() {
        directories.push(user_fonts);
    }
    let setting = std::env::var_os("CTP_USE_GAME_FONTS");
    let licensed_root = opted_in_root(root, setting.as_deref());
    find_with(licensed_root, name, bold, italic, &directories, |path| {
        path.metadata().is_ok_and(|metadata| metadata.is_file())
    })
}

/// Game-distributed font licenses are separate from the Windows system-font FAQ.
/// Only a deliberate user acknowledgement enables game-only font resources.
fn opted_in_root<'root>(
    root: Option<&'root Path>,
    setting: Option<&std::ffi::OsStr>,
) -> Option<&'root Path> {
    root.filter(|_root| setting == Some(std::ffi::OsStr::new("1")))
}

/// Injected directories and presence probes never write files or depend on host fonts.
/// The caller must already have authorized any supplied installation root.
pub fn find_with(
    root: Option<&Path>,
    name: &str,
    bold: bool,
    italic: bool,
    directories: &[PathBuf],
    mut present: impl FnMut(&Path) -> bool,
) -> Option<(PathBuf, bool)> {
    let name = name.to_ascii_lowercase();
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
    let face = Face::from_name(&name, bold, italic);
    let desktop_name = if known_desktop_face(&name) {
        Face::from_name(&name, false, false).desktop_name()
    } else {
        face.desktop_name()
    };
    for directory in directories.iter().map(PathBuf::as_path).chain([
        Path::new("C:/Windows/Fonts"),
        Path::new("/usr/share/fonts/truetype/msttcorefonts"),
        Path::new("/usr/share/fonts/msttcore"),
        Path::new("/Library/Fonts"),
        Path::new("/System/Library/Fonts/Supplemental"),
    ]) {
        for filename in [&name, &desktop_name] {
            let path = directory.join(filename);
            if present(&path) {
                return Some((path, filename == &name || known_desktop_face(&name)));
            }
        }
    }
    for filename in [
        face.liberation_name(),
        face.dejavu_name(),
        face.freefont_name(),
    ] {
        for directory in directories.iter().map(PathBuf::as_path).chain([
            Path::new("/usr/share/fonts/truetype/liberation2"),
            Path::new("/usr/share/fonts/truetype/liberation"),
            Path::new("/usr/share/fonts/liberation"),
            Path::new("/usr/share/fonts/liberation-serif"),
            Path::new("/usr/share/fonts/liberation-sans"),
            Path::new("/usr/share/fonts/truetype/dejavu"),
            Path::new("/usr/share/fonts/dejavu"),
            Path::new("/usr/share/fonts/truetype/freefont"),
            Path::new("/usr/share/fonts/gnu-free"),
            Path::new("/Library/Fonts"),
        ]) {
            let path = directory.join(&filename);
            if present(&path) {
                return Some((path, false));
            }
        }
    }
    None
}

/// A desktop alias is exact only for one of the original Times/Arial files.
fn known_desktop_face(name: &str) -> bool {
    matches!(
        name,
        "times.ttf"
            | "timesbd.ttf"
            | "timesi.ttf"
            | "timesbi.ttf"
            | "arial.ttf"
            | "arialbd.ttf"
            | "ariali.ttf"
            | "arialbi.ttf"
    )
}

/// Reject absolute/drive/parent paths and malformed names before filesystem probes.
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
