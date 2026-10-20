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
use crate::gameplay::model::{CityId, GameState};
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
pub(super) fn snapshot() -> Value {
    json!({
        "revision":17_i64,"turn":3_u32,"tiles":[{"id":"a","x":0_u16,"y":0_u16,"terrain":"TERRAIN_PLAINS"},{"id":"b","x":1_u16,"y":0_u16,"terrain":"TERRAIN_PLAINS"}],
        "units":[{"id":"guard-7","owner_id":"account-7","tile_id":"a","type_id":"UNIT_HOPLITE"}],
        "cities":[{"id":"city-7","owner_id":"account-7","tile_id":"a","population":3_u32,
            "queue":[{"id":"UNIT_HOPLITE","kind":"unit","cost":80_i32}]}],
        "research":{"target_id":"ADVANCE_BALLISTICS","available":["ADVANCE_AGRICULTURE"],"progress":25_i32},
        "diplomacy":{"science_percent":50_i64,"public_works_percent":10_i64,"workday":1_i64,"wages":-1_i64,"rations":0_i64},
        "legal_actions":[
            {"route":"queue-production","entity_id":"city-7","target_id":"UNIT_SETTLER","payload":{"city_id":"city-7","item":{"id":"UNIT_SETTLER","kind":"unit","cost":330_i32}}},
            {"route":"rush-buy","entity_id":"city-7","target_id":"UNIT_HOPLITE","payload":{"city_id":"city-7"}},
            {"route":"set-social-policy","entity_id":"account-7","fields":["workday","wages","rations"],"payload":{"workday":0_i64,"wages":0_i64,"rations":0_i64}},
            {"route":"set-science-rate","entity_id":"account-7","fields":["science_percent"],"payload":{"science_percent":100_i64}},
            {"route":"set-public-works-rate","entity_id":"account-7","fields":["public_works_percent"],"payload":{"public_works_percent":20_i64}}
        ]
    })
}

