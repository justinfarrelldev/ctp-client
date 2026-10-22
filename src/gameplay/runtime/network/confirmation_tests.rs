//! Original Build questions/queue guards with inert bounded command channels.
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
