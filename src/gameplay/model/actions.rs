//! Game-state actions, turn processing, fog, combat, and generation helpers.
#![expect(
    clippy::arithmetic_side_effects,
    reason = "Gameplay coordinate and movement math is bounded by small map dimensions and checked where it can cross storage boundaries"
)]
extern crate alloc;
use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::cmp::Ordering;
use crate::gameplay::audio::GameplaySound;
use crate::gameplay::server::ServerProjection;
use super::generation::{
    direction_delta, generated_contacts, generated_tiles, lcg_next, new_unit, next_id_from_len,
    owners_are_enemies, ruin_outcome, seeded_edge_point, start_popup, step_toward,
    straight_line_path,
};
use super::{
    CITY_VISION_RADIUS, City, CityId, FogState, GameState, GameplayEvent, GameplayPopup,
    LAND_STEP_COST, MIN_WORLD_HEIGHT, MIN_WORLD_WIDTH, MapFilters, MapPoint, MapTile, PlayerId,
    PopupKind, REACHABLE_LABEL_RADIUS, ReachableTileLabel, RuinOutcome, TileFeature, Unit, UnitId,
    UnitKind, UnitOrder,
};
impl GameState {
    /// Creates a deterministic game ready for the first render after Launch.
    #[must_use]
    pub fn new(width: u16, height: u16, empire_count: u8, seed: u64) -> Self {
        let width = width.max(MIN_WORLD_WIDTH);
        let height = height.max(MIN_WORLD_HEIGHT);
        let mut state = Self {
            width,
            height,
            tiles: generated_tiles(width, height, seed),
            units: Vec::new(),
            cities: Vec::new(),
            contacts: generated_contacts(empire_count),
            turn: 1,
            gold: 0,
            public_works: 0,
            selected_unit: None,
            popups: vec![start_popup()],
            filters: MapFilters::default(),
            seed,
            city_build_menu: None,
        };
        state.place_initial_actors(empire_count);
        state.refresh_fog();
        state.selected_unit = state.first_human_unit_id();
        state
    }

    /// Creates a renderable state from an authoritative server projection.
    #[must_use]
    pub fn from_server_projection(p: &ServerProjection, empire_count: u8, seed: u64) -> Self {
        super::server_projection::game_state_from_server_projection(p, empire_count, seed)
    }

    /// Queues a popup confirming server authority for this game.
    pub fn queue_server_connected_popup(&mut self, game_id: i64) {
        super::server_projection::queue_server_connected_popup(self, game_id);
    }

    /// Returns the number of non-barbarian empires, including the human empire.
    #[must_use]
    pub fn empire_count(&self) -> usize {
        self.contacts
            .iter()
            .filter(|contact| contact.owner.is_empire())
            .count()
    }

    /// Returns the tile at a point.
    #[must_use]
    pub fn tile(&self, point: MapPoint) -> Option<&MapTile> {
        self.tile_index(point)
            .and_then(|index| self.tiles.get(index))
    }

    /// Returns a mutable tile at a point.
    pub(super) fn tile_mut(&mut self, point: MapPoint) -> Option<&mut MapTile> {
        self.tile_index(point)
            .and_then(|index| self.tiles.get_mut(index))
    }

    /// Selects a unit by id if it is alive and human-owned.
    pub fn select_unit(&mut self, id: UnitId) -> bool {
        let selectable = self
            .units
            .iter()
            .any(|unit| unit.id == id && unit.alive && unit.owner == PlayerId::Human);
        if selectable {
            self.selected_unit = Some(id);
        }
        selectable
    }

    /// Returns the selected unit.
    #[must_use]
    pub fn selected_unit(&self) -> Option<&Unit> {
        let id = self.selected_unit?;
        self.units.iter().find(|unit| unit.id == id && unit.alive)
    }

    /// Returns labels for reachable tiles around the selected unit.
    #[must_use]
    pub fn reachable_tile_labels(&self) -> Vec<ReachableTileLabel> {
        let Some(unit) = self.selected_unit() else {
            return Vec::new();
        };
        let mut labels = Vec::new();
        for dy in -REACHABLE_LABEL_RADIUS..=REACHABLE_LABEL_RADIUS {
            for dx in -REACHABLE_LABEL_RADIUS..=REACHABLE_LABEL_RADIUS {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let Some(point) = unit.position.offset(dx, dy, self.width, self.height) else {
                    continue;
                };
                if !self.is_land_passable(point) {
                    continue;
                }
                let distance = unit.position.tile_distance(point);
                if distance == 0
                    || i16::try_from(distance).map_or(true, |value| value > REACHABLE_LABEL_RADIUS)
                {
                    continue;
                }
                labels.push(ReachableTileLabel {
                    position: point,
                    label: distance.to_string(),
                });
            }
        }
        labels.sort_by(|left, right| match left.label.cmp(&right.label) {
            Ordering::Equal => left.position.cmp(&right.position),
            other => other,
        });
        labels
    }

