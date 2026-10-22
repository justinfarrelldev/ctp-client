//! Installed Great Library articles and non-executable hyperlink/variable markup.
//!
//! The original bracketed text stays in the user's installation. Database
//! expressions are retained as data and resolved explicitly from viewer-safe
//! server definitions; no script, source expression or path is executed.

use alloc::collections::BTreeMap;
use std::path::Path;

use super::database::{field, records};
use crate::gameplay::{GameplayError, asset_error};

/// Original per-topic sections used by library, science and production descriptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LibrarySection {
    /// Requires/costs/abilities.
    Prerequisites,
    /// Effects or statistics.
    Statistics,
    /// Compact build-manager description.
    Summary,
    /// Gameplay explanation.
    Gameplay,
    /// Historical background.
    Historical,
}

/// One safe inline span; hyperlinks retain their canonical target separately.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibrarySpan {
    /// Plain localized text, including newlines.
    Text(String),
    /// Internal article reference, never a filesystem/network destination.
    Link {
        /// Canonical topic ID from the source database link.
        target: String,
        /// Original localized visible label.
        label: String,
    },
    /// Original database expression, interpreted only by explicit field mappings.
    Variable(String),
}

impl LibrarySpan {
    /// Borrows literal displayed text for native metric collection, excluding expressions.
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Borrow text payloads without duplicating installed articles"
    )]
    pub fn literal(&self) -> Option<&str> {
        match self {
            Self::Text(text) | Self::Link { label: text, .. } => Some(text.as_str()),
            Self::Variable(_) => None,
        }
    }
}

/// Parsed paragraph with links and explicit unresolved expressions.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LibraryDocument {
    /// Ordered spans; whitespace/newlines remain source-controlled.
    pub spans: Vec<LibrarySpan>,
}

impl LibraryDocument {
    /// Parses only known link/variable syntax; malformed markup remains literal.
    #[must_use]
    pub fn parse(source: &str) -> Self {
        let mut spans = Vec::new();
        let mut rest = source;
        while !rest.is_empty() {
            if let Some(link) = rest.strip_prefix("<L:") {
                if let Some((header, after_header)) = link.split_once('>') {
                    if let Some((_, target)) = header.split_once(',') {
                        if let Some((label, tail)) = after_header.split_once("<e>") {
                            spans.push(LibrarySpan::Link {
                                target: target.trim().to_owned(),
                                label: label.to_owned(),
                            });
                            rest = tail;
                            continue;
                        }
                    }
                }
            }
            if let Some(expression) = rest.strip_prefix('{') {
                if let Some((value, tail)) = expression.split_once('}') {
                    spans.push(LibrarySpan::Variable(value.trim().to_owned()));
                    rest = tail;
                    continue;
                }
            }
            let boundary = [rest.find("<L:"), rest.find('{')]
                .into_iter()
                .flatten()
                .filter(|index| *index > 0_usize)
                .min();
            if let Some((text, tail)) = boundary.and_then(|index| rest.split_at_checked(index)) {
                spans.push(LibrarySpan::Text(text.to_owned()));
                rest = tail;
            } else {
                spans.push(LibrarySpan::Text(rest.to_owned()));
                break;
            }
        }
        Self { spans }
    }

    /// Produces display text only through a caller-supplied safe variable resolver.
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Borrow non-Copy article spans instead of cloning the complete installed text to match by value"
    )]
    pub fn plain_text(&self, mut resolve: impl FnMut(&str) -> String) -> String {
        let mut result = String::new();
        for span in &self.spans {
            match span {
                LibrarySpan::Text(text) => result.push_str(text),
                LibrarySpan::Link { label, .. } => result.push_str(label),
                LibrarySpan::Variable(expression) => result.push_str(&resolve(expression)),
            }
        }
        result
    }
}

/// One installed topic; neither its text nor its name establishes game rules.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LibraryArticle {
    /// Canonical source content ID.
    pub id: String,
    /// Original section documents.
    pub sections: BTreeMap<LibrarySection, LibraryDocument>,
}

/// Source visual/article links from the active all-category uniticon database.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LibraryArt {
    /// Original library still picture, not a stale advanceicon movie placeholder.
    pub first_frame: Option<String>,
    /// Original science/management icon.
    pub icon: Option<String>,
    /// Original native management-list icon, with explicit medium-icon fallback.
    pub small_icon: Option<String>,
    /// Compact source `StatText` article tag used by the Build Manager.
    pub summary: Option<String>,
    /// Source Vari article tag used by research descriptions.
    pub statistics: Option<String>,
}

