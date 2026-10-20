//! Original GC building sprites and viewer-safe city/production name strips.
//!
//! Missing city names/style/age are server contract gaps: a machine identity or
//! hardcoded Warrior is never presented as an authoritative caption. Installed
//! default city art is an explicit visual fallback, not a simulated city state.

use super::{
    actors::unit_target_rect,
    text::{draw_centered_text, draw_text},
};
use crate::gameplay::model::{City, CityId, FogState, GameState, PlayerId, TerrainKind};
use crate::gameplay::runtime::{Camera, textures::GameplayTextures};
use crate::gameplay::server::ServerCityProjection;
use crate::gameplay::ui_model::InGameUiModel;
use crate::gameplay::{GameplayError, runtime_error};
use sdl2::{pixels::Color, rect::Rect, render::Canvas, ttf::Font, video::Window};

/// Exact projected identity; coordinates or a duplicate name are not identity.
fn source<'state>(state: &'state GameState, city: &City) -> Option<&'state ServerCityProjection> {
    state
        .viewer
        .as_ref()?
        .cities
        .iter()
        .find(|source| CityId::from_server_id(&source.id) == city.id)
}

/// Foreign remembered cities must use a stale observation, not current metadata.
fn visible(state: &GameState, city: &City) -> bool {
    let Some(tile) = state.tile(city.position) else {
        return false;
    };
    if tile.fog == FogState::Hidden {
        return false;
    }
    if !state.authoritative {
        return true;
    }
    let Some(projected) = source(state, city) else {
        return false;
    };
    if city.owner == PlayerId::Human && !projected.stale {
        return true;
    }
    projected.stale == (tile.fog == FogState::Explored)
}

/// Native building frames keep original hotspots, alpha and remembered shading.
pub(super) fn draw_cities(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
    camera: &Camera,
) -> Result<(), GameplayError> {
    let mut cities: Vec<_> = state
        .cities
        .iter()
        .filter(|city| visible(state, city))
        .collect();
    cities.sort_unstable_by_key(|city| {
        (
            u32::from(city.position.x).saturating_add(u32::from(city.position.y)),
            city.position.x,
        )
    });
    for city in cities {
        let point = city.position;
        let population = source(state, city).map_or(1_u32, |city| city.population);
        let explored = state
            .tile(point)
            .is_some_and(|tile| tile.fog == FogState::Explored);
        let water = state
            .tile(point)
            .is_some_and(|tile| tile.terrain == TerrainKind::Water);
        let Some(sprite) = textures.city_texture(population, water, explored)? else {
            continue;
        };
        let size = sprite.texture.query();
        let zoom = i32::try_from(camera.tile_width().div_euclid(94_u32))
            .unwrap_or(1_i32)
            .max(1_i32);
        canvas
            .copy(
                &sprite.texture,
                None,
                unit_target_rect(
                    size.width,
                    size.height,
                    sprite.hotpoint,
                    camera.tile_to_screen(point),
                    zoom,
                ),
            )
            .map_err(runtime_error)?;
    }
    Ok(())
}

/// City captions paint above actors and paths, as in the supplied release captures.
pub(super) fn draw_labels(
    canvas: &mut Canvas<Window>,
    state: &GameState,
    ui: &InGameUiModel,
    camera: &Camera,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    if !state.filters.city_names {
        return Ok(());
    }
    for city in state.cities.iter().filter(|city| visible(state, city)) {
        let source = source(state, city);
        let name = if state.authoritative {
            source.and_then(|source| source.facts["name"].as_str()).filter(|name| !name.is_empty()).unwrap_or("—")
        } else {
            city.name.as_str()
        };
        let (x, y) = camera.tile_to_screen(city.position);
        let zoom = i32::try_from(camera.tile_width().div_euclid(94_u32))
            .unwrap_or(1_i32)
            .max(1_i32);
        let y = y.saturating_sub(12_i32.saturating_mul(zoom));
        let height = u32::try_from(font.map_or(12_i32, Font::height))
            .unwrap_or(12_u32)
            .max(10_u32);
        let color = owner_color(
            city.owner,
            state
                .tile(city.position)
                .is_some_and(|tile| tile.fog == FogState::Explored),
        );
        let population = source
            .filter(|source| source.population > 0_u32)
            .map_or_else(|| "—".to_owned(), |source| source.population.to_string());
        let name_width = text_width(font, name).saturating_add(6_u32);
        let population_width = text_width(font, &population)
            .saturating_add(4_u32)
            .max(12_u32);
        let population_rect = Rect::new(x, y, population_width, height);
        canvas.set_draw_color(color);
        canvas.fill_rect(population_rect).map_err(runtime_error)?;
        draw_centered_text(
            canvas,
            font,
            &population,
            population_rect,
            Color::RGB(255_u8, 255_u8, 255_u8),
            false,
        )?;
        let name_rect = Rect::new(
            x.saturating_add(i32::try_from(population_width).unwrap_or(0_i32)),
            y,
            name_width,
            height,
        );
        strip(canvas, font, name, name_rect, color)?;
        if city.owner == PlayerId::Human {
            if let Some(item) = source
                .and_then(|source| source.facts["queue"].as_array())
                .and_then(|queue| queue.first())
                .and_then(|item| item["id"].as_str())
            {
                let label = ui.management.book.label(item);
                let production = if label == item {
                    item.trim_start_matches("UNIT_")
                        .trim_start_matches("IMPROVE_")
                        .replace('_', " ")
                } else {
                    label.to_owned()
                };
                let rect = Rect::new(
                    x,
                    y.saturating_add(i32::try_from(height).unwrap_or(12_i32)),
                    text_width(font, &production).saturating_add(6_u32),
                    height,
                );
                strip(canvas, font, &production, rect, color)?;
            }
        }
    }
    Ok(())
}

/// Small black name/production strip with the original player-colour frame.
fn strip(
    canvas: &mut Canvas<Window>,
    font: Option<&Font<'_, '_>>,
    text: &str,
    rect: Rect,
    color: Color,
) -> Result<(), GameplayError> {
    canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
    canvas.fill_rect(rect).map_err(runtime_error)?;
    canvas.set_draw_color(color);
    canvas.draw_rect(rect).map_err(runtime_error)?;
    draw_text(
        canvas,
        font,
        text,
        rect.x().saturating_add(3_i32),
        rect.y(),
        Color::RGB(255_u8, 255_u8, 255_u8),
    )
}

/// Actual font metrics; fallback is only for deliberate font-free tests.
fn text_width(font: Option<&Font<'_, '_>>, text: &str) -> u32 {
    font.and_then(|font| font.size_of(text).ok()).map_or_else(
        || {
            u32::try_from(text.chars().count())
                .unwrap_or(0_u32)
                .saturating_mul(6_u32)
        },
        |size| size.0,
    )
}

/// Static owner colours never imply territorial ownership of unknown tiles.
const fn owner_color(owner: PlayerId, explored: bool) -> Color {
    let channel = if explored { 123_u8 } else { 255_u8 };
    match owner {
        PlayerId::Human => Color::RGB(0_u8, 0_u8, channel),
        PlayerId::Empire(_) => Color::RGB(channel, 0_u8, 0_u8),
        PlayerId::Barbarians => Color::RGB(120_u8, 48_u8, 48_u8),
    }
}
