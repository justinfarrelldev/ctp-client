//! A real-format server start must reach selection, terrain, actors and radar.
//!
//! Wire data is synthetic and read-only art is installed locally. All canvas
//! comparisons stay in memory; no live server or screenshot file is used.

use crate::gameplay::model::{
    FogState, GameState, MapPoint, PlayerId, TerrainKind, UnitId, UnitKind,
};
use crate::gameplay::runtime::authority::{
    RuntimeAuthority, server_unit_id_for_unit, state_from_authority_projection,
};
use crate::gameplay::runtime::controls::{ControlIntent, intent_for_label};
use crate::gameplay::runtime::test_support::with_canvas;
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::runtime::{Camera, MovementInputState};
use crate::gameplay::server::test_support::{
    starting_resync_event, synthetic_starting_resync_event,
};
use crate::gameplay::server::{
    ServerClient, ServerConfig, ServerGameplaySession, ServerProjection,
};
use crate::gameplay::ui_model::{ControlPanelTab, InGameAction};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::path::Path;

#[test]
fn decoded_starting_snapshot_selects_and_renders_settler_terrain_and_minimap() {
    for event in [starting_resync_event(), synthetic_starting_resync_event()] {
        let result = with_canvas((800_u32, 600_u32), |canvas| {
            let projection =
                ServerProjection::from_resync_event(&event).map_err(|error| error.to_string())?;
            // Constructing a client parses configuration only; this test never sends requests.
            let authority = RuntimeAuthority {
                client: ServerClient::from_config(&ServerConfig::disabled_for_test())
                    .map_err(|error| error.to_string())?,
                session: ServerGameplaySession {
                    game_id: 44_i64,
                    session_id: 11_i64,
                    account_id: 7_i64,
                    projection: Some(projection.clone()),
                },
                empire_count: 2_u8,
                seed: 42_u64,
            };
            let state = state_from_authority_projection(&authority, &projection);
            let camera = validate_starting_state(&authority, &state)?;
            let creator = canvas.texture_creator();
            let mut textures = GameplayTextures::load(
                &Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"),
                &creator,
                true,
            )
            .map_err(|error| error.to_string())?;
            clear(canvas);
            super::actors::draw_units(
                canvas,
                &mut textures,
                &state,
                &camera,
                &MovementInputState::default(),
            )
            .map_err(|error| error.to_string())?;
            let actors = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
            if colored_pixels(&actors) < 300_usize {
                return Err("server starting actor/selection is still invisible".to_owned());
            }
            clear(canvas);
            super::draw_map(
                canvas,
                &mut textures,
                &state,
                &camera,
                &MovementInputState::default(),
            )
            .map_err(|error| error.to_string())?;
            let world = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
            if colored_pixels(&world) < 10_000_usize {
                return Err("server starting area is still black".to_owned());
            }
            clear(canvas);
            super::radar::draw_radar(canvas, &textures, &state)
                .map_err(|error| error.to_string())?;
            let radar = canvas.read_pixels(
                Rect::new(12_i32, 416_i32, 174_u32, 174_u32),
                PixelFormatEnum::RGBA32,
            )?;
            let color = super::tile_color(TerrainKind::Plains);
            if !radar
                .chunks_exact(4_usize)
                .any(|pixel| pixel[..3_usize] == [color.r, color.g, color.b])
            {
                return Err("server-projected terrain is absent from minimap".to_owned());
            }
            Ok(())
        });
        assert!(
            result.is_ok(),
            "starting projection pipeline regression: {result:?}"
        );
    }
}

/// Verifies wire facts reach owner mapping, command identity and initial camera.
fn validate_starting_state(
    authority: &RuntimeAuthority,
    state: &GameState,
) -> Result<Camera, String> {
    let selected = state
        .selected_unit()
        .ok_or_else(|| "decoded start has no selected settler".to_owned())?;
    if selected.owner != PlayerId::Human
        || selected.kind != UnitKind::Settler
        || selected.position != MapPoint::new(41_u16, 27_u16)
    {
        return Err("server starter ownership/type/spawn position was lost".to_owned());
    }
    if state.units.len() != 1_usize || selected.id != UnitId::from_server_id("settler-7-1") {
        return Err("missing/replaced authoritative unit identity".to_owned());
    }
    if server_unit_id_for_unit(authority, selected.id) != "settler-7-1" {
        return Err("selected settler cannot target the real server entity".to_owned());
    }
    if state
        .tiles
        .iter()
        .filter(|tile| tile.fog == FogState::Visible)
        .count()
        != 13_usize
    {
        return Err("starter sight must be the thirteen server-projected cells".to_owned());
    }
    if state
        .tile(MapPoint::new(0_u16, 0_u16))
        .is_none_or(|tile| tile.fog != FogState::Hidden)
    {
        return Err("unknown world was fabricated".to_owned());
    }
    if intent_for_label(state, ControlPanelTab::Unit, "Move") != Some(ControlIntent::ActivateUnit) {
        return Err("decoded settler's actual Move control is disabled".to_owned());
    }
    if intent_for_label(state, ControlPanelTab::Unit, "Settle")
        != Some(ControlIntent::Action(InGameAction::Settle))
    {
        return Err(
            "server settler capability did not enable the original Settle control".to_owned(),
        );
    }
    let camera = Camera::centered_on_selected(state);
    if camera.tile_to_screen(selected.position) != (353_i32, 252_i32) {
        return Err(
            "real starting position was not centered in native gameplay viewport".to_owned(),
        );
    }
    Ok(camera)
}

/// Clears between independent in-memory widget readbacks.
fn clear(canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
    canvas.clear();
}

/// Counts nonblack RGB pixels, ignoring the opaque buffer's alpha channel.
fn colored_pixels(pixels: &[u8]) -> usize {
    pixels
        .chunks_exact(4_usize)
        .filter(|pixel| pixel[..3_usize] != [0_u8; 3_usize])
        .count()
}
