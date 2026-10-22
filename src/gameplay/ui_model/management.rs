//! Source-layout managers with shared semantic, pointer and rendering descriptors.
//!
//! Selection, tabs and scroll positions are client presentation state. A manager
//! emits a revision-bound command; it never changes the authoritative snapshot.
//! Unsupported original operations retain a visible, explicit disabled reason.

use alloc::collections::BTreeMap;

use serde_json::Value;

use super::library::editor::metrics::EditMetrics;
use super::library::editor::{ClipboardRequest, Motion, TextEditor};
use super::library::{LibraryBrowser, LibraryCategory};
use super::{InGameAction, MenuRect};
use crate::gameplay::GameplayError;
use crate::gameplay::assets::layout::{LayoutBook, LayoutNode};
use crate::gameplay::assets::library::LibraryCatalog;
use crate::gameplay::model::GameState;

/// Source-attached advisor geometry/lifecycle and own-only accounting regressions.
#[cfg(test)]
mod advisor_tests;
/// Original docked advice windows with own-only facts and local callbacks.
mod advisors;
/// Source articles, measured fragments and viewer-safe variable mappings.
mod articles;
/// Source-derived button and editable-control bindings.
mod bindings;
/// Runtime-created original advance chart and viewer-safe local navigation.
mod chart;
/// Fileless source-chart regressions.
#[cfg(test)]
mod chart_tests;
/// Immutable command authorization, separate from manager presentation.
mod command;
/// Viewer-safe dynamic list rows and accounting labels.
mod data;
/// Source Empire Details/Summary lifecycle and government comparison regressions.
#[cfg(test)]
mod empire_tests;
/// Source-version presentation adaptation, independent of game authority.
mod fidelity;
/// Reference presentation and source-version layout deltas.
#[cfg(test)]
mod fidelity_tests;
/// Source government comparisons and exact currently eligible enactment identities.
mod government;
/// Source category-index sorting and noninteractive age headers.
mod index;
/// Age sorting and reference-control regressions.
#[cfg(test)]
mod index_tests;
/// Fileless list lifecycle and content/ranger geometry regressions.
#[cfg(test)]
mod list_tests;
/// Original list frame/pane separation and overflow-safe runtime sizing.
mod lists;
/// Atomic original option-slider presentation.
mod option_art;
/// Source-derived lower-panel bodies, with viewer-safe public captions.
mod panel;
/// Empty/populated lower-panel presentation regressions.
#[cfg(test)]
mod panel_tests;
/// Verified typed policy/rate parameters and source ranger controls.
mod policy;
/// Parameter schemas, revision guards and source slider-direction regressions.
#[cfg(test)]
mod policy_tests;
/// Stable-entry atomic queue requests and content resolution.
mod queue;
#[cfg(test)]
mod queue_tests;
/// Identity-preserving selection handoff after complete receipts/resyncs.
mod reconcile;
#[cfg(test)]
mod reconcile_tests;
/// Source row shapes, numeric values and research-list visibility regressions.
#[cfg(test)]
mod row_tests;
/// Native item-template columns populated only from current public projections.
mod rows;
/// Source list/hypertext rangers with shared range/geometry accounting.
mod scrollbar;
/// Empty/populated source ranger regressions.
#[cfg(test)]
mod scrollbar_tests;
/// View-only independent row/city/chart selection, separate from source navigation.
mod selection;
/// City selector/caption/popup and stale identity regressions.
#[cfg(test)]
mod selector_tests;
/// Source dropdown lifecycle and exact owned-city selection.
mod selectors;
/// Original control-tree geometry, active panels and dynamic list insertion.
mod tree;
/// Original per-control typography and fileless measured text metadata.
pub mod typography;
/// Native-font choice and style-metric regressions.
#[cfg(test)]
mod typography_tests;
/// Source unit statistics/tactical columns, readiness and exact disband regressions.
#[cfg(test)]
mod unit_tests;
/// Original unit statistics/tactical content and private-safe source callbacks.
mod units;
/// Source-expression/authority boundary regressions.
#[cfg(test)]
mod variable_tests;
/// Explicit non-executable mappings from original expressions to public rule data.
mod variables;

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

