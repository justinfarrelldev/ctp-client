//! Full-resolution CTP2 idle actors, numbered shields and original map-icon reticles.

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use super::movement;
use crate::gameplay::model::{FogState, GameState, MapPoint, PlayerId, UnitKind};
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::runtime::{Camera, MovementInputState};
use crate::gameplay::{GameplayError, runtime_error};

/// Same SPR bounds/alpha and presented identities for pointer input.
mod hits;
/// Pure presented-group and authoritative membership regressions.
#[cfg(test)]
mod stack_tests;
/// Prepared viewer-permitted poses keep motion and hidden members out of false stacks.
mod stacks;

/// Current sampled source masks and exact group IDs, never canonical-cell guesses.
pub(super) fn hit_targets(
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
    camera: &Camera,
    input: &MovementInputState,
) -> Result<Vec<crate::gameplay::runtime::actor_hits::ActorHit>, GameplayError> {
    hits::collect(textures, state, camera, input)
}

/// Draws original resource actors only when the viewer explicitly knows them.
pub(super) fn draw_goods(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
    camera: &Camera,
) -> Result<(), GameplayError> {
    let zoom = i32::try_from(camera.tile_width()).map_or(1_i32, |value| value.div_euclid(94_i32));
    for point in crate::gameplay::runtime::view::painter_order(state.width, state.height) {
        let Some(tile) = state
            .tile(point)
            .filter(|tile| tile.fog != FogState::Hidden)
        else {
            continue;
        };
        let Some(id) = tile.good_id.as_deref() else {
            continue;
        };
        let explored = state.filters.fog_of_war && tile.fog == FogState::Explored;
        let Some(actor) = textures.good_texture(id, explored)? else {
            continue;
        };
        let size = actor.texture.query();
        let position = camera.tile_to_screen(point);
        canvas
            .copy(
                &actor.texture,
                None,
                unit_target_rect(size.width, size.height, actor.hotpoint, position, zoom),
            )
            .map_err(runtime_error)?;
    }
    Ok(())
}

/// Returns the selected settler's original small city-radius preview.
/// Unknown/stale cells are excluded, so the overlay cannot reveal hidden terrain.
pub(super) fn settlement_preview_points(state: &GameState) -> Vec<MapPoint> {
    let Some(unit) = state.selected_unit().filter(|unit| {
        unit.kind == UnitKind::Settler
            && unit.owner == PlayerId::Human
            && state.city_at(unit.position).is_none()
    }) else {
        return Vec::new();
    };
    let mut points = Vec::with_capacity(9_usize);
    for dy in -1_i16..=1_i16 {
        for dx in -1_i16..=1_i16 {
            if let Some(point) = unit
                .position
                .offset(dx, dy, state.width, state.height)
                .filter(|point| {
                    state
                        .tile(*point)
                        .is_some_and(|tile| tile.fog == FogState::Visible)
                })
            {
                points.push(point);
            }
        }
    }
    points
}

/// Reproduces the green `DrawCityRadius` preview, not a permanent world grid.
pub(super) fn draw_settlement_preview(
    canvas: &mut Canvas<Window>,
    state: &GameState,
    camera: &Camera,
) -> Result<(), GameplayError> {
    for point in settlement_preview_points(state) {
        let (x, y) = camera.tile_to_screen(point);
        super::draw_tile_diamond(canvas, camera, x, y, Color::RGB(0, 255, 0))?;
    }
    Ok(())
}

/// Draws only the top visible actor of each stack, never overlapping whole sprites.
pub(super) fn draw_units(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
    camera: &Camera,
    input: &MovementInputState,
) -> Result<(), GameplayError> {
    for presented in stacks::prepare(textures, state, camera, input)? {
        let unit = presented.unit;
        let sprite = presented.sprite;
        let duration = presented.duration;
        let pose = presented.pose;
        let (x, y) = pose.position;
        let zoom = i32::try_from(camera.tile_width()).map_or(1_i32, |v| v.div_euclid(94_i32));
        draw_fortification(canvas, textures, state, unit, pose.position, camera)?;
        let sampled = if let Some(id) = sprite {
            textures.unit_texture(id, pose, duration, input.actors.enabled)?
        } else {
            None
        };
        let shield = if let Some(actor) = sampled {
            let size = actor.frame.sprite.texture.query();
            let target = hits::sprite_bounds(
                size.width,
                size.height,
                actor.frame.sprite.hotpoint,
                actor.delta,
                pose,
                camera,
            );
            canvas
                .copy_ex(
                    &actor.frame.sprite.texture,
                    None,
                    target,
                    0.0_f64,
                    None,
                    actor.flipped,
                    false,
                )
                .map_err(runtime_error)?;
            actor.frame.shield
        } else {
            canvas.set_draw_color(textures.player_color(state,&unit.owner,false));
            canvas
                .fill_rect(hits::fallback_bounds(pose))
                .map_err(runtime_error)?;
            (70_i32, 10_i32)
        };
        let stack = presented.count;
        let shield_x = x.saturating_add(shield.0.saturating_mul(zoom));
        let shield_y = y.saturating_add(shield.1.saturating_mul(zoom));
        /* Released source under ctp2_source/ctp2_code/:
         * gfx/spritesys/UnitActor.cpp::{DrawHerald,DrawStackingIndicator}
         * tint runtime MAPICON_HERALD silhouettes with ColorSet::GetPlayerColor.
         * Colors00.txt supplies the table; tilesys/tileset.h supplies icon IDs,
         * not LDL. The same color now feeds the existing client MAPICON_FLAG
         * overlay; that extra overlay/placement is not certified retail behavior
         * (retail tiledmap.cpp::DrawStartingLocations uses flags for the editor).
         * This change restores identity/color only, without changing geometry.
         */
        let color=textures.player_color(state,&unit.owner,false);
        draw_stack_shield(canvas,textures,stack,shield_x,shield_y,color,true)?;
        draw_flag(canvas,textures,color,shield_x,shield_y)?;
        if Some(unit.id) == input.active_unit() || Some(unit.id) == state.selected_unit {
            draw_selection(
                canvas,
                textures,
                camera,
                pose.position,
                unit.movement_points > 0_i16,
            )?;
        }
    }
    Ok(())
}

