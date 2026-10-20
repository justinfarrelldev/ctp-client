//! Read-only LDL inheritance and localized strings for native SDL management UI.
//!
//! This parses data, not original executable UI code. Resolution preserves child
//! order and recursively merges overrides, so installed frame art and control
//! positions remain the single visual reference. All parsing is available in
//! memory for tests; loading never extracts or writes game assets.

use alloc::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Native AUI image extents absent from explicit LDL fields.
mod geometry;

use crate::gameplay::{GameplayError, asset_error};

/// Bounded recursion for malformed or cyclic installed layouts.
const MAX_DEPTH: usize = 64;

/// One parsed or resolved LDL block, with original child draw order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LayoutNode {
    /// Instance/template identifier, local to its parent for nested controls.
    pub name: String,
    /// Ordered template inheritance; empty after resolution.
    pub bases: Vec<String>,
    /// Typed source values retained as strings without guessed defaults.
    pub fields: BTreeMap<String, String>,
    /// Nested blocks in original draw order.
    pub children: Vec<Self>,
}

impl LayoutNode {
    /// Looks up an immediate named child without confusing sibling scopes.
    #[must_use]
    pub fn child(&self, name: &str) -> Option<&Self> {
        self.children.iter().find(|child| child.name == name)
    }

    /// Reads one property, preserving an absent field as unknown.
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields.get(name).map(String::as_str)
    }

    /// Reads an explicitly declared integer property.
    #[must_use]
    pub fn integer(&self, name: &str) -> Option<i32> {
        self.field(name)?.parse().ok()
    }

    /// Implements `ctp2_Window::DoneInstantiatingThis`: centered main dialogs
    /// prefer x≥224/y≥28, then clamp wide/tall dialogs back onto the screen.
    #[must_use]
    pub fn centered_rect(&self, width: i32, height: i32) -> (i32, i32, i32, i32) {
        let window_width = self.integer("widthpix").unwrap_or(0_i32).max(0_i32);
        let window_height = self.integer("heightpix").unwrap_or(0_i32).max(0_i32);
        let x = width
            .saturating_sub(window_width)
            .div_euclid(2)
            .max(224_i32)
            .min(width.saturating_sub(window_width).max(0_i32));
        let y = height
            .saturating_sub(window_height)
            .div_euclid(2)
            .max(28_i32)
            .min(height.saturating_sub(window_height).max(0_i32));
        (x, y, window_width, window_height)
    }

    /// Fills absent fields/children in original first-template-wins order.
    fn inherit(&mut self, parent: Self) {
        for (name, value) in parent.fields {
            self.fields.entry(name).or_insert(value);
        }
        for child in parent.children {
            if let Some(inherited) = self
                .children
                .iter_mut()
                .find(|node| node.name == child.name)
            {
                inherited.inherit(child);
            } else {
                self.children.push(child);
            }
        }
    }
}

/// Installation-wide template dictionary and original localized labels.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LayoutBook {
    /// Top-level templates and dialog instances.
    nodes: BTreeMap<String, LayoutNode>,
    /// Installed string database; no localized strings are embedded as artwork.
    labels: BTreeMap<String, String>,
    /// Present only for installed-data resolution; in-memory parsing performs no I/O.
    root: Option<PathBuf>,
}

impl LayoutBook {
    /// Parses one or more LDL sources, without accessing any filesystem.
    ///
    /// # Errors
    /// Rejects malformed properties, unterminated strings/blocks and excess nesting.
    pub fn parse(sources: &[&str]) -> Result<Self, GameplayError> {
        let mut book = Self::default();
        for source in sources {
            let tokens = tokenize(source)?;
            let mut index = 0_usize;
            while let Some(token) = tokens.get(index) {
                match token.as_str() {
                    "$" => break,
                    "import" => index = index.saturating_add(2),
                    "template" => index = index.saturating_add(1),
                    _ => {
                        let node = parse_node(&tokens, &mut index, 0)?;
                        book.nodes.insert(node.name.clone(), node);
                    }
                }
            }
        }
        Ok(book)
    }