/// Semantic interaction available to real SDL input and accessible automation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagerIntent {
    /// Explicit revision-bound input/target dialog; opening never submits.
    Dialog(Box<super::operations::OperationDialog>),
    /// Close only the top manager, not the application or current turn.
    Close,
    /// Open a source-verified related manager.
    Open(ManagerKind),
    /// Open a related manager for the exact panel city, not the first owned city.
    ForCity(ManagerKind, String),
    /// Set a presentation-only category, active tab or selected row.
    Select(String, String),
    /// Scroll one list without moving the world camera.
    Scroll(String, i32),
    /// Set a source slider/list value, clamped against current content.
    ScrollTo(String, usize),
    /// Begin dragging the actual source thumb; keyboard value changes use `ScrollTo`.
    BeginScroll(String),
    /// Explicit source control path/value and the revision that displayed them.
    SetValue(String, i32, i64),
    /// Source thumb path/display revision; capture never mutates authoritative facts.
    BeginValue(String, i64),
    /// Focus an original local text field.
    Focus(String),
    /// Place/extend the local query caret at a validated UTF-8 boundary.
    EditCaret(usize, bool),
    /// Switch the library index category without changing game knowledge.
    LibraryCategory(LibraryCategory),
    /// Toggle the view-only source age-grouped category index.
    LibrarySort,
    /// Follow an installed internal article, opening the library when necessary.
    Article(String),
    /// Open the chart center's actual article instead of the original empty callback.
    ChartArticle(String),
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

/// Presentation-only source thumb capture; accepted receipts own real policy values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagerValueDrag {
    /// Exact original control path.
    pub path: String,
    /// Revision where the pointer capture began.
    pub revision: i64,
    /// Preview source index, never a mutated canonical level.
    pub value: i32,
    /// Pointer's offset inside the actual thumb, preserving grab position.
    pub grab_offset: i32,
}

