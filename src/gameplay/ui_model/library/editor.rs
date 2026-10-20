//! Bounded single-line Unicode editing without OS, filesystem or game authority.

use alloc::collections::VecDeque;
use core::ops::Range;

/// Native/fake font metrics for matching caret placement to painted glyphs.
pub mod metrics;

/// An explicit user clipboard request; constructing one never accesses the OS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardRequest {
    /// Copy the exact selected text.
    Copy(String),
    /// Cut only after a successful write and only while this selection is current.
    Cut { value: String, range: Range<usize> },
    /// Paste text through the same bounded editor operation as ordinary input.
    Paste,
}

/// Maximum accepted search length, in Unicode scalars rather than UTF-8 bytes.
const CHARACTER_LIMIT: usize = 128_usize;
/// Maximum reversible edits retained in either history direction.
const HISTORY_LIMIT: usize = 32_usize;

/// Original text-field keyboard movement, independent of platform keycodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    /// Previous Unicode scalar, or collapse a selection leftward.
    Left,
    /// Next Unicode scalar, or collapse a selection rightward.
    Right,
    /// Previous word boundary.
    WordLeft,
    /// Next word boundary, including intervening whitespace.
    WordRight,
    /// Start of this single line.
    Home,
    /// End of this single line.
    End,
}

/// An in-memory undo entry; selection restoration is part of the operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditSnapshot {
    /// Search value before the edit.
    pub value: String,
    /// UTF-8 caret offset.
    pub caret: usize,
    /// UTF-8 selection anchor.
    pub anchor: usize,
}

/// Search edit state; the browser owns the actual query to avoid duplicated values.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextEditor {
    /// UTF-8 caret byte offset, normalized at every operation.
    pub caret: usize,
    /// UTF-8 selection anchor; equal to caret when selection is empty.
    pub anchor: usize,
    /// Bounded undo entries, not persistent game history.
    pub past: VecDeque<EditSnapshot>,
    /// Bounded redo entries after Undo.
    pub future: Vec<EditSnapshot>,
    /// Transient SDL/IME composition; never submitted as search or a game order.
    pub composition: String,
    /// IME caret measured in Unicode scalars in the composition.
    pub composition_cursor: usize,
    /// IME selected scalar count after its composition caret.
    pub composition_selection: usize,
}

impl TextEditor {
    /// Returns a valid selection even if a stale caller supplied invalid offsets.
    #[must_use]
    pub fn range(&self, text: &str) -> Range<usize> {
        let caret = boundary(text, self.caret);
        let anchor = boundary(text, self.anchor);
        caret.min(anchor)..caret.max(anchor)
    }

