//! Measured rich-text lines and hyperlink hit rectangles, independent of SDL.

use crate::gameplay::assets::library::{LibraryDocument, LibrarySpan};
use alloc::collections::BTreeMap;

/// One visible word; its exact bounds are shared by drawing and link activation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryFragment {
    /// Source word or explicitly resolved variable value.
    pub text: String,
    /// Exact installed internal topic, never an external URL/path.
    pub target: Option<String>,
    /// Horizontal offset inside the text viewport.
    pub x: i32,
    /// Vertical offset after line-local scrolling.
    pub y: i32,
    /// Measured glyph extent, clipped for a single oversized glyph.
    pub width: u32,
    /// Font line spacing used for the semantic hit rectangle.
    pub height: u32,
}

/// Visible source words and the largest offset retaining a full final page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryLayout {
    /// Shared native glyph/hit rectangles.
    pub fragments: Vec<LibraryFragment>,
    /// Maximum whole-line scroll offset for the current width/font/content.
    pub max_scroll: usize,
}

/// Lays out visible words using injected metrics and non-executable variables.
#[must_use]
pub fn layout_document(
    document: &LibraryDocument,
    width: u32,
    height: u32,
    line_height: u32,
    measure: impl Fn(&str) -> u32,
    resolve: impl FnMut(&str) -> String,
    scroll: usize,
) -> Vec<LibraryFragment> {
    document_layout(
        document,
        width,
        height,
        line_height,
        measure,
        resolve,
        scroll,
    )
    .fragments
}

/// Measures content extent and clamps overscroll without evaluating variables twice.
#[must_use]
pub fn document_layout(
    document: &LibraryDocument,
    width: u32,
    height: u32,
    line_height: u32,
    measure: impl Fn(&str) -> u32,
    mut resolve: impl FnMut(&str) -> String,
    scroll: usize,
) -> LibraryLayout {
    if width == 0_u32 || height == 0_u32 {
        return LibraryLayout {
            fragments: Vec::new(),
            max_scroll: 0_usize,
        };
    }
    let mut values = BTreeMap::new();
    let mut cursor = Cursor {
        width,
        height,
        line_height: line_height.max(1_u32),
        scroll,
        x: 0_u32,
        line: 0_usize,
        measure: &measure,
        fragments: Vec::new(),
    };
    cursor.document(document, &mut resolve, &mut values);
    let visible = usize::try_from(height.div_ceil(cursor.line_height)).unwrap_or(usize::MAX);
    let max_scroll = cursor.line.saturating_add(1_usize).saturating_sub(visible);
    if scroll > max_scroll {
        cursor.scroll = max_scroll;
        cursor.x = 0_u32;
        cursor.line = 0_usize;
        cursor.fragments.clear();
        cursor.document(document, &mut resolve, &mut values);
    }
    LibraryLayout {
        fragments: cursor.fragments,
        max_scroll,
    }
}

/// Bounded viewport state kept separate from text/game ownership.
struct Cursor<'measure, Measure> {
    /// Available width.
    width: u32,
    /// Available height.
    height: u32,
    /// Native font line spacing.
    line_height: u32,
    /// Number of whole source lines hidden above the viewport.
    scroll: usize,
    /// Current horizontal glyph origin.
    x: u32,
    /// Current logical line, before scrolling.
    line: usize,
    /// Injected native or deterministic font metrics.
    measure: &'measure Measure,
    /// Visible words and their exact clickable extents.
    fragments: Vec<LibraryFragment>,
}

impl<Measure: Fn(&str) -> u32> Cursor<'_, Measure> {
    /// Memoizes explicit values so clamped retries see the same snapshot values.
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Borrow non-Copy spans rather than cloning the full source article for pattern matching"
    )]
    fn document(
        &mut self,
        document: &LibraryDocument,
        resolve: &mut impl FnMut(&str) -> String,
        values: &mut BTreeMap<String, String>,
    ) {
        for span in &document.spans {
            match span {
                LibrarySpan::Text(text) => self.text(text, None),
                LibrarySpan::Link { target, label } => self.text(label, Some(target)),
                LibrarySpan::Variable(expression) => self.text(
                    values
                        .entry(expression.clone())
                        .or_insert_with(|| resolve(expression)),
                    None,
                ),
            }
        }
    }

    /// Preserves explicit spaces/newlines instead of flattening paragraph breaks.
    fn text(&mut self, text: &str, target: Option<&str>) {
        for piece in text.split_inclusive(char::is_whitespace) {
            let word = piece.trim_end_matches(char::is_whitespace);
            if !word.is_empty() {
                self.word(word, target);
            }
            for space in piece.strip_prefix(word).unwrap_or("").chars() {
                match space {
                    '\n' => self.newline(),
                    '\r' => {}
                    '\t' => {
                        self.x = self
                            .x
                            .saturating_add((self.measure)(" ").saturating_mul(4_u32));
                    }
                    _ => self.x = self.x.saturating_add((self.measure)(" ")),
                }
            }
        }
    }

    /// Splits an oversized word at valid Unicode character boundaries.
    fn word(&mut self, word: &str, target: Option<&str>) {
        if (self.measure)(word) <= self.width {
            self.emit(word, target);
            return;
        }
        let mut chunk = String::new();
        for character in word.chars() {
            let mut next = chunk.clone();
            next.push(character);
            if !chunk.is_empty() && (self.measure)(&next) > self.width {
                self.emit(&chunk, target);
                self.newline();
                chunk.clear();
            }
            chunk.push(character);
        }
        if !chunk.is_empty() {
            self.emit(&chunk, target);
        }
    }

    /// Emits one clipped visible glyph rectangle; offscreen words are not stored.
    fn emit(&mut self, word: &str, target: Option<&str>) {
        let width = (self.measure)(word).min(self.width);
        if self.x > 0_u32 && self.x.saturating_add(width) > self.width {
            self.newline();
        }
        if let Some(relative) = self
            .line
            .checked_sub(self.scroll)
            .and_then(|line| u32::try_from(line).ok())
        {
            let y = relative.saturating_mul(self.line_height);
            if y < self.height {
                self.fragments.push(LibraryFragment {
                    text: word.to_owned(),
                    target: target.map(str::to_owned),
                    x: i32::try_from(self.x).unwrap_or(i32::MAX),
                    y: i32::try_from(y).unwrap_or(i32::MAX),
                    width,
                    height: self.line_height.min(self.height.saturating_sub(y)),
                });
            }
        }
        self.x = self.x.saturating_add(width);
    }

    /// Begins the next source line with saturating counters, never division by zero.
    const fn newline(&mut self) {
        self.x = 0_u32;
        self.line = self.line.saturating_add(1_usize);
    }
}
