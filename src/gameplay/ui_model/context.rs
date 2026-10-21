//! Map context actions from `controlpanelwindow.cpp:BuildCityList/BuildUnitList`.
//!
//! Visible source controls with no server operation remain explicitly disabled;
//! opening a menu cannot issue a command or mutate the authoritative game.

use super::MenuRect;
use crate::gameplay::model::{CityId, FogState, GameState, MapPoint, PlayerId, UnitId, UnitKind};
use crate::gameplay::server::interface::LegalAction;

/// Explicit intent, evaluated only after player selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextIntent {
    /// Open city management for this exact city.
    City(CityId),
    /// Open production for this exact city.
    Build(CityId),
    /// Open army composition for this exact unit.
    Army(UnitId),
    /// Activate movement for the clicked unit, not another selected unit.
    Move(UnitId),
    /// Execute a server-projected legal payload verbatim.
    Submit(LegalAction),
    /// Inspect viewer-known map information.
    MapInfo(MapPoint),
    /// Base-game operation absent from the current authoritative contract.
    Unavailable(&'static str),
}

/// Source-order menu row with a stable accessible label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextRow {
    /// Visible and accessible label.
    pub label: &'static str,
    /// Action owned by this row; disabled actions carry a reason.
    pub intent: ContextIntent,
}

impl ContextRow {
    /// Unavailable source actions are never falsely advertised as functional.
    #[must_use]
    pub const fn enabled(&self) -> bool {
        !matches!(self.intent, ContextIntent::Unavailable(_))
    }
}

/// Weakly-modal menu clamped inside the native logical game frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextView {
    /// Exact clicked map coordinate, retained while the camera moves.
    pub point: MapPoint,
    /// Window geometry shared by SDL and semantic interaction tests.
    pub rect: MenuRect,
    /// Source-order rows.
    pub rows: Vec<ContextRow>,
    /// Keyboard-selected row; pointer motion restores hover presentation.
    pub focused: usize,
    /// Whether keyboard navigation currently owns row highlighting.
    pub keyboard_focus: bool,
}

impl ContextView {
    /// Constructs a menu only from viewer-visible context; no command is sent.
    #[must_use]
    pub fn new(state: &GameState, point: MapPoint, x: i32, y: i32) -> Option<Self> {
        let rows = context_at(state, point);
        if rows.is_empty() {
            return None;
        }
        let width = rows
            .iter()
            .map(|row| {
                u32::try_from(row.label.len())
                    .unwrap_or(0)
                    .saturating_mul(7)
                    .saturating_add(14)
            })
            .max()
            .unwrap_or(100);
        let height = u32::try_from(rows.len())
            .unwrap_or(0)
            .saturating_mul(16)
            .saturating_add(4);
        Some(Self {
            point,
            rect: MenuRect {
                x: x.clamp(
                    0,
                    800_i32.saturating_sub(i32::try_from(width).unwrap_or(800)),
                ),
                y: y.clamp(
                    24,
                    600_i32
                        .saturating_sub(i32::try_from(height).unwrap_or(600))
                        .max(24),
                ),
                width,
                height,
            },
            rows,
            focused: 0,
            keyboard_focus: false,
        })
    }

    /// Moves keyboard focus through enabled rows only, with bounded wraparound.
    pub fn move_focus(&mut self, delta: i32) {
        let length = i32::try_from(self.rows.len()).unwrap_or(0_i32);
        if length == 0_i32 {
            return;
        }
        for _ in 0..self.rows.len() {
            self.focused = usize::try_from(
                i32::try_from(self.focused)
                    .unwrap_or(0_i32)
                    .saturating_add(delta)
                    .rem_euclid(length),
            )
            .unwrap_or(0_usize);
            if self.rows[self.focused].enabled() {
                break;
            }
        }
        self.keyboard_focus = true;
    }

    /// Resolves explicit Enter activation; merely focusing a row has no effect.
    #[must_use]
    pub fn focused_intent(&self) -> Option<ContextIntent> {
        self.rows
            .get(self.focused)
            .filter(|row| row.enabled())
            .map(|row| row.intent.clone())
    }

    /// Half-open menu row geometry, excluding the two-pixel frame.
    #[must_use]
    pub fn row_rect(&self, index: usize) -> Option<MenuRect> {
        self.rows.get(index)?;
        Some(MenuRect {
            x: self.rect.x.saturating_add(2),
            y: self
                .rect
                .y
                .saturating_add(2)
                .saturating_add(i32::try_from(index).ok()?.saturating_mul(16)),
            width: self.rect.width.saturating_sub(4),
            height: 16,
        })
    }

