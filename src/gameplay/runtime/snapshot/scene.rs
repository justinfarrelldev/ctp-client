//! Deterministic visual fixtures, explicitly separate from authoritative games.

use crate::gameplay::model::{FogState, GameState, MapPoint, PlayerId, TerrainKind};
use crate::gameplay::ui_model::management::ManagerKind;

/// Scene to inspect with the real renderer; neither variant contacts a server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewScene {
    /// The existing application-generated preview map.
    Generated,
    /// Known land, connected river, shore, stacked settlers and explored terrain.
    /// This is diagnostic fixture data, never a substitute for a server projection.
    Terrain,
    /// Source-aligned expanded stack control, using synthetic colocated units.
    Stack,
    /// Explicitly opened friendly-unit context on the synthetic terrain fixture.
    Context,
    /// Original gold-framed terrain information on the synthetic fixture.
    TerrainInfo,
    /// One lower-panel body over an explicit viewer-safe fixture.
    Panel {
        /// Original lower-panel tab to inspect.
        tab: crate::gameplay::ui_model::ControlPanelTab,
        /// Explicit city/research data, or a genuinely empty viewer projection.
        populated: bool,
    },
    /// Source-layout manager over an explicitly synthetic background fixture.
    Manager(ManagerKind),
    /// Explicit two-city projection with its original detached selector open.
    CityChooser,
    /// Populated descriptions backed by explicit synthetic public projection data.
    PopulatedManager {
        /// Source manager root to inspect.
        kind: ManagerKind,
        /// Select the advance chart rather than prose for a Library fixture.
        chart: bool,
    },
    /// A single confirmed synthetic move with original cursor and injected clock.
    Motion {
        /// Clockwise screen direction, north=0 through northwest=7 (clamped).
        direction: u8,
        /// Elapsed quarters of one travel/animation cycle, 0–4 (clamped).
        quarter: u8,
    },
}

/// Creates a reproducible scene without reading or writing configuration files.
pub(in crate::gameplay::runtime) fn preview_state(scene: PreviewScene) -> GameState {
    if let PreviewScene::Panel { populated, .. } = scene {
        return super::manager_fixture::panel_state(populated);
    }
    if scene == PreviewScene::CityChooser {
        return super::manager_fixture::chooser_state();
    }
    if matches!(
        scene,
        PreviewScene::PopulatedManager {
            kind: ManagerKind::Empire,
            ..
        }
    ) {
        return super::manager_fixture::policy_state();
    }
    if matches!(
        scene,
        PreviewScene::PopulatedManager {
            kind: ManagerKind::Unit
                | ManagerKind::Trade
                | ManagerKind::National
                | ManagerKind::Diplomacy,
            ..
        }
    ) {
        return super::manager_fixture::starting_state();
    }
    if matches!(scene, PreviewScene::PopulatedManager { .. }) {
        return super::manager_fixture::state();
    }
    let mut state = GameState::new(64_u16, 48_u16, 5_u8, 42_u64);
    state.popups.clear();
    if scene == PreviewScene::Generated {
        return state;
    }
    for tile in &mut state.tiles {
        tile.terrain = TerrainKind::Plains;
        tile.art_base_type = Some(1_u8);
        tile.feature = None;
        tile.fog = FogState::Hidden;
        tile.river = false;
    }
    state.units.retain(|unit| unit.owner == PlayerId::Human);
    let center = MapPoint::new(30_u16, 22_u16);
    for unit in &mut state.units {
        unit.position = center;
        unit.movement_points = 100_i16;
    }
    for y in 21_u16..=24_u16 {
        for x in 29_u16..=32_u16 {
            let index = usize::from(y)
                .saturating_mul(usize::from(state.width))
                .saturating_add(usize::from(x));
            let tile = &mut state.tiles[index];
            tile.fog = if y == 21_u16 && x > 30_u16 {
                FogState::Explored
            } else {
                FogState::Visible
            };
            if x == 32_u16 || y == 24_u16 {
                tile.terrain = TerrainKind::Water;
                tile.art_base_type = Some(13_u8);
            } else if x == 31_u16 && y == 21_u16 {
                tile.terrain = TerrainKind::Mountain;
                tile.art_base_type = Some(8_u8);
            }
            tile.river = matches!(
                (x, y),
                (30_u16, 21_u16..=23_u16) | (31_u16 | 32_u16, 23_u16)
            );
        }
    }
    state.gold = 100_i32;
    state
}

