//! Accessible gameplay controls shared by mouse dispatch and automation.
//!
//! Stable labels/IDs describe the same LDL rectangles used for rendering.
//! Selecting tabs or units is local view state; game orders still travel through
//! the authoritative command dispatcher. Unsupported orders never mutate state.

#![expect(
    clippy::default_numeric_fallback,
    reason = "Original order slots and LDL offsets are fixed i32 coordinates"
)]

use super::layout::{end_turn_button_rect, panel_rect};

use crate::gameplay::model::{GameState, UnitKind};
use crate::gameplay::ui_model::{ControlPanelTab, InGameAction, MenuRect};

/// Semantic intent produced by a labeled UI interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlIntent {
    /// A pre-existing authoritative order or local view action.
    Action(InGameAction),
    /// Switch visible panel without issuing a game command.
    Tab(ControlPanelTab),
    /// Activate the selected unit's original hover/click movement mode.
    ActivateUnit,
    /// Select the next living human unit.
    NextUnit,
    /// Preserve an original control whose server workflow is not yet integrated.
    Unavailable(&'static str),
}

/// One discoverable, labeled original-game control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameplayControl {
    /// Stable machine-readable identifier.
    pub id: &'static str,
    /// Human-readable accessible name.
    pub label: &'static str,
    /// Logical-frame rectangle, used by the actual pointer event path.
    pub rect: MenuRect,
    /// Semantic event dispatched by activation.
    pub intent: ControlIntent,
    /// Whether the currently selected unit can use this control.
    pub enabled: bool,
}

/// Original shortcut art, exact LDL offsets and accessible view actions.
pub(super) const SHORTCUTS: [(&str, i32, i32, &str, InGameAction); 8] = [
    (
        "upcb05u.rim",
        28,
        29,
        "City Manager",
        InGameAction::OpenCityManager,
    ),
    (
        "upcb06u.rim",
        68,
        13,
        "Empire Manager",
        InGameAction::OpenCivilizationStatus,
    ),
    (
        "upcb07u.rim",
        28,
        115,
        "Trade Summary",
        InGameAction::OutOfScope,
    ),
    (
        "upcb08u.rim",
        112,
        29,
        "Unit Manager",
        InGameAction::OpenUnitManager,
    ),
    (
        "upcb09u.rim",
        133,
        70,
        "Diplomacy",
        InGameAction::OpenDiplomacy,
    ),
    (
        "upcb10u.rim",
        112,
        115,
        "Science Manager",
        InGameAction::OpenScienceStatus,
    ),
    (
        "upcb11u.rim",
        13,
        70,
        "National Manager",
        InGameAction::OpenCivilizationStatus,
    ),
    (
        "upcb60u.rim",
        68,
        137,
        "Great Library",
        InGameAction::OpenGreatLibrary,
    ),
];
/// Original panel tab order and labels.
pub(super) const TABS: [(ControlPanelTab, &str); 5] = [
    (ControlPanelTab::Domestic, "Empire"),
    (ControlPanelTab::City, "City"),
    (ControlPanelTab::Unit, "Unit"),
    (ControlPanelTab::Messages, "Msgs"),
    (ControlPanelTab::TileImprovements, "Tile"),
];

/// Returns all fixed panel controls without touching SDL, files or a server.
#[must_use]
pub fn panel_controls(state: &GameState, active_tab: ControlPanelTab) -> Vec<GameplayControl> {
    let mut controls = Vec::new();
    controls.push(GameplayControl {
        id: "end-turn",
        label: "End Turn",
        rect: end_turn_button_rect(),
        intent: ControlIntent::Action(InGameAction::EndTurn),
        enabled: true,
    });
    for (_, x, y, label, action) in SHORTCUTS {
        controls.push(control(
            label,
            label,
            x,
            y,
            42_u32,
            42_u32,
            ControlIntent::Action(action),
            true,
        ));
    }
    for (index, (tab, label)) in TABS.into_iter().enumerate() {
        let x = 189_i32
            .saturating_add(i32::try_from(index).map_or(0_i32, |v| v.saturating_mul(65_i32)));
        controls.push(control(
            label,
            label,
            x,
            17_i32,
            65_u32,
            28_u32,
            ControlIntent::Tab(tab),
            true,
        ));
    }
    if active_tab != ControlPanelTab::Unit {
        return controls;
    }
    controls.push(control(
        "next-unit",
        "Next Unit",
        329_i32,
        50_i32,
        24_u32,
        25_u32,
        ControlIntent::NextUnit,
        state.selected_unit().is_some(),
    ));
    let settler = state
        .selected_unit()
        .is_some_and(|unit| unit.kind == UnitKind::Settler);
    for index in 0_i32..12_i32 {
        let (label, intent, enabled) = match index {
            0 => ("Move", ControlIntent::ActivateUnit, true),
            3 => ("Sleep", ControlIntent::Action(InGameAction::Sentinel), true),
            4 if settler => ("Settle", ControlIntent::Action(InGameAction::Settle), true),
            8 if settler => (
                "Add to City",
                ControlIntent::Unavailable("Add to City"),
                true,
            ),
            9 => (
                "Board Transport",
                ControlIntent::Unavailable("Board Transport"),
                true,
            ),
            11 => ("Disband", ControlIntent::Unavailable("Disband"), true),
            1 if !settler => ("Attack", ControlIntent::ActivateUnit, true),
            2 if !settler => (
                "Fortify",
                ControlIntent::Action(InGameAction::Sentinel),
                true,
            ),
            _ => (
                "Unavailable Order",
                ControlIntent::Unavailable("Unavailable Order"),
                false,
            ),
        };
        let rect = order_rect(index);
        controls.push(GameplayControl {
            id: label,
            label,
            rect,
            intent,
            enabled: enabled && state.selected_unit().is_some(),
        });
    }
    controls
}

