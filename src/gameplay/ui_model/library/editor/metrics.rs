//! Bounded prefix measurements shared by pointer hit testing and native painting.

/// Measurements for only the current query, never an unbounded keystroke cache.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EditMetrics {
    /// Exact measured query.
    pub value: String,
    /// UTF-8 caret boundaries and actual native prefix widths.
    pub prefixes: Vec<(usize, u32)>,
}

impl EditMetrics {
    /// Measures every valid caret prefix once after a query/font change.
    pub fn measure(&mut self, text: &str, measure: impl Fn(&str) -> u32) {
        if self.value == text && !self.prefixes.is_empty() {
            return;
        }
        text.clone_into(&mut self.value);
        self.prefixes.clear();
        self.prefixes.push((0_usize, 0_u32));
        for offset in text
            .char_indices()
            .map(|entry| entry.0)
            .skip(1_usize)
            .chain((!text.is_empty()).then_some(text.len()))
        {
            self.prefixes
                .push((offset, measure(text.get(..offset).unwrap_or(""))));
        }
    }

    /// Actual prefix widths when warmed; deterministic bitmap widths in pure tests.
    #[must_use]
    pub fn width(&self, text: &str, offset: usize) -> u32 {
        if self.value == text {
            if let Some(entry) = self.prefixes.iter().find(|entry| entry.0 == offset) {
                return entry.1;
            }
        }
        u32::try_from(text.get(..offset).unwrap_or("").chars().count())
            .unwrap_or(u32::MAX)
            .saturating_mul(6_u32)
    }

    /// Keeps the caret visible using exactly the renderer's single-line scrolling.
    #[must_use]
    pub fn scroll(&self, text: &str, caret: usize, width: u32) -> u32 {
        self.width(text, caret)
            .saturating_sub(width.saturating_sub(6_u32))
    }

    /// Chooses the nearest real UTF-8 boundary, including both ends of the query.
    #[must_use]
    pub fn hit(&self, text: &str, caret: usize, width: u32, x: i32) -> usize {
        let position = u32::try_from(x.saturating_sub(3_i32))
            .unwrap_or(0_u32)
            .saturating_add(self.scroll(text, caret, width));
        core::iter::once(0_usize)
            .chain(text.char_indices().map(|entry| entry.0).skip(1_usize))
            .chain(core::iter::once(text.len()))
            .min_by_key(|offset| self.width(text, *offset).abs_diff(position))
            .unwrap_or(0_usize)
    }
}
