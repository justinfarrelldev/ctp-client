//! Native source research journeys with inert production-shaped command channels.
//! No sockets, workers, files, service calls, or coordinate-coded target choices.
/// Actual panel/shortcut input entry, including fresh games with no research target.
mod entry_tests;
use super::{Network, Outcome};
use crate::gameplay::{
    assets::layout::LayoutBook,
    model::{GameState, NetworkStatus},
    runtime::{
        MovementInputState, RuntimeAuthority, authority::state_from_authority_projection,
        management_input, sync,
    },
    server::{ServerClient, ServerConfig, ServerGameplaySession, envelope::PreparedCommand},
    ui_model::{
        InGameUiModel,
        management::{ManagementUi, ManagerIntent, ManagerKind},
    },
};
use alloc::sync::Arc;
use core::sync::atomic::AtomicBool;
use sdl2::{
    event::Event,
    keyboard::{Keycode, Mod},
    mouse::MouseButton,
};
use serde_json::json;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, TryRecvError};

/// UI and authority with observable bounded channels; transport is never started.
struct Harness {
    /// Inert worker sender is the real production enqueue boundary.
    authority: RuntimeAuthority,
    /// Complete received facts, unchanged by selection or queued intent.
    state: GameState,
    /// Real cached source managers/descriptors read from the installed fixture.
    ui: InGameUiModel,
    /// Pure pointer/gesture capture, never a real operating-system pointer.
    movement: MovementInputState,
    /// None only when testing a disconnected command channel.
    jobs: Option<Receiver<PreparedCommand>>,
}

/// Accurate current-research/available/phase contract with a non-first candidate.
fn fixture() -> Harness {
    let projection = serde_json::from_value(json!({
        "revision":7_i64,"turn":1_u32,"contract":{"version":2_u32},
        "research":{"target_id":"ADVANCE_BALLISTICS","goal_id":"ADVANCE_RELIGION","progress":25_i64,"advances":["ADVANCE_AGRICULTURE"],"available":["ADVANCE_BRONZE_WORKING","ADVANCE_IRON_WORKING","ADVANCE_BALLISTICS"]},
        "rules":{"advances":[{"id":"ADVANCE_BRONZE_WORKING","cost":50_i64},{"id":"ADVANCE_IRON_WORKING","cost":120_i64},{"id":"ADVANCE_BALLISTICS","cost":570_i64}]},
        "legal_actions":[{"route":"pass","payload":{}}]
    })).expect("viewer-private current research/planning schema");
    let (sender, jobs) = mpsc::sync_channel(1_usize);
    let (result_sink, receiver) = mpsc::sync_channel(1_usize);
    let authority = RuntimeAuthority {
        io: Some(Network {
            jobs: sender,
            results: receiver,
            delivery: Arc::new(super::workers::Delivery::new(7_i64, result_sink)),
            stop: Arc::new(AtomicBool::new(false)),
        }),
        client: ServerClient::from_config(&ServerConfig {
            base_url: "http://127.0.0.1:1".to_owned(),
            required: true,
        })
        .expect("unused endpoint, never connected"),
        session: ServerGameplaySession {
            roster:crate::gameplay::server::roster::PlayerRoster::default(),
            game_id: 44_i64,
            session_id: 71_i64,
            account_id: 7_i64,
            projection: Some(projection),
        },
        empire_count: 1_u8,
        seed: 1_u64,
    };
    let mut state = state_from_authority_projection(
        &authority,
        authority
            .session
            .projection
            .as_ref()
            .expect("complete fixture"),
    );
    state.popups.clear();
    let book = LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"))
        .expect("read-only source definitions");
    let ui = InGameUiModel {
        management: ManagementUi::new(book).expect("source manager trees"),
        ..InGameUiModel::default()
    };
    Harness {
        authority,
        state,
        ui,
        movement: MovementInputState::default(),
        jobs: Some(jobs),
    }
}

/// Native semantic keyboard event, no OS input injection or coordinates.
fn key(code: Keycode) -> Event {
    Event::KeyDown {
        timestamp: 0_u32,
        window_id: 0_u32,
        keycode: Some(code),
        scancode: None,
        keymod: Mod::NOMOD,
        repeat: false,
    }
}

/// Native keyboard activation of an exact semantic ID, not an arbitrary first match.
fn activate(harness: &mut Harness, id: &str) {
    harness.ui.management.focused = Some(id.to_owned());
    assert!(
        management_input::consume(
            &key(Keycode::RETURN),
            &mut harness.state,
            &mut harness.ui,
            &mut harness.movement,
            Some(&mut harness.authority)
        ),
        "native modal keyboard activation"
    );
}

/// Source `ChangeResearch` opens its actual chooser from a selected discovery.
fn open(harness: &mut Harness) {
    harness
        .ui
        .management
        .open(ManagerKind::Science, &harness.state, None);
    assert!(
        harness.ui.management.apply_local(ManagerIntent::Select(
            "advance".to_owned(),
            "ADVANCE_AGRICULTURE".to_owned()
        )),
        "parent known selection"
    );
    activate(harness, "ChangeResearch");
    assert_eq!(
        harness.ui.management.view.as_ref().map(|view| view.kind),
        Some(ManagerKind::Research),
        "source entry opens chooser"
    );
}

