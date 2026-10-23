//! Original Build questions/queue guards with inert bounded command channels.
/// Original saved-file browser/preview/confirmed-load journeys, with fake storage.
mod saved_load_tests;
/// Source custom-mode editing is local, never an authoritative queue operation.
mod custom_tests;
/// Source Save-name/native/storage journeys with an entirely fake file boundary.
mod save_tests;
/// Original explicit profile-file Delete question; all storage effects are fake.
mod delete_tests;
/// Original exact-name Overwrite from Save-name, using only fake profile storage.
mod overwrite_tests;
/// Released source queue-first descriptions/Library and explicit empty-pane deselection.
mod description_tests;
/// Repaired complete city append contract, separate from local Custom authoring.
mod production_tests;
/// Safe rejection and subsequent successful replication remain distinct facts.
mod rejection_tests;
use super::{Network, Outcome};
use crate::gameplay::{assets::layout::LayoutBook, model::{GameState, NetworkStatus}, runtime::{MovementInputState, RuntimeAuthority, authority::state_from_authority_projection, management_input, operation_input, sync}, server::{ServerClient, ServerConfig, ServerGameplaySession, envelope::PreparedCommand}, ui_model::{InGameUiModel, management::{ManagementUi, ManagerKind}}};
use alloc::sync::Arc;
use core::sync::atomic::AtomicBool;
use sdl2::{event::Event, keyboard::{Keycode, Mod}, mouse::MouseButton};
use serde_json::json;
use std::{path::Path, sync::mpsc::{self, Receiver, TryRecvError}};

/*
 * Source: EditQueue.cpp::{ClearButton,ClearMessageCallback} and
 * MessageBoxDialog.{cpp,h}::{Query,constructor,LeftButtonActionCallback,
 * RightButtonActionCallback} in ctp2_source/ctp2_code/ui/interface/;
 * messagebox.ldl::MessageBoxDialog provides actual OK/Cancel controls/geometry.
 * The tests drive those semantic IDs/native SDL events and captured revision,
 * not coordinates or direct queue mutation. No workers/sockets/files are created.
 */
/// Original affirmative control, not an implicit first command.
const OK: &str = "MessageBoxDialog.DialogBackground.LeftButton";
/// Original cancel control is visually leftmost despite its `RightButton` name.
const CANCEL: &str = "MessageBoxDialog.DialogBackground.RightButton";

/// Observable production-shaped channel boundary without live transport.
struct Harness {
    /// Actual session metadata, inert network sender and no worker threads.
    authority: RuntimeAuthority,
    /// Complete owner-only facts.
    state: GameState,
    /// Cached original source dialogs and current Build view.
    ui: InGameUiModel,
    /// Only captured jobs; None when simulating disconnect.
    jobs: Option<Receiver<PreparedCommand>>,
}

