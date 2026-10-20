//! Session-owned source fonts: correct faces/sizes without leaks or per-frame loading.

use crate::gameplay::assets::layout::LayoutNode;
use crate::gameplay::ui_model::management::{ManagementUi, typography::SourceFont};
use crate::gameplay::{GameplayError, runtime_error};

/// Source font/word-geometry agreement without filesystem writes.
#[cfg(test)]
mod tests;
use alloc::collections::BTreeMap;
use sdl2::ttf::{Font, FontStyle, Hinting, Sdl2TtfContext};
use std::path::{Path, PathBuf};

/// Loaded once per SDL session/capture; lifetimes enforce shutdown before TTF context.
pub(super) struct ManagementFonts<'ttf> {
    /// Resolved source styles; no raw pointers or globally leaked TTF resources.
    faces: BTreeMap<SourceFont, Font<'ttf, 'static>>,
}
impl<'ttf> ManagementFonts<'ttf> {
    /// Loads existing system fonts read-only, reporting explicit fallback choices.
    pub(super) fn load(
        context: &'ttf Sdl2TtfContext,
        ui: &ManagementUi,
    ) -> Result<Self, GameplayError> {
        let mut faces = BTreeMap::new();
        for key in ui.font_keys() {
            let (path, exact) = find_font(&key)
                .ok_or_else(|| runtime_error(format!("No installed font for {}", key.name)))?;
            if !exact {
                log::warn!(
                    "Source font {} unavailable; using {}. Exact font pixel parity cannot be certified.",
                    key.name,
                    path.display()
                );
            }
            let mut font = context
                .load_font(&path, key.pixel_size())
                .map_err(runtime_error)?;
            font.set_hinting(Hinting::Mono);
            let mut style = FontStyle::NORMAL;
            if key.bold {
                style |= FontStyle::BOLD;
            }
            if key.italic {
                style |= FontStyle::ITALIC;
            }
            font.set_style(style);
            faces.insert(key, font);
        }
        Ok(Self { faces })
    }
    /// Loads and measures once before accepting interactive source-control input.
    pub(super) fn load_measured(
        context: &'ttf Sdl2TtfContext,
        ui: &mut ManagementUi,
    ) -> Result<Self, GameplayError> {
        let fonts = Self::load(context, ui)?;
        fonts.measure(ui);
        Ok(fonts)
    }

    /// Same style for native painting and measured semantic link rectangles.
    pub(super) fn get(&self, key: &SourceFont) -> Option<&Font<'ttf, 'static>> {
        self.faces.get(key)
    }
    /// Warms the pure UI metadata using actual loaded source-style glyph metrics.
    pub(super) fn measure(&self, ui: &mut ManagementUi) {
        ui.measure_source_fonts(
            |key, text| {
                self.get(key)
                    .and_then(|font| font.size_of(text).ok())
                    .map_or(0_u32, |size| size.0)
            },
            |key| {
                self.get(key)
                    .and_then(|font| u32::try_from(font.height()).ok())
                    .unwrap_or(16_u32)
            },
        );
    }
    /// Query typing/pointer prefix measurements use the actual field style too.
    pub(super) fn measure_query(&self, ui: &mut ManagementUi) {
        let key = ui
            .layouts
            .get(&crate::gameplay::ui_model::management::ManagerKind::Library)
            .and_then(|root| root.child("SearchWord"))
            .map_or_else(SourceFont::default, SourceFont::from_node);
        if let Some(font) = self.get(&key) {
            ui.measure_query(|text| font.size_of(text).map_or(0_u32, |size| size.0));
        }
    }
}

/// Borrowed frame font choices keep legacy map/HUD drawing unchanged.
#[derive(Clone, Copy)]
pub(super) struct RenderFonts<'draw, 'ttf, 'rwops> {
    /// Existing gameplay font for map/panel and explicit bitmap-only tests.
    pub body: Option<&'draw Font<'ttf, 'rwops>>,
    /// Source controls use the session's actual per-control font styles.
    pub management: Option<&'draw ManagementFonts<'ttf>>,
}
impl<'draw, 'ttf, 'rwops> From<Option<&'draw Font<'ttf, 'rwops>>>
    for RenderFonts<'draw, 'ttf, 'rwops>
{
    fn from(body: Option<&'draw Font<'ttf, 'rwops>>) -> Self {
        Self {
            body,
            management: None,
        }
    }
}
impl<'draw, 'ttf, 'rwops> RenderFonts<'draw, 'ttf, 'rwops> {
    /// Native source style when available, or the caller's explicit fallback.
    pub(super) fn control(self, node: &LayoutNode) -> Option<&'draw Font<'ttf, 'rwops>> {
        self.management
            .and_then(|fonts| fonts.get(&SourceFont::from_node(node)))
            .or(self.body)
    }
}

/// Source family names may select only ordinary existing font locations.
fn find_font(key: &SourceFont) -> Option<(PathBuf, bool)> {
    let basename = Path::new(&key.name).file_name()?;
    for directory in [
        "C:/Windows/Fonts",
        "/usr/share/fonts/truetype/msttcorefonts",
    ] {
        let path = Path::new(directory).join(basename);
        if path.try_exists().unwrap_or(false) {
            return Some((path, true));
        }
    }
    let serif = key.name.contains("times") || key.name.contains("serif");
    let bold = key.bold
        || key.name.contains("bd")
        || key.name.contains("bold")
        || key.name.contains("bi.");
    let italic = key.italic || key.name.contains("italic") || key.name.ends_with("i.ttf");
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
        if path.try_exists().unwrap_or(false) {
            return Some((path, false));
        }
    }
    super::find_existing_font().map(|path| (path, false))
}
