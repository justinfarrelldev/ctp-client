//! Source-layout managers with shared semantic, pointer and rendering descriptors.
//!
//! Selection, tabs and scroll positions are client presentation state. A manager
//! emits a revision-bound command; it never changes the authoritative snapshot.
//! Unsupported original operations retain a visible, explicit disabled reason.

use alloc::collections::BTreeMap;

use serde_json::Value;

use super::library::{LibraryBrowser, LibraryCategory};
use super::{InGameAction, MenuRect};
use crate::gameplay::GameplayError;
use crate::gameplay::assets::layout::{LayoutBook, LayoutNode};
use crate::gameplay::assets::library::LibraryCatalog;
use crate::gameplay::model::GameState;
use crate::gameplay::server::interface::LegalAction;

/// Source articles, measured fragments and viewer-safe variable mappings.
mod articles;
/// Source-derived button and editable-control bindings.
mod bindings;
/// Viewer-safe dynamic list rows and accounting labels.
mod data;
/// Original control-tree geometry, active panels and dynamic list insertion.
mod tree;

/// Original manager/dialog identity, independent of the lower control-panel tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ManagerKind {
    /// City production queue editor.
    Build,
    /// Domestic policy and government.
    Empire,
    /// National city summary.
    National,
    /// Individual city management.
    City,
    /// Unit statistics and tactical information.
    Unit,
    /// Army composition.
    Army,
    /// Trade market and existing routes.
    Trade,
    /// Contacts and diplomatic actions.
    Diplomacy,
    /// Current research and available advances.
    Science,
    /// Explicit available-research selection/confirmation dialog.
    Research,
    /// Installed Great Library articles.
    Library,
    /// Score, wonders and ranking.
    Statistics,
    /// Live local graphics options.
    Graphics,
    /// Live local sound options.
    Sound,
}

impl ManagerKind {
    /// All implemented source-layout roots, including nested research selection.
    pub const ALL: [Self; 14] = [
        Self::Build,
        Self::Empire,
        Self::National,
        Self::City,
        Self::Unit,
        Self::Army,
        Self::Trade,
        Self::Diplomacy,
        Self::Science,
        Self::Research,
        Self::Library,
        Self::Statistics,
        Self::Graphics,
        Self::Sound,
    ];

    /// Diagnostic/automation name, independent of localized dialog titles.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Empire => "empire",
            Self::National => "national",
            Self::City => "city",
            Self::Unit => "unit",
            Self::Army => "army",
            Self::Trade => "trade",
            Self::Diplomacy => "diplomacy",
            Self::Science => "science",
            Self::Research => "research",
            Self::Library => "library",
            Self::Statistics => "statistics",
            Self::Graphics => "graphics",
            Self::Sound => "sound",
        }
    }

    /// Parses an explicit diagnostic name without falling back to another manager.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.name().eq_ignore_ascii_case(name))
    }

    /// Original LDL instance whose inherited children form this dialog.
    #[must_use]
    pub const fn root(self) -> &'static str {
        match self {
            Self::Build => "BuildEditorWindow",
            Self::Empire => "DomesticDialog",
            Self::National => "CityStatusWin",
            Self::City => "CityWindow",
            Self::Unit => "UnitManager",
            Self::Army => "ArmyManager",
            Self::Trade => "TradeManager",
            Self::Diplomacy => "DiplomacyWindow",
            Self::Science => "ScienceManagementDialog",
            Self::Research => "SciAdvanceScreen",
            Self::Library => "GreatLibrary",
            Self::Statistics => "InfoDialog",
            Self::Graphics => "GraphicsWindow",
            Self::Sound => "SoundWindow",
        }
    }
}

/// Resolves distinct original manager entry points without collapsing them to tabs.
#[must_use]
pub const fn kind_for_action(action: InGameAction) -> Option<ManagerKind> {
    match action {
        InGameAction::OpenCityBuild => Some(ManagerKind::Build),
        InGameAction::OpenCityManager => Some(ManagerKind::City),
        InGameAction::OpenNationalManager => Some(ManagerKind::National),
        InGameAction::OpenTradeManager => Some(ManagerKind::Trade),
        InGameAction::OpenCivilizationStatus => Some(ManagerKind::Empire),
        InGameAction::OpenUnitManager => Some(ManagerKind::Unit),
        InGameAction::OpenArmyManager => Some(ManagerKind::Army),
        InGameAction::OpenDiplomacy => Some(ManagerKind::Diplomacy),
        InGameAction::OpenScienceStatus => Some(ManagerKind::Science),
        InGameAction::OpenGreatLibrary => Some(ManagerKind::Library),
        InGameAction::OpenInfo | InGameAction::OpenScore | InGameAction::OpenWonders => {
            Some(ManagerKind::Statistics)
        }
        InGameAction::OpenGraphicsOptions => Some(ManagerKind::Graphics),
        InGameAction::OpenSoundOptions => Some(ManagerKind::Sound),
        _ => None,
    }
}

