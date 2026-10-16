//! Source/screenshot contracts independent of the accidental reference map pan.

use super::{
    DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH, click_in_non_map_ui, end_turn_button_rect,
};

#[test]
fn gameplay_reference_frame_is_original_800_by_600() {
    assert_eq!(
        (DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT),
        (800_u32, 600_u32),
        "640×480 reference is an 80% scaled 800×600 game, not 1024×768"
    );
    let end = end_turn_button_rect();
    assert_eq!(
        (end.x, end.y, end.width, end.height),
        (762_i32, 476_i32, 38_u32, 86_u32),
        "AdjustToScreen anchors the entire 601×184 panel bottom-right"
    );
}

#[test]
fn hidden_unit_order_buttons_cannot_be_clicked_through_another_tab() {
    let mut state = crate::gameplay::model::GameState::new(24_u16, 18_u16, 5_u8, 42_u64);
    state.popups.clear();
    let settle =
        super::controls::panel_controls(&state, crate::gameplay::ui_model::ControlPanelTab::Unit)
            .into_iter()
            .find(|control| control.label == "Settle")
            .expect("settle descriptor exists");
    let mut ui = crate::gameplay::ui_model::InGameUiModel {
        active_tab: crate::gameplay::ui_model::ControlPanelTab::Domestic,
        ..crate::gameplay::ui_model::InGameUiModel::default()
    };
    let mut camera = super::Camera::centered_on_selected(&state);
    let mut movement = super::MovementInputState::default();
    let outcome = super::handle_left_click(
        settle.rect.x.saturating_add(8_i32),
        settle.rect.y.saturating_add(8_i32),
        &mut state,
        &mut ui,
        &mut camera,
        &mut movement,
        None,
    );
    assert_eq!(outcome, None, "hidden control cannot close the game");
    assert!(
        state.cities.is_empty(),
        "Empire-tab clicks must never activate the hidden Settle order"
    );
}

#[test]
fn map_input_is_not_blocked_by_old_full_width_bottom_and_right_panels() {
    assert!(
        !click_in_non_map_ui(780_i32, 100_i32),
        "upper right is map, not the old debug radar panel"
    );
    assert!(
        !click_in_non_map_ui(100_i32, 280_i32),
        "space above the actual radar remains map"
    );
    assert!(
        click_in_non_map_ui(100_i32, 450_i32),
        "bottom-left radar consumes map input"
    );
    assert!(
        click_in_non_map_ui(600_i32, 510_i32),
        "unit/order panel consumes map input"
    );
    assert!(
        click_in_non_map_ui(60_i32, 12_i32),
        "top menus consume map input"
    );
}