    /// Reads installed LDL text and localization; never modifies the installation.
    ///
    /// # Errors
    /// Returns contextual read/parse errors rather than substituting invented UI.
    pub fn load(root: &Path) -> Result<Self, GameplayError> {
        let directory = root.join("ctp2_data/english/uidata/layouts");
        // Only active manager dependencies: archived diplomacy.ldl is explicitly
        // commented out in civ3.ldl and contains incomplete legacy declarations.
        let paths = [
            "ctp_template.ldl",
            "civ3.ldl",
            "fancy.ldl",
            "controlpanel.ldl",
            "spnewgame.ldl",
            "citywindow.ldl",
            "citymanager.ldl",
            "citystatus.ldl",
            "editqueue.ldl",
            "civstatus.ldl",
            "unitmanager.ldl",
            "armymanager.ldl",
            "trademanager.ldl",
            "diplomacywindow.ldl",
            "DiplomacyDetails.ldl",
            "science.ldl",
            "greatlibrary.ldl",
            "infoscreen.ldl",
            "graphics.ldl",
            "sound.ldl",
        ]
        .map(|name| directory.join(name));
        let sources = paths
            .iter()
            .map(|path| {
                std::fs::read(path)
                    .map(|bytes| super::text::decode(&bytes))
                    .map_err(|error| asset_error(format!("{}: {error}", path.display())))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut book = Self::default();
        for (path, source) in paths.iter().zip(&sources) {
            let parsed = Self::parse(&[source])
                .map_err(|error| asset_error(format!("{}: {error}", path.display())))?;
            book.nodes.extend(parsed.nodes);
        }
        for filename in ["gl_str.txt", "ldl_str.txt"] {
            let strings = std::fs::read(root.join("ctp2_data/english/gamedata").join(filename))
                .map_err(|error| asset_error(error.to_string()))?;
            let tokens = tokenize(&super::text::decode(&strings))?;
            for pair in tokens.windows(2) {
                if pair[0].starts_with("str_")
                    || pair[0].chars().all(|character| {
                        character.is_ascii_uppercase()
                            || character.is_ascii_digit()
                            || character == '_'
                    })
                {
                    book.labels.insert(pair[0].clone(), pair[1].clone());
                }
            }
        }
        book.root = Some(root.to_path_buf());
        Ok(book)
    }

    /// Resolves named template inheritance, recursive child overrides and cycles.
    ///
    /// # Errors
    /// Fails for unknown roots/templates or inheritance deeper than the safe limit.
    pub fn resolve(&self, name: &str) -> Result<LayoutNode, GameplayError> {
        let node = self
            .nodes
            .get(name)
            .ok_or_else(|| asset_error(format!("unknown LDL block {name}")))?;
        let mut resolved = self.resolve_node(node, 0)?;
        if let Some(root) = self.root.as_ref() {
            geometry::complete(&mut resolved, root)?;
        }
        Ok(resolved)
    }

    /// Returns the read-only installation origin, absent for in-memory layouts.
    #[must_use]
    pub(in crate::gameplay) fn installation_root(&self) -> Option<&Path> {
        self.root.as_deref()
    }

    /// Resolves a localized label while keeping missing keys visibly identifiable.
    #[must_use]
    pub fn label<'book>(&'book self, key: &'book str) -> &'book str {
        self.labels.get(key).map_or(key, String::as_str)
    }

    /// Bounded resolver for both standalone templates and local child blocks.
    fn resolve_node(&self, node: &LayoutNode, depth: usize) -> Result<LayoutNode, GameplayError> {
        if depth >= MAX_DEPTH {
            return Err(asset_error(format!(
                "cyclic/excessive LDL inheritance at {}",
                node.name
            )));
        }
        // ldl_datablock::CopyAttributesFrom fills only absent attributes. Local
        // children stay unexpanded until inherited child attributes have filled
        // them; eagerly resolving their fallback bases would reverse precedence.
        let mut resolved = node.clone();
        resolved.bases.clear();
        for base in &node.bases {
            let parent = self.nodes.get(base).ok_or_else(|| {
                asset_error(format!("unknown LDL template {base} for {}", node.name))
            })?;
            resolved.inherit(self.resolve_node(parent, depth.saturating_add(1))?);
        }
        resolved.children = resolved
            .children
            .iter()
            .map(|child| self.resolve_node(child, depth.saturating_add(1)))
            .collect::<Result<_, _>>()?;
        Ok(resolved)
    }
}

/// Parses a complete block; properties require an explicit scalar value.
fn parse_node(
    tokens: &[String],
    index: &mut usize,
    depth: usize,
) -> Result<LayoutNode, GameplayError> {
    if depth >= MAX_DEPTH {
        return Err(asset_error("excessive LDL child nesting".to_owned()));
    }
    let name = next_token(tokens, index)?;
    if matches!(name.as_str(), "{" | "}" | ":" | "$") {
        return Err(asset_error(format!("invalid LDL block name {name}")));
    }
    let mut node = LayoutNode {
        name,
        ..LayoutNode::default()
    };
    while tokens.get(*index).is_some_and(|token| token == ":") {
        *index = index.saturating_add(1);
        node.bases.push(next_token(tokens, index)?);
    }
    if next_token(tokens, index)? != "{" {
        return Err(asset_error(format!(
            "missing opening brace for {}",
            node.name
        )));
    }
    loop {
        let token = tokens
            .get(*index)
            .ok_or_else(|| asset_error(format!("unterminated LDL block {}", node.name)))?;
        if token == "}" {
            *index = index.saturating_add(1);
            break;
        }
        if ["int", "bool", "string"]
            .iter()
            .any(|kind| token.eq_ignore_ascii_case(kind))
        {
            *index = index.saturating_add(1);
            let key = next_token(tokens, index)?;
            let value = next_token(tokens, index)?;
            if matches!(value.as_str(), "{" | "}" | ":" | "$") {
                return Err(asset_error(format!("missing LDL property value for {key}")));
            }
            node.fields.insert(key.to_ascii_lowercase(), value);
        } else {
            node.children
                .push(parse_node(tokens, index, depth.saturating_add(1))?);
        }
    }
    Ok(node)
}

/// Advances a token cursor with a descriptive EOF instead of indexing/panicking.
fn next_token(tokens: &[String], index: &mut usize) -> Result<String, GameplayError> {
    let token = tokens
        .get(*index)
        .cloned()
        .ok_or_else(|| asset_error("unexpected end of LDL input".to_owned()))?;
    *index = index.saturating_add(1);
    Ok(token)
}

/// Splits punctuation and scalars while retaining quoted whitespace/comments.
fn tokenize(source: &str) -> Result<Vec<String>, GameplayError> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '#' => {
                for next in chars.by_ref() {
                    if next == '\n' {
                        break;
                    }
                }
            }
            '"' => {
                let mut value = String::new();
                let mut terminated = false;
                while let Some(next) = chars.next() {
                    if next == '"' {
                        terminated = true;
                        break;
                    }
                    if next == '\\' && chars.peek() == Some(&'"') {
                        value.push(
                            chars
                                .next()
                                .ok_or_else(|| asset_error("unterminated LDL escape".to_owned()))?,
                        );
                    } else {
                        value.push(next);
                    }
                }
                if !terminated {
                    return Err(asset_error("unterminated LDL string".to_owned()));
                }
                tokens.push(value);
            }
            '{' | '}' | ':' | '$' => tokens.push(character.to_string()),
            whitespace if whitespace.is_whitespace() => {}
            first => {
                let mut value = first.to_string();
                while chars.peek().is_some_and(|next| {
                    !next.is_whitespace() && !matches!(*next, '#' | '"' | '{' | '}' | ':' | '$')
                }) {
                    if let Some(next) = chars.next() {
                        value.push(next);
                    }
                }
                tokens.push(value);
            }
        }
    }
    Ok(tokens)
}
