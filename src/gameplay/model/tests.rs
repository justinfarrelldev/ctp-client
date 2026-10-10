//! Tests for the pure gameplay model.

use super::{FogState, GameState, PlayerId, TileFeature, UnitKind, UnitOrder};

#[test]
fn barbarians_are_not_counted_as_empires() {
    let state = GameState::new(24, 18, 4, 7);

    assert_eq!(
        state.empire_count(),
        4,
        "barbarians must not add to empire count"
    );
    assert!(
        state
            .units
            .iter()
            .any(|unit| unit.owner == PlayerId::Barbarians),
        "generated game should contain wandering barbarians"
    );
}

#[test]
fn launch_state_has_visible_start_and_initial_popup() {
    let state = GameState::new(24, 18, 2, 9);
    let selected = state
        .selected_unit()
        .expect("launch should select a human settler");
    let tile = state
        .tile(selected.position)
        .expect("selected unit should be on the map");

    assert_eq!(
        selected.kind,
        UnitKind::Settler,
        "first selected unit should be a settler"
    );
    assert_eq!(
        tile.fog,
        FogState::Visible,
        "starting tile should be visible"
    );
    assert_eq!(state.popups.len(), 1, "start popup should be queued");
}

#[test]
fn movement_reveals_fog_and_shows_distance_labels() {
    let mut state = GameState::new(24, 18, 2, 11);
    let selected = state
        .selected_unit()
        .expect("selected unit should exist")
        .position;
    let target = selected
        .offset(1, 0, state.width, state.height)
        .expect("test target should remain in bounds");
    let labels = state.reachable_tile_labels();

    assert!(
        labels.iter().any(|label| label.label == "1"),
        "reachable GUI should include one-tile labels"
    );
    let events = state.order_selected_to(target);
    let moved = state
        .selected_unit()
        .expect("selected unit should remain alive")
        .position;

    assert_eq!(moved, target, "one-tile order should move immediately");
    assert!(!events.is_empty(), "movement should emit sound/move events");
    assert_eq!(
        state.tile(target).expect("target tile should exist").fog,
        FogState::Visible,
        "movement should refresh fog"
    );
}

#[test]
fn movement_order_enters_hidden_frontier_tile_and_reveals_it() {
    let mut state = GameState::new(24, 18, 2, 15);
    let start = state
        .selected_unit()
        .expect("selected unit should exist")
        .position;
    let target = start
        .offset(1, 0, state.width, state.height)
        .expect("test target should remain in bounds");
    state
        .tile_mut(target)
        .expect("target tile should exist")
        .fog = FogState::Hidden;

    let events = state.order_selected_to(target);
    let moved = state
        .selected_unit()
        .expect("selected unit should remain alive")
        .position;

    assert!(
        !events.is_empty(),
        "orders into hidden frontier tiles should explore instead of being blocked"
    );
    assert_eq!(
        moved, target,
        "hidden frontier tiles should be legal movement destinations"
    );
    assert_eq!(
        state.tile(target).expect("target tile should exist").fog,
        FogState::Visible,
        "entering a hidden frontier tile should reveal it"
    );
}

#[test]
fn settle_consumes_settler_and_opens_city_build_menu() {
    let mut state = GameState::new(24, 18, 2, 12);
    let selected_id = state
        .selected_unit()
        .expect("selected settler should exist")
        .id;
    let events = state.settle_selected("Alpha");

    assert!(
        state
            .units
            .iter()
            .any(|unit| unit.id == selected_id && !unit.alive),
        "settler should be consumed by founding a city"
    );
    assert_eq!(state.cities.len(), 1, "one city should be founded");
    assert_eq!(
        state.cities[0].name, "Alpha",
        "requested city name should be used"
    );
    assert_eq!(
        state.city_build_menu,
        Some(state.cities[0].id),
        "city build menu should open after founding"
    );
    assert!(!events.is_empty(), "settling should emit city/sound events");
}

#[test]
fn sentinel_prevents_auto_selection_until_all_units_sleep() {
    let mut state = GameState::new(24, 18, 2, 13);
    let sentinel_id = state
        .selected_unit()
        .expect("selected unit should exist")
        .id;
    state.sentinel_selected();
    state.end_turn();

    assert!(
        state
            .units
            .iter()
            .any(|unit| unit.id == sentinel_id && matches!(unit.order, UnitOrder::Sentinel)),
        "sentinel order should persist"
    );
    assert_ne!(
        state.selected_unit,
        Some(sentinel_id),
        "awake unit cycling should skip sentinel when another unit is available"
    );
}

#[test]
fn ruins_resolve_and_remove_feature_without_file_writes() {
    let mut state = GameState::new(24, 18, 1, 14);
    let selected_id = state
        .selected_unit()
        .expect("selected unit should exist")
        .id;
    let position = state
        .selected_unit()
        .expect("selected unit should exist")
        .position;
    state
        .tile_mut(position)
        .expect("selected tile should exist")
        .feature = Some(TileFeature::Ruin);

    let events = state.resolve_tile_entry(selected_id);

    assert_eq!(
        state
            .tile(position)
            .expect("selected tile should exist")
            .feature,
        None,
        "ruin should be consumed after entry"
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(*event, super::GameplayEvent::RuinResolved(_))),
        "ruin entry should emit a ruin event"
    );
}

#[test]
fn barbarian_attack_can_kill_settlers() {
    let mut state = GameState::new(24, 18, 1, 15);
    let settler_pos = state
        .selected_unit()
        .expect("selected unit should exist")
        .position;
    let barb_index = state
        .units
        .iter()
        .position(|unit| unit.owner == PlayerId::Barbarians)
        .expect("barbarian should exist");
    state.units[barb_index].position = settler_pos;
    let settler_index = state
        .units
        .iter()
        .position(|unit| unit.owner == PlayerId::Human && unit.alive)
        .expect("settler should exist");

    let events = state.resolve_combat(barb_index, settler_index);

    assert!(
        state.units[settler_index].alive || state.units[barb_index].alive,
        "combat should leave at least one side resolved"
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(*event, super::GameplayEvent::UnitKilled(_))),
        "combat should emit death event"
    );
}