/// Configures a deterministic accepted movement without invoking a network client.
/// This fixture is intentionally synthetic, not evidence about server movement.
pub(in crate::gameplay::runtime) fn preview_presentation(
    scene: PreviewScene,
    state: &mut GameState,
    textures: &super::super::textures::GameplayTextures<'_>,
    ui: &mut crate::gameplay::ui_model::InGameUiModel,
) -> (super::super::Camera, super::super::MovementInputState) {
    let mut camera = super::super::Camera::centered_on_selected(state);
    let mut input = super::super::MovementInputState::default();
    match scene {
        PreviewScene::Panel { tab, .. } => ui.active_tab = tab,
        PreviewScene::Stack => ui.stack_expanded = true,
        PreviewScene::Manager(kind) => {
            if kind == crate::gameplay::ui_model::management::ManagerKind::Library {
                // Explicit installed reference article, not invented game research.
                ui.management.library.navigate("ADVANCE_BALLISTICS");
            }
            ui.management.open(kind, state, None);
        }
        PreviewScene::PopulatedManager { kind, chart } => {
            ui.management.open(kind, state, None);
            if matches!(
                kind,
                ManagerKind::Unit
                    | ManagerKind::Trade
                    | ManagerKind::National
                    | ManagerKind::Diplomacy
            ) {
                ui.active_tab = crate::gameplay::ui_model::ControlPanelTab::TileImprovements;
                // The reference inspects a black viewport; observed units still
                // exist elsewhere, rather than hiding their authorized facts.
                camera.center_on(MapPoint::new(63, 47));
            }
            let control = if kind == ManagerKind::Build {
                Some("ItemsBox.UnitsList/UNIT_SETTLER")
            } else if kind == ManagerKind::Library && chart {
                Some("Tabs.TechTreeTab.TabButton")
            } else {
                None
            };
            if let Some(intent) = control.and_then(|id| ui.management.activate_id(id, state)) {
                ui.management.apply_local(intent);
            }
        }
        PreviewScene::CityChooser => {
            ui.management
                .open(ManagerKind::Build, state, Some("fixture-city"));
            if let Some(intent) = ui
                .management
                .activate_id("SingleCityChooser.Pulldown.autobutton", state)
            {
                ui.management.apply_local(intent);
            }
        }
        PreviewScene::Context => {
            if let Some(point) = state.selected_unit().map(|unit| unit.position) {
                ui.context =
                    crate::gameplay::ui_model::context::ContextView::new(state, point, 450, 300);
            }
        }
        PreviewScene::TerrainInfo => ui.map_info = state.selected_unit().map(|unit| unit.position),
        PreviewScene::Generated | PreviewScene::Terrain | PreviewScene::Motion { .. } => {}
    }
    if let PreviewScene::Motion { direction, quarter } = scene {
        state.units.truncate(1);
        let before = state.units.clone();
        if let Some(unit) = state.units.first_mut() {
            let deltas: [(i16, i16); 8] = [
                (-1, -1),
                (0, -1),
                (1, -1),
                (1, 0),
                (1, 1),
                (0, 1),
                (-1, 1),
                (-1, 0),
            ];
            let (dx, dy) = deltas[usize::from(direction.min(7))];
            if let Some(target) = unit.position.offset(dx, dy, state.width, state.height) {
                unit.position = target;
                let duration = textures.move_duration(unit.kind, 5, true);
                input.actors.confirm(&before, state, &[target]);
                input.actors.now_ms = duration
                    .saturating_mul(u64::from(quarter.min(4)))
                    .div_euclid(4);
                ui.pointer = Some((650_i32, 260_i32));
            }
        }
    }
    (camera, input)
}