/// Non-first owned city, current quote and exact atomic queue-edit permission.
fn fixture() -> Harness {
    let projection = serde_json::from_value(json!({"revision":7_i64,"turn":1_u32,"contract":{"version":2_u32},"tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"plains"},{"id":"b","x":2_u16,"y":1_u16,"terrain":"plains"}],"cities":[{"id":"c1","owner_id":"account-7","tile_id":"a","queue":[]},{"id":"c2","owner_id":"account-7","tile_id":"b","name":"Rome","queue":[{"entry_id":"c2:head","id":"UNIT_HOPLITE","kind":"unit"},{"entry_id":"c2:future","id":"UNIT_WARRIOR","kind":"unit"}],"stored_production":25_i64,"rush_buy_quote":{"revision":7_i64,"status":"available","cost":30_i64}}],"diplomacy":{"gold":100_i32},"legal_actions":[{"route":"edit-production","entity_id":"c2","fields":["city_id","entries"],"payload":{"city_id":"c2","entries":[]}}]})).expect("exact viewer/editor schema");
    let (sender, jobs) = mpsc::sync_channel(1_usize);
    let (_results, results) = mpsc::sync_channel(1_usize);
    let authority = RuntimeAuthority { io: Some(Network { jobs:sender, results, stop:Arc::new(AtomicBool::new(false)) }), client:ServerClient::from_config(&ServerConfig { base_url:"http://127.0.0.1:1".to_owned(), required:true }).expect("unused endpoint"), session:ServerGameplaySession { game_id:44_i64, session_id:71_i64, account_id:7_i64, projection:Some(projection) }, empire_count:1_u8, seed:1_u64 };
    let mut state = state_from_authority_projection(&authority, authority.session.projection.as_ref().expect("viewer"));
    state.popups.clear();
    let book = LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2")).expect("source assets read only");
    let mut ui = InGameUiModel { management:ManagementUi::new(book).expect("cached original roots"), ..InGameUiModel::default() };
    ui.management.open(ManagerKind::Build, &state, Some("c2"));
    Harness { authority, state, ui, jobs:Some(jobs) }
}

/// Opening invokes only the original question callback, never an order.
fn open(harness: &mut Harness) {
    let intent = harness.ui.management.activate_id("QueueGroup.ClearButton", &harness.state).expect("original Clear control");
    management_input::apply(intent, &mut harness.state, &mut harness.ui, Some(&mut harness.authority));
    assert!(harness.ui.operation.is_some(), "source query above the same still-visible Build view");
    assert_eq!(harness.ui.management.view.as_ref().map(|view| view.kind), Some(ManagerKind::Build), "parent doesn't disappear behind query");
    assert_eq!(harness.jobs.as_ref().expect("observable channel").try_recv(), Err(TryRecvError::Empty), "opening isn't approval");
}

/// Real SDL keyboard event with no OS injection.
fn key(code: Keycode) -> Event {
    Event::KeyDown { timestamp:0_u32, window_id:0_u32, keycode:Some(code), scancode:None, keymod:Mod::NOMOD, repeat:false }
}

/// The production modal dispatcher consumes all question input.
fn input(harness: &mut Harness, event: &Event) {
    assert!(operation_input::consume(event, &mut harness.state, &mut harness.ui, Some(&mut harness.authority)), "native source question input");
}

#[test]
fn build_clear_question_cancel_escape_and_unfocused_enter_never_approve_or_dispatch_map_actions() {
    for mode in 0_u8..3_u8 {
        let mut harness = fixture();
        let before = harness.state.clone();
        let parent = harness.ui.management.view.clone();
        open(&mut harness);
        input(&mut harness, &key(Keycode::RETURN));
        assert!(harness.ui.operation.is_some(), "Enter without explicit affirmative focus cannot choose first command");
        if mode == 0_u8 { input(&mut harness, &key(Keycode::ESCAPE)); }
        else if mode == 1_u8 {
            input(&mut harness, &key(Keycode::TAB));
            assert_eq!(harness.ui.operation.as_ref().expect("query").confirmation_focus.as_deref(), Some(CANCEL), "source left-to-right first explicit focus is Cancel");
            input(&mut harness, &key(Keycode::RETURN));
        } else {
            let rect = harness.ui.operation.as_ref().expect("query").confirmation.iter().find(|widget| widget.id == CANCEL).expect("source cancel descriptor").rect;
            input(&mut harness, &Event::MouseButtonDown { timestamp:0_u32, window_id:0_u32, which:0_u32, mouse_btn:MouseButton::Left, clicks:1_u8, x:rect.x.saturating_add(1_i32), y:rect.y.saturating_add(1_i32) });
        }
        assert!(harness.ui.operation.is_none(), "cancel/ESC dismisses only query");
        assert_eq!(harness.ui.management.view, parent, "exact original Build selection preserved");
        assert_eq!(harness.state, before, "no queue/gold/turn mutation on query/cancel");
        assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(), Err(TryRecvError::Empty), "no hidden operation/turn/map action");
    }
}

#[test]
fn build_clear_question_explicit_source_ok_enqueues_exact_revision_once_without_applying_queued_result() {
    let mut harness = fixture();
    open(&mut harness);
    let received = harness.state.viewer.clone();
    input(&mut harness, &key(Keycode::TAB));
    input(&mut harness, &key(Keycode::TAB));
    assert_eq!(harness.ui.operation.as_ref().expect("query").confirmation_focus.as_deref(), Some(OK), "two actual native focus steps explicitly select source OK");
    input(&mut harness, &key(Keycode::RETURN));
    assert!(harness.ui.operation.is_none(), "accepted local job dismisses question, not manager");
    assert_eq!(harness.state.network, NetworkStatus::Pending, "enqueue isn't gameplay execution");
    let command = harness.jobs.as_ref().expect("jobs").try_recv().expect("one captured queue operation");
    assert_eq!(command.path, "/game/44/edit-production", "registered atomic queue route");
    assert_eq!(command.body["city_id"], "c2", "exact non-first selected city");
    assert_eq!(command.body["entries"], json!([]), "captured explicit complete clear");
    assert_eq!((command.body["session_id"].as_i64(),command.body["expected_revision"].as_i64()), (Some(71_i64),Some(7_i64)), "immutable displayed revision/membership");
    assert_eq!(harness.state.viewer, received, "no local queue clearing or stored-production loss");
    let mut queued = harness.authority.session.projection.clone().expect("receipt");
    queued.revision=8_i64;queued.interface.planning_status="queued".to_owned();
    sync::apply(&mut harness.authority, &mut harness.state, &mut harness.ui, &mut MovementInputState::default(), Outcome::Receipt(Box::new(queued)));
    assert_eq!(harness.state.gold, 100_i32, "queued result doesn't spend gold");
    let city = harness.state.viewer.as_ref().expect("viewer").cities.iter().find(|city| city.id == "c2").expect("same city");
    assert_eq!(city.facts["queue"].as_array().map(Vec::len), Some(2_usize), "queued receipt isn't local execution");
    assert_eq!(city.facts["stored_production"], 25_i64, "no speculative production loss");
    assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(), Err(TryRecvError::Empty), "no duplicate request");
}

#[test]
fn build_clear_question_stale_session_revision_or_failed_enqueue_keeps_captured_confirmation_open() {
    for mode in 0_u8..6_u8 {
        let mut harness=fixture();open(&mut harness);
        harness.ui.operation.as_mut().expect("query").confirmation_focus=Some(OK.to_owned());
        match mode {
            0_u8 => harness.state.viewer.as_mut().expect("viewer").revision=8_i64,
            1_u8 => harness.authority.session.projection.as_mut().expect("session").revision=8_i64,
            2_u8 => { let filler=PreparedCommand::new(&harness.authority.session,"pass",json!({})).expect("queue sentinel");harness.authority.io.as_ref().expect("inert network").submit(filler).expect("fill bounded channel"); },
            3_u8 => drop(harness.jobs.take()),
            4_u8 => harness.state.network=NetworkStatus::Pending,
            _ => {},
        }
        let before=harness.state.viewer.clone();
        if mode == 5_u8 {
            assert!(operation_input::consume(&key(Keycode::RETURN), &mut harness.state, &mut harness.ui, None), "absent authority still owns modal input without submitting");
        } else {
            input(&mut harness, &key(Keycode::RETURN));
        }
        assert!(harness.ui.operation.is_some(), "no silent close/rebase on stale or failed request");
        assert_eq!(harness.ui.operation.as_ref().expect("query").revision,7_i64,"original revision retained");
        assert_eq!(harness.state.viewer,before,"no canonical mutation from invalid/failed approval");
        if let Some(jobs)=harness.jobs.as_ref() {
            if mode==2_u8 { assert!(jobs.try_recv().is_ok_and(|command| command.path=="/game/44/pass"),"only pre-existing filler"); }
            assert_eq!(jobs.try_recv(),Err(TryRecvError::Empty),"no new/rebased command");
        }
    }
}

/*
 * Source: ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::UpdateButtons,
 * RemoveItem/Remove and ItemUp/Up guard bought index0/index1->0, while future
 * removal stays enabled. Original queue edits call CityData/bldque directly;
 * this native path instead captures one entry-addressed authoritative request.
 * Completed quote means funded, not a fabricated original paid-history flag.
 */
#[test]
fn build_queue_edit_native_funded_guard_does_not_fallthrough_or_mutate_queue_on_safe_future_enqueue() {
    let mut harness=fixture();
    let quote=harness.state.viewer.as_mut().expect("viewer").cities.iter_mut().find(|city| city.id=="c2").expect("exact city").rush_buy_quote.as_mut().expect("quote");
    quote.status="completed".to_owned();quote.cost=Some(0_i64);
    harness.authority.session.projection=harness.state.viewer.clone();
    let select=harness.ui.management.activate_id("QueueGroup.QueueList/c2:future",&harness.state).expect("explicit actual source queue row");
    assert!(harness.ui.management.apply_local(select),"select entry, not its content or another city");
    harness.ui.management.focused=Some("QueueGroup.UpButton".to_owned());
    assert!(management_input::consume(&key(Keycode::RETURN),&mut harness.state,&mut harness.ui,&mut MovementInputState::default(),Some(&mut harness.authority)),"disabled source Up consumes native key before map");
    assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"index1 cannot displace funded head");
    assert_eq!(harness.state.network,NetworkStatus::Ready,"disabled attempt creates no pending action");
    let before=harness.state.viewer.clone();
    harness.ui.management.focused=Some("QueueGroup.RemoveButton".to_owned());
    assert!(management_input::consume(&key(Keycode::RETURN),&mut harness.state,&mut harness.ui,&mut MovementInputState::default(),Some(&mut harness.authority)),"future Remove uses actual source native callback without invented Query");
    let command=harness.jobs.as_ref().expect("jobs").try_recv().expect("one safe future removal");
    assert_eq!(command.path,"/game/44/edit-production","atomic editor, not dangerous legacy cancel");
    assert_eq!(command.body["city_id"],"c2","exact non-first selected city");
    assert_eq!(command.body["entries"],json!([{"entry_id":"c2:head"}]),"funded head unchanged, exact future removed");
    assert_eq!(command.body["expected_revision"],7_i64,"captured receipt revision");
    assert_eq!(harness.state.network,NetworkStatus::Pending,"enqueue isn't execution");
    assert_eq!(harness.state.viewer,before,"canonical queue and funding are not locally changed");
    assert!(harness.ui.operation.is_none(),"ordinary original Remove has no Query");
    assert_eq!(harness.ui.management.view.as_ref().and_then(|view|view.selections.get("queue")).map(String::as_str),Some("c2:future"),"do not replace pending removed entry with first/current head");
    assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"one exact request");
}