/// Source layouts and local management-window state; no game simulation/storage.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ManagementUi {
    /// Installed localized source data, absent only in explicit asset-free tests.
    pub book: LayoutBook,
    /// Resolved manager roots cached once, not reparsed every SDL frame.
    pub layouts: BTreeMap<ManagerKind, LayoutNode>,
    /// Cached source docked advice windows, sharing the main manager lifecycle.
    pub advisor_layouts: BTreeMap<ManagerKind, LayoutNode>,
    /// Original lower-panel body tree, resolved once without frame-time file I/O.
    pub panel: LayoutNode,
    /// Original runtime-created Chart viewport/button template, resolved once.
    pub chart: LayoutNode,
    /// Original dynamic item templates, resolved once rather than doing frame-time I/O.
    pub row_templates: BTreeMap<&'static str, LayoutNode>,
    /// Open manager, independent of map/control-panel selections.
    pub view: Option<ManagerView>,
    /// Explicit keyboard focus; Enter never activates an arbitrary first action.
    pub focused: Option<String>,
    /// Installed article browser, separate from authoritative game research.
    pub library: LibraryBrowser,
    /// Bounded query selection, composition and undo/redo, not game authority.
    pub editor: TextEditor,
    /// Explicit clipboard work deferred to an injectable native boundary.
    pub clipboard_request: Option<ClipboardRequest>,
    /// Native current-query prefix metrics used for pointer/caret parity.
    pub edit_metrics: EditMetrics,
    /// Whether a pointer selection began inside the actual query field.
    pub search_dragging: bool,
    /// Latest keyboard Shift state for extending a pointer selection.
    pub pointer_extend: bool,
    /// Captured source thumb and grab offset; no world movement capture is shared.
    pub scroll_drag: Option<(String, i32)>,
    /// Pending policy thumb preview, submitted only on explicit release.
    pub value_drag: Option<ManagerValueDrag>,
    /// Native font word/glyph widths used by both painting and link rectangles.
    pub text_widths: BTreeMap<String, u32>,
    /// Actual per-source-face metrics, retaining regular 8-point requirements versus body text.
    pub font_metrics: BTreeMap<typography::SourceFont, typography::FontMetrics>,
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
        let mut layouts: BTreeMap<_, _> = ManagerKind::ALL
            .into_iter()
            .map(|kind| book.resolve(kind.root()).map(|node| (kind, node)))
            .collect::<Result<_, _>>()?;
        for (kind, root) in &mut layouts {
            fidelity::configure(*kind, root, &book)?;
        }
        if let Some(root) = layouts.get_mut(&ManagerKind::Library) {
            index::configure(root, &book)?;
        }
        let catalog = book
            .installation_root()
            .map(LibraryCatalog::load)
            .transpose()?
            .unwrap_or_default();
        let chart = book.resolve("TechTree")?;
        let panel = book.resolve("ControlPanelWindow")?;
        let row_templates = rows::templates(&book)?;
        let advisor_layouts = advisors::templates(&book)?;
        Ok(Self {
            advisor_layouts,
            book,
            layouts,
            chart,
            row_templates,
            panel,
            view: None,
            focused: None,
            library: LibraryBrowser::new(catalog),
            editor: TextEditor::default(),
            clipboard_request: None,
            edit_metrics: EditMetrics::default(),
            search_dragging: false,
            pointer_extend: false,
            scroll_drag: None,
            value_drag: None,
            text_widths: BTreeMap::new(),
            font_metrics: BTreeMap::new(),
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
        self.clipboard_request = None;
        self.search_dragging = false;
        self.scroll_drag = None;
        self.value_drag = None;
        if kind == ManagerKind::Library {
            self.editor.reset(&self.library.query);
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

    /// Collects the resolved original styles without introducing SDL into the model.
    #[must_use]
    pub fn font_keys(&self) -> alloc::collections::BTreeSet<typography::SourceFont> {
        typography::keys(self)
    }
    /// Warms native geometry using exactly the source styles used for painting.
    pub fn measure_source_fonts(
        &mut self,
        measure: impl Fn(&typography::SourceFont, &str) -> u32,
        line_height: impl Fn(&typography::SourceFont) -> u32,
    ) {
        typography::measure(self, measure, line_height);
    }

    /// Warms only changed query prefixes using the actual native field font.
    pub fn measure_query(&mut self, measure: impl Fn(&str) -> u32) {
        self.edit_metrics.measure(&self.library.query, measure);
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
        let Some(maximum) = scrollbar::maximum(self, state, path) else {
            return false;
        };
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

    /// Accessible absolute range changes never bypass the current source bounds.
    pub fn scroll_to(&mut self, path: &str, value: usize, state: &GameState) -> bool {
        let Some(maximum) = scrollbar::maximum(self, state, path) else {
            return false;
        };
        let Some(view) = self.view.as_mut() else {
            return false;
        };
        view.scroll.insert(path.to_owned(), value.min(maximum));
        true
    }
    /// Captures the actual thumb under the pointer, or its midpoint for accessibility.
    pub fn begin_scroll(&mut self, path: &str, pointer_y: Option<i32>, state: &GameState) -> bool {
        scrollbar::begin(self, path, pointer_y, state)
    }
    /// Updates only the captured source range using its current content/geometry.
    pub fn drag_scroll(&mut self, y: i32, state: &GameState) -> bool {
        scrollbar::drag(self, y, state)
    }

    /// Typed parameter command for this manager; unknown control paths never submit.
    #[must_use]
    pub fn value_command(
        &self,
        path: &str,
        value: i32,
        state: &GameState,
    ) -> Option<ManagerCommand> {
        self.view.as_ref().filter(|view| {
            (view.kind == ManagerKind::Empire && path != "Slider")
                || (view.kind == ManagerKind::Unit && path == "Slider")
        })?;
        policy::command(state, path, value)
    }

    /// Revalidates city selection against the current receipt before changing any view.
    pub fn select_city(&mut self, id: &str, state: &GameState) -> bool {
        if !data::owned_city_ids(state).iter().any(|owned| owned == id) {
            return false;
        }
        self.apply_local(ManagerIntent::Select("city".to_owned(), id.to_owned()))
    }

    /// Revalidates comparison identity against this receipt before changing the view.
    pub fn select_government(&mut self, id: &str, state: &GameState) -> bool {
        government::select(self, state, id)
    }

    /// Current authorized popup choices; arbitrary keys cannot open a chooser.
    #[must_use]
    pub fn dropdown_choices(&self, state: &GameState) -> Vec<(String, String)> {
        self.view
            .as_ref()
            .filter(|view| {
                view.selections
                    .get("dropdown")
                    .is_some_and(|path| selectors::valid_path(view.kind, path))
            })
            .map_or_else(Vec::new, |view| {
                view.selections
                    .get("dropdown")
                    .map_or_else(Vec::new, |path| selectors::options(self, state, view, path))
            })
    }

    /// Flattens the active source tree and inserts only viewer-safe dynamic rows.
    #[must_use]
    pub fn widgets(&self, state: &GameState) -> Vec<ManagerWidget> {
        tree::widgets(self, state)
    }

    /// Lower-panel artwork/captions, separate from open management windows.
    #[must_use]
    pub fn panel_widgets(
        &self,
        state: &GameState,
        ui: &super::InGameUiModel,
    ) -> Vec<ManagerWidget> {
        panel::widgets(self, state, ui)
    }

    /// Exact accessible source ID activation, including duplicate-label controls.
    #[must_use]
    pub fn activate_id(&self, id: &str, state: &GameState) -> Option<ManagerIntent> {
        self.widgets(state)
            .into_iter()
            .find(|widget| widget.id == id)?
            .intent
    }

    /// Keeps independent selections only while their exact identities still exist.
    pub fn reconcile(&mut self, state: &GameState) {
        reconcile::apply(self, state);
    }

    /// Accessible label activation requires a unique enabled match.
    #[must_use]
    pub fn activate(&self, label: &str, state: &GameState) -> Option<ManagerIntent> {
        let mut matches = self.widgets(state).into_iter().filter(|widget| {
            (widget.label.eq_ignore_ascii_case(label)
                || widget
                    .node
                    .field("aria-label")
                    .is_some_and(|name| name.eq_ignore_ascii_case(label)))
                && widget.intent.is_some()
        });
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

    /// Restores a parent while cancelling native capture, composition and deferred I/O.
    fn close(&mut self) {
        self.search_dragging = false;
        self.scroll_drag = None;
        self.value_drag = None;
        self.clipboard_request = None;
        self.editor.clear_composition();
        self.view = self.parents.pop();
        self.focused = None;
    }

    /// Applies presentation-only interactions; submissions/opening require the
    /// runtime and return false rather than claiming a game order was executed.
    pub fn apply_local(&mut self, intent: ManagerIntent) -> bool {
        if intent == ManagerIntent::Close {
            self.close();
            return true;
        }
        let Some(view) = self.view.as_mut() else {
            return false;
        };
        match intent {
            ManagerIntent::Select(key, value) => selection::apply(view, key, value),
            ManagerIntent::EditCaret(offset, extend) => {
                self.editor.set_caret(&self.library.query, offset, extend);
                self.focused = Some("SearchWord".to_owned());
                true
            }
            ManagerIntent::Focus(id) => {
                if id == "SearchWord" {
                    self.editor
                        .move_caret(&self.library.query, Motion::End, false);
                }
                self.focused = Some(id);
                true
            }
            ManagerIntent::LibrarySort => {
                self.library.sort_by_age = !self.library.sort_by_age;
                self.library.index_open = true;
                view.scroll.remove("IndexSheet");
                true
            }
            ManagerIntent::LibraryCategory(category) => {
                self.library.set_category(category);
                self.editor.reset(&self.library.query);
                self.focused = None;
                view.scroll.remove("IndexSheet");
                true
            }
            ManagerIntent::ChartArticle(id) if view.kind == ManagerKind::Library => {
                if !self.library.navigate(&id) {
                    return false;
                }
                view.selections.insert("article".to_owned(), id);
                view.selections
                    .insert("tab:Tabs".to_owned(), "Gameplay".to_owned());
                chart::reset(view);
                view.scroll.clear();
                self.focused = None;
                true
            }
            ManagerIntent::Article(id) if view.kind == ManagerKind::Library => {
                let changed = self.library.navigate(&id);
                if changed {
                    self.focused = None;
                    chart::reset(view);
                    view.selections.insert("article".to_owned(), id);
                    view.scroll.clear();
                }
                changed
            }
            ManagerIntent::LibraryBack => {
                let changed = self.library.go_back();
                if changed {
                    chart::reset(view);
                    view.scroll.clear();
                }
                changed
            }
            ManagerIntent::LibraryForward => {
                let changed = self.library.go_forward();
                if changed {
                    chart::reset(view);
                    view.scroll.clear();
                }
                changed
            }
            ManagerIntent::Close
            | ManagerIntent::Open(_)
            | ManagerIntent::ForCity(..)
            | ManagerIntent::Dialog(_)
            | ManagerIntent::Submit(_)
            | ManagerIntent::Article(_)
            | ManagerIntent::ChartArticle(_)
            | ManagerIntent::Scroll(_, _)
            | ManagerIntent::ScrollTo(_, _)
            | ManagerIntent::BeginScroll(_)
            | ManagerIntent::SetValue(..)
            | ManagerIntent::BeginValue(..) => false,
        }
    }
}
