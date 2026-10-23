//! Observed overlays and political boundaries; never query an undiscovered tile.
use crate::gameplay::model::{FogState, GameState, MapPoint, TileFeature};
use crate::gameplay::runtime::{Camera, textures::GameplayTextures};
use crate::gameplay::{GameplayError, runtime_error};
use sdl2::{
    rect::{Point, Rect},
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
            if state.filters.political_borders {
                if let Some(owner) = source.owner_id.as_deref().filter(|owner| !owner.is_empty()) {
                    borders(canvas, textures, state, camera, point, owner)?;
                }
            }
        }
    }
    Ok(())
}

/// Edges require two explicit observed owners; nearby cities confer no ownership.
fn borders(
    canvas: &mut Canvas<Window>,
    textures: &GameplayTextures<'_>,
    state: &GameState,
    camera: &Camera,
    point: MapPoint,
    owner: &str,
) -> Result<(), GameplayError> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(());
    };
    /* Released source under ctp2_source/ctp2_code/:
     * gfx/tilesys/tiledraw.cpp::TiledMap::{DrawNationalBorders,
     * GetVisibleCellOwner} uses NORMAL ColorSet::GetPlayerColor even for a
     * remembered owner; city captions alone use its dark variant. Colors00.txt
     * defines those runtime colors, not LDL. Frozen public setup slots supply
     * only presentation: they never create the observed owner or make an absent
     * neighboring owner known. Keep the existing safer observed-neighbor gate,
     * not the original canonical-world/city lookup or full border parity.
     */
    let color=textures.observed_owner_color(state,owner);
    let (x, y) = camera.tile_to_screen(point);
    let scale = i32::try_from(camera.tile_width().div_euclid(94).max(1)).unwrap_or(1_i32);
    for (dx, dy, start, end) in [
        (1_i16, 0_i16, (94_i32, 48_i32), (47_i32, 72_i32)),
        (0_i16, 1_i16, (47_i32, 72_i32), (0_i32, 48_i32)),
        (-1_i16, 0_i16, (0_i32, 48_i32), (47_i32, 24_i32)),
        (0_i16, -1_i16, (47_i32, 24_i32), (94_i32, 48_i32)),
    ] {
        let neighbor = point
            .offset(dx, dy, state.width, state.height)
            .and_then(|neighbor| textures.observed_tile(viewer, neighbor));
        if neighbor
            .and_then(|neighbor| neighbor.owner_id.as_deref())
            .is_some_and(|other| other != owner)
        {
            canvas.set_draw_color(color);
            canvas
                .draw_line(
                    Point::new(
                        x.saturating_add(start.0.saturating_mul(scale)),
                        y.saturating_add(start.1.saturating_mul(scale)),
                    ),
                    Point::new(
                        x.saturating_add(end.0.saturating_mul(scale)),
                        y.saturating_add(end.1.saturating_mul(scale)),
                    ),
                )
                .map_err(runtime_error)?;
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