    /// Orders or immediately moves the selected unit to a target tile.
    pub fn order_selected_to(&mut self, target: MapPoint) -> Vec<GameplayEvent> {
        let mut events = Vec::new();
        if !self.in_bounds(target) || !self.is_land_passable(target) {
            return events;
        }
        let Some(unit_id) = self.selected_unit else {
            return events;
        };
        let Some(start) = self.unit_position(unit_id) else {
            return events;
        };
        let path = straight_line_path(start, target, self.width, self.height);
        if path.is_empty() {
            return events;
        }
        self.set_unit_order(unit_id, UnitOrder::MoveTo { path });
        events.extend(self.process_unit_path(unit_id));
        self.refresh_fog();
        events
    }

    /// Moves the selected unit one tile by signed offset.
    pub fn move_selected_by(&mut self, dx: i16, dy: i16) -> Vec<GameplayEvent> {
        let Some(unit) = self.selected_unit() else {
            return Vec::new();
        };
        let Some(target) = unit.position.offset(dx, dy, self.width, self.height) else {
            return Vec::new();
        };
        self.order_selected_to(target)
    }

    /// Sets the selected unit to sentinel.
    pub fn sentinel_selected(&mut self) -> Vec<GameplayEvent> {
        let mut events = Vec::new();
        if let Some(id) = self.selected_unit {
            self.set_unit_order(id, UnitOrder::Sentinel);
            events.push(GameplayEvent::Sound(GameplaySound::UnitAcknowledge));
        }
        events
    }

    /// Founds a city with the selected settler.
    pub fn settle_selected(&mut self, requested_name: &str) -> Vec<GameplayEvent> {
        let mut events = Vec::new();
        let Some(unit_id) = self.selected_unit else {
            return events;
        };
        let Some(unit_index) = self.unit_index(unit_id) else {
            return events;
        };
        if self.units[unit_index].kind != UnitKind::Settler || !self.units[unit_index].alive {
            return events;
        }
        let position = self.units[unit_index].position;
        if !self.is_land_passable(position) || self.city_at(position).is_some() {
            return events;
        }
        self.units[unit_index].alive = false;
        let city_id = CityId(next_id_from_len(self.cities.len()));
        let name = if requested_name.trim().is_empty() {
            format!("City {}", self.cities.len().saturating_add(1))
        } else {
            requested_name.trim().to_owned()
        };
        self.cities.push(City {
            id: city_id,
            owner: PlayerId::Human,
            position,
            name,
            build_item: "Warrior".to_owned(),
        });
        self.city_build_menu = Some(city_id);
        self.selected_unit = self.first_human_unit_id();
        self.refresh_fog();
        let popup = GameplayPopup {
            title: "City Founded".to_owned(),
            body: "Your settlers have founded a new city. Choose a build item from the city build menu.".to_owned(),
            kind: PopupKind::City,
        };
        self.popups.push(popup.clone());
        events.push(GameplayEvent::Sound(GameplaySound::Settle));
        events.push(GameplayEvent::CityFounded(city_id));
        events.push(GameplayEvent::Popup(popup));
        events
    }

    /// Advances all players by one turn.
    pub fn end_turn(&mut self) -> Vec<GameplayEvent> {
        let mut events = Vec::new();
        let human_ids: Vec<UnitId> = self
            .units
            .iter()
            .filter(|unit| unit.owner == PlayerId::Human && unit.alive)
            .map(|unit| unit.id)
            .collect();
        for id in human_ids {
            events.extend(self.process_unit_path(id));
            self.reset_unit_movement(id);
        }
        events.extend(self.advance_barbarians());
        events.extend(self.advance_empires());
        self.turn = self.turn.saturating_add(1);
        self.refresh_fog();
        self.selected_unit = self.first_awake_human_unit_id();
        events.push(GameplayEvent::Sound(GameplaySound::EndTurn));
        events.push(GameplayEvent::TurnAdvanced(self.turn));
        events
    }