    /// Returns an enabled intent by exact semantic label.
    #[must_use]
    pub fn activate(&self, label: &str) -> Option<ContextIntent> {
        self.rows
            .iter()
            .find(|row| row.enabled() && row.label.eq_ignore_ascii_case(label))
            .map(|row| row.intent.clone())
    }

    /// Resolves the same row as semantic activation for physical pointer input.
    #[must_use]
    pub fn intent_at(&self, x: i32, y: i32) -> Option<ContextIntent> {
        self.rows
            .iter()
            .enumerate()
            .find(|&(index, row)| {
                row.enabled() && self.row_rect(index).is_some_and(|rect| rect.contains(x, y))
            })
            .map(|(_, row)| row.intent.clone())
    }
}

/// Chooses city-over-garrison priority and never discloses unseen occupants.
#[must_use]
pub fn context_at(state: &GameState, point: MapPoint) -> Vec<ContextRow> {
    let Some(tile) = state
        .tile(point)
        .filter(|tile| tile.fog != FogState::Hidden)
    else {
        return Vec::new();
    };
    let info = ContextRow {
        label: "Map Info",
        intent: ContextIntent::MapInfo(point),
    };
    if let Some(city) = state
        .city_at(point)
        .filter(|city| city.owner == PlayerId::Human && tile.fog == FogState::Visible)
    {
        return vec![
            ContextRow {
                label: "City Manager",
                intent: ContextIntent::City(city.id),
            },
            ContextRow {
                label: "Build Manager",
                intent: ContextIntent::Build(city.id),
            },
            unavailable("Rename City", "Server has no rename-city route"),
            unavailable("Disband City", "Server has no disband-city route"),
            unavailable("Turn Mayor ON", "Server has no city-governor route"),
            info,
        ];
    }
    let unit = state
        .selected_unit()
        .filter(|unit| unit.position == point && unit.owner == PlayerId::Human)
        .or_else(|| {
            state
                .units
                .iter()
                .find(|unit| unit.alive && unit.position == point && unit.owner == PlayerId::Human)
        });
    let Some(unit) = unit.filter(|_| tile.fog == FogState::Visible) else {
        return vec![info];
    };
    let mut rows = vec![
        ContextRow {
            label: "Army Manager",
            intent: ContextIntent::Army(unit.id),
        },
        ContextRow {
            label: "Move",
            intent: ContextIntent::Move(unit.id),
        },
    ];
    if unit.kind == UnitKind::Settler {
        let legal = legal_for_unit(state, unit.id, "found-city");
        rows.push(ContextRow {
            label: "Settle",
            intent: legal.cloned().map_or(
                ContextIntent::Unavailable("No legal settlement action for this unit"),
                ContextIntent::Submit,
            ),
        });
    }
    if unit.kind != UnitKind::Settler {
        rows.push(unavailable(
            "Attack",
            "Choose a legal target through combat planning",
        ));
    }
    for (label, route) in [
        ("Fortify", "fortify"),
        ("Sentinel", "sentinel"),
        ("Pillage", "pillage"),
        ("Pirate Trade", "pirate-trade"),
        ("Expel", "expel"),
        ("Reform City", "reform"),
        ("Cancel Movement", "cancel-movement-order"),
        ("Board Transport", "embark"),
        ("Disband Army", "disband"),
    ] {
        let legal = legal_for_unit(state, unit.id, route);
        rows.push(ContextRow {
            label,
            intent: legal
                .filter(|action| action.http_supported())
                .cloned()
                .map_or(
                    ContextIntent::Unavailable("No legal authoritative action for this unit"),
                    ContextIntent::Submit,
                ),
        });
    }
    rows.push(info);
    rows
}

/// Resolves only an unambiguous projected target; do not silently select the
/// first transport/trade target when a separate target chooser is required.
fn legal_for_unit<'viewer>(
    state: &'viewer GameState,
    id: UnitId,
    route: &str,
) -> Option<&'viewer LegalAction> {
    let viewer = state.viewer.as_ref()?;
    let source = viewer
        .units
        .iter()
        .find(|source| UnitId::from_server_id(&source.id) == id)?;
    let mut actions = viewer
        .interface
        .legal_actions
        .iter()
        .filter(|action| action.entity_id == source.id && action.route == route);
    let action = actions.next()?;
    if actions.next().is_some() {
        None
    } else {
        Some(action)
    }
}

/// Preserves a source control without substituting a different engine command.
const fn unavailable(label: &'static str, reason: &'static str) -> ContextRow {
    ContextRow {
        label,
        intent: ContextIntent::Unavailable(reason),
    }
}