/// Two authorized unit definitions make non-first Add an explicit real row.
fn buildable_fixture() -> Harness {
    let mut harness=fixture();
    let interface=&mut harness.state.viewer.as_mut().expect("viewer").interface;
    interface.rules=Some(json!({"units":[{"id":"UNIT_WARRIOR","production_cost":50_i64},{"id":"UNIT_SETTLER","production_cost":330_i64},{"id":"UNIT_HOPLITE","production_cost":100_i64}]}));
    interface.contract.as_mut().expect("v2 opt-in").features.insert("atomic_production_queue".to_owned(),true);
    for id in ["UNIT_WARRIOR","UNIT_SETTLER","UNIT_HOPLITE"] {
        interface.legal_actions.push(crate::gameplay::server::interface::LegalAction { route:"queue-production".to_owned(),entity_id:"c2".to_owned(),target_id:id.to_owned(),payload:json!({"city_id":"c2","item":{"id":id,"kind":"unit","cost":50_i64}}),..crate::gameplay::server::interface::LegalAction::default() });
    }
    harness.authority.session.projection=harness.state.viewer.clone();
    harness
}

/// Native SDL input chooses a semantic current source row, never a hardcoded point.
fn row_event(harness: &Harness, id: &str, clicks: u8, pressed: bool) -> Event {
    let row=harness.ui.management.widgets(&harness.state).into_iter().find(|widget| widget.id==id).expect("exact displayed original source row");
    assert!(row.rect.width>4_u32 && row.rect.height>4_u32,"nonempty visible original row");
    let x=row.rect.x.saturating_add(2_i32);let y=row.rect.y.saturating_add(2_i32);
    if pressed { Event::MouseButtonDown {timestamp:0_u32,window_id:0_u32,which:0_u32,mouse_btn:MouseButton::Left,clicks,x,y} }
    else { Event::MouseButtonUp {timestamp:0_u32,window_id:0_u32,which:0_u32,mouse_btn:MouseButton::Left,clicks,x,y} }
}

/// First down/up selects without ordering; SDL then reports count2 on next down.
fn begin_row_press(harness: &mut Harness, id: &str) {
    let down=row_event(harness,id,1_u8,true);manager_event(harness,&down);
    let up=row_event(harness,id,1_u8,false);manager_event(harness,&up);
    assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"first source row click selects only");
}

/// Production manager input remains strongly modal and uses only inert channels.
fn manager_event(harness: &mut Harness, event: &Event) {
    assert!(management_input::consume(event,&mut harness.state,&mut harness.ui,&mut MovementInputState::default(),Some(&mut harness.authority)),"native original Build callback before world/map handlers");
}

/*
 * Source: ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::{ListCallback,Add,
 * InsertInQueue,QueueListCallback,Remove,ShowSelectedInfo,UpdateButtons};
 * CityData.cpp::InsertBuildItem and bldque.cpp::{InsertBefore,RemoveNodeByIndex}.
 * editqueue.ldl::BuildEditorWindow.ItemsBox.{UnitsList,BuildingList,WonderList}
 * and QueueGroup.QueueList keep source hit/selection rows and styles/positions.
 * SDL two-click callbacks select their exact current row BEFORE invoking the same
 * Add/Remove control and guards; they do not use a previous/first selection or
 * directly mutate original-engine queues. Pending/queued isn't execution.
 * Higher click counts cannot cascade removals after a receipt shifts rows.
 */