/// Creates a client pointing only at the test's ephemeral listener (or unused port).
pub(super) fn authority(port: u16, value: Value) -> RuntimeAuthority {
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
pub(super) fn ui() -> InGameUiModel {
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
pub(super) fn key(keycode: Keycode) -> Event {
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

/// Accessible control activation passes through the production dispatcher.
fn activate(ui: &mut InGameUiModel, state: &mut crate::gameplay::model::GameState, id: &str) {
    let intent = ui
        .management
        .activate_id(id, state)
        .expect("enabled original control");
    apply(intent, state, ui, None);
}

/// Link accessibility and pixel-hit descriptors must identify the same article.
fn follow_link(
    ui: &mut InGameUiModel,
    state: &mut crate::gameplay::model::GameState,
    target: &str,
) {
    let link = ui
        .management
        .widgets(state)
        .into_iter()
        .find(|widget| {
            widget.node.field("richtext") == Some("true")
                && widget.intent == Some(ManagerIntent::Article(target.to_owned()))
        })
        .expect("visible installed link");
    assert_eq!(
        ui.management.intent_at(
            link.rect.x.saturating_add(1_i32),
            link.rect.y.saturating_add(1_i32),
            state
        ),
        link.intent,
        "glyph and pointer targets agree"
    );
    activate(ui, state, &link.id);
}

#[test]
fn library_search_links_history_and_close_are_fileless_view_only_workflows() {
    let authority = authority(9_u16, snapshot());
    let mut state = state_from_authority_projection(
        &authority,
        authority.session.projection.as_ref().expect("viewer"),
    );
    state.popups.clear();
    let before = state.clone();
    let mut ui = ui();
    let mut movement = MovementInputState::default();
    ui.management.open(ManagerKind::Library, &state, None);
    assert_eq!(
        ui.management.focused.as_deref(),
        Some("SearchWord"),
        "original library immediately accepts search typing"
    );
    assert!(
        ui.management.library.entries(str::to_owned).is_empty(),
        "article display does not fill the index"
    );
    let text = Event::TextInput {
        timestamp: 0_u32,
        window_id: 0_u32,
        text: "UNIT_SETTLERx\n".to_owned(),
    };
    assert!(
        super::consume(&text, &mut state, &mut ui, &mut movement, None),
        "focused Unicode input consumed"
    );
    assert!(
        super::consume(
            &key(Keycode::BACKSPACE),
            &mut state,
            &mut ui,
            &mut movement,
            None
        ),
        "backspace consumed"
    );
    assert_eq!(
        ui.management.library.query, "UNIT_SETTLER",
        "control characters are not pasted"
    );
    assert!(
        ui.management.library.search_results.is_none(),
        "typing alone does not execute a search"
    );
    assert!(
        super::consume(
            &key(Keycode::RETURN),
            &mut state,
            &mut ui,
            &mut movement,
            None
        ),
        "Enter executes search, not End Turn"
    );
    activate(&mut ui, &mut state, "IndexSheet/UNIT_SETTLER");
    assert_eq!(
        ui.management.library.current.as_deref(),
        Some("UNIT_SETTLER"),
        "exact search result opened"
    );
    follow_link(&mut ui, &mut state, "CONCEPT_CITY");
    assert_eq!(
        ui.management.library.current.as_deref(),
        Some("CONCEPT_CITY"),
        "internal hyperlink follows its own target"
    );
    activate(&mut ui, &mut state, "BackButton");
    assert_eq!(
        ui.management.library.current.as_deref(),
        Some("UNIT_SETTLER"),
        "Back restores actual article"
    );
    activate(&mut ui, &mut state, "ForwardButton");
    assert_eq!(
        ui.management.library.current.as_deref(),
        Some("CONCEPT_CITY"),
        "Forward restores article without a new branch"
    );
    assert!(
        super::consume(
            &key(Keycode::ESCAPE),
            &mut state,
            &mut ui,
            &mut movement,
            None
        ),
        "Escape consumed"
    );
    assert!(ui.management.view.is_none(), "only local browser closes");
    assert_eq!(
        state, before,
        "no game mutation, real HTTP or research substitution"
    );
}

#[test]
fn article_navigation_from_build_returns_to_the_exact_parent_selection() {
    let authority = authority(9_u16, snapshot());
    let mut state = state_from_authority_projection(
        &authority,
        authority.session.projection.as_ref().expect("viewer"),
    );
    state.popups.clear();
    let mut ui = ui();
    ui.management
        .open(ManagerKind::Build, &state, Some("city-7"));
    activate(&mut ui, &mut state, "ItemsBox.UnitsList/UNIT_SETTLER");
    let parent = ui.management.view.clone();
    activate(&mut ui, &mut state, "LibraryButton");
    assert_eq!(
        ui.management.library.current.as_deref(),
        Some("UNIT_SETTLER"),
        "selected description opens its own article"
    );
    activate(&mut ui, &mut state, "OkButton");
    assert_eq!(
        ui.management.view, parent,
        "Close restores independent city/build/queue state"
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
pub(super) struct Request {
    /// Exact HTTP request line.
    pub(super) line: String,
    /// Parsed authenticated/revision-bound command envelope.
    pub(super) body: Value,
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
pub(super) fn respond(listener: &TcpListener, projection: &Value) -> Result<Request, String> {
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
pub(super) fn payload_after_validating_envelope(body: &Value) -> Value {
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
fn city_selector_handoff_submits_only_the_new_citys_exact_legal_production_payload() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("isolated listener");
    let port = listener.local_addr().expect("address").port();
    let mut initial = snapshot();
    initial["cities"].as_array_mut().expect("cities").push(
        json!({"id":"city-8","owner_id":"account-7","tile_id":"b","population":2_u32,"queue":[]}),
    );
    initial["legal_actions"].as_array_mut().expect("actions").push(json!({"route":"queue-production","entity_id":"city-8","target_id":"UNIT_SETTLER","payload":{"city_id":"city-8","item":{"id":"UNIT_SETTLER","kind":"unit","cost":440_i32}}}));
    let mut authority = authority(port, initial.clone());
    let mut state = state_from_authority_projection(
        &authority,
        authority.session.projection.as_ref().expect("viewer"),
    );
    state.popups.clear();
    let before = state.clone();
    let mut ui = ui();
    ui.management
        .open(ManagerKind::Build, &state, Some("city-7"));
    let next = ui
        .management
        .activate_id("SingleCityChooser.Next", &state)
        .expect("next city");
    apply(next, &mut state, &mut ui, Some(&mut authority));
    assert_eq!(state, before, "city browsing cannot issue game commands");
    let select = ui
        .management
        .activate_id("ItemsBox.UnitsList/UNIT_SETTLER", &state)
        .expect("explicit new-city buildable");
    apply(select, &mut state, &mut ui, None);
    let add = ui
        .management
        .activate_id("ItemsBox.AddButton", &state)
        .expect("new-city add");
    let mut after = initial;
    after["revision"] = json!(18_i64);
    let worker = thread::spawn(move || respond(&listener, &after));
    apply(add, &mut state, &mut ui, Some(&mut authority));
    let request = worker
        .join()
        .expect("stub thread")
        .expect("received command");
    assert_eq!(
        request.line, "POST /game/44/queue-production HTTP/1.1\r\n",
        "exact route"
    );
    assert_eq!(
        payload_after_validating_envelope(&request.body),
        json!({"city_id":"city-8","item":{"id":"UNIT_SETTLER","kind":"unit","cost":440_i32}}),
        "never send the old city identity or reuse its 330-point production payload"
    );
    assert_eq!(authority.session.revision(), 18_i64, "accepted revision");
    assert_eq!(
        ui.management
            .view
            .as_ref()
            .expect("view")
            .city_id
            .as_deref(),
        Some("city-8"),
        "receipt retains the new city identity"
    );
}

/// Confirms receipt replacement cannot select a same-label or different city.
fn assert_selected_city(ui: &InGameUiModel, city: &str) {
    assert_eq!(
        ui.management
            .view
            .as_ref()
            .and_then(|view| view.city_id.as_deref()),
        Some(city),
        "manager retains exact city"
    );
}

/// Receipts can contain effective values different from the explicit requested rate.
fn operation_receipt(route: &str, payload: &Value) -> Value {
    let mut after = snapshot();
    after["revision"] = json!(18_i64);
    if matches!(
        route,
        "set-social-policy" | "set-science-rate" | "set-public-works-rate"
    ) {
        if let Some(fields) = payload.as_object() {
            for (name, value) in fields {
                after["diplomacy"][name] = value.clone();
            }
        }
        if route == "set-science-rate" {
            after["diplomacy"]["science_percent"] = json!(57_i64);
        }
    }
    after
}

/// Native labels follow the receipt, never the outgoing requested value or sample.
fn assert_received_policy(ui: &InGameUiModel, state: &GameState, route: &str) {
    if route == "set-science-rate" {
        assert_eq!(
            ui.management
                .widgets(state)
                .iter()
                .find(|widget| widget.id
                    == "TabGroup.Tab1.TabPanel.Commerce.Modify.Secondary.Spinner/display")
                .map(|widget| widget.label.as_str()),
            Some("57"),
            "server-owned effective response, not the requested 60 or suggested 100"
        );
    }
}

#[test]
fn semantic_production_research_and_policies_send_exact_mocked_routes_revisions_and_payloads() {
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
        (
            ManagerKind::Empire,
            None,
            "TabGroup.Tab1.TabPanel.Production.Modify.Primary.Slider/value/3",
            "set-social-policy",
            json!({"workday":-1_i64,"wages":-1_i64,"rations":0_i64}),
        ),
        (
            ManagerKind::Empire,
            None,
            "TabGroup.Tab1.TabPanel.Commerce.Modify.Secondary.Spinner/increase",
            "set-science-rate",
            json!({"science_percent":60_i64}),
        ),
        (
            ManagerKind::Empire,
            None,
            "TabGroup.Tab1.TabPanel.Production.Modify.Secondary.Spinner/increase",
            "set-public-works-rate",
            json!({"public_works_percent":20_i64}),
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
        let after = operation_receipt(route, &expected);
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
        assert_selected_city(&ui, "city-7");
        assert_received_policy(&ui, &state, route);
        assert!(state.popups.is_empty(), "no hidden command failure");
    }
}
