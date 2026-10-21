//! Native SDL readback for observed art/forms/messages/history; no capture files.
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::runtime::{Camera, test_support::with_canvas, textures::GameplayTextures};
use crate::gameplay::ui_model::{
    InGameUiModel,
    management::{ManagementUi, ManagerIntent, ManagerKind},
    operations::{InputKind, OperationDialog},
};
use crate::gameplay::{
    assets::tile::TileSet,
    model::{GameState, MapPoint},
    server::ServerProjection,
};
use sdl2::{
    pixels::{Color, PixelFormatEnum},
    rect::Rect,
    render::BlendMode,
};
use serde_json::json;
use std::path::Path;

#[test]
fn observed_ruin_art_matches_original_hut_and_remembered_pixels_without_hidden_leaks() {
    let result = with_canvas((800, 600), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let tiles = TileSet::load_from_installation(&root).map_err(|error| error.to_string())?;
        let camera = Camera::for_test_identity();
        let point = MapPoint::new(1, 1);
        for (stale, hidden) in [(false, false), (true, false), (false, true)] {
            let visible = json!({"id":"a","x":1_u16,"y":1_u16,"terrain":"TERRAIN_PLAINS","stale":stale,"ruin":true});
            let records = if hidden { Vec::new() } else { vec![visible] };
            let projection: ServerProjection =
                serde_json::from_value(json!({"revision":1_i64,"turn":1_u32,"tiles":records}))
                    .map_err(|error| error.to_string())?;
            let state = GameState::from_server_projection(&projection, 1, 1);
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            if !hidden {
                let mut source = tiles
                    .improvement_image(150)
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "original hut absent".to_owned())?;
                if stale {
                    crate::gameplay::assets::pixel::shade_explored(&mut source.data, true);
                }
                let mut expected = creator
                    .create_texture_static(PixelFormatEnum::RGBA32, source.width, source.height)
                    .map_err(|error| error.to_string())?;
                expected.set_blend_mode(BlendMode::Blend);
                expected
                    .update(None, &source.data, source.pitch)
                    .map_err(|error| error.to_string())?;
                let (x, y) = camera.tile_to_screen(point);
                canvas.copy(
                    &expected,
                    None,
                    Rect::new(x, y, source.width, source.height),
                )?;
            }
            let expected = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            super::observations::draw(canvas, &mut textures, &state, &camera, point)
                .map_err(|error| error.to_string())?;
            let actual = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
            if actual != expected {
                return Err(format!("hut pixels differ: stale={stale}, hidden={hidden}"));
            }
        }
        Ok(())
    });
    assert!(result.is_ok(), "observed hut readback: {result:?}");
}

#[test]
fn source_message_ranger_and_eye_share_semantic_descriptors_and_private_coordinates() {
    use_message_controls();
}