#[test]
fn build_double_click_left_nonfirst_unit_appends_exact_item_once_and_queued_receipt_is_not_execution() {
    let mut harness=buildable_fixture();
    for id in ["ItemsBox.UnitsList/UNIT_WARRIOR","QueueGroup.QueueList/c2:future"] {
        let intent=harness.ui.management.activate_id(id,&harness.state).expect("explicit independent source selection");
        assert!(harness.ui.management.apply_local(intent),"selection alone isn't a command");
    }
    let before=harness.state.viewer.clone();
    begin_row_press(&mut harness,"ItemsBox.UnitsList/UNIT_SETTLER");
    let event=row_event(&harness,"ItemsBox.UnitsList/UNIT_SETTLER",2_u8,true);
    manager_event(&mut harness,&event);
    let command=harness.jobs.as_ref().expect("jobs").try_recv().expect("original ListCallback double-click adds exact clicked unit");
    assert_eq!(command.path,"/game/44/edit-production","same atomic current Add control");
    assert_eq!(command.body["city_id"],"c2","non-first selected city");
    assert_eq!(command.body["entries"],json!([{"entry_id":"c2:head"},{"entry_id":"c2:future"},{"item_id":"UNIT_SETTLER"}]),"append clicked non-first buildable, not old Warrior or selected queue entry");
    assert_eq!(command.body["expected_revision"],7_i64,"immutable current source revision");
    let view=harness.ui.management.view.as_ref().expect("Build stays visible");
    assert_eq!(view.kind,ManagerKind::Build,"do not close the manager after Add");
    assert_eq!(view.selections.get("buildable").map(String::as_str),Some("UNIT_SETTLER"),"left selection is actual clicked ID");
    assert_eq!(view.selections.get("queue").map(String::as_str),Some("c2:future"),"right selection remains independent");
    assert_eq!(harness.state.viewer,before,"no local append, gold or production mutation");
    let release=row_event(&harness,"ItemsBox.UnitsList/UNIT_SETTLER",2_u8,false);
    manager_event(&mut harness,&release);
    assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"mouse-up creates no duplicate/map order");
    let mut queued=harness.authority.session.projection.clone().expect("receipt");queued.revision=8_i64;queued.interface.planning_status="queued".to_owned();
    sync::apply(&mut harness.authority,&mut harness.state,&mut harness.ui,&mut MovementInputState::default(),Outcome::Receipt(Box::new(queued)));
    assert_eq!(harness.state.gold,100_i32,"queued isn't spending");
    assert_eq!(harness.state.viewer.as_ref().expect("viewer").cities.iter().find(|city|city.id=="c2").expect("exact city").facts["queue"].as_array().map(Vec::len),Some(2_usize),"no generated entry/new execution until actual received queue changes");
}

#[test]
fn build_double_click_queue_removes_exact_duplicate_future_but_never_funded_or_unknown_head_or_new_first_row() {
    for mode in 0_u8..3_u8 {
        let mut harness=buildable_fixture();
        let projected_city=harness.state.viewer.as_mut().expect("viewer").cities.iter_mut().find(|city|city.id=="c2").expect("exact city");
        projected_city.facts["queue"][1_usize]["id"]=json!("UNIT_HOPLITE");
        if mode==2_u8 { projected_city.rush_buy_quote=None; } else { let quote=projected_city.rush_buy_quote.as_mut().expect("quote");quote.status="completed".to_owned();quote.cost=Some(0_i64); }
        harness.authority.session.projection=harness.state.viewer.clone();
        let prior=if mode==0_u8 {"QueueGroup.QueueList/c2:head"} else {"QueueGroup.QueueList/c2:future"};
        for id in ["ItemsBox.UnitsList/UNIT_SETTLER",prior] {
            let intent=harness.ui.management.activate_id(id,&harness.state).expect("explicit source row");
            assert!(harness.ui.management.apply_local(intent),"local independent selection");
        }
        let target=if mode==0_u8 {"QueueGroup.QueueList/c2:future"} else {"QueueGroup.QueueList/c2:head"};
        let before=harness.state.viewer.clone();
        begin_row_press(&mut harness,target);
        let event=row_event(&harness,target,2_u8,true);manager_event(&mut harness,&event);
        assert_eq!(harness.state.viewer,before,"double-click isn't local production execution");
        assert!(harness.ui.operation.is_none(),"ordinary original queue removal has no Query");
        if mode==0_u8 {
            let command=harness.jobs.as_ref().expect("jobs").try_recv().expect("exact duplicate future removal");
            assert_eq!(command.body["entries"],json!([{"entry_id":"c2:head"}]),"never remove identical-content funded head or reuse prior selected head");
            assert_eq!(harness.state.network,NetworkStatus::Pending,"queued intent only");
            let mut received=harness.authority.session.projection.clone().expect("later actually changed queue");received.revision=8_i64;
            let current=received.cities.iter_mut().find(|city|city.id=="c2").expect("exact city");current.facts["queue"].as_array_mut().expect("queue").truncate(1_usize);
            let quote=current.rush_buy_quote.as_mut().expect("quote");quote.revision=8_i64;quote.status="available".to_owned();quote.cost=Some(30_i64);
            sync::apply(&mut harness.authority,&mut harness.state,&mut harness.ui,&mut MovementInputState::default(),Outcome::Receipt(Box::new(received)));
            let view=harness.ui.management.view.as_ref().expect("same parent");
            assert!(!view.selections.contains_key("queue") && view.selections.get("description").is_some_and(|id|id=="UNIT_SETTLER"),"removed queue identity never selects first/head; source falls back only to already explicit left choice");
            assert_eq!(view.selections.get("buildable").map(String::as_str),Some("UNIT_SETTLER"),"independent left choice survives actual removal");
            let third=row_event(&harness,"QueueGroup.QueueList/c2:head",3_u8,true);manager_event(&mut harness,&third);
            assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"higher click count must not cascade into now-first unlocked head");
        } else {
            assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"funded/unknown head guard shared by double-click and source button");
            assert_eq!(harness.state.network,NetworkStatus::Ready,"disabled head never submits");
        }
    }
}