/// Select an explicit different/non-first canonical candidate through native input.
fn select(harness: &mut Harness) {
    activate(harness, "Background.SciAdvanceList/ADVANCE_IRON_WORKING");
}

/// Request inspection preserves exact payload/envelope identities without a service.
fn assert_request(command: PreparedCommand) {
    assert_eq!(
        command.path, "/game/44/set-research",
        "registered research route"
    );
    let mut body = command.body;
    assert_eq!(body["session_id"], 71_i64, "exact membership");
    assert_eq!(
        body["expected_revision"], 7_i64,
        "captured displayed revision"
    );
    for field in ["idempotency_key", "command_id"] {
        assert!(
            body[field].as_str().is_some_and(|value| !value.is_empty()),
            "immutable nonempty {field}"
        );
        body.as_object_mut().expect("envelope object").remove(field);
    }
    body.as_object_mut().expect("object").remove("session_id");
    body.as_object_mut()
        .expect("object")
        .remove("expected_revision");
    assert_eq!(
        body,
        json!({"advance_id":"ADVANCE_IRON_WORKING"}),
        "no first/default target, cost, goal or executable metadata"
    );
}

#[test]
fn native_cancel_and_escape_restore_exact_science_parent_without_any_order_or_research_change() {
    for cancel in [true, false] {
        let mut harness = fixture();
        let before = harness.state.clone();
        open(&mut harness);
        let parent = harness.ui.management.parents.last().cloned();
        select(&mut harness);
        if cancel {
            activate(&mut harness, "Background.CancelButton");
        } else {
            assert!(
                management_input::consume(
                    &key(Keycode::ESCAPE),
                    &mut harness.state,
                    &mut harness.ui,
                    &mut harness.movement,
                    Some(&mut harness.authority)
                ),
                "original chooser Escape handler"
            );
        }
        assert_eq!(
            harness.ui.management.view, parent,
            "cancel/ESC restore exact known selection, not another target/window"
        );
        assert_eq!(
            harness.state, before,
            "never replay old target or apply candidate research on cancel"
        );
        assert_eq!(
            harness
                .jobs
                .as_ref()
                .expect("observable channel")
                .try_recv(),
            Err(TryRecvError::Empty),
            "zero gameplay operations"
        );
    }
}

#[test]
fn native_ok_closes_only_chooser_on_actual_enqueue_and_queued_receipt_never_executes_selected_research()
 {
    let mut harness = fixture();
    open(&mut harness);
    let parent = harness.ui.management.parents.last().cloned();
    let received = harness
        .state
        .viewer
        .as_ref()
        .expect("viewer")
        .interface
        .research
        .clone();
    select(&mut harness);
    activate(&mut harness, "Background.BackButton");
    assert_eq!(
        harness.ui.management.view, parent,
        "source OK returns to Science only after accepted local enqueue"
    );
    assert_eq!(
        harness.state.network,
        NetworkStatus::Pending,
        "enqueue is pending, not a gameplay outcome"
    );
    assert_request(
        harness
            .jobs
            .as_ref()
            .expect("jobs")
            .try_recv()
            .expect("one exact request"),
    );
    assert_eq!(
        harness
            .state
            .viewer
            .as_ref()
            .expect("viewer")
            .interface
            .research,
        received,
        "don't change target/goal/progress/known set optimistically"
    );
    let mut queued = harness
        .authority
        .session
        .projection
        .clone()
        .expect("complete receipt");
    queued.revision = 8_i64;
    queued.interface.planning_status = "queued".to_owned();
    sync::apply(
        &mut harness.authority,
        &mut harness.state,
        &mut harness.ui,
        &mut harness.movement,
        Outcome::Receipt(Box::new(queued)),
    );
    assert_eq!(
        harness.state.network,
        NetworkStatus::Ready,
        "receipt acknowledges request only"
    );
    assert_eq!(
        harness
            .state
            .viewer
            .as_ref()
            .expect("viewer")
            .interface
            .research,
        received,
        "queued current snapshot retains actual Ballistics/old goal and spending"
    );
    assert_eq!(
        harness.ui.management.view, parent,
        "receipt preserves exact parent selection"
    );
    assert_eq!(
        harness.jobs.as_ref().expect("jobs").try_recv(),
        Err(TryRecvError::Empty),
        "no replay or extra operation after receipt"
    );
}