/// Paints the source flag above its traveling herald anchor.
fn draw_flag(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    color: Color,
    x: i32,
    y: i32,
) -> Result<(), GameplayError> {
    if let Some(flag) = textures.map_icon(23, color)? {
        let size = flag.query();
        let top = y
            .saturating_sub(i32::try_from(size.height).unwrap_or(0_i32))
            .saturating_sub(5);
        canvas
            .copy(flag, None, Rect::new(x, top, size.width, size.height))
            .map_err(runtime_error)?;
    }
    Ok(())
}

/// Uses the actual fortified fact and `UnitActor::DrawFortified` overlay 34.
fn draw_fortification(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
    unit: &crate::gameplay::model::Unit,
    position: (i32, i32),
    camera: &Camera,
) -> Result<(), GameplayError> {
    let fortified = state
        .viewer
        .as_ref()
        .and_then(|viewer| {
            viewer.units.iter().find(|source| {
                crate::gameplay::model::UnitId::from_server_id(&source.id) == unit.id
            })
        })
        .is_some_and(|source| source.facts["fortified"].as_bool() == Some(true));
    if fortified {
        if let Some(texture) = textures.improvement_texture(34)? {
            canvas
                .copy(
                    texture,
                    None,
                    Rect::new(
                        position.0,
                        position.1,
                        camera.tile_width(),
                        camera.tile_height(),
                    ),
                )
                .map_err(runtime_error)?;
        }
    }
    Ok(())
}

/// Original actor center is (48,48); subtract the SPR hotspot, then zoom uniformly.
pub(super) fn unit_target_rect(
    width: u32,
    height: u32,
    hotpoint: (i32, i32),
    position: (i32, i32),
    zoom: i32,
) -> Rect {
    let scale = u32::try_from(zoom).map_or(1_u32, |v| v);
    Rect::new(
        position
            .0
            .saturating_add(48_i32.saturating_sub(hotpoint.0).saturating_mul(zoom)),
        position
            .1
            .saturating_add(48_i32.saturating_sub(hotpoint.1).saturating_mul(zoom)),
        width.saturating_mul(scale),
        height.saturating_mul(scale),
    )
}

/// Paints original herald silhouettes/numbers, including a four-pixel health bar.
pub(super) fn draw_stack_shield(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    count: usize,
    x: i32,
    y: i32,
    color: Color,
    health_bar: bool,
) -> Result<(), GameplayError> {
    let index = if count <= 1_usize {
        6_u16
    } else {
        10_u16.saturating_add(u16::try_from(count.min(9_usize)).map_or(2_u16, |v| v))
    };
    if let Some(icon) = textures.map_icon(index, color)? {
        let size = icon.query();
        canvas
            .copy(icon, None, Rect::new(x, y, size.width, size.height))
            .map_err(runtime_error)?;
        if health_bar {
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas
                .fill_rect(Rect::new(x, y.saturating_sub(4_i32), size.width, 3))
                .map_err(runtime_error)?;
            canvas.set_draw_color(Color::RGB(0, 255, 0));
            canvas
                .fill_rect(Rect::new(
                    x.saturating_add(1_i32),
                    y.saturating_sub(3_i32),
                    size.width.saturating_sub(2_u32),
                    1,
                ))
                .map_err(runtime_error)?;
        }
    }
    Ok(())
}

/// Paints the actual four shaped bracket icons; lines are only a missing-art fallback.
fn draw_selection(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    camera: &Camera,
    position: (i32, i32),
    can_move: bool,
) -> Result<(), GameplayError> {
    let color = movement::selection_feedback_color(can_move);
    let (x, y) = position;
    let center_x = x
        .saturating_add(i32::try_from(camera.tile_width()).map_or(47_i32, |v| v.div_euclid(2_i32)));
    let center_y = y.saturating_add(
        i32::try_from(camera.tile_height()).map_or(36_i32, |v| v.div_euclid(2_i32)),
    );
    for (index, right, bottom) in [
        (8_u16, false, false),
        (9, true, false),
        (10, true, true),
        (11, false, true),
    ] {
        let Some(icon) = textures.map_icon(index, color)? else {
            return movement::draw_screen_selection_brackets(canvas, camera, position, color);
        };
        let size = icon.query();
        let left = if right {
            center_x
                .saturating_add(25_i32)
                .saturating_sub(i32::try_from(size.width).map_or(0_i32, |v| v))
        } else {
            center_x.saturating_sub(25_i32)
        };
        let top = if bottom {
            center_y
                .saturating_add(25_i32)
                .saturating_sub(i32::try_from(size.height).map_or(0_i32, |v| v))
        } else {
            center_y.saturating_sub(25_i32)
        };
        canvas
            .copy(icon, None, Rect::new(left, top, size.width, size.height))
            .map_err(runtime_error)?;
    }
    Ok(())
}
