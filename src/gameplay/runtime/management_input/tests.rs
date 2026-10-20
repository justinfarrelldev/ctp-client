//! Pure focus tests and an isolated HTTP stub: never contact a running game server.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use serde_json::{Value, json};

use super::{apply, open_pending_build};
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::model::CityId;
use crate::gameplay::runtime::authority::state_from_authority_projection;
use crate::gameplay::runtime::{
    Camera, MovementInputState, RuntimeAuthority, handle_event, replace_state_from_projection,
};
use crate::gameplay::server::{
    ServerClient, ServerConfig, ServerGameplaySession, ServerProjection,
};
use crate::gameplay::ui_model::InGameUiModel;
use crate::gameplay::ui_model::management::{ManagementUi, ManagerIntent, ManagerKind};

/// Exact viewer contract, using authenticated account ownership and non-index IDs.
fn snapshot() -> Value {
    json!({
        "revision":17_i64,"turn":3_u32,"tiles":[{"id":"a","x":0_u16,"y":0_u16,"terrain":"TERRAIN_PLAINS"},{"id":"b","x":1_u16,"y":0_u16,"terrain":"TERRAIN_PLAINS"}],
        "units":[{"id":"guard-7","owner_id":"account-7","tile_id":"a","type_id":"UNIT_HOPLITE"}],
        "cities":[{"id":"city-7","owner_id":"account-7","tile_id":"a","population":3_u32,
            "queue":[{"id":"UNIT_HOPLITE","kind":"unit","cost":80_i32}]}],
        "research":{"target_id":"ADVANCE_BALLISTICS","available":["ADVANCE_AGRICULTURE"],"progress":25_i32},
        "legal_actions":[
            {"route":"queue-production","entity_id":"city-7","target_id":"UNIT_SETTLER","payload":{"city_id":"city-7","item":{"id":"UNIT_SETTLER","kind":"unit","cost":330_i32}}},
            {"route":"rush-buy","entity_id":"city-7","target_id":"UNIT_HOPLITE","payload":{"city_id":"city-7"}}
        ]
    })
}

/// Creates a client pointing only at the test's ephemeral listener (or unused port).
fn authority(port: u16, value: Value) -> RuntimeAuthority {
    RuntimeAuthority {
        client: ServerClient::from_config(&ServerConfig {
            base_url: format!("http://127.0.0.1:{port}"),
            required: true,
        })
        .expect("stub endpoint"),
        session: ServerGameplaySession {
            game_id: 44_i64,
            session_id: 71_i64,
            account_id: 7_i64,
            projection: Some(serde_json::from_value(value).expect("viewer fixture")),
        },
        empire_count: 1_u8,
        seed: 42_u64,
    }
}

/// Reads installed source definitions without changing configuration or any files.
fn ui() -> InGameUiModel {
    InGameUiModel {
        management: ManagementUi::new(
            LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"))
                .expect("source layouts"),
        )
        .expect("resolve managers"),
        ..InGameUiModel::default()
    }
}

/// Explicit keyboard event, avoiding coordinate-dependent manager automation.
fn key(keycode: Keycode) -> Event {
    Event::KeyDown {
        timestamp: 0_u32,
        window_id: 0_u32,
        keycode: Some(keycode),
        scancode: None,
        keymod: Mod::NOMOD,
        repeat: false,
    }
}

#[test]
fn manager_focus_blocks_turn_orders_mouse_release_and_zoom_until_explicit_close() {
    let authority = authority(9_u16, snapshot());
    let mut state = state_from_authority_projection(
        &authority,
        authority.session.projection.as_ref().expect("viewer"),
    );
    state.popups.clear();
    let before = state.clone();
    let mut ui = ui();
    ui.management.open(ManagerKind::Science, &state, None);
    let mut camera = Camera::centered_on_selected(&state);
    let initial_camera = camera;
    let mut movement = MovementInputState::default();
    for event in [
        key(Keycode::A),
        key(Keycode::RETURN),
        key(Keycode::B),
        Event::MouseWheel {
            timestamp: 0_u32,
            window_id: 0_u32,
            which: 0_u32,
            x: 0_i32,
            y: 1_i32,
            direction: sdl2::mouse::MouseWheelDirection::Normal,
            precise_x: 0.0_f32,
            precise_y: 1.0_f32,
            mouse_x: 0_i32,
            mouse_y: 0_i32,
        },
    ] {
        assert_eq!(
            handle_event(
                &event,
                &mut state,
                &mut ui,
                &mut camera,
                &mut movement,
                None
            ),
            None,
            "manager keeps focus"
        );
    }
    assert_eq!(
        state, before,
        "no End Turn/settlement/movement through manager"
    );
    assert_eq!(
        camera, initial_camera,
        "wheel cannot zoom world through manager"
    );
    assert_eq!(
        handle_event(
            &key(Keycode::TAB),
            &mut state,
            &mut ui,
            &mut camera,
            &mut movement,
            None
        ),
        None,
        "Tab focus"
    );
    assert!(
        ui.management.focused.is_some(),
        "explicit enabled keyboard focus"
    );
    assert_eq!(
        handle_event(
            &key(Keycode::ESCAPE),
            &mut state,
            &mut ui,
            &mut camera,
            &mut movement,
            None
        ),
        None,
        "Escape closes view"
    );
    assert!(
        ui.management.view.is_none(),
        "closed without dismissing the game"
    );
}

