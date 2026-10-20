//! Fileless Great Library browsing, filtering and measured hyperlink layout.

use alloc::collections::VecDeque;

use crate::gameplay::assets::library::LibraryCatalog;

/// Safe rich-text layout shared by native rendering and clickable link descriptors.
pub mod layout;

/// Bounded presentation history; navigating never grows an unbounded game log.
const HISTORY_LIMIT: usize = 256;

/// Original Great Library category rail, including terrain-named resource topics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LibraryCategory {
    /// Advances.
    #[default]
    Advances,
    /// Unit types.
    Units,
    /// City improvements.
    Buildings,
    /// Wonders.
    Wonders,
    /// Ordinary terrain, excluding commodity records.
    Terrain,
    /// Resource topics whose original IDs contain `_GOOD_`.
    Goods,
    /// Tile improvements.
    TileImprovements,
    /// Governments.
    Governments,
    /// Orders/special actions.
    Orders,
    /// General concepts.
    Concepts,
}

impl LibraryCategory {
    /// Original category label, independent of canonical article IDs.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Advances => "Advances",
            Self::Units => "Units",
            Self::Buildings => "Buildings",
            Self::Wonders => "Wonders",
            Self::Terrain => "Terrain",
            Self::Goods => "Goods",
            Self::TileImprovements => "Tile Improvements",
            Self::Governments => "Governments",
            Self::Orders => "Orders",
            Self::Concepts => "Concepts",
        }
    }

    /// Exact localized category keys used by the original `SetCategoryName` callback.
    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::Advances => "str_ldl_CAPS_ADVANCES",
            Self::Units => "str_ldl_CAPS_UNITS",
            Self::Buildings => "str_ldl_IMPROVE",
            Self::Wonders => "str_ldl_CAPS_WONDERS",
            Self::Terrain => "str_ldl_TERRAIN",
            Self::Goods => "str_ldl_CAPS_GOODS",
            Self::TileImprovements => "str_ldl_CAPS_TILE_IMP",
            Self::Governments => "str_ldl_GOVERN",
            Self::Orders => "str_ldl_ORDERS",
            Self::Concepts => "str_ldl_CONCEPTS",
        }
    }

    /// Resolves actual source control names, including singular Concept/Govern.
    #[must_use]
    pub fn from_button(name: &str) -> Option<Self> {
        match name {
            "AdvancesButton" => Some(Self::Advances),
            "UnitsButton" => Some(Self::Units),
            "ImproveButton" => Some(Self::Buildings),
            "WondersButton" => Some(Self::Wonders),
            "TerrainButton" => Some(Self::Terrain),
            "GoodsButton" => Some(Self::Goods),
            "TileImpButton" => Some(Self::TileImprovements),
            "GovernButton" => Some(Self::Governments),
            "OrdersButton" => Some(Self::Orders),
            "ConceptButton" => Some(Self::Concepts),
            _ => None,
        }
    }

    /// Maps a known source article to its original category without using game state.
    #[must_use]
    pub fn for_topic(id: &str) -> Option<Self> {
        if id.contains("_GOOD_") {
            return Some(Self::Goods);
        }
        [
            ("ADVANCE_", Self::Advances),
            ("UNIT_", Self::Units),
            ("IMPROVE_", Self::Buildings),
            ("WONDER_", Self::Wonders),
            ("TERRAIN_", Self::Terrain),
            ("TILEIMP_", Self::TileImprovements),
            ("GOVERNMENT_", Self::Governments),
            ("ORDER_", Self::Orders),
            ("CONCEPT_", Self::Concepts),
        ]
        .into_iter()
        .find_map(|(prefix, category)| id.starts_with(prefix).then_some(category))
    }
}

/// Local browser state, not research/production authority or persistent game data.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LibraryBrowser {
    /// Read-only installed article index.
    pub catalog: LibraryCatalog,
    /// Exact currently displayed article ID.
    pub current: Option<String>,
    /// Active category rail entry.
    pub category: LibraryCategory,
    /// Search text; bounded in characters and never treated as a path.
    pub query: String,
    /// The original index stays empty until a category or search is explicitly used.
    pub index_open: bool,
    /// Cached global search matches; None means the category index, not search mode.
    pub search_results: Option<Vec<String>>,
    /// Previous article IDs in browser order.
    pub back: VecDeque<String>,
    /// Forward destinations after Back.
    pub forward: Vec<String>,
}

