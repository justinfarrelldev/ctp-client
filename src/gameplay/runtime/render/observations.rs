//! Observed overlays and political boundaries; never query an undiscovered tile.
use crate::gameplay::model::{FogState, GameState, MapPoint, TileFeature};
use crate::gameplay::runtime::{Camera, textures::GameplayTextures};
use crate::gameplay::{GameplayError, runtime_error};
use sdl2::{
    rect::Rect,
    render::Canvas,
    video::Window,
};

/// Draws only server-observed construction, ruins and permitted boundary edges.
pub(super) fn draw(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
    camera: &Camera,
    point: MapPoint,
) -> Result<(), GameplayError> {
    let Some(tile) = state
        .tile(point)
        .filter(|tile| tile.fog != FogState::Hidden)
    else {
        return Ok(());
    };
    let explored = tile.fog == FogState::Explored;
    let (x, y) = camera.tile_to_screen(point);
    let zoom = camera.tile_width().div_euclid(94).max(1);
    if tile.feature == Some(TileFeature::Ruin) {
        // tiledraw.cpp selects two original hut variants by x parity.
        overlay(
            canvas,
            textures,
            if point.x & 1 == 1 { 150 } else { 151 },
            explored,
            x,
            y,
            zoom,
        )?;
    }
    if let Some(viewer) = state.viewer.as_ref() {
        textures.observe_tiles(viewer);
        if let Some(source) = textures.observed_tile(viewer, point) {
            if let Some(overlays) = source.facts["overlays"].as_array() {
                for observed in overlays {
                    if let Some(id) = observed["improvement_id"].as_str() {
                        // TileOverlayProjection deliberately omits remaining at zero.
                        // This default applies only inside an explicitly observed overlay.
                        let remaining = observed["remaining"]
                            .as_u64()
                            .and_then(|value| u32::try_from(value).ok())
                            .unwrap_or(0);
                        let total = viewer
                            .interface
                            .rules
                            .as_ref()
                            .and_then(|rules| rules["terrain_improvements"].as_array())
                            .and_then(|definitions| {
                                definitions
                                    .iter()
                                    .find(|definition| definition["id"].as_str() == Some(id))
                            })
                            .and_then(|definition| definition["build_turns"].as_u64())
                            .and_then(|value| u32::try_from(value).ok())
                            .unwrap_or_else(|| remaining.max(1));
                        if let Some(index) =
                            textures.improvement_art(id, &source.terrain, remaining, total)
                        {
                            overlay(canvas, textures, index, explored, x, y, zoom)?;
                        }
                    }
                }
            }
            if state.filters.political_borders||state.filters.city_influence {
                super::borders::draw(canvas,textures,state,camera,point)?;
            }
        }
    }
    Ok(())
}

/// Standard overlays retain full native dimensions and original fog shading.
fn overlay(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    id: u16,
    explored: bool,
    x: i32,
    y: i32,
    zoom: u32,
) -> Result<(), GameplayError> {
    if let Some(texture) = textures.observed_improvement(id, explored)? {
        let size = texture.query();
        canvas
            .copy(
                texture,
                None,
                Rect::new(
                    x,
                    y,
                    size.width.saturating_mul(zoom),
                    size.height.saturating_mul(zoom),
                ),
            )
            .map_err(runtime_error)?;
    }
    Ok(())
}