    /// Dismisses the oldest popup, if any.
    pub fn dismiss_popup(&mut self) -> Option<GameplayPopup> {
        if self.popups.is_empty() {
            return None;
        }
        Some(self.popups.remove(0))
    }

    /// Toggles city-name map filter.
    pub const fn toggle_city_names(&mut self) {
        self.filters.city_names = !self.filters.city_names;
    }

    /// Toggles trade-route map filter.
    pub const fn toggle_trade_routes(&mut self) {
        self.filters.trade_routes = !self.filters.trade_routes;
    }

    /// Toggles fog-of-war rendering filter.
    pub const fn toggle_fog_of_war(&mut self) {
        self.filters.fog_of_war = !self.filters.fog_of_war;
    }

    /// Toggles radar/minimap rendering.
    pub const fn toggle_radar(&mut self) {
        self.filters.radar = !self.filters.radar;
    }

    /// Toggles grid overlay.
    pub const fn toggle_grid(&mut self) {
        self.filters.grid = !self.filters.grid;
    }

    /// Returns whether a point is on the map.
    #[must_use]
    pub const fn in_bounds(&self, point: MapPoint) -> bool {
        point.x < self.width && point.y < self.height
    }

    /// Returns the city at a point, if any.
    #[must_use]
    pub fn city_at(&self, point: MapPoint) -> Option<&City> {
        self.cities
            .iter()
            .find(|city| city.position == point && city.owner.is_empire())
    }

    /// Returns alive units at a point.
    #[must_use]
    pub fn units_at(&self, point: MapPoint) -> Vec<&Unit> {
        self.units
            .iter()
            .filter(|unit| unit.alive && unit.position == point)
            .collect()
    }

    /// Places deterministic launch actors.
    fn place_initial_actors(&mut self, empire_count: u8) {
        let start = self.nearest_passable(MapPoint::new(
            self.width.div_euclid(2),
            self.height.div_euclid(2),
        ));
        self.units.push(new_unit(
            UnitId(1),
            UnitKind::Settler,
            PlayerId::Human,
            start,
        ));
        if let Some(second) = start.offset(1, 0, self.width, self.height) {
            let second = self.nearest_passable(second);
            self.units.push(new_unit(
                UnitId(2),
                UnitKind::Settler,
                PlayerId::Human,
                second,
            ));
        }
        let normal_empires = empire_count.saturating_sub(1);
        for index in 0..normal_empires {
            let owner = PlayerId::Empire(index.saturating_add(1));
            let base = seeded_edge_point(self.width, self.height, self.seed, index);
            let point = self.nearest_passable(base);
            self.units.push(new_unit(
                UnitId(u32::from(index).saturating_add(10)),
                UnitKind::Settler,
                owner,
                point,
            ));
        }
        for index in 0_u8..3_u8 {
            let base = seeded_edge_point(
                self.width,
                self.height,
                self.seed ^ 0xBABA_u64,
                index.saturating_add(9),
            );
            let point = self.nearest_passable(base);
            self.units.push(new_unit(
                UnitId(u32::from(index).saturating_add(100)),
                UnitKind::Barbarian,
                PlayerId::Barbarians,
                point,
            ));
        }
    }

    /// Processes one unit's queued path until movement is exhausted.
    fn process_unit_path(&mut self, unit_id: UnitId) -> Vec<GameplayEvent> {
        let mut events = Vec::new();
        loop {
            let Some(next) = self.next_step_for_unit(unit_id) else {
                break;
            };
            let Some(unit_index) = self.unit_index(unit_id) else {
                break;
            };
            if self.units[unit_index].movement_points < LAND_STEP_COST {
                break;
            }
            if !self.is_land_passable(next) {
                self.set_unit_order(unit_id, UnitOrder::None);
                break;
            }
            if let Some(defender_index) =
                self.enemy_unit_index_at(next, self.units[unit_index].owner)
            {
                events.extend(self.resolve_combat(unit_index, defender_index));
                self.pop_unit_path_front(unit_id);
                break;
            }
            self.units[unit_index].position = next;
            self.units[unit_index].movement_points = self.units[unit_index]
                .movement_points
                .saturating_sub(LAND_STEP_COST);
            self.pop_unit_path_front(unit_id);
            events.push(GameplayEvent::Sound(GameplaySound::UnitMove));
            events.push(GameplayEvent::UnitMoved(unit_id, next));
            events.extend(self.resolve_tile_entry(unit_id));
            if !self.unit_alive(unit_id) {
                break;
            }
        }
        events
    }