/// Exact command metadata displayed and confirmed by a manager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagerCommand {
    /// Viewer revision at which this intent was formed.
    pub revision: i64,
    /// Registered HTTP command name, never an engine-only advertised route.
    pub route: String,
    /// Exact selected entity/policy payload.
    pub payload: Value,
}

impl ManagerCommand {
    /// Rejects stale confirmations and commands with no current authority.
    #[must_use]
    pub fn valid_for(&self, state: &GameState) -> bool {
        let Some(viewer) = state
            .viewer
            .as_ref()
            .filter(|viewer| viewer.revision == self.revision)
        else {
            return false;
        };
        if self.route == "set-research" {
            if self
                .payload
                .as_object()
                .is_none_or(|fields| fields.len() != 1_usize)
            {
                return false;
            }
            return self.payload["advance_id"].as_str().is_some_and(|target| {
                viewer
                    .interface
                    .research
                    .as_ref()
                    .is_some_and(|research| research.available.iter().any(|id| id == target))
            });
        }
        viewer.interface.legal_actions.iter().any(|action| {
            action.http_supported() && action.route == self.route && action.payload == self.payload
        })
    }

    /// Carries through an advertised action without reconstructing its metadata.
    pub(super) fn advertised(state: &GameState, action: &LegalAction) -> Option<Self> {
        action.http_supported().then(|| Self {
            revision: state.viewer.as_ref().map_or(0, |viewer| viewer.revision),
            route: action.route.clone(),
            payload: action.payload.clone(),
        })
    }
}

/// Semantic interaction available to real SDL input and accessible automation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagerIntent {
    /// Close only the top manager, not the application or current turn.
    Close,
    /// Open a source-verified related manager.
    Open(ManagerKind),
    /// Set a presentation-only category, active tab or selected row.
    Select(String, String),
    /// Scroll one list without moving the world camera.
    Scroll(String, i32),
    /// Focus an original local text field.
    Focus(String),
    /// Switch the library index category without changing game knowledge.
    LibraryCategory(LibraryCategory),
    /// Follow an installed internal article, opening the library when necessary.
    Article(String),
    /// Restore the previous library article.
    LibraryBack,
    /// Restore an article after Back.
    LibraryForward,
    /// Submit one exact command through the authoritative runtime.
    Submit(ManagerCommand),
}

impl ManagerIntent {
    /// Extracts a command for mock transport/receipt tests without a server call.
    #[must_use]
    pub fn into_command(self) -> Option<ManagerCommand> {
        if let Self::Submit(command) = self {
            Some(command)
        } else {
            None
        }
    }
}

/// One source-aligned control, including noninteractive art/text and dynamic rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagerWidget {
    /// Stable original LDL path, with `/canonical-id` appended for a dynamic row.
    pub id: String,
    /// Original localized label or authoritative dynamic value.
    pub label: String,
    /// Shared logical rectangle for drawing and pointer activation.
    pub rect: MenuRect,
    /// Resolved source properties (children are flattened into other widgets).
    pub node: LayoutNode,
    /// Enabled semantic interaction; absent for text/art and disabled controls.
    pub intent: Option<ManagerIntent>,
    /// Precise disabled reason, never a pretend successful operation.
    pub unavailable: Option<String>,
    /// Whether this is an active tab/selected row/toggle.
    pub selected: bool,
}

/// Independent selections survive repaint and authoritative receipts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagerView {
    /// Visible manager identity.
    pub kind: ManagerKind,
    /// Exact canonical city used by production/city screens.
    pub city_id: Option<String>,
    /// Independent named selections, not a single shared row index.
    pub selections: BTreeMap<String, String>,
    /// List-local scroll offsets.
    pub scroll: BTreeMap<String, usize>,
}

