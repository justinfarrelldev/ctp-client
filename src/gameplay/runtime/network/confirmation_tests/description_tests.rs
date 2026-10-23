//! Complete released queue-first/blank-pane selection workflow through native input.
use super::{Harness,buildable_fixture,begin_row_press,manager_event,key};
use sdl2::keyboard::Keycode;
use crate::gameplay::ui_model::management::ManagerIntent;
use sdl2::{event::Event,mouse::MouseButton};
use std::sync::mpsc::TryRecvError;

/// Exact source Library article; no guessed first/current queue position.
fn article(harness:&Harness)->Option<String>{match harness.ui.management.activate_id("LibraryButton",&harness.state){Some(ManagerIntent::Article(id))=>Some(id),_=>None}}
/* Released ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::{ShowSelectedInfo,
 * ListCallback,QueueListCallback,Library}, ui/aui_common/aui_listbox.cpp::
 * {GetSelectedItem,MouseLGrabInside,DeselectItem}, ui/aui_ctp2/ctp2_listbox.cpp::Draw
 * and editqueue.ldl::QueueGroup.QueueList/ItemDescription/LibraryButton. Focus
 * neither clears selection nor changes source queue-first article priority.
 */
#[test]
fn build_description_source_queue_wins_both_explicit_selections_until_empty_queue_pane_deselects_only_that_role(){
    let mut harness=buildable_fixture();let canonical=harness.state.viewer.clone();
    begin_row_press(&mut harness,"QueueGroup.QueueList/c2:head");begin_row_press(&mut harness,"ItemsBox.UnitsList/UNIT_SETTLER");
    assert_eq!(article(&harness).as_deref(),Some("UNIT_HOPLITE"),"released queue-first ShowSelectedInfo, not last-interacted focus");
    let view=harness.ui.management.view.as_ref().expect("view");assert_eq!(view.selections.get("queue").map(String::as_str),Some("c2:head"),"queue identity retained");assert_eq!(view.selections.get("buildable").map(String::as_str),Some("UNIT_SETTLER"),"independent explicit left choice retained");
    let blank=harness.ui.management.widgets(&harness.state).into_iter().find(|widget|widget.id=="QueueGroup.QueueList.EmptyPane").expect("source non-force-select blank queue pane").rect;
    manager_event(&mut harness,&Event::MouseButtonDown {timestamp:0_u32,window_id:0_u32,which:0_u32,mouse_btn:MouseButton::Left,clicks:1_u8,x:blank.x.saturating_add(1_i32),y:blank.y.saturating_add(1_i32)});
    assert_eq!(article(&harness).as_deref(),Some("UNIT_SETTLER"),"explicit blank-pane deselection reveals known left content, not a neighbor");
    assert!(harness.ui.management.view.as_ref().is_some_and(|current|!current.selections.contains_key("queue")&&current.selections.get("buildable").is_some_and(|id|id=="UNIT_SETTLER")),"only queue role deselected");
    assert_eq!(harness.ui.management.focused.as_deref(),Some("QueueGroup.QueueList"),"source blank click gives the actual queue keyboard focus");
    manager_event(&mut harness,&key(Keycode::DOWN));assert_eq!(article(&harness).as_deref(),Some("UNIT_HOPLITE"),"explicit Down after deselection selects source first queue row, not a background control");
    assert_eq!(harness.state.viewer,canonical,"selection only, no canonical queue alteration");assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"no edit/cancel/map/turn request");
}

#[test]
fn build_description_source_empty_queue_pane_has_exact_semantic_deselect_not_header_frame_or_another_row(){
    let mut harness=buildable_fixture();begin_row_press(&mut harness,"QueueGroup.QueueList/c2:head");
    let widget=harness.ui.management.widgets(&harness.state).into_iter().find(|widget|widget.id=="QueueGroup.QueueList.EmptyPane").expect("source MouseLGrabInside empty area must deselect non-force-selected queue");
    let captured=widget.intent.expect("exact presentation deselect intent");begin_row_press(&mut harness,"QueueGroup.QueueList/c2:future");
    assert!(!harness.ui.management.apply_local(captured),"stale blank-pane descriptor cannot deselect a newer explicitly selected identity");
    assert_eq!(harness.ui.management.view.as_ref().and_then(|view|view.selections.get("queue")).map(String::as_str),Some("c2:future"),"not rebased");
}

#[test]
fn build_description_source_custom_queue_turns_are_blank_not_unknown_city_forecast_and_local_article_resolves(){
    let mut harness=buildable_fixture();harness.ui.management.focused=Some("NormalModeButtons.CustomButton".to_owned());manager_event(&mut harness,&key(Keycode::RETURN));
    begin_row_press(&mut harness,"ItemsBox.UnitsList/UNIT_SETTLER");harness.ui.management.focused=Some("ItemsBox.AddButton".to_owned());manager_event(&mut harness,&key(Keycode::RETURN));
    let entry=harness.ui.management.saved_queues.custom.draft.as_ref().expect("draft").entries[0_usize].id.clone();begin_row_press(&mut harness,&format!("QueueGroup.QueueList/{entry}"));
    let widgets=harness.ui.management.widgets(&harness.state);let turns=widgets.iter().find(|widget|widget.id.starts_with(&format!("QueueGroup.QueueList/{entry}/"))&&widget.node.name=="Turns").expect("original custom Turns control");
    assert!(turns.label.is_empty(),"released UpdateQueueList explicitly blanks custom Turns, no fabricated city ETA or unknown dash");assert_eq!(article(&harness).as_deref(),Some("UNIT_SETTLER"),"owned typed local entry resolves content; original Custom inner-info shadow bug is not ported");
}
