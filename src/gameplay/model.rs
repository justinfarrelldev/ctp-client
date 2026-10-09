//! Pure single-player gameplay state for the CTP2 in-game screen.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use super::audio::GameplaySound;

/// Minimum generated world width.
const MIN_WORLD_WIDTH: u16 = 24;
/// Minimum generated world height.
const MIN_WORLD_HEIGHT: u16 = 18;
/// Default human unit movement points per turn.
const DEFAULT_MOVE_POINTS: i16 = 100;
/// Movement points consumed by one land step.
const LAND_STEP_COST: i16 = 100;
/// Maximum distance shown by number overlays around the selected unit.
const REACHABLE_LABEL_RADIUS: i16 = 2;
/// Visibility radius for ordinary ancient units.
const UNIT_VISION_RADIUS: i16 = 2;
/// Visibility radius for cities.
const CITY_VISION_RADIUS: i16 = 3;
/// Owner id used by the human player.
pub const HUMAN_PLAYER_ID: PlayerId = PlayerId::Human;

/// Coordinate on the generated rectangular world map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MapPoint {
    /// Zero-based x coordinate.
    pub x: u16,
    /// Zero-based y coordinate.
    pub y: u16,
}

impl MapPoint {
    /// Creates a map point.
    #[must_use]
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    /// Returns a new point offset by signed deltas if it stays inside the map.
    #[must_use]
    pub fn offset(self, dx: i16, dy: i16, width: u16, height: u16) -> Option<Self> {
        let next_x = i32::from(self.x).checked_add(i32::from(dx))?;
        let next_y = i32::from(self.y).checked_add(i32::from(dy))?;
        if next_x < 0_i32
            || next_y < 0_i32
            || next_x >= i32::from(width)
            || next_y >= i32::from(height)
        {
            return None;
        }
        Some(Self {
            x: u16::try_from(next_x).ok()?,
            y: u16::try_from(next_y).ok()?,
        })
    }

    /// Returns Chebyshev distance, matching the eight-direction movement grid.
    #[must_use]
    pub fn tile_distance(self, other: Self) -> u16 {
        self.x.abs_diff(other.x).max(self.y.abs_diff(other.y))
    }
}

/// Terrain families rendered on the main map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainKind {
    /// Open grassland.
    Grassland,
    /// Plains.
    Plains,
    /// Forest.
    Forest,
    /// Hill.
    Hills,
    /// Mountain.
    Mountain,
    /// Desert.
    Desert,
    /// Swamp/wetland.
    Swamp,
    /// Ocean/water.
    Water,
}

impl TerrainKind {
    /// Returns whether ancient land units can enter this terrain.
    #[must_use]
    pub const fn passable_by_land(self) -> bool {
        !matches!(self, Self::Water)
    }

    /// Returns an RGB color used when original terrain art is unavailable.
    #[must_use]
    pub const fn fallback_color(self) -> (u8, u8, u8) {
        match self {
            Self::Grassland => (74, 143, 54),
            Self::Plains => (148, 132, 72),
            Self::Forest => (40, 104, 42),
            Self::Hills => (118, 103, 74),
            Self::Mountain => (112, 112, 112),
            Self::Desert => (190, 165, 92),
            Self::Swamp => (64, 105, 82),
            Self::Water => (42, 84, 145),
        }
    }
}

/// Special object placed on a tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileFeature {
    /// Ancient ruin/goody hut.
    Ruin,
}

/// Fog-of-war visibility state for a tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FogState {
    /// The tile has never been explored.
    Hidden,
    /// The tile was explored but is not currently visible.
    Explored,
    /// The tile is currently visible.
    Visible,
}

/// One world-map tile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapTile {
    /// Tile terrain.
    pub terrain: TerrainKind,
    /// Optional special feature.
    pub feature: Option<TileFeature>,
    /// Fog state from the human player's perspective.
    pub fog: FogState,
}

/// Owner of a unit or city.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PlayerId {
    /// Human player.
    Human,
    /// Normal non-barbarian empire, counted in empire totals.
    Empire(u8),
    /// Barbarian faction, never counted as an empire.
    Barbarians,
}

impl PlayerId {
    /// Returns whether this owner is a normal empire.
    #[must_use]
    pub const fn is_empire(self) -> bool {
        matches!(self, Self::Human | Self::Empire(_))
    }

    /// Returns whether this owner is barbarian.
    #[must_use]
    pub const fn is_barbarian(self) -> bool {
        matches!(self, Self::Barbarians)
    }
}

/// Stable unit identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnitId(pub u32);

/// Stable city identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CityId(pub u32);

/// Gameplay unit families needed by the launch-era implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitKind {
    /// CTP2 settler.
    Settler,
    /// Basic warrior used by early empires/barbarians.
    Warrior,
    /// Dedicated barbarian raider.
    Barbarian,
}

impl UnitKind {
    /// Returns the CTP2 sprite id from `spriteID.txt`.
    #[must_use]
    pub const fn sprite_id(self) -> u16 {
        match self {
            Self::Settler => 2,
            Self::Warrior | Self::Barbarian => 3,
        }
    }