/// Resolves a semantic interaction by accessible label, with no coordinate clicking.
#[must_use]
pub fn intent_for_label(
    state: &GameState,
    active_tab: ControlPanelTab,
    label: &str,
) -> Option<ControlIntent> {
    panel_controls(state, active_tab)
        .into_iter()
        .find(|control| {
            control.enabled && (control.label.eq_ignore_ascii_case(label) || control.id == label)
        })
        .map(|control| control.intent)
}

/// Finds the same semantic interaction under a real SDL logical pointer position.
pub(super) fn intent_at(
    state: &GameState,
    active_tab: ControlPanelTab,
    x: i32,
    y: i32,
) -> Option<ControlIntent> {
    panel_controls(state, active_tab)
        .into_iter()
        .find(|control| control.enabled && control.rect.contains(x, y))
        .map(|control| control.intent)
}

/// Applies both accessible and pointer intents through the same runtime dispatch.
pub(super) fn apply_intent(
    intent: ControlIntent,
    state: &mut GameState,
    ui: &mut crate::gameplay::ui_model::InGameUiModel,
    camera: &mut super::Camera,
    movement: &mut super::MovementInputState,
    authority: Option<&mut super::RuntimeAuthority>,
) -> Option<crate::gameplay::GameplayOutcome> {
    match intent {
        ControlIntent::Action(action) => {
            return super::apply_ui_action(action, state, ui, camera, movement, authority);
        }
        ControlIntent::Tab(tab) => {
            ui.active_tab = tab;
            ui.status_text.clear();
        }
        ControlIntent::ActivateUnit => {
            if let Some(position) = state.selected_unit().map(|unit| unit.position) {
                super::begin_or_activate_unit_drag(state, movement, position);
                movement.cancel_drag();
            }
        }
        ControlIntent::NextUnit => {
            let ids: Vec<_> = state
                .units
                .iter()
                .filter(|unit| unit.alive && unit.owner == crate::gameplay::model::PlayerId::Human)
                .map(|unit| unit.id)
                .collect();
            if !ids.is_empty() {
                let next = ids
                    .iter()
                    .position(|id| Some(*id) == state.selected_unit)
                    .map_or(0_usize, |index| {
                        index
                            .saturating_add(1_usize)
                            .checked_rem(ids.len())
                            .unwrap_or(0_usize)
                    });
                if state.select_unit(ids[next]) {
                    *camera = super::Camera::centered_on_selected(state);
                    *movement = super::MovementInputState::default();
                }
            }
        }
        ControlIntent::Unavailable(label) => {
            ui.status_text = format!("{label}: server workflow not yet integrated");
        }
    }
    None
}

/// Source 4×3 order-button geometry shared by painting and event dispatch.
pub(super) fn order_rect(index: i32) -> MenuRect {
    let x = 366_i32.saturating_add(index.rem_euclid(4_i32).saturating_mul(42_i32));
    let y = 57_i32.saturating_add(index.div_euclid(4_i32).saturating_mul(32_i32));
    let rect = panel_rect(x, y, 40_u32, 30_u32);
    MenuRect {
        x: rect.x(),
        y: rect.y(),
        width: rect.width(),
        height: rect.height(),
    }
}

/// Constructs one panel-relative descriptor with explicitly sized geometry.
#[expect(
    clippy::too_many_arguments,
    reason = "Flat source control descriptions are easier to audit against LDL coordinates"
)]
fn control(
    id: &'static str,
    label: &'static str,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    intent: ControlIntent,
    enabled: bool,
) -> GameplayControl {
    let rect = panel_rect(x, y, width, height);
    GameplayControl {
        id,
        label,
        rect: MenuRect {
            x: rect.x(),
            y: rect.y(),
            width: rect.width(),
            height: rect.height(),
        },
        intent,
        enabled,
    }
}

