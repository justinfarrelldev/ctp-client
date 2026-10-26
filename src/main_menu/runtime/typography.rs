//! Original shell face inheritance, loaded once without changing current scaling.
use crate::gameplay::GameplayError;
use crate::gameplay::assets::layout::{LayoutBook, LayoutNode};
use crate::gameplay::ui_model::management::typography::SourceFont;
use alloc::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::Path;

/// Each original shell root has its own namespace for local control identifiers.
const ROOTS: [&str; 19] = [
    "InitPlayWindow",
    "SPWindow",
    "SPNewGameWindow",
    "OptionsWindow",
    "GamePlayOptionsWindow",
    "GraphicsWindow",
    "ScreenResScreen",
    "SoundWindow",
    "MusicScreen",
    "MusicTrackScreen",
    "KmScreen",
    "SPNewGameTribeScreen",
    "SPNewGameDiffScreen",
    "SPNewGameMapSizeScreen",
    "SPNewGameMapShapeScreen",
    "SPNewGamePlayersScreen",
    "SPNewGameRulesScreen",
    "CustomMapWindow",
    "SPNewGameRiskScreen",
];

/// Exact source face/style for each unique screen-local modeled control name.
#[derive(Default)]
pub(super) struct ShellTypography {
    /// Ambiguous nested local names remain absent rather than borrowing a neighbor.
    faces: BTreeMap<String, BTreeMap<String, Option<SourceFont>>>,
}

impl ShellTypography {
    /* Original source: ctp2_source/ctp2_code/ui/interface/spnewgamescreen.cpp::
     * spnewgamescreen_Initialize and optionsscreen.cpp::optionsscreen_Initialize
     * construct SPNewGameWindow/C3Window and the source-named child controls.
     * ui/aui_common/aui_textbase.cpp::InitCommonLdl/TextReloadFont uses inherited
     * fontname/boldfont/italicfont. English layouts ctp_template.ldl::
     * CTP2_STANDARD_FONT is timesbd.ttf, CTP_GREY_TITLE_FONT is times.ttf;
     * spnewgame.ldl::SPNewGameWindow::{CivilizationLabel,LeaderNameLabel,...}
     * uses GreyTitle, while {TribeButton,Name,...} retains Standard bold.
     * sound.ldl::SoundWindow::Name uses GreyTitle but SFXName uses Standard.
     * Do not infer font weight from all Label/Name controls indiscriminately.
     * This cache changes faces only: current physical point sizing, per-label
     * percentages, coordinates, art and navigation are intentionally retained.
     * Native edit exception: ui/interface/spnewgamewindow.cpp::SPNewGameWindow
     * creates C3TextField; ui/aui_ctp2/c3textfield.cpp::C3TextField delegates to
     * ui/aui_common/aui_textfield.cpp::InitCommonLdl/InitCommon, which mistakenly
     * sends the LDL TTF filename to Win32 EnumFonts (expects a family), and reads
     * fontname instead of fontheight. Honor the declared font here, not the old
     * accidental stock-GDI fallback; native edit raster equality is not claimed.
     */
    /// Reads the bounded active shell layouts once; no font or art assets are copied.
    pub(super) fn load(root: &Path) -> Self {
        let directory = root.join("ctp2_data/english/uidata/layouts");
        let mut sources = Vec::new();
        for name in [
            "ctp_template.ldl",
            "fancy.ldl",
            "spnewgame.ldl",
            "spnewgamepopups.ldl",
            "options.ldl",
            "gameplayoptions.ldl",
            "graphics.ldl",
            "sound.ldl",
            "musicscreen.ldl",
            "km_screen.ldl",
            "custommap.ldl",
        ] {
            let path = directory.join(name);
            let read = std::fs::File::open(&path).and_then(|file| {
                let mut bytes = Vec::new();
                file.take(512_u64 * 1024_u64 + 1_u64).read_to_end(&mut bytes)?;
                Ok(bytes)
            });
            match read {
                Ok(bytes) if bytes.len() <= 512_usize * 1024_usize => {
                    sources.push(crate::gameplay::assets::text::decode(&bytes));
                }
                Ok(_) => log::warn!("Shell font layout too large: {}", path.display()),
                Err(error) => {
                    log::warn!("Shell font layout unavailable: {}: {error}", path.display());
                }
            }
        }
        let borrowed: Vec<&str> = sources.iter().map(String::as_str).collect();
        Self::parse(&borrowed).unwrap_or_else(|error| {
            log::warn!("Original shell typography unavailable; using standard Times face: {error}");
            Self::default()
        })
    }

    /// Resolves inherited source faces in memory, also used by fileless regressions.
    pub(super) fn parse(sources: &[&str]) -> Result<Self, GameplayError> {
        let book = LayoutBook::parse(sources)?;
        let mut typography = Self::default();
        for root in ROOTS {
            if let Ok(node) = book.resolve(root) {
                for child in &node.children {
                    typography.collect(root, child);
                }
            }
        }
        Ok(typography)
    }

    /// Unique screen-local font; missing/ambiguous controls retain explicit fallback.
    pub(super) fn face(&self, screen: &str, control: &str) -> Option<&SourceFont> {
        self.faces
            .get(screen)
            .and_then(|controls| controls.get(control))
            .and_then(Option::as_ref)
    }

    /// Source faces to load once at the current physical menu size.
    pub(super) fn keys(&self) -> BTreeSet<SourceFont> {
        self.faces
            .values()
            .flat_map(BTreeMap::values)
            .filter_map(Option::as_ref)
            .cloned()
            .chain(core::iter::once(SourceFont::default()))
            .collect()
    }

    /// Descend only resolved trees; duplicate leaf names cannot choose a sibling.
    fn collect(&mut self, root: &str, node: &LayoutNode) {
        let mut face = SourceFont::from_node(node);
        // Face-only restoration: leave existing shell point-size/scaling policy intact.
        face.points = SourceFont::default().points;
        self.faces
            .entry(root.to_owned())
            .or_default()
            .entry(node.name.clone())
            .and_modify(|previous| *previous = None)
            .or_insert(Some(face));
        for child in &node.children {
            self.collect(root, child);
        }
    }
}
