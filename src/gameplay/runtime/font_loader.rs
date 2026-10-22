//! Session font loading shared by live gameplay and fileless native previews.
use super::fonts::discovery;
use crate::gameplay::ui_model::management::typography::SourceFont;
use sdl2::ttf::{Font, Hinting, Sdl2TtfContext};
use std::path::Path;
/// The original selected-installation face precedes explicit system fallbacks.
/// Preserve existing body raster size/hinting; discovery never copies font files.
pub(super) fn load_gameplay_font<'ttf>(
    context: &'ttf Sdl2TtfContext,
    root: Option<&Path>,
) -> Option<Font<'ttf, 'static>> {
    let (path, exact) = discovery::find(root, &SourceFont::default())?;
    if !exact {
        log::warn!(
            "Original gameplay Times face unavailable; using {}. Exact font parity cannot be certified.",
            path.display()
        );
    }
    let mut font = context.load_font(path, 13_u16).ok()?;
    font.set_hinting(Hinting::Mono);
    Some(font)
}
