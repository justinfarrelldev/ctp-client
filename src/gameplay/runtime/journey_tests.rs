//! Retained destinations/unknown frontier attempts never fabricate geography.
use super::{journeys,animation::ActorAnimations,Camera};
use crate::gameplay::{model::{GameState,MapPoint},server::ServerProjection};
use serde_json::json;

#[test]
#[expect(clippy::default_numeric_fallback, reason = "JSON fixture literals represent server wire integers")]
fn retained_destination_uses_shape_intent_and_public_coordinates_even_when_exhausted() {
    let mut viewer:ServerProjection=serde_json::from_value(json!({"revision":5,"turn":3,"contract":{"version":2,"features":{"retained_movement_orders":true}},"world":{"width":20,"height":12,"min_x":0,"min_y":0},"tiles":[{"id":"origin","x":1,"y":1,"terrain":"plains"}],"units":[{"id":"u","owner_id":"human","tile_id":"origin","movement":0}],"action_intents":[{"route":"set-movement-order","entity_id":"u","fields":["unit_id","target_x","target_y"],"payload":{"unit_id":"u"}}]})).expect("wire fixture");
    let state=GameState::from_server_projection(&viewer,1,1);
    let unit=state.units[0].id;
    let request=journeys::request(&state,unit,&[MapPoint::new(10,8)]).expect("retained destination");
    assert_eq!(request.0,"set-movement-order","not a guessed multi-turn path");
    assert_eq!(request.1,json!({"unit_id":"u","target_x":10_u16,"target_y":8_u16}),"no hidden tile IDs or outcomes");
    viewer.interface.action_intents.clear();
    let waiting=GameState::from_server_projection(&viewer,1,1);
    assert!(journeys::request(&waiting,unit,&[MapPoint::new(10,8)]).is_none(),"feature alone cannot authorize input while waiting");
}

#[test]
fn provisional_motion_and_rollback_never_mutate_canonical_units() {
    let state=GameState::new(20,12,1,1);
    let unit=state.selected_unit().expect("starter").clone();
    let target=unit.position.offset(1,0,state.width,state.height).expect("adjacent");
    let camera=Camera::centered_on_selected(&state);
    let mut actors=ActorAnimations::default();
    actors.preview(&unit,target);
    actors.begin_frame(0);
    actors.begin_frame(200);
    assert_ne!(actors.pose(&unit,&camera,500).position,camera.tile_to_screen(unit.position),"optimistic pixels before confirmation");
    assert_eq!(state.selected_unit(),Some(&unit),"authority remains at origin");
    actors.rollback();
    assert_eq!(actors.pose(&unit,&camera,500).position,camera.tile_to_screen(unit.position),"failed/queued attempt returns to authority");
}