#[test]
fn newly_founded_city_opens_its_build_manager_once_without_stealing_old_selection() {
    let authority = authority(9_u16, snapshot());
    let mut state = state_from_authority_projection(
        &authority,
        authority.session.projection.as_ref().expect("viewer"),
    );
    state.popups.clear();
    let selected = state.selected_unit;
    let mut next = snapshot();
    next["revision"] = json!(18_i64);
    next["cities"]
        .as_array_mut()
        .expect("cities")
        .push(json!({"id":"second-city","owner_id":"account-7","tile_id":"b"}));
    let projection: ServerProjection = serde_json::from_value(next).expect("receipt");
    replace_state_from_projection(&authority, &mut state, &projection);
    assert_eq!(
        state.selected_unit, selected,
        "exact unit identity survives receipt"
    );
    let mut ui = ui();
    open_pending_build(&mut state, &mut ui);
    let view = ui.management.view.as_ref().expect("automatic build window");
    assert_eq!(
        view.kind,
        ManagerKind::Build,
        "original settlement workflow"
    );
    assert_eq!(
        view.city_id.as_deref(),
        Some("second-city"),
        "new city, not first pre-existing city"
    );
    assert_eq!(
        ui.selected_city,
        Some(CityId::from_server_id("second-city")),
        "correct city selection"
    );
    assert!(
        state.city_build_menu.is_none(),
        "pending notification consumed"
    );
    assert!(ui.management.apply_local(ManagerIntent::Close), "close");
    open_pending_build(&mut state, &mut ui);
    assert!(
        ui.management.view.is_none(),
        "ordinary frames do not reopen the manager"
    );
}

#[test]
fn settle_uses_server_recommended_unique_identity_and_name_then_opens_that_city() {
    let mut initial = snapshot();
    initial["units"][0_usize]["type_id"] = json!("UNIT_SETTLER");
    initial["units"][0_usize]["effects"] = json!([{"kind":"settler"}]);
    initial["units"][0_usize]["tile_id"] = json!("b");
    let payload =
        json!({"unit_id":"guard-7","city_id":"client-city-guard-7","name":"City guard-7"});
    initial["legal_actions"] =
        json!([{"route":"found-city","entity_id":"guard-7","target_id":"b","payload":payload}]);
    let listener = TcpListener::bind("127.0.0.1:0").expect("isolated listener");
    let port = listener.local_addr().expect("stub address").port();
    let mut authority = authority(port, initial.clone());
    let mut state = state_from_authority_projection(
        &authority,
        authority.session.projection.as_ref().expect("viewer"),
    );
    state.popups.clear();
    let mut after = initial;
    after["revision"] = json!(18_i64);
    after["units"] = json!([]);
    after["cities"].as_array_mut().expect("cities").push(
        json!({"id":"client-city-guard-7","owner_id":"account-7","tile_id":"b","population":1_u32}),
    );
    after["legal_actions"] = json!([]);
    let worker = thread::spawn(move || respond(&listener, &after));
    assert!(
        crate::gameplay::runtime::apply_authoritative_action(
            &mut authority,
            &mut state,
            crate::gameplay::ui_model::InGameAction::Settle
        ),
        "settlement handled authoritatively"
    );
    let request = worker
        .join()
        .expect("stub thread")
        .expect("received settlement");
    assert_eq!(
        request.line, "POST /game/44/found-city HTTP/1.1\r\n",
        "exact founding route"
    );
    assert_eq!(
        payload_after_validating_envelope(&request.body),
        payload,
        "never repeat hardcoded New City for every settlement"
    );
    let mut ui = ui();
    open_pending_build(&mut state, &mut ui);
    assert_eq!(
        ui.management
            .view
            .as_ref()
            .and_then(|view| view.city_id.as_deref()),
        Some("client-city-guard-7"),
        "accepted city, not first old city"
    );
    assert_eq!(
        ui.active_tab,
        crate::gameplay::ui_model::ControlPanelTab::City,
        "original post-settlement city panel"
    );
}

/// Request observed by the isolated gateway stub, not by a real server/database.
struct Request {
    /// Exact HTTP request line.
    line: String,
    /// Parsed authenticated/revision-bound command envelope.
    body: Value,
}