    /// Borrows only selected characters; arbitrary strings never become paths.
    #[must_use]
    pub fn selected<'text>(&self, text: &'text str) -> &'text str {
        text.get(self.range(text)).unwrap_or("")
    }

    /// Returns the normalized active caret independently of selection direction.
    #[must_use]
    pub fn caret_offset(&self, text: &str) -> usize {
        boundary(text, self.caret)
    }

    /// Moves/extends the caret using character boundaries, not unchecked slicing.
    pub fn move_caret(&mut self, text: &str, motion: Motion, extend: bool) {
        let selection = self.range(text);
        let caret = boundary(text, self.caret);
        let next = match motion {
            Motion::Left if !extend && !selection.is_empty() => selection.start,
            Motion::Right if !extend && !selection.is_empty() => selection.end,
            Motion::Left => previous(text, caret),
            Motion::Right => next(text, caret),
            Motion::WordLeft => word_left(text, caret),
            Motion::WordRight => word_right(text, caret),
            Motion::Home => 0_usize,
            Motion::End => text.len(),
        };
        self.set_caret(text, next, extend);
    }

    /// Pointer/accessibility placement uses the same normalized byte boundaries.
    pub fn set_caret(&mut self, text: &str, offset: usize, extend: bool) {
        self.caret = boundary(text, offset);
        self.anchor = if extend {
            boundary(text, self.anchor)
        } else {
            self.caret
        };
        self.clear_composition();
    }

    /// Selects all Unicode characters without copying the query.
    pub fn select_all(&mut self, text: &str) {
        self.anchor = 0_usize;
        self.caret = text.len();
        self.clear_composition();
    }

    /// Replaces selection with bounded single-line input and records one undo step.
    pub fn insert(&mut self, text: &mut String, input: &str) -> bool {
        let range = self.range(text);
        let removed = text.get(range.clone()).unwrap_or("").chars().count();
        let available =
            CHARACTER_LIMIT.saturating_sub(text.chars().count().saturating_sub(removed));
        let value: String = input
            .chars()
            .filter(|character| !character.is_control())
            .take(available)
            .collect();
        if !input.is_empty() && value.is_empty() {
            return false;
        }
        if text.get(range.clone()) == Some(value.as_str()) {
            return false;
        }
        self.remember(text);
        // range() has normalized both bounds against this exact UTF-8 string.
        text.replace_range(range.clone(), &value);
        self.caret = range.start.saturating_add(value.len());
        self.anchor = self.caret;
        self.clear_composition();
        true
    }

    /// Removes a selection, scalar or whole word in the chosen direction.
    pub fn erase(&mut self, text: &mut String, backward: bool, word: bool) -> bool {
        if self.range(text).is_empty() {
            let motion = match (backward, word) {
                (true, true) => Motion::WordLeft,
                (true, false) => Motion::Left,
                (false, true) => Motion::WordRight,
                (false, false) => Motion::Right,
            };
            self.move_caret(text, motion, true);
        }
        self.insert(text, "")
    }

    /// Restores value and selection while keeping the reverse history bounded.
    pub fn undo(&mut self, text: &mut String) -> bool {
        let Some(snapshot) = self.past.pop_back() else {
            return false;
        };
        self.future.push(self.snapshot(text));
        self.restore(text, snapshot);
        true
    }

    /// Restores an undone edit; editing a new branch clears this future.
    pub fn redo(&mut self, text: &mut String) -> bool {
        let Some(snapshot) = self.future.pop() else {
            return false;
        };
        self.push_past(self.snapshot(text));
        self.restore(text, snapshot);
        true
    }

    /// Field resets do not allow Undo to resurrect a previous category's query.
    pub fn reset(&mut self, text: &str) {
        *self = Self {
            caret: text.len(),
            anchor: text.len(),
            ..Self::default()
        };
    }

    /// Keeps composition separate until SDL commits actual `TextInput`.
    pub fn compose(&mut self, text: &str, start: i32, length: i32) {
        self.composition = text
            .chars()
            .filter(|character| !character.is_control())
            .take(CHARACTER_LIMIT)
            .collect();
        let count = self.composition.chars().count();
        self.composition_cursor = usize::try_from(start).unwrap_or(0_usize).min(count);
        self.composition_selection = usize::try_from(length)
            .unwrap_or(0_usize)
            .min(count.saturating_sub(self.composition_cursor));
    }

    /// Cancels an unfinished IME composition without closing the Library window.
    pub fn clear_composition(&mut self) {
        self.composition.clear();
        self.composition_cursor = 0_usize;
        self.composition_selection = 0_usize;
    }

    /// Captures normalized positions before a reversible mutation.
    fn snapshot(&self, text: &str) -> EditSnapshot {
        EditSnapshot {
            value: text.to_owned(),
            caret: boundary(text, self.caret),
            anchor: boundary(text, self.anchor),
        }
    }
    /// A new edit invalidates only redo history, not unrelated browser navigation.
    fn remember(&mut self, text: &str) {
        self.push_past(self.snapshot(text));
        self.future.clear();
    }
    /// Evicts the oldest undo entry before exceeding the fixed capacity.
    fn push_past(&mut self, snapshot: EditSnapshot) {
        if self.past.len() >= HISTORY_LIMIT {
            self.past.pop_front();
        }
        self.past.push_back(snapshot);
    }
    /// History entries are always created from valid strings and normalized offsets.
    fn restore(&mut self, text: &mut String, snapshot: EditSnapshot) {
        *text = snapshot.value;
        self.caret = boundary(text, snapshot.caret);
        self.anchor = boundary(text, snapshot.anchor);
        self.clear_composition();
    }
}

/// Clamps stale byte offsets to the previous actual UTF-8 boundary.
fn boundary(text: &str, offset: usize) -> usize {
    let mut position = offset.min(text.len());
    while !text.is_char_boundary(position) {
        position = position.saturating_sub(1_usize);
    }
    position
}
/// Finds the previous complete scalar without subtracting arbitrary byte counts.
fn previous(text: &str, position: usize) -> usize {
    text.char_indices()
        .take_while(|entry| entry.0 < position)
        .last()
        .map_or(0_usize, |entry| entry.0)
}
/// Finds the next complete scalar, including the valid terminal boundary.
fn next(text: &str, position: usize) -> usize {
    text.char_indices()
        .find(|entry| entry.0 > position)
        .map_or(text.len(), |entry| entry.0)
}
/// Unicode letters/numbers and underscores form a word, not just ASCII bytes.
fn word(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}
/// Matches a single-line Ctrl+Left: skip trailing whitespace, then one word/group.
fn word_left(text: &str, from: usize) -> usize {
    let mut position = from;
    while text
        .get(..position)
        .and_then(|prefix| prefix.chars().next_back())
        .is_some_and(char::is_whitespace)
    {
        position = previous(text, position);
    }
    let Some(group) = text
        .get(..position)
        .and_then(|prefix| prefix.chars().next_back())
        .map(word)
    else {
        return position;
    };
    while text
        .get(..position)
        .and_then(|prefix| prefix.chars().next_back())
        .is_some_and(|character| !character.is_whitespace() && word(character) == group)
    {
        position = previous(text, position);
    }
    position
}
/// Matches Ctrl+Right: pass the current word/group, then intervening whitespace.
fn word_right(text: &str, from: usize) -> usize {
    let mut position = from;
    if let Some(group) = text
        .get(position..)
        .and_then(|suffix| suffix.chars().next())
        .filter(|character| !character.is_whitespace())
        .map(word)
    {
        while text
            .get(position..)
            .and_then(|suffix| suffix.chars().next())
            .is_some_and(|character| !character.is_whitespace() && word(character) == group)
        {
            position = next(text, position);
        }
    }
    while text
        .get(position..)
        .and_then(|suffix| suffix.chars().next())
        .is_some_and(char::is_whitespace)
    {
        position = next(text, position);
    }
    position
}
