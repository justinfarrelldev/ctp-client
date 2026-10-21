//! Local read/unread and semantic result presentation. Rewards execute only server-side.
use super::MenuRect;
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::model::{GameState, MapPoint};
use crate::gameplay::server::v2::Notification;
use alloc::collections::{BTreeMap, BTreeSet};

/// Presentation-only capture of the original message ranger thumb.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageDrag {
    /// Pixel offset grabbed inside the thumb.
    pub grab: i32,
    /// Original track's top coordinate.
    pub top: i32,
    /// Available native thumb travel.
    pub travel: u32,
}

/// Original parchment message list and source message-window state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MessageLog {
    /// Current bounded server window, newest first.
    pub items: Vec<Notification>,
    /// Stable IDs already presented in this process/window.
    pub seen: BTreeSet<String>,
    /// Explicit local read marks, never written back as gameplay state.
    pub read: BTreeSet<String>,
    /// Exact open result, not a queue position or canonical event.
    pub selected: Option<String>,
    /// First visible message row.
    pub start: usize,
    /// Last complete snapshot processed; stable frames perform no repeated cloning.
    pub revision: Option<i64>,
    /// Previously received authoritative dates; never extrapolated from current time.
    pub dates: BTreeMap<u32, String>,
    /// Local native ranger capture, unrelated to world movement.
    pub drag: Option<MessageDrag>,
}
impl MessageLog {
    /// Updates only the server's retained window and returns newly seen notices.
    pub fn ingest(&mut self, notices: &[Notification]) -> Vec<Notification> {
        let notices = &notices[notices.len().saturating_sub(128)..];
        let ids = notices
            .iter()
            .map(|notice| notice.id.clone())
            .collect::<BTreeSet<_>>();
        let fresh = notices
            .iter()
            .filter(|notice| !self.seen.contains(&notice.id))
            .cloned()
            .collect();
        self.items = notices.iter().rev().cloned().collect();
        self.seen = ids;
        self.read.retain(|id| self.seen.contains(id));
        if self
            .selected
            .as_ref()
            .is_some_and(|id| !self.seen.contains(id))
        {
            self.selected = None;
        }
        self.start = self.start.min(self.items.len().saturating_sub(5));
        fresh
    }
    /// Number of currently retained unread notices.
    #[must_use]
    pub fn unread(&self) -> usize {
        self.items
            .iter()
            .filter(|notice| !self.read.contains(&notice.id))
            .count()
    }
    /// Accessible/native selection marks precisely one stable result as read.
    pub fn open(&mut self, id: &str) {
        if self.items.iter().any(|notice| notice.id == id) {
            self.drag = None;
            self.selected = Some(id.to_owned());
            self.read.insert(id.to_owned());
        }
    }
    /// Current semantic result, if still in the permitted retention window.
    #[must_use]
    pub fn current(&self) -> Option<&Notification> {
        self.items
            .iter()
            .find(|notice| self.selected.as_deref() == Some(notice.id.as_str()))
    }
    /// Bounds arrow/page/wheel movement against the actual retained notice count.
    pub fn scroll(&mut self, delta: i32) {
        self.start = self
            .start
            .saturating_add_signed(isize::try_from(delta).unwrap_or(0))
            .min(self.items.len().saturating_sub(5));
    }
    /// Source lower-panel list at native `ControlTabPanel` coordinates.
    #[must_use]
    pub const fn list() -> MenuRect {
        MenuRect {
            x: 397,
            y: 466,
            width: 318,
            height: 107,
        }
    }
    /// Exactly five complete 20-pixel rows, with no coordinate-dependent semantics.
    #[must_use]
    pub fn row(&self, index: usize) -> Option<MenuRect> {
        self.items.get(index)?;
        if index < self.start || index >= self.start.saturating_add(5) {
            return None;
        }
        Some(MenuRect {
            x: 399,
            y: 467_i32.saturating_add(
                i32::try_from(index.saturating_sub(self.start))
                    .ok()?
                    .saturating_mul(20),
            ),
            width: 310,
            height: 20,
        })
    }
    /// Source `StandardMessageWindow` is 530x188; ruin modal follows the reference.
    #[must_use]
    pub fn window(&self) -> MenuRect {
        if self
            .current()
            .is_some_and(|notice| notice.kind == "ruin.explored")
        {
            MenuRect {
                x: 365,
                y: 80,
                width: 380,
                height: 185,
            }
        } else {
            MenuRect {
                x: 0,
                y: 48,
                width: 530,
                height: 188,
            }
        }
    }
    /// Original message eye; absent terrain never becomes a location lookup.
    #[must_use]
    pub fn location(&self, state: &GameState) -> Option<MapPoint> {
        let id = &self.current()?.tile_id;
        state
            .viewer
            .as_ref()?
            .tiles
            .iter()
            .find(|tile| tile.id == *id)
            .map(|tile| MapPoint::new(tile.x, tile.y))
    }
}

/// Installed content labels and only legally projected semantic facts.
#[must_use]
pub fn text(notice: &Notification, state: &GameState, book: &LayoutBook) -> String {
    let content = book.label(&notice.content_id);
    let city = state
        .viewer
        .as_ref()
        .and_then(|viewer| viewer.cities.iter().find(|city| city.id == notice.city_id))
        .and_then(|city| city.facts["name"].as_str())
        .unwrap_or("your city");
    match notice.kind.as_str() {
        "production.completed" => format!("{content} is ready in {city}."),
        "governor.production_selected" => format!("The mayor selected {content} in {city}."),
        "ruin.explored" => match notice.reward_kind.as_str() {
            "gold" => format!("Exploration reward: {} gold.", notice.amount),
            "advance" => format!("Knowledge of {content} was discovered among the ruins."),
            "unit" => format!("A {content} has joined your civilization."),
            "city" => "The inhabitants have joined your civilization as a city.".to_owned(),
            "empty" => "The ruins have been explored; nothing was found.".to_owned(),
            _ => "The ruins have been explored; reward details are unavailable.".to_owned(),
        },
        "order.rejected" => {
            "An order could not be completed. Resynchronizing with the server.".to_owned()
        }
        "movement_order.completed" => "The army has reached its destination.".to_owned(),
        "movement_order.stopped" => {
            "The army's journey has stopped. Select a new destination to continue.".to_owned()
        }
        _ => "An authoritative game result was received.".to_owned(),
    }
}
