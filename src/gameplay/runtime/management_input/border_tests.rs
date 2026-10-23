//! Real semantic Graphics activation must toggle overlays, never send a game order.
use super::super::{Camera,MovementInputState,handle_event};
use crate::gameplay::{model::GameState,ui_model::management::ManagerKind};
use super::tests::{key,ui};
use sdl2::keyboard::Keycode;
#[test]
fn nation_borders_graphics_source_controls_are_real_local_toggles_not_disabled_art() {
    let mut state=GameState::new(8_u16,8_u16,2_u8,1_u64);state.popups.clear();state.network=crate::gameplay::model::NetworkStatus::Pending;let units=state.units.clone();let mut ui=ui();let mut camera=Camera::for_test_identity();let mut movement=MovementInputState::default();ui.management.open(ManagerKind::Graphics,&state,None);
    for id in ["PoliticalBordersButton","CityInflenceButton"] {
        let widget=ui.management.widgets(&state).into_iter().find(|widget|widget.id==id).expect("source Graphics switch");assert!(widget.intent.is_some(),"{id} must use the original local display callback, not a fake server operation");assert!(widget.selected,"initial ON check state");assert_eq!(widget.node.field("image0"),Some("upbt11u1.tga"),"actual source checked artwork");assert!(widget.intent.expect("display intent").into_command().is_none(),"never a canonical order");ui.management.focused=Some(id.to_owned());handle_event(&key(Keycode::SPACE),&mut state,&mut ui,&mut camera,&mut movement,None);
        let off=ui.management.widgets(&state).into_iter().find(|candidate|candidate.id==id).expect("same source switch");assert!(!off.selected,"actual descriptor reflects OFF");assert_eq!(off.node.field("image0"),Some("upbt11u0.tga"),"unchecked source art");assert!(!off.label.contains("str_ldl_"),"source caption is localized");
    }
    assert!(!state.filters.political_borders&&!state.filters.city_influence,"actual semantic activation changes both independently");assert_eq!(state.units,units,"no gameplay execution");assert_eq!(state.network,crate::gameplay::model::NetworkStatus::Pending,"local display never acknowledges an outstanding order");
    handle_event(&key(Keycode::ESCAPE),&mut state,&mut ui,&mut camera,&mut movement,None);ui.management.open(ManagerKind::Graphics,&state,None);assert!(ui.management.widgets(&state).into_iter().filter(|widget|matches!(widget.id.as_str(),"PoliticalBordersButton"|"CityInflenceButton")).all(|widget|!widget.selected),"Close/reopen preserves actual session settings");
}