    /// Resolves tile features and contacts after a unit enters a tile.
    pub(super) fn resolve_tile_entry(&mut self, unit_id: UnitId) -> Vec<GameplayEvent> {
        let mut events = Vec::new();
        let Some(position) = self.unit_position(unit_id) else {
            return events;
        };
        events.extend(self.resolve_ruin(position));
        events.extend(self.resolve_contacts(position));
        events
    }

    /// Resolves a ruin if present.
    fn resolve_ruin(&mut self, position: MapPoint) -> Vec<GameplayEvent> {
        let mut events = Vec::new();
        let Some(tile) = self.tile_mut(position) else {
            return events;
        };
        if tile.feature != Some(TileFeature::Ruin) {
            return events;
        }
        tile.feature = None;
        let outcome = ruin_outcome(self.seed, self.turn, position);
        match outcome {
            RuinOutcome::Gold => {
                self.gold = self.gold.saturating_add(100);
                self.queue_ruin_popup("Ruin Explored", "Your people discover ancient gold.");
            }
            RuinOutcome::Knowledge => {
                self.reveal_radius(position, 5);
                self.queue_ruin_popup(
                    "Ruin Explored",
                    "Ancient maps reveal the surrounding lands.",
                );
            }
            RuinOutcome::Barbarians => {
                self.spawn_barbarian_near(position);
                self.queue_ruin_popup("Barbarians!", "Wandering barbarians pour out of the ruins.");
            }
            RuinOutcome::City => {
                let city_id = CityId(next_id_from_len(self.cities.len()));
                self.cities.push(City {
                    id: city_id,
                    owner: PlayerId::Human,
                    position,
                    name: format!("Ruin City {}", self.cities.len().saturating_add(1)),
                    build_item: "Warrior".to_owned(),
                });
                self.queue_ruin_popup(
                    "Ruin Becomes City",
                    "The ruins become a small friendly city.",
                );
            }
        }
        let popup = self.popups.last().cloned();
        events.push(GameplayEvent::Sound(GameplaySound::Ruin));
        events.push(GameplayEvent::RuinResolved(outcome));
        if let Some(popup) = popup {
            events.push(GameplayEvent::Popup(popup));
        }
        events
    }

    /// Queues a ruin popup.
    fn queue_ruin_popup(&mut self, title: &str, body: &str) {
        self.popups.push(GameplayPopup {
            title: title.to_owned(),
            body: body.to_owned(),
            kind: PopupKind::Ruin,
        });
    }

    /// Resolves empire contacts near a position.
    fn resolve_contacts(&mut self, position: MapPoint) -> Vec<GameplayEvent> {
        let mut events = Vec::new();
        let owners: Vec<PlayerId> = self
            .units
            .iter()
            .filter(|unit| unit.alive && unit.owner != PlayerId::Human && unit.owner.is_empire())
            .filter(|unit| unit.position.tile_distance(position) <= 2)
            .map(|unit| unit.owner)
            .collect();
        for owner in owners {
            if let Some(contact) = self
                .contacts
                .iter_mut()
                .find(|contact| contact.owner == owner)
            {
                if !contact.encountered {
                    contact.encountered = true;
                    let popup = GameplayPopup {
                        title: "Empire Encountered".to_owned(),
                        body: "Scouts report contact with another empire.".to_owned(),
                        kind: PopupKind::Encounter,
                    };
                    self.popups.push(popup.clone());
                    events.push(GameplayEvent::EmpireEncountered(owner));
                    events.push(GameplayEvent::Popup(popup));
                }
            }
        }
        events
    }

    /// Advances barbarian AI one step.
    fn advance_barbarians(&mut self) -> Vec<GameplayEvent> {
        let mut events = Vec::new();
        let barbarian_ids: Vec<UnitId> = self
            .units
            .iter()
            .filter(|unit| unit.alive && unit.owner == PlayerId::Barbarians)
            .map(|unit| unit.id)
            .collect();
        for id in barbarian_ids {
            let Some(position) = self.unit_position(id) else {
                continue;
            };
            let target = self.nearest_human_unit(position);
            let next = target
                .and_then(|target_position| {
                    step_toward(position, target_position, self.width, self.height)
                })
                .or_else(|| self.random_neighbor(position));
            let Some(next) = next else {
                continue;
            };
            if !self.is_land_passable(next) {
                continue;
            }
            let Some(barb_index) = self.unit_index(id) else {
                continue;
            };
            if let Some(defender_index) = self.enemy_unit_index_at(next, PlayerId::Barbarians) {
                events.extend(self.resolve_combat(barb_index, defender_index));
            } else {
                self.units[barb_index].position = next;
            }
        }
        events
    }

