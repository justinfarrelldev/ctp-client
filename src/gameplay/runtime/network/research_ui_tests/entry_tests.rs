//! Real panel/shortcut entry to research, not a direct manager-open shortcut in tests.
use super::{Harness,fixture,key};
use crate::gameplay::{
    model::{CityId,NetworkStatus},
    runtime::{Camera,authority::state_from_authority_projection,handle_event,management_input,sync},
    server::ServerProjection,
    ui_model::{ControlPanelTab,management::ManagerKind,operations::OperationDialog},
};
use super::super::Outcome;
use sdl2::{event::Event,keyboard::{Keycode,Mod},mouse::MouseButton};
use serde_json::json;
use std::sync::mpsc::TryRecvError;

/// Exact current owned city is deliberately not the first; fresh research has no target.
fn entry_fixture(cities:bool,active:bool)->Harness {
    let mut harness=fixture();
    let mut viewer:ServerProjection=serde_json::from_value(json!({
        "revision":7_i64,"turn":1_u32,"contract":{"version":2_u32},
        "research":{"target_id":"","progress":0_i64,"advances":[],"available":["ADVANCE_AGRICULTURE","ADVANCE_TOOLMAKING"]},
        "rules":{"advances":[{"id":"ADVANCE_AGRICULTURE","cost":10_i64},{"id":"ADVANCE_TOOLMAKING","cost":20_i64}]},
        "legal_actions":[{"route":"pass","payload":{}}]
    })).expect("received research contract");
    if cities {
        viewer.tiles=serde_json::from_value(json!([{"id":"a","x":1_u16,"y":1_u16,"terrain":"plains"},{"id":"b","x":2_u16,"y":1_u16,"terrain":"plains"}])).expect("observed city tiles");
        viewer.cities=serde_json::from_value(json!([{"id":"c1","owner_id":"account-7","tile_id":"a","name":"City 1","queue":[]},{"id":"c2","owner_id":"account-7","tile_id":"b","name":"City 2","queue":[]}])).expect("owned cities");
        harness.ui.selected_city=Some(CityId::from_server_id("c2"));
    }
    if active {"ADVANCE_TOOLMAKING".clone_into(&mut viewer.interface.research.as_mut().expect("research").target_id);}
    harness.state=state_from_authority_projection(&harness.authority,&viewer);harness.state.popups.clear();
    harness.authority.session.projection=Some(viewer);harness.ui.active_tab=ControlPanelTab::Domestic;harness
}
/// Native runtime dispatcher, including modal owners and normal gameplay hit testing.
fn dispatch(harness:&mut Harness,event:&Event) {
    let mut camera=Camera::centered_on_selected(&harness.state);
    assert!(handle_event(event,&mut harness.state,&mut harness.ui,&mut camera,&mut harness.movement,Some(&mut harness.authority)).is_none(),"navigation cannot quit or become a gameplay order");
}
/// Resolve the real current semantic panel control before generating its native click.
fn panel_click(harness:&mut Harness,id:&str) {
    let rect=harness.ui.management.panel_widgets(&harness.state,&harness.ui).into_iter().find(|widget|widget.id==id).expect("original panel control").rect;
    dispatch(harness,&Event::MouseButtonDown {timestamp:0_u32,window_id:0_u32,which:0_u32,mouse_btn:MouseButton::Left,clicks:1_u8,x:rect.x.saturating_add(2_i32),y:rect.y.saturating_add(2_i32)});
}
/// Focus the exact visible source descriptor and use the full native event chain.
fn activate(harness:&mut Harness,id:&str) {
    assert!(harness.ui.management.widgets(&harness.state).iter().any(|widget|widget.id==id&&widget.intent.is_some()),"explicit current source control {id}");
    harness.ui.management.focused=Some(id.to_owned());dispatch(harness,&key(Keycode::RETURN));
}
/// Source science shortcut plus its existing client F6 alias.
fn shortcut(code:Keycode,modifiers:Mod)->Event {
    Event::KeyDown {timestamp:0_u32,window_id:0_u32,keycode:Some(code),scancode:None,keymod:modifiers,repeat:false}
}
/* Released ui/interface/DomesticControlPanel.cpp::{constructor,
 * EditResearchButtonActionCallback,ClearScience} wires portrait AND radial to
 * sci_advancescreen_displayMyWindow even without current research or a city.
 * CityControlPanel.cpp::EditBuildQueueButtonActionCallback is a different route.
 * controlpanel.ldl::{DomesticTab.TabPanel.AdvanceProgress,CityTab.TabPanel.BuildProgress}
 * and ctp_template.ldl::CTP2_PROGRESS_DISPLAY own the unchanged native controls.
 */
