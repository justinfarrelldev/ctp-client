//! Source saved-load browser through fake storage, native controls and inert jobs.
use super::{Harness, begin_row_press, buildable_fixture, input, key, manager_event};
use crate::gameplay::{model::NetworkStatus, runtime::{MovementInputState, network::Outcome, operation_input, saved_queues::{Entry, QueueReader, service_with}, sync}, server::envelope::PreparedCommand, ui_model::management::saved};
use alloc::collections::BTreeMap;
use sdl2::keyboard::Keycode;
use serde_json::json;
use std::{path::{Path, PathBuf}, sync::mpsc::TryRecvError};

/// Fake local template source; no actual directory or file is touched.
struct Files {
    /// Explicit user-visible names and bytes, not canonical production state.
    bytes: BTreeMap<String, Vec<u8>>,
}
impl QueueReader for Files {
    fn list(&mut self, _directory: &Path) -> Result<Vec<Entry>, String> {Ok(self.bytes.keys().map(|name|Entry {name:name.clone(),regular:true}).collect())}
    fn read(&mut self, path: &Path, maximum: usize) -> Result<Vec<u8>, String> {
        if maximum != saved::codec::MAX_BYTES { return Err("Unbounded native load request".to_owned()); }
        self.bytes.get(path.file_name().and_then(|name|name.to_str()).unwrap_or("")).cloned().ok_or_else(||"fake missing file".to_owned())
    }
}
/// Opens actual source `LoadBox`, then services its explicit request with fake bytes.
fn open(harness: &mut Harness, text: &str) {
    harness.ui.management.focused=Some("ItemsBox.LoadSaveButton".to_owned());manager_event(harness,&key(Keycode::RETURN));
    assert!(harness.ui.operation.is_none(),"inline source browser, not raw custom input");
    let mut files=Files {bytes:BTreeMap::from([("00-other".to_owned(),b"U UNIT_WARRIOR".to_vec()),("chosen".to_owned(),text.as_bytes().to_vec())])};
    assert!(service_with(&mut harness.ui,&[("memory".to_owned(),PathBuf::from("/memory/queues"))],&mut files),"actual injected storage service");
    begin_row_press(harness,"LoadBox.QueuesList/memory:chosen");
}
/// Same source LoadButton/Query callback for every positive or negative journey.
fn question(harness: &mut Harness) {
    harness.ui.management.focused=Some("LoadBox.LoadButton".to_owned());manager_event(harness,&key(Keycode::RETURN));
    assert!(harness.ui.operation.is_some(),"exact current source QueryLoadQueue");
}
/// Source no-default affirmative focus: explicit Tab Cancel, Tab OK, Return.
fn approve(harness: &mut Harness) {input(harness,&key(Keycode::TAB));input(harness,&key(Keycode::TAB));input(harness,&key(Keycode::RETURN));}

/*
 * Released source: EditQueue.cpp::{LoadModeCallback,QueueFileList,
 * DisplayQueueContents,LoadCallback,LoadQueryCallback,ExitLoadMode} under
 * ctp2_source/ctp2_code/ui/interface/; CityData.cpp::LoadQueue -> bldque.cpp::Load.
 * editqueue.ldl::{LoadBox,eq_LoadListItem,eq_ContentsItem} and
 * messagebox.ldl::MessageBoxDialog provide actual controls/fonts/bounds/OK/Cancel.
 * No first-file choice, mutable filename re-read, local queue clearing or hidden
 * production calculations. Captured command is immutable, queued != executed.
 */
#[test]
fn build_saved_load_exact_file_preview_cancel_and_immutable_query_submit_once_then_return_to_items() {
    let mut harness=buildable_fixture();begin_row_press(&mut harness,"ItemsBox.UnitsList/UNIT_SETTLER");
    open(&mut harness,"U UNIT_SETTLER");
    manager_event(&mut harness,&key(Keycode::UP));
    assert_eq!(harness.ui.management.view.as_ref().and_then(|view|view.selections.get("saved-queue")).map(String::as_str),Some("memory:00-other"),"source file-list arrows update the exact preview selection");
    manager_event(&mut harness,&key(Keycode::DOWN));manager_event(&mut harness,&key(Keycode::SPACE));
    assert!(harness.ui.operation.is_none(),"file-list Space has no source Add/load keyboard action; explicit Load required");
    assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"preview navigation sends no command");
    let widgets=harness.ui.management.widgets(&harness.state);
    assert!(widgets.iter().any(|widget|widget.id=="LoadBox.QueueName" && widget.label=="chosen"),"filename is actual selected local name, not a content ID");
    assert!(widgets.iter().any(|widget|widget.id.starts_with("LoadBox.Contents/preview:0") && widget.label=="Settler"),"source exact selected file preview");
    let before=harness.state.viewer.clone();question(&mut harness);
    let dialog=harness.ui.operation.as_ref().expect("query");
    assert!(dialog.close_load_on_submit,"actual enqueue returns to ItemsBox");
    let message=dialog.confirmation.iter().find(|widget|widget.node.name=="MessageDisplay").expect("source message");
    let expected=saved::prompt(harness.ui.management.book.label("str_ldl_EditQueueReallyLoad"),&["chosen","Rome"]).expect("original cached string slots");
    assert_eq!(message.label,expected,"original localized filename/city query");
    input(&mut harness,&key(Keycode::ESCAPE));
    assert!(harness.ui.operation.is_none() && harness.ui.management.widgets(&harness.state).iter().any(|widget|widget.id=="LoadBox"),"query Cancel keeps original browser and selection");
    assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"no load on cancel");
    question(&mut harness);
    harness.ui.management.saved_queues.files.get_mut("memory:chosen").expect("captured file").parsed=saved::codec::parse("U UNIT_WARRIOR");
    approve(&mut harness);
    let command=harness.jobs.as_ref().expect("jobs").try_recv().expect("exact captured load");
    assert_eq!(command.path,"/game/44/edit-production","registered atomic editor");
    assert_eq!(command.body["city_id"],"c2","exact non-first selected city");
    assert_eq!(command.body["entries"],json!([{"item_id":"UNIT_SETTLER"}]),"immutable preview/query, not first file or re-read changed contents");
    assert_eq!((command.body["session_id"].as_i64(),command.body["expected_revision"].as_i64()),(Some(71_i64),Some(7_i64)),"captured session/revision");
    assert!(harness.ui.operation.is_none() && harness.ui.management.widgets(&harness.state).iter().any(|widget|widget.id=="ItemsBox"),"only accepted enqueue leaves browser");
    assert_eq!(harness.state.viewer,before,"no local replacement or treasury/production change");
    assert_eq!(harness.state.network,NetworkStatus::Pending,"enqueue not execution");
    assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"one exact request");
}