/// Bounded read of one mock HTTP command, retaining content-length semantics.
fn read_request(stream: &mut TcpStream) -> Result<Request, String> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|error| error.to_string())?;
    let mut content_length = 0_usize;
    for _ in 0_u16..64_u16 {
        let mut header = String::new();
        reader
            .read_line(&mut header)
            .map_err(|error| error.to_string())?;
        if header == "\r\n" {
            break;
        }
        if let Some(value) = header.strip_prefix("Content-Length:") {
            content_length = value
                .trim()
                .parse::<usize>()
                .map_err(|error| error.to_string())?;
        }
    }
    if content_length > 0x4000_usize {
        return Err("oversized mock request".to_owned());
    }
    let mut bytes = vec![0_u8; content_length];
    reader
        .read_exact(&mut bytes)
        .map_err(|error| error.to_string())?;
    let body = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    Ok(Request { line, body })
}

/// Mock-only responder with a bounded wait, so a missing dispatch fails not hangs.
fn respond(listener: &TcpListener, projection: &Value) -> Result<Request, String> {
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    let started = Instant::now();
    let mut stream = loop {
        match listener.accept() {
            Ok((stream, _address)) => break stream,
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock
                    && started.elapsed() < Duration::from_secs(5_u64) =>
            {
                thread::sleep(Duration::from_millis(1_u64));
            }
            Err(error) => return Err(format!("mock received no command: {error}")),
        }
    };
    stream
        .set_read_timeout(Some(Duration::from_secs(2_u64)))
        .map_err(|error| error.to_string())?;
    let request = read_request(&mut stream)?;
    let body =
        json!({"revision":18_i64,"receipt_id":"mock-receipt","viewer_projection":projection})
            .to_string();
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|error| error.to_string())?;
    Ok(request)
}

/// Checks gateway metadata separately from operation-specific payload assertions.
fn payload_after_validating_envelope(body: &Value) -> Value {
    let mut fields = body.as_object().expect("command envelope").clone();
    assert_eq!(
        fields.remove("session_id"),
        Some(json!(71_i64)),
        "authenticated mock session"
    );
    assert_eq!(
        fields.remove("expected_revision"),
        Some(json!(17_i64)),
        "displayed revision, not next receipt revision"
    );
    for field in ["command_id", "idempotency_key"] {
        assert!(
            fields
                .remove(field)
                .and_then(|value| value.as_str().map(str::to_owned))
                .is_some_and(|value| !value.is_empty()),
            "required {field}"
        );
    }
    Value::Object(fields)
}

#[test]
fn semantic_add_rush_and_research_send_exact_mocked_routes_revisions_and_payloads() {
    for (kind, selection, button, route, expected) in [
        (
            ManagerKind::Build,
            Some("ItemsBox.UnitsList/UNIT_SETTLER"),
            "ItemsBox.AddButton",
            "queue-production",
            json!({"city_id":"city-7","item":{"id":"UNIT_SETTLER","kind":"unit","cost":330_i32}}),
        ),
        (
            ManagerKind::Build,
            None,
            "RushBuyButton",
            "rush-buy",
            json!({"city_id":"city-7"}),
        ),
        (
            ManagerKind::Research,
            Some("Background.SciAdvanceList/ADVANCE_AGRICULTURE"),
            "Background.BackButton",
            "set-research",
            json!({"advance_id":"ADVANCE_AGRICULTURE"}),
        ),
    ] {
        let listener = TcpListener::bind("127.0.0.1:0").expect("isolated listener");
        let port = listener.local_addr().expect("stub address").port();
        let mut authority = authority(port, snapshot());
        let mut state = state_from_authority_projection(
            &authority,
            authority.session.projection.as_ref().expect("viewer"),
        );
        state.popups.clear();
        let selected = state.selected_unit;
        let mut ui = ui();
        ui.management.open(kind, &state, Some("city-7"));
        if let Some(id) = selection {
            let intent = ui
                .management
                .activate_id(id, &state)
                .expect("explicit row selection");
            assert!(ui.management.apply_local(intent), "local selection");
        }
        let intent = ui
            .management
            .activate_id(button, &state)
            .expect("eligible command");
        let mut after = snapshot();
        after["revision"] = json!(18_i64);
        let worker = thread::spawn(move || respond(&listener, &after));
        apply(intent, &mut state, &mut ui, Some(&mut authority));
        let request = worker
            .join()
            .expect("stub thread")
            .expect("received command");
        assert_eq!(
            request.line,
            format!("POST /game/44/{route} HTTP/1.1\r\n"),
            "exact gateway route"
        );
        assert_eq!(
            payload_after_validating_envelope(&request.body),
            expected,
            "exact metadata/entity payload, no local substitutions"
        );
        assert_eq!(
            authority.session.revision(),
            18_i64,
            "accepted receipt revision"
        );
        assert_eq!(
            state.selected_unit, selected,
            "receipt preserves exact selected unit"
        );
        assert_eq!(
            ui.management
                .view
                .as_ref()
                .and_then(|view| view.city_id.as_deref()),
            Some("city-7"),
            "manager retains exact city"
        );
        assert!(state.popups.is_empty(), "no hidden command failure");
    }
}
