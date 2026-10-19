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

/// Draws visible city markers (city sprites remain a separate gameplay feature).
pub(super) fn draw_cities(
    canvas: &mut Canvas<Window>,
    state: &GameState,
    camera: &Camera,
) -> Result<(), GameplayError> {
    for city in &state.cities {
        if state
            .tile(city.position)
            .is_none_or(|tile| tile.fog == FogState::Hidden)
        {
            continue;
        }
        let (x, y) = camera.tile_to_screen(city.position);
        canvas.set_draw_color(owner_color(city.owner));
        canvas
            .fill_rect(Rect::new(
                x.saturating_add(36_i32),
                y.saturating_add(28_i32),
                24,
                20,
            ))
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
    for unit in state.units.iter().filter(|unit| unit.alive) {
        if unit.owner != PlayerId::Human
            && state
                .tile(unit.position)
                .is_none_or(|tile| tile.fog != FogState::Visible)
        {
            continue;
        }
        let top = state
            .selected_unit()
            .filter(|selected| selected.position == unit.position)
            .or_else(|| {
                state
                    .units
                    .iter()
                    .find(|candidate| candidate.alive && candidate.position == unit.position)
            });
        if top.is_some_and(|top| top.id != unit.id) {
            continue;
        }
        let duration = textures.move_duration(unit.kind, input.actors.speed, input.actors.enabled);
        let pose = input.actors.pose(unit, camera, duration);
        let (x, y) = pose.position;
        let zoom = i32::try_from(camera.tile_width()).map_or(1_i32, |v| v.div_euclid(94_i32));
        let shield = if let Some(actor) =
            textures.unit_texture(unit.kind, pose, duration, input.actors.enabled)?
        {
            let size = actor.frame.sprite.texture.query();
            let hotpoint = pose.facing.hotpoint(
                actor.frame.sprite.hotpoint,
                u16::try_from(size.width).map_or(u16::MAX, |width| width),
            );
            let position = (
                x.saturating_add(actor.delta.0.saturating_mul(zoom)),
                y.saturating_add(actor.delta.1.saturating_mul(zoom)),
            );
            let target = unit_target_rect(size.width, size.height, hotpoint, position, zoom);
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
            canvas.set_draw_color(owner_color(unit.owner));
            canvas
                .fill_rect(Rect::new(
                    x.saturating_add(40_i32),
                    y.saturating_add(20_i32),
                    12,
                    18,
                ))
                .map_err(runtime_error)?;
            (70_i32, 10_i32)
        };
        let stack = state
            .units
            .iter()
            .filter(|candidate| candidate.alive && candidate.position == unit.position)
            .count();
        let shield_x = x.saturating_add(shield.0.saturating_mul(zoom));
        let shield_y = y.saturating_add(shield.1.saturating_mul(zoom));
        draw_stack_shield(
            canvas, textures, stack, shield_x, shield_y, unit.owner, true,
        )?;
        if let Some(flag) = textures.map_icon(23_u16, owner_color(unit.owner))? {
            let size = flag.query();
            let flag_y = shield_y
                .saturating_sub(i32::try_from(size.height).map_or(0_i32, |value| value))
                .saturating_sub(5_i32);
            canvas
                .copy(
                    flag,
                    None,
                    Rect::new(shield_x, flag_y, size.width, size.height),
                )
                .map_err(runtime_error)?;
        }
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
    owner: PlayerId,
    health_bar: bool,
) -> Result<(), GameplayError> {
    let index = if count <= 1_usize {
        6_u16
    } else {
        10_u16.saturating_add(u16::try_from(count.min(9_usize)).map_or(2_u16, |v| v))
    };
    if let Some(icon) = textures.map_icon(index, owner_color(owner))? {
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

/// Quantized original player palette, shared by map and portrait heralds.
const fn owner_color(owner: PlayerId) -> Color {
    match owner {
        PlayerId::Human => Color::RGB(0, 0, 255),
        PlayerId::Empire(_) => Color::RGB(255, 0, 0),
        PlayerId::Barbarians => Color::RGB(120, 48, 48),
    }
}