#[test]
fn build_double_click_queue_cannot_bypass_pending_absent_session_or_stale_session_even_after_selecting_exact_row() {
    for mode in 0_u8..3_u8 {
        let mut harness=fixture();
        if mode==0_u8 { harness.state.network=NetworkStatus::Pending; }
        if mode==2_u8 { harness.authority.session.projection.as_mut().expect("session").revision=8_i64; }
        let before=harness.state.viewer.clone();
        begin_row_press(&mut harness,"QueueGroup.QueueList/c2:future");
        let event=row_event(&harness,"QueueGroup.QueueList/c2:future",2_u8,true);
        if mode==1_u8 {
            assert!(management_input::consume(&event,&mut harness.state,&mut harness.ui,&mut MovementInputState::default(),None),"absent authority still consumes source modal input");
        } else { manager_event(&mut harness,&event); }
        assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"no blocked/rebased request");
        assert_eq!(harness.state.viewer,before,"failed dispatch never removes a canonical entry");
        assert_eq!(harness.ui.management.view.as_ref().and_then(|view|view.selections.get("queue")).map(String::as_str),Some("c2:future"),"failed action retains explicit clicked row, not another candidate");
    }
}

#[test]
fn build_double_click_interrupted_by_receipt_row_shift_city_focus_or_keyboard_never_rebases_first_press() {
    for mode in 0_u8..6_u8 {
        let mut harness=buildable_fixture();
        if mode==2_u8 {
            harness.state.viewer.as_mut().expect("viewer").interface.legal_actions.push(serde_json::from_value(json!({"route":"edit-production","entity_id":"c1","fields":["city_id","entries"],"payload":{"city_id":"c1","entries":[]}})).expect("other owned city permission"));
            harness.state.viewer.as_mut().expect("viewer").interface.legal_actions.push(serde_json::from_value(json!({"route":"queue-production","entity_id":"c1","target_id":"UNIT_SETTLER","payload":{"city_id":"c1","item":{"id":"UNIT_SETTLER","kind":"unit","cost":330_i64}}})).expect("other city candidate authority"));
            harness.authority.session.projection=harness.state.viewer.clone();
        }
        let first=if mode==1_u8 {"QueueGroup.QueueList/c2:head"} else if mode==2_u8 {"ItemsBox.UnitsList/UNIT_SETTLER"} else {"QueueGroup.QueueList/c2:future"};
        begin_row_press(&mut harness,first);
        match mode {
            0_u8 | 1_u8 => {
                let mut snapshot=harness.authority.session.projection.clone().expect("current snapshot");snapshot.revision=8_i64;
                let city=snapshot.cities.iter_mut().find(|city|city.id=="c2").expect("same city");
                city.rush_buy_quote.as_mut().expect("quote").revision=8_i64;
                if mode==1_u8 { city.facts["queue"].as_array_mut().expect("queue").remove(0_usize); }
                sync::apply(&mut harness.authority,&mut harness.state,&mut harness.ui,&mut MovementInputState::default(),Outcome::Snapshot(Box::new(snapshot)));
            }
            2_u8 => assert!(harness.ui.management.select_city("c1",&harness.state),"authorized city switch between presses"),
            3_u8 => {
                let lost=Event::Window {timestamp:0_u32,window_id:0_u32,win_event:sdl2::event::WindowEvent::FocusLost};
                assert!(!management_input::consume(&lost,&mut harness.state,&mut harness.ui,&mut MovementInputState::default(),Some(&mut harness.authority)),"window focus propagates after canceling held UI press");
            }
            4_u8 => manager_event(&mut harness,&key(Keycode::TAB)),
            _ => { open(&mut harness); input(&mut harness,&key(Keycode::ESCAPE)); },
        }
        let second=if mode==1_u8 {"QueueGroup.QueueList/c2:future"} else {first};
        let event=row_event(&harness,second,2_u8,true);manager_event(&mut harness,&event);
        assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"mode{mode}: no rebased double-click on new revision, shifted row, city, or interrupted focus");
        assert_eq!(harness.state.network,NetworkStatus::Ready,"interrupted gesture remains presentation-only");
        assert_eq!(harness.state.viewer.as_ref().expect("viewer").cities.iter().find(|city|city.id=="c1").expect("other city").facts["queue"],json!([]),"never append in another city");
    }
}

/*
 * Source: EditQueue.cpp::{SelectChoiceList,ShowSelectedInfo,Library}
 * and citywindow.cpp::CityWindow::SetItemDescription (original ui/interface/);
 * aui_HyperTextBox::{SetHyperText,RepositionRanger}/aui_Ranger::SetMaximum retain
 * in-range scroll and clamp excess. editqueue.ldl::BuildEditorWindow.
 * {ItemDescription,LibraryButton,ItemsBox,QueueGroup} provides source controls.
 * Received removal while Library is open must not invent a replacement queue
 * selection, erase a different content role or execute any operation on return.
 */