/// Source layouts and local management-window state; no game simulation/storage.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ManagementUi {
    /// Installed localized source data, absent only in explicit asset-free tests.
    pub book: LayoutBook,
    /// Resolved manager roots cached once, not reparsed every SDL frame.
    pub layouts: BTreeMap<ManagerKind, LayoutNode>,
    /// Open manager, independent of map/control-panel selections.
    pub view: Option<ManagerView>,
    /// Explicit keyboard focus; Enter never activates an arbitrary first action.
    pub focused: Option<String>,
    /// Installed article browser, separate from authoritative game research.
    pub library: LibraryBrowser,
    /// Native font word/glyph widths used by both painting and link rectangles.
    pub text_widths: BTreeMap<String, u32>,
    /// Native font line spacing, or a deterministic fallback in asset-free tests.
    pub line_height: u32,
    /// Related windows to restore on Close; duplicate kinds are not stacked.
    pub parents: Vec<ManagerView>,
}

impl ManagementUi {
    /// Resolves all source manager trees before accepting real interactions.
    ///
    /// # Errors
    /// Returns source/template errors; never silently replaces a real dialog.
    pub fn new(book: LayoutBook) -> Result<Self, GameplayError> {
        let layouts = ManagerKind::ALL
            .into_iter()
            .map(|kind| book.resolve(kind.root()).map(|node| (kind, node)))
            .collect::<Result<_, _>>()?;
        let catalog = book
            .installation_root()
            .map(LibraryCatalog::load)
            .transpose()?
            .unwrap_or_default();
        Ok(Self {
            book,
            layouts,
            view: None,
            focused: None,
            library: LibraryBrowser::new(catalog),
            text_widths: BTreeMap::new(),
            line_height: 16_u32,
            parents: Vec::new(),
        })
    }

    /// Opens a view without changing units/cities/queues/research in the model.
    pub fn open(&mut self, kind: ManagerKind, state: &GameState, city_id: Option<&str>) {
        self.focused = (kind == ManagerKind::Library).then(|| "SearchWord".to_owned());
        self.parents.clear();
        if kind == ManagerKind::Library && self.library.current.is_none() {
            if let Some(id) = state
                .viewer
                .as_ref()
                .and_then(|viewer| viewer.interface.research.as_ref())
                .map(|research| research.target_id.as_str())
            {
                self.library.navigate(id);
            }
        }
        let mut selections = BTreeMap::new();
        selections.insert("build-category".to_owned(), "Units".to_owned());
        let selected_city = city_id
            .map(str::to_owned)
            .or_else(|| data::owned_city_ids(state).into_iter().next());
        self.view = Some(ManagerView {
            kind,
            city_id: selected_city,
            selections,
            scroll: BTreeMap::new(),
        });
    }

    /// Opens a related source window while retaining its parent's selections.
    /// Returning to an existing kind restores it rather than growing a loop of windows.
    pub fn open_related(&mut self, kind: ManagerKind, state: &GameState) {
        if self.view.as_ref().is_some_and(|view| view.kind == kind) {
            return;
        }
        if let Some(index) = self.parents.iter().position(|view| view.kind == kind) {
            self.view = self.parents.get(index).cloned();
            self.parents.truncate(index);
            self.focused = None;
            return;
        }
        let previous = self.view.take();
        let parents = core::mem::take(&mut self.parents);
        let city = previous.as_ref().and_then(|view| view.city_id.clone());
        self.open(kind, state, city.as_deref());
        self.parents = parents;
        if let Some(previous) = previous {
            self.parents.push(previous);
        }
    }

    /// Collects installed article metrics once from the same native font used to draw.
    pub fn measure_text(&mut self, measure: impl Fn(&str) -> u32, line_height: u32) {
        articles::measure(self, measure, line_height);
    }

    /// Executes the original all-category text search once, caching its matches.
    pub fn search_library(&mut self) {
        let labels: BTreeMap<_, _> = self
            .library
            .catalog
            .articles
            .keys()
            .map(|id| (id.clone(), data::label(self, id)))
            .collect();
        self.library
            .execute_search(|id| labels.get(id).cloned().unwrap_or_else(|| id.to_owned()));
        if let Some(view) = self.view.as_mut() {
            view.scroll.remove("IndexSheet");
        }
    }