#[cfg(test)]
mod tests {
    use super::{ControlIntent, apply_intent, intent_at, intent_for_label, panel_controls};
    use crate::gameplay::model::GameState;
    use crate::gameplay::runtime::{Camera, MovementInputState};
    use crate::gameplay::ui_model::InGameUiModel;
    use crate::gameplay::ui_model::{ControlPanelTab, InGameAction};

    #[test]
    fn accessible_activation_and_real_pointer_dispatch_share_original_rectangles() {
        let state = GameState::new(24_u16, 18_u16, 5_u8, 42_u64);
        for label in ["End Turn", "Settle", "Move", "Great Library", "Unit"] {
            let descriptor = panel_controls(&state, ControlPanelTab::Unit)
                .into_iter()
                .find(|control| control.label == label)
                .expect("original control is discoverable");
            assert_eq!(
                intent_for_label(&state, ControlPanelTab::Unit, label),
                intent_at(
                    &state,
                    ControlPanelTab::Unit,
                    descriptor.rect.x.saturating_add(10_i32),
                    descriptor.rect.y.saturating_add(10_i32)
                ),
                "accessible and mouse paths must agree for {label}"
            );
        }
        assert_eq!(
            intent_for_label(&state, ControlPanelTab::Unit, "Settle"),
            Some(ControlIntent::Action(InGameAction::Settle)),
            "Settle routes through the same authoritative action as B"
        );
    }

    #[test]
    fn labeled_buttons_drive_selection_tabs_and_nonmutating_unsupported_orders() {
        let mut state = GameState::new(24_u16, 18_u16, 5_u8, 42_u64);
        state.popups.clear();
        let initial = state.selected_unit;
        let initial_units = state.units.clone();
        let mut ui = InGameUiModel::default();
        let mut camera = Camera::centered_on_selected(&state);
        let mut movement = MovementInputState::default();
        for label in [
            "Move",
            "Next Unit",
            "Next Unit",
            "Empire",
            "Unit",
            "Disband",
            "Great Library",
        ] {
            let intent = intent_for_label(&state, ui.active_tab, label)
                .expect("original labeled control enabled");
            assert_eq!(
                apply_intent(
                    intent,
                    &mut state,
                    &mut ui,
                    &mut camera,
                    &mut movement,
                    None
                ),
                None,
                "view interaction remains in game"
            );
            match label {
                "Move" => assert_eq!(
                    movement.active_unit(),
                    initial,
                    "move activates selected unit"
                ),
                "Next Unit" => assert_eq!(
                    movement.active_unit(),
                    None,
                    "cycling clears old movement input"
                ),
                "Empire" => assert_eq!(
                    ui.active_tab,
                    ControlPanelTab::Domestic,
                    "real tab switches"
                ),
                "Unit" => assert_eq!(
                    ui.active_tab,
                    ControlPanelTab::Unit,
                    "unit controls restored"
                ),
                "Disband" => assert!(
                    ui.status_text.contains("not yet integrated"),
                    "unsupported orders are explicit"
                ),
                "Great Library" => assert!(
                    ui.great_library.is_some(),
                    "shortcut reaches existing library model"
                ),
                _ => {}
            }
        }
        assert_eq!(
            state.selected_unit, initial,
            "next-unit cycling wraps to the first owned unit"
        );
        assert_eq!(
            state.units, initial_units,
            "view controls and unsupported Disband never mutate real units"
        );
        state.units.clear();
        state.selected_unit = None;
        for intent in [ControlIntent::NextUnit, ControlIntent::ActivateUnit] {
            assert_eq!(
                apply_intent(
                    intent,
                    &mut state,
                    &mut ui,
                    &mut camera,
                    &mut movement,
                    None
                ),
                None,
                "empty selection is safe"
            );
        }
    }

    #[test]
    fn disabled_order_slots_and_missing_selection_cannot_issue_commands() {
        let mut state = GameState::new(24_u16, 18_u16, 5_u8, 42_u64);
        assert_eq!(
            intent_for_label(&state, ControlPanelTab::Unit, "Unavailable Order"),
            None,
            "blank cells are inert"
        );
        state.selected_unit = None;
        assert_eq!(
            intent_for_label(&state, ControlPanelTab::Unit, "Settle"),
            None,
            "no selection cannot settle"
        );
        assert_eq!(
            intent_for_label(&state, ControlPanelTab::Unit, "Unknown"),
            None,
            "unknown labels never activate a neighbor"
        );
    }
}