#[test]
fn build_lifecycle_native_library_receipt_return_and_category_switch_preserve_exact_description_and_clamp_source_scroll() {
    let mut harness=buildable_fixture();
    for (id,lines) in [("UNIT_SETTLER",40_usize),("UNIT_HOPLITE",40_usize)] {
        if let Some(art)=harness.ui.management.library.catalog.art.get_mut(id) {art.summary=None;}
        let text=core::iter::repeat_n("Synthetic summary line.\n",lines).collect::<String>();
        harness.ui.management.library.catalog.articles.entry(id.to_owned()).or_default().sections.insert(crate::gameplay::assets::library::LibrarySection::Summary,crate::gameplay::assets::library::LibraryDocument::parse(&text));
    }
    let queue=harness.ui.management.activate_id("QueueGroup.QueueList/c2:head",&harness.state).expect("exact source queue row");
    assert!(harness.ui.management.apply_local(queue),"independent queue");
    begin_row_press(&mut harness,"ItemsBox.UnitsList/UNIT_SETTLER");
    assert!(harness.ui.management.scroll_to("ItemDescription",2_usize,&harness.state),"source measured long article ranger");
    let words=harness.ui.management.widgets(&harness.state);
    let pane=words.iter().find(|widget|widget.id=="ItemDescription").expect("original text viewport").rect;
    let fragments=words.iter().filter(|widget|widget.id.starts_with("ItemDescription/word-")).collect::<Vec<_>>();
    assert!(!fragments.is_empty(),"long selected description really supplies visible words");
    for word in fragments {
        assert!(word.rect.x>=pane.x && word.rect.y>=pane.y && word.rect.x.saturating_add(i32::try_from(word.rect.width).expect("word width"))<=pane.x.saturating_add(i32::try_from(pane.width).expect("pane width")) && word.rect.y.saturating_add(i32::try_from(word.rect.height).expect("word height"))<=pane.y.saturating_add(i32::try_from(pane.height).expect("pane height")),"every visible source text descriptor stays inside ItemDescription");
    }
    harness.ui.management.focused=Some("LibraryButton".to_owned());manager_event(&mut harness,&key(Keycode::RETURN));
    assert_eq!(harness.ui.management.view.as_ref().map(|view|view.kind),Some(ManagerKind::Library),"source Library button opens related window");
    assert_eq!(harness.ui.management.library.current.as_deref(),Some("UNIT_HOPLITE"),"released queue-first actual content, not last-focused left choice");
    let mut snapshot=harness.authority.session.projection.clone().expect("current full snapshot");snapshot.revision=8_i64;
    let city=snapshot.cities.iter_mut().find(|city|city.id=="c2").expect("same city");city.facts["queue"].as_array_mut().expect("queue").remove(0_usize);city.rush_buy_quote.as_mut().expect("quote").revision=8_i64;
    sync::apply(&mut harness.authority,&mut harness.state,&mut harness.ui,&mut MovementInputState::default(),Outcome::Snapshot(Box::new(snapshot)));
    manager_event(&mut harness,&key(Keycode::ESCAPE));
    let view=harness.ui.management.view.as_ref().expect("exact parent restored");
    assert_eq!((view.kind,view.city_id.as_deref()),(ManagerKind::Build,Some("c2")),"same original Build/city parent");
    assert!(!view.selections.contains_key("queue"),"removed queue identity doesn't select new first entry");
    assert_eq!(view.selections.get("buildable").map(String::as_str),Some("UNIT_SETTLER"),"independent left choice survives actual received removal");
    assert_eq!(view.scroll.get("ItemDescription").copied(),Some(2_usize),"source retains in-range ranger on long document return");
    harness.ui.management.focused=Some("ItemsBox.BuildingsButton".to_owned());manager_event(&mut harness,&key(Keycode::RETURN));
    let switched=harness.ui.management.view.as_ref().expect("Build");
    assert!(!switched.selections.contains_key("buildable") && !switched.selections.contains_key("description"),"hidden Units choice/description cannot survive source category switch");
    assert_eq!(switched.scroll.get("ItemDescription").copied(),Some(0_usize),"empty description clamps actual stored value");
    assert!(harness.ui.management.activate_id("ItemsBox.AddButton",&harness.state).is_none() && harness.ui.management.activate_id("LibraryButton",&harness.state).is_none(),"no hidden choice can add or open a stale article");
    let bounds=harness.ui.management.widgets(&harness.state).into_iter().find(|widget|widget.id=="ItemDescription").expect("original source description control").rect;
    assert_eq!((bounds.width,bounds.height),(246_u32,65_u32),"source dimensions and current scaling unchanged");
    assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"navigation/return/category never submits a gameplay command");
}

/// Enough exact owner queue entries to cross the source viewport without assets.
fn navigation_fixture() -> Harness {
    let mut harness=buildable_fixture();
    let entries=(0_u8..24_u8).map(|index|json!({"entry_id":format!("c2:{index}"),"id":"UNIT_HOPLITE","kind":"unit"})).collect::<Vec<_>>();
    harness.state.viewer.as_mut().expect("viewer").cities.iter_mut().find(|city|city.id=="c2").expect("same city").facts["queue"]=json!(entries);
    harness.authority.session.projection=harness.state.viewer.clone();
    harness
}

/// Actual currently visible semantic row roots, excluding numeric children/rangers.
fn visible_queue(harness: &Harness) -> Vec<String> {
    harness.ui.management.widgets(&harness.state).into_iter().filter_map(|widget| {
        match widget.intent {
            Some(crate::gameplay::ui_model::management::ManagerIntent::Select(key,id)) if key=="queue" && widget.id==format!("QueueGroup.QueueList/{id}") => Some(id),
            _=>None,
        }
    }).collect()
}

/*
 * Original source: ctp2_source/ctp2_code/ui/interface/EditQueue.cpp constructor
 * wires visible left lists' SetKeyboardActionControl(m_addButton). Original
 * ui/aui_common/aui_listbox.cpp::{HandleKey,SendKeyboardAction,SelectItem} wraps
 * Up/Down and delegates Space; queue has no Remove keyboard action control.
 * EditQueue::{Up,Down,InsertInQueue} explicitly EnsureItemVisible after a change.
 * editqueue.ldl::BuildEditorWindow.{ItemsBox,QueueGroup.QueueList} defines the
 * unchanged list panes/rangers. Client queues never move before received facts.
 */