impl LibraryBrowser {
    /// Creates a browser without opening files, an SDL window or a server session.
    #[must_use]
    pub fn new(catalog: LibraryCatalog) -> Self {
        Self {
            catalog,
            ..Self::default()
        }
    }

    /// Navigates only to installed topics and records a bounded history.
    pub fn navigate(&mut self, id: &str) -> bool {
        if self.catalog.article(id).is_none() {
            return false;
        }
        if self.current.as_deref() == Some(id) {
            return true;
        }
        if let Some(previous) = self.current.replace(id.to_owned()) {
            if self.back.len() >= HISTORY_LIMIT {
                self.back.pop_front();
            }
            self.back.push_back(previous);
        }
        self.forward.clear();
        if self.search_results.is_none() {
            self.query.clear();
        }
        if let Some(category) = LibraryCategory::for_topic(id) {
            self.category = category;
        }
        true
    }

    /// Restores the previous article, not the previous selected queue entry.
    pub fn go_back(&mut self) -> bool {
        let Some(id) = self.back.pop_back() else {
            return false;
        };
        if let Some(current) = self.current.replace(id.clone()) {
            self.forward.push(current);
        }
        self.restore_category(&id);
        true
    }

    /// Restores an article after Back, without manufacturing a new history branch.
    pub fn go_forward(&mut self) -> bool {
        let Some(id) = self.forward.pop() else {
            return false;
        };
        if let Some(current) = self.current.replace(id.clone()) {
            self.back.push_back(current);
        }
        self.restore_category(&id);
        true
    }

    /// Category switching changes only the index, not the displayed article.
    pub fn set_category(&mut self, category: LibraryCategory) {
        self.category = category;
        self.query.clear();
        self.index_open = true;
        self.search_results = None;
    }

    /// Bounds pasted Unicode search text; editing never accesses a path or network.
    pub fn set_query(&mut self, text: &str) {
        self.query = text.chars().take(128_usize).collect();
    }

    /// Searches titles and article text across all categories, as the original
    /// `Search_Great_Library` callback does. Work is cached on explicit execution,
    /// not repeated across every native frame or hypothetical game mutation.
    pub fn execute_search(&mut self, label: impl Fn(&str) -> String) {
        let query = self.query.trim().to_lowercase();
        let matches = self
            .catalog
            .articles
            .iter()
            .filter(|entry| {
                let id = entry.0;
                let article = entry.1;
                label(id).to_lowercase().contains(&query)
                    || id.to_lowercase().contains(&query)
                    || article.sections.values().any(|document| {
                        document
                            .plain_text(|_| String::new())
                            .to_lowercase()
                            .contains(&query)
                    })
            })
            .map(|(id, _)| id.clone())
            .collect();
        self.index_open = true;
        self.search_results = Some(matches);
    }

    /// Returns the explicitly opened category index or cached global search results.
    #[must_use]
    pub fn entries(&self, label: impl Fn(&str) -> String) -> Vec<(String, String)> {
        if !self.index_open {
            return Vec::new();
        }
        let mut entries: Vec<_> = self.search_results.as_ref().map_or_else(
            || {
                self.catalog
                    .articles
                    .keys()
                    .filter(|id| LibraryCategory::for_topic(id) == Some(self.category))
                    .map(|id| (id.clone(), label(id)))
                    .collect()
            },
            |results| results.iter().map(|id| (id.clone(), label(id))).collect(),
        );
        entries.sort_by_cached_key(|entry| entry.1.to_lowercase());
        entries
    }

    /// Restores an article's index category and clears obsolete search restrictions.
    fn restore_category(&mut self, id: &str) {
        if self.search_results.is_none() {
            self.query.clear();
        }
        if let Some(category) = LibraryCategory::for_topic(id) {
            self.category = category;
        }
    }
}