    /// Returns the display label used by fallback rendering.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Settler => "Settler",
            Self::Warrior => "Warrior",
            Self::Barbarian => "Barbarian",
        }
    }

    /// Returns combat strength for deterministic simple combat.
    #[must_use]
    pub const fn strength(self) -> i16 {
        match self {
            Self::Settler => 1,
            Self::Warrior | Self::Barbarian => 3,
        }
    }
}

/// Unit standing order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitOrder {
    /// No long-lived order.
    None,
    /// Unit will be skipped by automatic cycling until activated.
    Sentinel,
    /// Unit has a queued movement path.
    MoveTo { path: Vec<MapPoint> },
}

/// A unit on the world map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unit {
    /// Stable identifier.
    pub id: UnitId,
    /// Unit kind.
    pub kind: UnitKind,
    /// Owning player.
    pub owner: PlayerId,
    /// Current tile position.
    pub position: MapPoint,
    /// Remaining movement points this turn.
    pub movement_points: i16,
    /// Maximum movement points per turn.
    pub max_movement_points: i16,
    /// Vision radius in tiles.
    pub vision_radius: i16,
    /// Whether the unit is alive.
    pub alive: bool,
    /// Current order.
    pub order: UnitOrder,
}

/// A founded city.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct City {
    /// Stable identifier.
    pub id: CityId,
    /// Owning player.
    pub owner: PlayerId,
    /// City tile.
    pub position: MapPoint,
    /// Display name.
    pub name: String,
    /// Current build queue head.
    pub build_item: String,
}

/// Empire metadata discovered by the local game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmpireContact {
    /// Empire owner id.
    pub owner: PlayerId,
    /// Whether this empire has been encountered by the human player.
    pub encountered: bool,
}

/// Ruin outcome families.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuinOutcome {
    /// Gold was found.
    Gold,
    /// Knowledge/map information was found.
    Knowledge,
    /// Barbarians emerged.
    Barbarians,
    /// The ruin became a small friendly city.
    City,
}

/// Popup category shown by the in-game message system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupKind {
    /// Beginning-of-game message.
    GameStart,
    /// Ruin/hut result.
    Ruin,
    /// Encounter with another empire.
    Encounter,
    /// Combat/death result.
    Combat,
    /// City/build menu notification.
    City,
    /// Generic information.
    Info,
}

/// One modal/non-modal gameplay popup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameplayPopup {
    /// Popup title.
    pub title: String,
    /// Popup body text.
    pub body: String,
    /// Popup category.
    pub kind: PopupKind,
}

/// Event emitted by a model action for runtime sound/popup routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameplayEvent {
    /// Sound should be played.
    Sound(GameplaySound),
    /// Popup should be shown.
    Popup(GameplayPopup),
    /// Unit moved.
    UnitMoved(UnitId, MapPoint),
    /// Unit died.
    UnitKilled(UnitId),
    /// City was founded.
    CityFounded(CityId),
    /// Ruin was resolved.
    RuinResolved(RuinOutcome),
    /// An empire was encountered.
    EmpireEncountered(PlayerId),
    /// Turn advanced.
    TurnAdvanced(u32),
}

/// Reachability label rendered over a tile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReachableTileLabel {
    /// Tile position.
    pub position: MapPoint,
    /// Number shown on the GUI (`1`, `2`, etc.).
    pub label: String,
}

/// Map-filter toggle state.
#[expect(
    clippy::struct_excessive_bools,
    reason = "CTP2 exposes these independent map overlay toggles as separate UI switches"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapFilters {
    /// Show city-name labels.
    pub city_names: bool,
    /// Show trade routes.
    pub trade_routes: bool,
    /// Show tile grid.
    pub grid: bool,
    /// Show political borders.
    pub political_borders: bool,
    /// Apply fog-of-war darkening.
    pub fog_of_war: bool,
    /// Show minimap/radar.
    pub radar: bool,
}

impl Default for MapFilters {
    fn default() -> Self {
        Self {
            city_names: true,
            trade_routes: true,
            grid: false,
            political_borders: true,
            fog_of_war: true,
            radar: true,
        }
    }
}

/// Complete pure gameplay state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    /// World width in tiles.
    pub width: u16,
    /// World height in tiles.
    pub height: u16,
    /// Tiles in row-major order.
    pub tiles: Vec<MapTile>,
    /// Units on the map.
    pub units: Vec<Unit>,
    /// Founded cities.
    pub cities: Vec<City>,
    /// Empire contacts, excluding barbarians.
    pub contacts: Vec<EmpireContact>,
    /// Current one-based turn number.
    pub turn: u32,
    /// Human gold reserve.
    pub gold: i32,
    /// Human public works reserve.
    pub public_works: i32,
    /// Currently selected unit.
    pub selected_unit: Option<UnitId>,
    /// Pending popups.
    pub popups: Vec<GameplayPopup>,
    /// Active map filters.
    pub filters: MapFilters,
    /// Deterministic RNG seed/state.
    pub seed: u64,
    /// Whether the city build menu is open.
    pub city_build_menu: Option<CityId>,
}

/// Game-state actions, turn processing, fog, combat, and generation helpers.
mod actions;
/// Deterministic world generation and movement helper functions.
mod generation;

#[cfg(test)]
mod tests;
