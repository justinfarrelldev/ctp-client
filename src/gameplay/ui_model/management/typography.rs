//! Original source font metadata and measured word geometry, independent of SDL.

use super::ManagementUi;
use crate::gameplay::assets::layout::LayoutNode;
use crate::gameplay::assets::library::LibrarySpan;
use alloc::collections::{BTreeMap, BTreeSet};

/// Exact source face/point/style identity, not a gameplay rule or system-font handle.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceFont {
    /// Original installed face name, e.g. regular times.ttf versus timesbd.ttf.
    pub name: String,
    /// Original point size at the original AUI font renderer's 96 DPI.
    pub points: u16,
    /// Optional explicit bold style in addition to the named face.
    pub bold: bool,
    /// Optional explicit italic style in addition to the named face.
    pub italic: bool,
}
impl Default for SourceFont {
    fn default() -> Self {
        Self {
            name: "timesbd.ttf".to_owned(),
            points: 10_u16,
            bold: false,
            italic: false,
        }
    }
}
impl SourceFont {
    /// Reads already-inherited metadata; absent fields use the actual standard font.
    #[must_use]
    pub fn from_node(node: &LayoutNode) -> Self {
        Self {
            name: node
                .field("fontname")
                .unwrap_or("timesbd.ttf")
                .to_ascii_lowercase(),
            points: node
                .integer("fontsize")
                .and_then(|value| u16::try_from(value).ok())
                .filter(|size| *size > 0_u16)
                .unwrap_or(10_u16),
            bold: node
                .field("boldfont")
                .is_some_and(|value| value.eq_ignore_ascii_case("true")),
            italic: node
                .field("italicfont")
                .is_some_and(|value| value.eq_ignore_ascii_case("true")),
        }
    }
    /// Original `aui_BitmapFont` sets 96 DPI, not one raster pixel per source point.
    #[must_use]
    pub const fn pixel_size(&self) -> u16 {
        let size = self.points.saturating_mul(4_u16).div_euclid(3_u16);
        if size == 0_u16 { 1_u16 } else { size }
    }
}

/// One source style's native word/glyph extents for shared drawing/link hit testing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontMetrics {
    /// Exact literal words plus glyph advances for new runtime numeric values.
    pub words: BTreeMap<String, u32>,
    /// Native line height; the same style is used by the renderer.
    pub line_height: u32,
}
impl Default for FontMetrics {
    fn default() -> Self {
        Self {
            words: BTreeMap::new(),
            line_height: 16_u32,
        }
    }
}
impl FontMetrics {
    /// Prefers native whole-word kerning; unknown numeric values use native glyphs.
    #[must_use]
    pub fn width(&self, text: &str) -> u32 {
        self.words.get(text).copied().unwrap_or_else(|| {
            text.chars()
                .map(|character| {
                    self.words
                        .get(&character.to_string())
                        .copied()
                        .unwrap_or(6_u32)
                })
                .fold(0_u32, u32::saturating_add)
        })
    }
}

/// Resolves all source styles once, including runtime-created chart/age headers.
pub(super) fn keys(ui: &ManagementUi) -> BTreeSet<SourceFont> {
    let mut keys = BTreeSet::new();
    keys.insert(SourceFont::default());
    for root in ui.layouts.values().chain(core::iter::once(&ui.chart)) {
        collect(root, &mut keys);
    }
    let mut header = LayoutNode::default();
    header
        .fields
        .insert("fontname".to_owned(), "times.ttf".to_owned());
    header.fields.insert("fontsize".to_owned(), "14".to_owned());
    keys.insert(SourceFont::from_node(&header));
    keys
}

/// Mirrors inherited source trees; there is no per-frame file/font parsing.
fn collect(node: &LayoutNode, keys: &mut BTreeSet<SourceFont>) {
    keys.insert(SourceFont::from_node(node));
    for child in &node.children {
        collect(child, keys);
    }
}

/// Warms literal words and glyphs per style using injected native/fake font metrics.
pub(super) fn measure(
    ui: &mut ManagementUi,
    measure: impl Fn(&SourceFont, &str) -> u32,
    line_height: impl Fn(&SourceFont) -> u32,
) {
    let mut texts: BTreeSet<String> = (' '..='~').map(|character| character.to_string()).collect();
    for span in ui
        .library
        .catalog
        .articles
        .values()
        .flat_map(|article| article.sections.values())
        .flat_map(|document| document.spans.iter())
    {
        if let Some(text) = LibrarySpan::literal(span) {
            texts.extend(text.split_whitespace().map(str::to_owned));
            texts.extend(
                text.chars()
                    .filter(|character| !character.is_control())
                    .map(|character| character.to_string()),
            );
        }
    }
    ui.font_metrics = keys(ui)
        .into_iter()
        .map(|font| {
            let metrics = FontMetrics {
                words: texts
                    .iter()
                    .map(|text| (text.clone(), measure(&font, text)))
                    .collect(),
                line_height: line_height(&font).max(1_u32),
            };
            (font, metrics)
        })
        .collect();
}