#[test]
fn build_saved_load_protects_exact_funded_head_and_queued_receipt_does_not_apply_template() {
    let mut harness=buildable_fixture();
    let quote=harness.state.viewer.as_mut().expect("viewer").cities.iter_mut().find(|city|city.id=="c2").expect("city").rush_buy_quote.as_mut().expect("quote");quote.status="completed".to_owned();quote.cost=Some(0_i64);
    harness.authority.session.projection=harness.state.viewer.clone();
    open(&mut harness,"U UNIT_SETTLER\nU UNIT_SETTLER");question(&mut harness);approve(&mut harness);
    let command=harness.jobs.as_ref().expect("jobs").try_recv().expect("funded-aware load");
    assert_eq!(command.body["entries"],json!([{"entry_id":"c2:head"},{"item_id":"UNIT_SETTLER"},{"item_id":"UNIT_SETTLER"}]),"retain exact paid/funded head, preserve duplicate new items");
    let mut queued=harness.authority.session.projection.clone().expect("queued receipt");queued.revision=8_i64;queued.interface.planning_status="queued".to_owned();
    sync::apply(&mut harness.authority,&mut harness.state,&mut harness.ui,&mut MovementInputState::default(),Outcome::Receipt(Box::new(queued)));
    let city=harness.state.viewer.as_ref().expect("viewer").cities.iter().find(|city|city.id=="c2").expect("same city");
    assert_eq!(city.facts["queue"].as_array().map(Vec::len),Some(2_usize),"queued receipt isn't replaced template");assert_eq!(city.facts["stored_production"],25_i64,"production remains received");assert_eq!(harness.state.gold,100_i32,"no local spending");
}

#[test]
fn build_saved_load_bad_file_wrong_kind_unknown_definition_or_funding_never_opens_a_destructive_query() {
    for mode in 0_u8..6_u8 {
        let mut harness=buildable_fixture();
        let text=match mode {0_u8=>"U UNIT_SETTLER\nBAD",1_u8=>"B UNIT_SETTLER",2_u8=>"U UNKNOWN",_=>"U UNIT_SETTLER"};
        if mode==3_u8 {harness.state.viewer.as_mut().expect("viewer").cities.iter_mut().find(|city|city.id=="c2").expect("city").rush_buy_quote=None;}
        if mode==4_u8 {harness.state.viewer.as_mut().expect("viewer").interface.planning_status="queued".to_owned();}
        if mode==5_u8 {harness.state.viewer.as_mut().expect("viewer").interface.legal_actions.clear();}
        open(&mut harness,text);
        assert!(harness.ui.management.activate_id("LoadBox.LoadButton",&harness.state).is_none(),"mode{mode}: unsupported complete file/funding/phase/authorization never becomes partial destructive load");
        assert!(harness.ui.management.widgets(&harness.state).iter().any(|widget|widget.id=="LoadBox.LoadButton" && widget.unavailable.is_some()),"honest disabled reason");
        assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"no hidden request");
    }
}

#[test]
fn build_saved_load_failed_approval_keeps_source_query_browser_and_captured_revision() {
    for mode in 0_u8..4_u8 {
        let mut harness=buildable_fixture();open(&mut harness,"U UNIT_SETTLER");question(&mut harness);
        match mode {
            0_u8 => {harness.state.viewer.as_mut().expect("viewer").revision=8_i64;harness.authority.session.projection.as_mut().expect("session").revision=8_i64;},
            1_u8 => harness.state.network=NetworkStatus::Pending,
            2_u8 => {let filler=PreparedCommand::new(&harness.authority.session,"pass",json!({})).expect("bounded-channel sentinel");harness.authority.io.as_ref().expect("network").submit(filler).expect("fill channel");},
            _=>{},
        }
        let before=harness.state.viewer.clone();input(&mut harness,&key(Keycode::TAB));input(&mut harness,&key(Keycode::TAB));
        if mode==3_u8 {assert!(operation_input::consume(&key(Keycode::RETURN),&mut harness.state,&mut harness.ui,None),"no authority still owns source question");}else{input(&mut harness,&key(Keycode::RETURN));}
        assert!(harness.ui.operation.is_some() && harness.ui.management.widgets(&harness.state).iter().any(|widget|widget.id=="LoadBox"),"failure never leaves query/browser or silently rebases");
        assert_eq!(harness.ui.operation.as_ref().expect("query").revision,7_i64,"captured revision retained");assert_eq!(harness.state.viewer,before,"no mutation");
        let jobs=harness.jobs.as_ref().expect("jobs");if mode==2_u8 {assert!(jobs.try_recv().is_ok_and(|command|command.path=="/game/44/pass"),"only original filler");}
        assert_eq!(jobs.try_recv(),Err(TryRecvError::Empty),"no failed/rebased load");
    }
}