#[test]
fn build_navigation_native_pointer_focus_arrows_wrap_exact_left_list_and_space_submits_selected_item_once() {
    let mut harness=navigation_fixture();
    begin_row_press(&mut harness,"QueueGroup.QueueList/c2:0");
    begin_row_press(&mut harness,"ItemsBox.UnitsList/UNIT_SETTLER");
    assert_eq!(harness.ui.management.focused.as_deref(),Some("ItemsBox.UnitsList"),"source pointer click gives keyboard focus to that list");
    let before=harness.state.viewer.clone();
    manager_event(&mut harness,&key(Keycode::UP));
    assert_eq!(harness.ui.management.view.as_ref().and_then(|view|view.selections.get("buildable")).map(String::as_str),Some("UNIT_HOPLITE"),"Up selects previous source row, not another global button");
    manager_event(&mut harness,&key(Keycode::UP));
    assert_eq!(harness.ui.management.view.as_ref().and_then(|view|view.selections.get("buildable")).map(String::as_str),Some("UNIT_WARRIOR"),"source Up wraps first to last");
    manager_event(&mut harness,&key(Keycode::DOWN));manager_event(&mut harness,&key(Keycode::DOWN));
    assert_eq!(harness.ui.management.view.as_ref().and_then(|view|view.selections.get("buildable")).map(String::as_str),Some("UNIT_SETTLER"),"Down wraps then returns exact non-first target");
    assert_eq!(harness.ui.management.view.as_ref().and_then(|view|view.selections.get("queue")).map(String::as_str),Some("c2:0"),"independent queue selection remains");
    manager_event(&mut harness,&key(Keycode::SPACE));
    let command=harness.jobs.as_ref().expect("jobs").try_recv().expect("source Space dispatches same Add control");
    assert_eq!(command.path,"/game/44/edit-production","atomic editor");
    let entries=command.body["entries"].as_array().expect("complete captured order");
    assert_eq!(entries.len(),25_usize,"one appended item");assert_eq!(entries.last(),Some(&json!({"item_id":"UNIT_SETTLER"})),"never old/first unit or selected queue content");
    assert_eq!(command.body["city_id"],"c2","exact non-first city");assert_eq!(command.body["expected_revision"],7_i64,"current immutable revision");
    assert_eq!(harness.state.viewer,before,"keyboard navigation/submission is not canonical execution");
    manager_event(&mut harness,&key(Keycode::SPACE));
    assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"Pending Space cannot duplicate or act on map");
}

#[test]
fn build_navigation_queue_wrap_keeps_exact_offscreen_entry_visible_and_space_never_removes() {
    let mut harness=navigation_fixture();begin_row_press(&mut harness,"QueueGroup.QueueList/c2:0");
    let before=harness.state.viewer.clone();
    manager_event(&mut harness,&key(Keycode::UP));
    assert_eq!(harness.ui.management.view.as_ref().and_then(|view|view.selections.get("queue")).map(String::as_str),Some("c2:23"),"source Up wraps to exact last entry");
    assert!(visible_queue(&harness).iter().any(|id|id=="c2:23"),"fix original offscreen keyboard selection instead of hiding focus");
    assert!(harness.ui.management.view.as_ref().and_then(|view|view.scroll.get("QueueGroup.QueueList")).is_some_and(|offset|*offset>0_usize),"only source queue viewport moves");
    manager_event(&mut harness,&key(Keycode::SPACE));
    assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"queue has no source keyboard Remove action");
    manager_event(&mut harness,&key(Keycode::DOWN));
    assert_eq!(harness.ui.management.view.as_ref().and_then(|view|view.selections.get("queue")).map(String::as_str),Some("c2:0"),"source Down wraps last to first");
    assert!(visible_queue(&harness).iter().any(|id|id=="c2:0"),"first entry returns visibly");
    assert_eq!(harness.state.viewer,before,"no canonical queue/order changes from selection");
}

#[test]
fn build_navigation_received_reorder_ensures_same_selected_entry_visible_without_queued_execution_or_unrelated_scroll_snap() {
    let mut harness=navigation_fixture();
    let page=visible_queue(&harness).len();assert!(page>1_usize && page<24_usize,"real source queue viewport");
    let index=page.saturating_sub(1_usize);let selected=format!("c2:{index}");
    begin_row_press(&mut harness,&format!("QueueGroup.QueueList/{selected}"));
    harness.ui.management.focused=Some("QueueGroup.DownButton".to_owned());
    let before=harness.state.viewer.clone();manager_event(&mut harness,&key(Keycode::RETURN));
    let command=harness.jobs.as_ref().expect("jobs").try_recv().expect("one explicit source move down");
    assert_eq!(command.body["entries"][page]["entry_id"],selected,"captured exact entry moved down once");
    assert_eq!(harness.state.viewer,before,"no local reorder");
    let mut queued=harness.authority.session.projection.clone().expect("queued receipt");queued.revision=8_i64;queued.interface.planning_status="queued".to_owned();
    queued.cities.iter_mut().find(|city|city.id=="c2").expect("city").rush_buy_quote.as_mut().expect("quote").revision=8_i64;
    sync::apply(&mut harness.authority,&mut harness.state,&mut harness.ui,&mut MovementInputState::default(),Outcome::Receipt(Box::new(queued)));
    assert_eq!(harness.ui.management.view.as_ref().and_then(|view|view.scroll.get("QueueGroup.QueueList")).copied().unwrap_or(0_usize),0_usize,"queued intent cannot scroll an unexecuted reorder");
    let mut actual=harness.authority.session.projection.clone().expect("later actual order");actual.revision=9_i64;actual.interface.planning_status=String::new();
    let changed_city=actual.cities.iter_mut().find(|city|city.id=="c2").expect("city");changed_city.facts["queue"].as_array_mut().expect("queue").swap(index,page);changed_city.rush_buy_quote.as_mut().expect("quote").revision=9_i64;
    sync::apply(&mut harness.authority,&mut harness.state,&mut harness.ui,&mut MovementInputState::default(),Outcome::Snapshot(Box::new(actual)));
    assert_eq!(harness.ui.management.view.as_ref().and_then(|view|view.selections.get("queue")).map(String::as_str),Some(selected.as_str()),"same entry, never neighboring duplicate");
    assert!(visible_queue(&harness).contains(&selected),"actual received move brings selected entry into view");
    assert!(harness.ui.management.scroll_to("QueueGroup.QueueList",0_usize,&harness.state),"user deliberately scrolls away");
    let mut unchanged=harness.authority.session.projection.clone().expect("unrelated current update");unchanged.revision=10_i64;
    sync::apply(&mut harness.authority,&mut harness.state,&mut harness.ui,&mut MovementInputState::default(),Outcome::Snapshot(Box::new(unchanged)));
    assert_eq!(harness.ui.management.view.as_ref().and_then(|view|view.scroll.get("QueueGroup.QueueList")).copied(),Some(0_usize),"unchanged entry position cannot snap user scroll back");
    let mut removed=harness.authority.session.projection.clone().expect("removed entry update");removed.revision=11_i64;
    removed.cities.iter_mut().find(|city|city.id=="c2").expect("city").facts["queue"].as_array_mut().expect("queue").remove(page);
    sync::apply(&mut harness.authority,&mut harness.state,&mut harness.ui,&mut MovementInputState::default(),Outcome::Snapshot(Box::new(removed)));
    assert!(!harness.ui.management.view.as_ref().expect("view").selections.contains_key("queue"),"vanished selected ID never becomes new first row");
    assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"receipts/scroll never submit another order");
}