#[test]
fn native_double_click_uses_exact_visible_row_not_current_or_first_candidate_and_submits_once() {
    let mut harness = fixture();
    open(&mut harness);
    let parent = harness.ui.management.parents.last().cloned();
    let row = harness
        .ui
        .management
        .widgets(&harness.state)
        .into_iter()
        .find(|widget| widget.id == "Background.SciAdvanceList/ADVANCE_IRON_WORKING")
        .expect("exact presented row");
    assert!(
        row.rect.width > 4_u32 && row.rect.height > 4_u32,
        "actual visible source row"
    );
    let event = Event::MouseButtonDown {
        timestamp: 0_u32,
        window_id: 0_u32,
        which: 0_u32,
        mouse_btn: MouseButton::Left,
        clicks: 2_u8,
        x: row.rect.x.saturating_add(2_i32),
        y: row.rect.y.saturating_add(2_i32),
    };
    assert!(
        management_input::consume(
            &event,
            &mut harness.state,
            &mut harness.ui,
            &mut harness.movement,
            Some(&mut harness.authority)
        ),
        "native source list double-click callback"
    );
    assert_request(
        harness
            .jobs
            .as_ref()
            .expect("jobs")
            .try_recv()
            .expect("double-click confirms its explicit row"),
    );
    assert_eq!(
        harness.ui.management.view, parent,
        "double-click returns original parent"
    );
    let release = Event::MouseButtonUp {
        timestamp: 0_u32,
        window_id: 0_u32,
        which: 0_u32,
        mouse_btn: MouseButton::Left,
        clicks: 2_u8,
        x: row.rect.x,
        y: row.rect.y,
    };
    assert!(
        management_input::consume(
            &release,
            &mut harness.state,
            &mut harness.ui,
            &mut harness.movement,
            Some(&mut harness.authority)
        ),
        "release remains modal, never a world move"
    );
    assert_eq!(
        harness.jobs.as_ref().expect("jobs").try_recv(),
        Err(TryRecvError::Empty),
        "mouse up cannot submit duplicate or map order"
    );
}

#[test]
fn full_or_disconnected_enqueue_keeps_chooser_and_received_facts_without_claiming_success() {
    for disconnected in [false, true] {
        let mut harness = fixture();
        open(&mut harness);
        select(&mut harness);
        let before = harness.state.viewer.clone();
        if disconnected {
            drop(harness.jobs.take());
        } else {
            let filler = PreparedCommand::new(&harness.authority.session, "pass", json!({}))
                .expect("in-memory queue sentinel");
            harness
                .authority
                .io
                .as_ref()
                .expect("inert network")
                .submit(filler)
                .expect("fill bounded channel without gameplay dispatch");
        }
        activate(&mut harness, "Background.BackButton");
        assert_eq!(
            harness.ui.management.view.as_ref().map(|view| view.kind),
            Some(ManagerKind::Research),
            "failed dispatch must not close source confirmation UI"
        );
        assert_eq!(
            harness.state.network,
            NetworkStatus::Ready,
            "no accepted job, no false Pending"
        );
        assert_eq!(
            harness.state.viewer, before,
            "failed channel can't change any received fact"
        );
        assert!(
            harness
                .state
                .popups
                .iter()
                .any(|popup| popup.title == "Order not submitted"),
            "actionable real failure remains visible"
        );
        if let Some(jobs) = harness.jobs.as_ref() {
            assert!(
                jobs.try_recv()
                    .is_ok_and(|command| command.path == "/game/44/pass"),
                "only original filler occupies full queue"
            );
            assert_eq!(
                jobs.try_recv(),
                Err(TryRecvError::Empty),
                "failed research enqueue is not secretly accepted"
            );
        }
    }
}

#[test]
fn captured_confirmation_stale_missing_session_pending_removed_or_lost_viewer_stays_open_without_order()
 {
    for mode in 0_u8..6_u8 {
        let mut harness = fixture();
        open(&mut harness);
        select(&mut harness);
        let intent = harness
            .ui
            .management
            .activate_id("Background.BackButton", &harness.state)
            .expect("captured explicit current confirmation");
        match mode {
            0_u8 => harness.state.viewer.as_mut().expect("viewer").revision = 8_i64,
            1_u8 => {
                harness
                    .authority
                    .session
                    .projection
                    .as_mut()
                    .expect("session viewer")
                    .revision = 8_i64;
            }
            2_u8 => harness.state.network = NetworkStatus::Pending,
            3_u8 => harness
                .state
                .viewer
                .as_mut()
                .expect("viewer")
                .interface
                .research
                .as_mut()
                .expect("research")
                .available
                .retain(|id| id != "ADVANCE_IRON_WORKING"),
            4_u8 => {}
            _ => harness.state.viewer = None,
        }
        let received = harness.state.viewer.clone();
        management_input::apply(
            intent,
            &mut harness.state,
            &mut harness.ui,
            (mode != 4_u8).then_some(&mut harness.authority),
        );
        assert_eq!(
            harness.ui.management.view.as_ref().map(|view| view.kind),
            Some(ManagerKind::Research),
            "invalid confirmation must not close/retarget chooser"
        );
        assert_eq!(
            harness.state.viewer, received,
            "no local research result from stale/absent authority"
        );
        assert_eq!(
            harness.jobs.as_ref().expect("jobs").try_recv(),
            Err(TryRecvError::Empty),
            "no hidden retry/new envelope or inferred command"
        );
    }
}
