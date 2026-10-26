//! Shared read-only font discovery for source-backed controls and gameplay text.
use crate::gameplay::ui_model::management::typography::SourceFont;
use std::path::{Path, PathBuf};

/// Retains original installation-first precedence without shipping font bytes.
pub(in crate::gameplay::runtime) fn find(
    root: Option<&Path>,
    key: &SourceFont,
) -> Option<(PathBuf, bool)> {
    crate::fonts::find(root, &key.name, key.bold, key.italic)
}

/// Deterministic adapter used by existing fileless original-font regressions.
#[cfg(test)]
pub(super) fn find_with(
    root: Option<&Path>,
    key: &SourceFont,
    present: impl FnMut(&Path) -> bool,
) -> Option<(PathBuf, bool)> {
    crate::fonts::find_with(root, &key.name, key.bold, key.italic, &[], present)
}