/// Full read-only article index, usable without SDL or a live server.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LibraryCatalog {
    /// Source topic IDs mapped to sections; no artwork is embedded or extracted.
    pub articles: BTreeMap<String, LibraryArticle>,
    /// Active original visual/text associations, loaded read-only when installed.
    pub art: BTreeMap<String, LibraryArt>,
}

impl LibraryCatalog {
    /// Reads the original localized library, preserving retail Windows-1252 text.
    ///
    /// # Errors
    /// Reports missing text or malformed section delimiters rather than writing a cache.
    pub fn load(root: &Path) -> Result<Self, GameplayError> {
        let path = root.join("ctp2_data/english/gamedata/Great_Library.txt");
        let bytes = std::fs::read(&path)
            .map_err(|error| asset_error(format!("{}: {error}", path.display())))?;
        let mut catalog = Self::parse(&super::text::decode(&bytes))?;
        let icons = std::fs::read(root.join("ctp2_data/default/gamedata/uniticon.txt"))
            .map_err(|error| asset_error(error.to_string()))?;
        for (key, fields) in records(&super::text::decode(&icons)) {
            if let Some(id) = key.strip_prefix("ICON_") {
                let image = |name| {
                    field(&fields, name)
                        .filter(|value| !value.eq_ignore_ascii_case("null"))
                        .map(|value| value.to_ascii_lowercase().replace(".tga", ".rim"))
                };
                let article = |name| {
                    field(&fields, name)
                        .filter(|value| !value.eq_ignore_ascii_case("null"))
                        .map(str::to_owned)
                };
                catalog.art.insert(
                    id.to_owned(),
                    LibraryArt {
                        first_frame: image("FirstFrame"),
                        icon: image("Icon"),
                        small_icon: image("SmallIcon"),
                        summary: article("StatText"),
                        statistics: article("Vari"),
                    },
                );
            }
        }
        Ok(catalog)
    }

    /// Parses bracketed source sections entirely in memory.
    ///
    /// # Errors
    /// Rejects unmatched `[END]`, text outside a section and missing terminators.
    pub fn parse(text: &str) -> Result<Self, GameplayError> {
        let mut catalog = Self::default();
        let mut current: Option<String> = None;
        let mut body = String::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed == "[END]" {
                let name = current
                    .take()
                    .ok_or_else(|| asset_error("library END without a section".to_owned()))?;
                let (id, section) = topic_and_section(&name);
                let article =
                    catalog
                        .articles
                        .entry(id.to_owned())
                        .or_insert_with(|| LibraryArticle {
                            id: id.to_owned(),
                            sections: BTreeMap::new(),
                        });
                article
                    .sections
                    .insert(section, LibraryDocument::parse(body.trim()));
                body.clear();
            } else if current.is_none() {
                if let Some(name) = trimmed
                    .strip_prefix('[')
                    .and_then(|value| value.strip_suffix(']'))
                {
                    current = Some(name.to_owned());
                } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    return Err(asset_error(
                        "library text outside a bracketed section".to_owned(),
                    ));
                }
            } else {
                if !body.is_empty() {
                    body.push('\n');
                }
                body.push_str(line);
            }
        }
        if let Some(name) = current {
            return Err(asset_error(format!("unterminated library section {name}")));
        }
        Ok(catalog)
    }

    /// Resolves exact topics; unknown links never read arbitrary paths or URLs.
    #[must_use]
    pub fn article(&self, id: &str) -> Option<&LibraryArticle> {
        self.articles.get(id)
    }

    /// Resolves an exact source section tag; it can never become a filename or URL.
    #[must_use]
    pub fn document(&self, tag: &str) -> Option<&LibraryDocument> {
        let (id, section) = topic_and_section(tag);
        self.article(id)?.sections.get(&section)
    }
}

/// Unsuffixed concept topics use their source text as gameplay explanation.
fn topic_and_section(name: &str) -> (&str, LibrarySection) {
    for (suffix, section) in [
        ("_PREREQ", LibrarySection::Prerequisites),
        ("_STATISTICS", LibrarySection::Statistics),
        ("_SUMMARY", LibrarySection::Summary),
        ("_GAMEPLAY", LibrarySection::Gameplay),
        ("_HISTORICAL", LibrarySection::Historical),
    ] {
        if let Some(id) = name.strip_suffix(suffix) {
            return (id, section);
        }
    }
    (name, LibrarySection::Gameplay)
}