#[test]
fn build_navigation_space_never_reuses_hidden_stale_or_unauthorized_left_focus_and_empty_queue_has_no_phantom_row() {
    for mode in 0_u8..5_u8 {
        let mut harness=navigation_fixture();begin_row_press(&mut harness,"ItemsBox.UnitsList/UNIT_SETTLER");
        match mode {
            0_u8 => {},
            1_u8 => harness.authority.session.projection.as_mut().expect("session").revision=8_i64,
            2_u8 => { assert!(harness.ui.management.apply_local(crate::gameplay::ui_model::management::ManagerIntent::Select("build-category".to_owned(),"Wonders".to_owned())),"switch current source category");harness.ui.management.focused=Some("ItemsBox.UnitsList".to_owned()); },
            3_u8 => harness.ui.management.focused=Some("ItemsBox.UnitsList/removed-id".to_owned()),
            _ => {
                harness.state.viewer.as_mut().expect("viewer").cities.iter_mut().find(|city|city.id=="c2").expect("city").facts["queue"]=json!([]);
                harness.ui.management.view.as_mut().expect("view").selections.remove("queue");harness.ui.management.focused=Some("QueueGroup.QueueList".to_owned());
                manager_event(&mut harness,&key(Keycode::UP));
                assert!(!harness.ui.management.view.as_ref().expect("view").selections.contains_key("queue"),"empty current queue cannot manufacture a selected entry");
            }
        }
        let before=harness.state.viewer.clone();
        if mode==0_u8 {assert!(management_input::consume(&key(Keycode::SPACE),&mut harness.state,&mut harness.ui,&mut MovementInputState::default(),None),"modal input still consumed without an authority");}
        else {manager_event(&mut harness,&key(Keycode::SPACE));}
        assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"mode{mode}: no hidden/stale/rebased/no-session request");
        assert_eq!(harness.state.viewer,before,"invalid keyboard action cannot mutate queue/rules");
    }
}

/* Original source: EditQueue.cpp::{LoadModeCallback,ExitLoadMode} and
 * editqueue.ldl::BuildEditorWindow.{ItemsBox.LoadSaveButton,LoadBox,
 * LoadBox.CancelButton}. Loading is an original inline browser/preview mode,
 * not the unrelated generic comma-separated authoritative queue editor.
 */
#[test]
fn build_saved_load_enters_original_file_browser_without_raw_editor_or_command_and_cancel_restores_items() {
    let mut harness=buildable_fixture();begin_row_press(&mut harness,"ItemsBox.UnitsList/UNIT_SETTLER");
    let before=harness.state.viewer.clone();
    harness.ui.management.focused=Some("ItemsBox.LoadSaveButton".to_owned());manager_event(&mut harness,&key(Keycode::RETURN));
    assert!(harness.ui.operation.is_none(),"source Load opens LoadBox, never raw custom queue form");
    let widgets=harness.ui.management.widgets(&harness.state);
    let load=widgets.iter().find(|widget|widget.id=="LoadBox").expect("actual original LoadBox shown");
    assert_eq!((load.rect.width,load.rect.height),(435_u32,193_u32),"source mode bounds, current scaling unchanged");
    assert!(!widgets.iter().any(|widget|widget.id=="ItemsBox.UnitsList"),"original ItemsBox hidden while loading");
    assert!(widgets.iter().any(|widget|widget.id=="LoadBox.QueuesList") && widgets.iter().any(|widget|widget.id=="LoadBox.Contents"),"source file and preview lists visible");
    assert!(harness.ui.management.activate_id("LoadBox.LoadButton",&harness.state).is_none(),"missing explicit known file selection cannot apply anything");
    harness.ui.management.focused=Some("LoadBox.CancelButton".to_owned());manager_event(&mut harness,&key(Keycode::RETURN));
    let view=harness.ui.management.view.as_ref().expect("source Build stays open");
    assert_eq!((view.kind,view.city_id.as_deref()),(ManagerKind::Build,Some("c2")),"same original city/manager");
    assert_eq!(view.selections.get("buildable").map(String::as_str),Some("UNIT_SETTLER"),"source visible item category/selection returns");
    assert!(harness.ui.management.widgets(&harness.state).iter().any(|widget|widget.id=="ItemsBox.UnitsList"),"actual original item list restored");
    assert_eq!(harness.state.viewer,before,"mode entry/cancel never replaces queue");
    assert_eq!(harness.jobs.as_ref().expect("jobs").try_recv(),Err(TryRecvError::Empty),"no game operation on open/cancel");
}