    /// Scrolls only a current list/text box, clamping the stored offset as well as
    /// its rendering so a single reverse event immediately leaves the last page.
    pub fn scroll(&mut self, path: &str, delta: i32, state: &GameState) -> bool {
        let Some(view) = self.view.as_ref() else {
            return false;
        };
        let Some(widget) = self.widgets(state).into_iter().find(|widget| {
            widget.id == path
                && widget.node.field("objecttype").is_some_and(|kind| {
                    kind.eq_ignore_ascii_case("ctp2_ListBox")
                        || kind.eq_ignore_ascii_case("ctp2_hypertextbox")
                })
        }) else {
            return false;
        };
        let maximum =
            articles::scroll_limit(self, state, view, path, widget.rect).unwrap_or_else(|| {
                data::rows(self, state, view, path)
                    .len()
                    .saturating_sub(tree::visible_rows(view.kind, path, widget.rect))
            });
        let Some(current) = self.view.as_mut() else {
            return false;
        };
        let offset = current.scroll.entry(path.to_owned()).or_default();
        *offset = (*offset).min(maximum);
        if delta < 0_i32 {
            *offset =
                offset.saturating_sub(usize::try_from(delta.unsigned_abs()).unwrap_or(usize::MAX));
        } else {
            *offset = offset
                .saturating_add(usize::try_from(delta).unwrap_or(usize::MAX))
                .min(maximum);
        }
        true
    }

    /// Flattens the active source tree and inserts only viewer-safe dynamic rows.
    #[must_use]
    pub fn widgets(&self, state: &GameState) -> Vec<ManagerWidget> {
        tree::widgets(self, state)
    }

    /// Exact accessible source ID activation, including duplicate-label controls.
    #[must_use]
    pub fn activate_id(&self, id: &str, state: &GameState) -> Option<ManagerIntent> {
        self.widgets(state)
            .into_iter()
            .find(|widget| widget.id == id)?
            .intent
    }

    /// Accessible label activation requires a unique enabled match.
    #[must_use]
    pub fn activate(&self, label: &str, state: &GameState) -> Option<ManagerIntent> {
        let mut matches = self
            .widgets(state)
            .into_iter()
            .filter(|widget| widget.label.eq_ignore_ascii_case(label) && widget.intent.is_some());
        let intent = matches.next()?.intent;
        matches.next().is_none().then_some(intent).flatten()
    }

    /// Pointer activation uses exactly the same descriptors as semantic tests.
    #[must_use]
    pub fn intent_at(&self, x: i32, y: i32, state: &GameState) -> Option<ManagerIntent> {
        self.widgets(state)
            .into_iter()
            .rev()
            .find(|widget| widget.rect.contains(x, y) && widget.intent.is_some())?
            .intent
    }

    /// Applies presentation-only interactions; submissions/opening require the
    /// runtime and return false rather than claiming a game order was executed.
    pub fn apply_local(&mut self, intent: ManagerIntent) -> bool {
        if intent == ManagerIntent::Close {
            self.view = self.parents.pop();
            self.focused = None;
            return true;
        }
        let Some(view) = self.view.as_mut() else {
            return false;
        };
        match intent {
            ManagerIntent::Select(key, value) => {
                if key == "city" {
                    view.city_id = Some(value.clone());
                }
                if matches!(key.as_str(), "buildable" | "queue") {
                    view.selections
                        .insert("description".to_owned(), value.clone());
                }
                view.selections.insert(key, value);
                true
            }
            ManagerIntent::Focus(id) => {
                self.focused = Some(id);
                true
            }
            ManagerIntent::LibraryCategory(category) => {
                self.library.set_category(category);
                self.focused = None;
                view.scroll.remove("IndexSheet");
                true
            }
            ManagerIntent::Article(id) if view.kind == ManagerKind::Library => {
                let changed = self.library.navigate(&id);
                if changed {
                    self.focused = None;
                    view.selections.insert("article".to_owned(), id);
                    view.scroll.clear();
                }
                changed
            }
            ManagerIntent::LibraryBack => {
                let changed = self.library.go_back();
                if changed {
                    view.scroll.clear();
                }
                changed
            }
            ManagerIntent::LibraryForward => {
                let changed = self.library.go_forward();
                if changed {
                    view.scroll.clear();
                }
                changed
            }
            ManagerIntent::Close
            | ManagerIntent::Open(_)
            | ManagerIntent::Submit(_)
            | ManagerIntent::Article(_)
            | ManagerIntent::Scroll(_, _) => false,
        }
    }
}
