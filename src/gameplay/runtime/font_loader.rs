//! Session font discovery shared by live gameplay and fileless native previews.
use std::path::{Path,PathBuf};
use sdl2::ttf::{Font,Hinting,Sdl2TtfContext};

/// Original Times-compatible fonts, with explicit platform fallbacks.
const CANDIDATES:[&str;8]=[
    "C:/Windows/Fonts/timesbd.ttf",
    "/usr/share/fonts/truetype/msttcorefonts/timesbd.ttf",
    "/usr/share/fonts/truetype/liberation2/LiberationSerif-Bold.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSerif-Bold.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
    "/usr/share/fonts/truetype/liberation2/LiberationSans-Bold.ttf",
    "/usr/share/fonts/truetype/freefont/FreeSansBold.ttf",
    "C:/Windows/Fonts/arialbd.ttf",
];
/// Loads identical original-size font/hinting for live gameplay and previews.
pub(super) fn load_gameplay_font(context:&Sdl2TtfContext)->Option<Font<'_, 'static>> {
    let mut font=context.load_font(find_existing_font()?,13_u16).ok()?;
    font.set_hinting(Hinting::Mono);
    Some(font)
}
/// Finds an installed font without creating or changing any resource.
pub(super) fn find_existing_font()->Option<PathBuf> {
    CANDIDATES.iter().map(Path::new).find(|path| path.try_exists().unwrap_or(false)).map(Path::to_path_buf)
}
