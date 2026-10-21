//! Public bounds constrain camera/input without inspecting hidden terrain.
use super::Camera;
use crate::gameplay::{model::GameState, server::ServerProjection};
use serde_json::json;

#[test]
fn camera_clamps_to_public_bounds_at_every_zoom_without_revealing_cells() {
    let viewer:ServerProjection=serde_json::from_value(json!({"revision":1_i64,"turn":1_u32,"world":{"min_x":0_u16,"min_y":0_u16,"width":20_u16,"height":12_u16},"tiles":[]})).expect("public extents");
    let state = GameState::from_server_projection(&viewer, 1, 1);
    let mut camera = Camera::centered_on_selected(&state);
    for zoom in 0_u8..3_u8 {
        for amount in [-100_000_i32, 100_000_i32] {
            camera.pan_pixels(amount, amount);
            camera.constrain(&state);
            let center = camera
                .screen_to_tile(400, 288)
                .expect("representable center");
            assert!(
                center.x < 20 && center.y < 12,
                "clamped public coordinate at zoom {zoom}"
            );
            assert!(
                state
                    .tiles
                    .iter()
                    .all(|tile| tile.fog == crate::gameplay::model::FogState::Hidden),
                "camera never discovers tiles"
            );
        }
        camera.zoom_in();
    }
}