/// Resolves native pointer targets by semantic IDs rather than fixed click coordinates.
fn use_message_controls() {
    use crate::gameplay::runtime::{MovementInputState, message_input};
    use crate::gameplay::ui_model::ControlPanelTab;
    use sdl2::{
        event::Event,
        mouse::{MouseButton, MouseState},
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
    let notices=(0_u32..8_u32).map(|index| json!({"id":format!("n{index}"),"kind":"ruin.explored","reward_kind":"gold","amount":250_i64,"turn":1_u32,"tile_id":if index == 7 {"absent"} else {"a"}})).collect::<Vec<_>>();
    let projection:ServerProjection=serde_json::from_value(json!({"revision":1_i64,"turn":1_u32,"world":{"width":20_u16,"height":12_u16},"tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"plains"}],"calendar":{"version":"ctp2-timescale-v1","turn":1_u32,"year":-4000_i64},"notifications":notices})).expect("in-memory notification fixture");
    let mut state = GameState::from_server_projection(&projection, 1, 1);
    let mut ui = InGameUiModel {
        management: ManagementUi::new(LayoutBook::load(&root).expect("source layouts read-only"))
            .expect("source panel cache"),
        ..InGameUiModel::default()
    };
    let mut movement = MovementInputState::default();
    message_input::refresh(None, &mut state, &mut ui, &mut movement);
    assert_eq!(
        ui.messages.unread(),
        7,
        "automatic ruin popup marks only its result read"
    );
    assert_eq!(
        ui.messages.dates.get(&1).map(String::as_str),
        Some("4000BC"),
        "only observed calendar is cached"
    );
    ui.messages.selected = None;
    ui.active_tab = ControlPanelTab::Messages;
    let mut camera = Camera::for_test_identity();
    let widgets = ui.management.panel_widgets(&state, &ui);
    let down = widgets
        .iter()
        .find(|widget| widget.id == "MessageListBox/down")
        .expect("source down arrow");
    let click = Event::MouseButtonDown {
        timestamp: 0,
        window_id: 0,
        which: 0,
        mouse_btn: MouseButton::Left,
        clicks: 1,
        x: down.rect.x.saturating_add(1),
        y: down.rect.y.saturating_add(1),
    };
    assert!(
        message_input::consume(&click, &state, &mut ui, &mut camera),
        "native arrow consumes world input"
    );
    assert_eq!(ui.messages.start, 1, "arrow and semantic scrolling agree");
    let shifted = ui.management.panel_widgets(&state, &ui);
    let thumb = shifted
        .iter()
        .find(|widget| widget.id == "MessageListBox/ranger/thumb")
        .expect("source thumb");
    let grab = Event::MouseButtonDown {
        timestamp: 0,
        window_id: 0,
        which: 0,
        mouse_btn: MouseButton::Left,
        clicks: 1,
        x: thumb.rect.x.saturating_add(1),
        y: thumb.rect.y.saturating_add(1),
    };
    assert!(
        message_input::consume(&grab, &state, &mut ui, &mut camera),
        "thumb captures its own pointer"
    );
    let drag = Event::MouseMotion {
        timestamp: 0,
        window_id: 0,
        which: 0,
        mousestate: MouseState::from_sdl_state(1),
        x: thumb.rect.x,
        y: 1_000_i32,
        xrel: 0_i32,
        yrel: 0_i32,
    };
    assert!(
        message_input::consume(&drag, &state, &mut ui, &mut camera),
        "drag remains exclusive outside the list"
    );
    assert_eq!(
        ui.messages.start, 3,
        "thumb clamps to actual retained results, not empty pages"
    );
    assert!(
        message_input::activate("message-scroll-up", &state, &mut ui, &mut camera),
        "accessible scroll action"
    );
    check_message_eye(&state, &mut ui, &mut camera);
}

/// Popup capture and location navigation never infer an unavailable tile.
fn check_message_eye(state: &GameState, ui: &mut InGameUiModel, camera: &mut Camera) {
    use crate::gameplay::runtime::message_input;
    ui.messages.open("n7");
    assert!(
        ui.messages.drag.is_none(),
        "opening a popup releases a thumb interrupted by an asynchronous result"
    );
    let before = *camera;
    assert!(
        !message_input::activate("message-eye", state, ui, camera),
        "absent location never triggers a geography lookup"
    );
    assert_eq!(
        *camera, before,
        "unknown result location cannot move the camera"
    );
    ui.messages.open("n0");
    assert!(
        message_input::activate("message-eye", state, ui, camera),
        "known observed location can center"
    );
    assert_eq!(
        camera.screen_to_tile(400, 288),
        Some(MapPoint::new(1, 1)),
        "eye centers the actual known tile"
    );
    assert!(
        ui.messages.selected.is_none(),
        "eye dismisses the window, not the log entry"
    );
    assert_eq!(
        crate::gameplay::ui_model::messages::text(
            ui.messages
                .items
                .iter()
                .find(|notice| notice.id == "n0")
                .expect("result remains in log"),
            state,
            &ui.management.book
        ),
        "Exploration reward: 250 gold.",
        "exact confirmed reward text"
    );
}

#[test]
fn native_forms_messages_and_own_history_are_real_nonempty_source_chrome() {
    let result = with_canvas((800, 600), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let projection:ServerProjection=serde_json::from_value(json!({"revision":1_i64,"turn":2_u32,"tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"plains"}],"cities":[{"id":"c","owner_id":"human","tile_id":"a","name":"Rome"}],"legal_actions":[{"route":"rename-city","entity_id":"c","payload":{"city_id":"c","name":"Rome"}}],"notifications":[{"id":"n","kind":"production.completed","turn":2_u32,"content_id":"UNIT_HOPLITE","city_id":"c","tile_id":"a"}],"statistics":[{"turn":1_u32,"score":0_i64,"city_count":1_u32,"unit_count":1_u32,"population":1_u64},{"turn":2_u32,"score":10_i64,"city_count":1_u32,"unit_count":2_u32,"population":2_u64}]})).map_err(|error| error.to_string())?;
        let state = GameState::from_server_projection(&projection, 1, 1);
        let mut ui = InGameUiModel {
            management: ManagementUi::new(
                LayoutBook::load(&root).map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?,
            ..InGameUiModel::default()
        };
        ui.operation = OperationDialog::input(&state, "c", InputKind::Rename);
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        super::operations::draw(canvas, &textures, &state, &ui, None)
            .map_err(|error| error.to_string())?;
        let field = OperationDialog::field();
        let field_pixel = canvas.read_pixels(
            Some(Rect::new(
                field.x.saturating_add(190),
                field.y.saturating_add(5),
                1,
                1,
            )),
            PixelFormatEnum::RGBA32,
        )?;
        if field_pixel != vec![255_u8, 255, 255, 255] {
            return Err("native original-size input field missing".to_owned());
        }
        ui.operation = None;
        ui.messages.ingest(&projection.interface.notifications);
        ui.messages.open("n");
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        super::messages::draw(canvas, &mut textures, &state, &ui, None)
            .map_err(|error| error.to_string())?;
        let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        if !pixels
            .chunks_exact(4)
            .any(|pixel| pixel[0] > 150 && pixel[1] > 80 && pixel[2] < 120)
        {
            return Err("original gold message frame missing".to_owned());
        }
        ui.messages.selected = None;
        ui.management.open(ManagerKind::Statistics, &state, None);
        ui.management.apply_local(ManagerIntent::Select(
            "tab:TabGroup".to_owned(),
            "Tab3".to_owned(),
        ));
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        super::history::draw(canvas, &state, &ui, None).map_err(|error| error.to_string())?;
        let graph_pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        if !graph_pixels
            .chunks_exact(4)
            .any(|pixel| pixel[0] == 40 && pixel[1] == 90 && pixel[2] == 255)
        {
            return Err("actual own history line missing".to_owned());
        }
        Ok(())
    });
    assert!(result.is_ok(), "native v2 UI readback: {result:?}");
}