#[test]
fn research_entry_panel_portrait_and_radial_open_actual_chooser_without_requiring_city_or_target() {
    for cities in [false,true] {for active in [false,true] {for id in ["AdvanceProgress.IconBorder.IconButton","AdvanceProgress.IconBorder.IconButton.RadialButton"] {
        let mut harness=entry_fixture(cities,active);let before=harness.state.clone();let selected_city=harness.ui.selected_city;
        panel_click(&mut harness,id);
        let view=harness.ui.management.view.as_ref().expect("actual panel entry opens research chooser");
        assert_eq!(view.kind,ManagerKind::Research,"Domestic research is not city Build or just the Science manager");
        assert_eq!(view.selections.get("advance").map(String::as_str),active.then_some("ADVANCE_TOOLMAKING"),"no target never chooses the first available advance");
        dispatch(&mut harness,&key(Keycode::ESCAPE));
        assert!(harness.ui.management.view.is_none(),"panel Cancel returns to the same gameplay panel");
        assert_eq!(harness.ui.active_tab,ControlPanelTab::Domestic,"same panel");assert_eq!(harness.ui.selected_city,selected_city,"no implicit first-city substitution");
        assert_eq!(harness.state,before,"open/cancel never researches or changes a city");
        assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"no hidden order");
    }}}
}
#[test]
fn research_entry_city_portrait_and_radial_still_open_exact_selected_city_build_manager() {
    for id in ["BuildProgress.IconBorder.IconButton","BuildProgress.IconBorder.IconButton.RadialButton"] {
        let mut harness=entry_fixture(true,false);harness.ui.active_tab=ControlPanelTab::City;let before=harness.state.viewer.clone();panel_click(&mut harness,id);
        let view=harness.ui.management.view.as_ref().expect("city Build entry");assert_eq!(view.kind,ManagerKind::Build,"city production is not research");assert_eq!(view.city_id.as_deref(),Some("c2"),"exact non-first owned city");assert_eq!(harness.state.viewer,before,"navigation only");
    }
}
#[test]
fn research_entry_panel_to_explicit_toolmaking_posts_once_and_only_received_execution_updates_progress() {
    let mut harness=entry_fixture(true,false);let before=harness.state.viewer.clone();
    panel_click(&mut harness,"AdvanceProgress.IconBorder.IconButton.RadialButton");
    activate(&mut harness,"Background.SciAdvanceList/ADVANCE_TOOLMAKING");activate(&mut harness,"Background.BackButton");
    let command=harness.jobs.as_ref().expect("jobs").try_recv().expect("one actual set-research enqueue");
    assert_eq!(command.path,"/game/44/set-research","not edit-production");
    assert_eq!(command.body["advance_id"],"ADVANCE_TOOLMAKING","exact non-first explicitly selected choice");
    assert_eq!(command.body["expected_revision"],7_i64,"current displayed revision");assert_eq!(command.body["session_id"],71_i64,"exact session");
    assert_eq!(command.body.as_object().map(serde_json::Map::len),Some(5_usize),"advance plus shared envelope only; no city, cost or inferred grant");
    assert!(command.body["command_id"].as_str().is_some_and(|id|!id.is_empty())&&command.body["idempotency_key"].as_str().is_some_and(|id|!id.is_empty()),"immutable operation identity");
    assert_eq!(harness.state.network,NetworkStatus::Pending,"enqueued, not executed");assert_eq!(harness.state.viewer,before,"no local research grant/target/progress change");assert!(harness.ui.management.view.is_none(),"accepted panel chooser returns to panel");
    let mut queued=harness.authority.session.projection.clone().expect("receipt");queued.revision=8_i64;"queued".clone_into(&mut queued.interface.planning_status);
    sync::apply(&mut harness.authority,&mut harness.state,&mut harness.ui,&mut harness.movement,Outcome::Receipt(Box::new(queued)));
    assert!(harness.state.viewer.as_ref().and_then(|viewer|viewer.interface.research.as_ref()).is_some_and(|research|research.target_id.is_empty()),"queued acknowledgement does not execute research");
    let mut received=harness.authority.session.projection.clone().expect("current projection");received.revision=9_i64;received.interface.planning_status.clear();let research=received.interface.research.as_mut().expect("research");"ADVANCE_TOOLMAKING".clone_into(&mut research.target_id);research.progress=3_i64;
    sync::apply(&mut harness.authority,&mut harness.state,&mut harness.ui,&mut harness.movement,Outcome::Snapshot(Box::new(received)));
    assert!(harness.ui.management.panel_widgets(&harness.state,&harness.ui).iter().any(|widget|widget.id=="AdvanceProgress.Title"&&widget.label=="Toolmaking"),"only received execution updates the actual panel title");
    assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"no duplicate or production command");
}
#[test]
fn research_entry_science_shortcuts_escape_build_and_pending_auto_build_cannot_hijack_explicit_navigation() {
    for event in [shortcut(Keycode::F6,Mod::NOMOD),shortcut(Keycode::NUM_6,Mod::LCTRLMOD)] {
        let mut harness=entry_fixture(true,false);harness.ui.active_tab=ControlPanelTab::City;panel_click(&mut harness,"BuildProgress.IconBorder.IconButton");
        harness.state.city_build_menu=Some(CityId::from_server_id("c2"));
        dispatch(&mut harness,&event);management_input::open_pending_build(&mut harness.state,&mut harness.ui);
        assert_eq!(harness.ui.management.view.as_ref().map(|view|view.kind),Some(ManagerKind::Science),"real global science navigation replaces Build and consumes only stale presentation");
        assert!(harness.ui.management.parents.is_empty(),"source close_AllScreens does not create a stale Build parent");
        activate(&mut harness,"ChangeResearch");assert_eq!(harness.ui.management.view.as_ref().map(|view|view.kind),Some(ManagerKind::Research),"Science's real chooser entry still works");
        dispatch(&mut harness,&key(Keycode::ESCAPE));assert_eq!(harness.ui.management.view.as_ref().map(|view|view.kind),Some(ManagerKind::Science),"Cancel returns to Science, not Build");
        assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"shortcuts/cancel never submit");
    }
}
#[test]
fn research_entry_confirmation_modal_and_modified_or_repeat_shortcuts_cannot_be_bypassed() {
    let mut harness=entry_fixture(true,false);harness.ui.active_tab=ControlPanelTab::City;panel_click(&mut harness,"BuildProgress.IconBorder.IconButton");
    for event in [shortcut(Keycode::F6,Mod::LALTMOD),shortcut(Keycode::NUM_6,Mod::NOMOD),Event::KeyDown {timestamp:0_u32,window_id:0_u32,keycode:Some(Keycode::F6),scancode:None,keymod:Mod::NOMOD,repeat:true}] {
        dispatch(&mut harness,&event);assert_eq!(harness.ui.management.view.as_ref().map(|view|view.kind),Some(ManagerKind::Build),"only a real unmodified F6 or Ctrl6 activates science");
    }
    let action=harness.state.viewer.as_ref().expect("viewer").interface.legal_actions[0_usize].clone();
    harness.ui.operation=OperationDialog::actions(&harness.state,"Confirm an order",core::iter::once(action));assert!(harness.ui.operation.is_some(),"protected modal");
    dispatch(&mut harness,&key(Keycode::F6));
    assert!(harness.ui.operation.is_some(),"shortcut neither approves nor dismisses a confirmation");assert_eq!(harness.ui.management.view.as_ref().map(|view|view.kind),Some(ManagerKind::Build),"underlying view not replaced");assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"no implicit approval");
}
#[test]
fn research_entry_explicit_choice_with_stale_session_pending_or_missing_candidate_never_submits() {
    for mode in 0_u8..4_u8 {
        let mut harness=entry_fixture(false,false);panel_click(&mut harness,"AdvanceProgress.IconBorder.IconButton");activate(&mut harness,"Background.SciAdvanceList/ADVANCE_TOOLMAKING");
        match mode {0_u8=>harness.authority.session.projection.as_mut().expect("session").revision=8_i64,1_u8=>harness.state.network=NetworkStatus::Pending,2_u8=>harness.state.viewer.as_mut().expect("viewer").interface.research.as_mut().expect("research").available.clear(),_=>{}}
        harness.ui.management.focused=Some("Background.BackButton".to_owned());
        if mode==3_u8 {
            let mut camera=Camera::centered_on_selected(&harness.state);
            assert!(handle_event(&key(Keycode::RETURN),&mut harness.state,&mut harness.ui,&mut camera,&mut harness.movement,None).is_none(),"absent session consumes only modal input");
        }else{dispatch(&mut harness,&key(Keycode::RETURN));}
        assert_eq!(harness.ui.management.view.as_ref().map(|view|view.kind),Some(ManagerKind::Research),"failed/unavailable confirmation keeps the explicit choice dialog");assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"no stale/rebased/unauthorized command");
    }
}