    /// Advances simple non-barbarian AI settlers.
    fn advance_empires(&mut self) -> Vec<GameplayEvent> {
        let mut events = Vec::new();
        let ids: Vec<UnitId> = self
            .units
            .iter()
            .filter(|unit| unit.alive && matches!(unit.owner, PlayerId::Empire(_)))
            .map(|unit| unit.id)
            .collect();
        for id in ids {
            let Some(index) = self.unit_index(id) else {
                continue;
            };
            if self.units[index].kind == UnitKind::Settler && self.turn > 2 {
                let city_id = CityId(next_id_from_len(self.cities.len()));
                let owner = self.units[index].owner;
                let position = self.units[index].position;
                self.cities.push(City {
                    id: city_id,
                    owner,
                    position,
                    name: "Foreign City".to_owned(),
                    build_item: "Warrior".to_owned(),
                });
                self.units[index].alive = false;
                events.push(GameplayEvent::CityFounded(city_id));
            }
        }
        events
    }

    /// Resolves deterministic combat between two unit indexes.
    pub(super) fn resolve_combat(
        &mut self,
        attacker_index: usize,
        defender_index: usize,
    ) -> Vec<GameplayEvent> {
        let mut events = Vec::new();
        if attacker_index >= self.units.len() || defender_index >= self.units.len() {
            return events;
        }
        let attacker_id = self.units[attacker_index].id;
        let defender_id = self.units[defender_index].id;
        let attacker_strength = self.units[attacker_index].kind.strength();
        let defender_strength = self.units[defender_index].kind.strength();
        let attacker_wins = attacker_strength >= defender_strength
            && self.units[attacker_index].kind != UnitKind::Settler;
        if attacker_wins {
            self.units[defender_index].alive = false;
            self.units[attacker_index].position = self.units[defender_index].position;
            events.push(GameplayEvent::UnitKilled(defender_id));
        } else {
            self.units[attacker_index].alive = false;
            events.push(GameplayEvent::UnitKilled(attacker_id));
        }
        let popup = GameplayPopup {
            title: "Combat".to_owned(),
            body: "A unit has died in combat.".to_owned(),
            kind: PopupKind::Combat,
        };
        self.popups.push(popup.clone());
        events.push(GameplayEvent::Sound(GameplaySound::Combat));
        events.push(GameplayEvent::Sound(GameplaySound::UnitDeath));
        events.push(GameplayEvent::Popup(popup));
        events
    }

    /// Recomputes human fog of war.
    fn refresh_fog(&mut self) {
        for tile in &mut self.tiles {
            tile.fog = match tile.fog {
                FogState::Visible | FogState::Explored => FogState::Explored,
                FogState::Hidden => FogState::Hidden,
            };
        }
        let unit_positions: Vec<(MapPoint, i16)> = self
            .units
            .iter()
            .filter(|unit| unit.alive && unit.owner == PlayerId::Human)
            .map(|unit| (unit.position, unit.vision_radius))
            .collect();
        for (position, radius) in unit_positions {
            self.reveal_radius(position, radius);
        }
        let city_positions: Vec<MapPoint> = self
            .cities
            .iter()
            .filter(|city| city.owner == PlayerId::Human)
            .map(|city| city.position)
            .collect();
        for position in city_positions {
            self.reveal_radius(position, CITY_VISION_RADIUS);
        }
    }

    /// Reveals a radius around a point.
    fn reveal_radius(&mut self, center: MapPoint, radius: i16) {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx.unsigned_abs().max(dy.unsigned_abs()) > radius.unsigned_abs() {
                    continue;
                }
                if let Some(point) = center.offset(dx, dy, self.width, self.height) {
                    if let Some(tile) = self.tile_mut(point) {
                        tile.fog = FogState::Visible;
                    }
                }
            }
        }
    }

    /// Returns a row-major index for a point.
    fn tile_index(&self, point: MapPoint) -> Option<usize> {
        if !self.in_bounds(point) {
            return None;
        }
        usize::from(point.y)
            .checked_mul(usize::from(self.width))?
            .checked_add(usize::from(point.x))
    }

    /// Returns whether a land unit may enter a point.
    fn is_land_passable(&self, point: MapPoint) -> bool {
        self.tile(point)
            .is_some_and(|tile| tile.terrain.passable_by_land())
    }

    /// Returns the first human unit id.
    fn first_human_unit_id(&self) -> Option<UnitId> {
        self.units
            .iter()
            .find(|unit| unit.owner == PlayerId::Human && unit.alive)
            .map(|unit| unit.id)
    }

    /// Returns first awake human unit id.
    fn first_awake_human_unit_id(&self) -> Option<UnitId> {
        self.units
            .iter()
            .find(|unit| {
                unit.owner == PlayerId::Human
                    && unit.alive
                    && !matches!(unit.order, UnitOrder::Sentinel)
            })
            .map(|unit| unit.id)
            .or_else(|| self.first_human_unit_id())
    }

    /// Returns an alive unit index by id.
    fn unit_index(&self, id: UnitId) -> Option<usize> {
        self.units
            .iter()
            .position(|unit| unit.id == id && unit.alive)
    }

    /// Returns whether a unit id is alive.
    fn unit_alive(&self, id: UnitId) -> bool {
        self.unit_index(id).is_some()
    }

    /// Returns a unit position by id.
    fn unit_position(&self, id: UnitId) -> Option<MapPoint> {
        self.unit_index(id).map(|index| self.units[index].position)
    }

    /// Resets one unit movement.
    fn reset_unit_movement(&mut self, id: UnitId) {
        if let Some(index) = self.unit_index(id) {
            self.units[index].movement_points = self.units[index].max_movement_points;
        }
    }

    /// Sets one unit order.
    fn set_unit_order(&mut self, id: UnitId, order: UnitOrder) {
        if let Some(index) = self.unit_index(id) {
            self.units[index].order = order;
        }
    }

    /// Returns next path step for a unit.
    fn next_step_for_unit(&self, id: UnitId) -> Option<MapPoint> {
        let unit = self.units.get(self.unit_index(id)?)?;
        match unit.order.clone() {
            UnitOrder::MoveTo { path } => path.first().copied(),
            UnitOrder::None | UnitOrder::Sentinel => None,
        }
    }

    /// Pops next path step for a unit.
    fn pop_unit_path_front(&mut self, id: UnitId) {
        if let Some(index) = self.unit_index(id) {
            let order = core::mem::replace(&mut self.units[index].order, UnitOrder::None);
            if let UnitOrder::MoveTo { mut path } = order {
                if !path.is_empty() {
                    path.remove(0);
                }
                if path.is_empty() {
                    self.units[index].order = UnitOrder::None;
                } else {
                    self.units[index].order = UnitOrder::MoveTo { path };
                }
            }
        }
    }

    /// Returns an enemy unit at a point.
    fn enemy_unit_index_at(&self, point: MapPoint, owner: PlayerId) -> Option<usize> {
        self.units.iter().position(|unit| {
            unit.alive && unit.position == point && owners_are_enemies(unit.owner, owner)
        })
    }

    /// Returns nearest human unit position.
    fn nearest_human_unit(&self, position: MapPoint) -> Option<MapPoint> {
        self.units
            .iter()
            .filter(|unit| unit.alive && unit.owner == PlayerId::Human)
            .min_by_key(|unit| unit.position.tile_distance(position))
            .map(|unit| unit.position)
    }

    /// Returns a deterministic neighboring point.
    fn random_neighbor(&mut self, position: MapPoint) -> Option<MapPoint> {
        self.seed = lcg_next(self.seed);
        let direction = u8::try_from(self.seed % 8).ok()?;
        let (dx, dy) = direction_delta(direction);
        position.offset(dx, dy, self.width, self.height)
    }

    /// Spawns one barbarian near a point.
    fn spawn_barbarian_near(&mut self, position: MapPoint) {
        let spawn = position
            .offset(1, 0, self.width, self.height)
            .filter(|point| self.is_land_passable(*point))
            .unwrap_or(position);
        let id = UnitId(next_id_from_len(self.units.len()).saturating_add(1000));
        self.units.push(new_unit(
            id,
            UnitKind::Barbarian,
            PlayerId::Barbarians,
            spawn,
        ));
    }

    /// Finds nearest passable point to a requested point.
    fn nearest_passable(&self, point: MapPoint) -> MapPoint {
        if self.is_land_passable(point) {
            return point;
        }
        for radius in 1_i16..8_i16 {
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    if let Some(candidate) = point.offset(dx, dy, self.width, self.height) {
                        if self.is_land_passable(candidate) {
                            return candidate;
                        }
                    }
                }
            }
        }
        MapPoint::new(0, 0)
    }
}
