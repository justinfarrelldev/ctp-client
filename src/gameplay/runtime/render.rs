//! Gameplay frame rendering.

#![expect(
    clippy::arithmetic_side_effects,
    reason = "Renderer uses fixed pixel offsets for SDL rectangles after camera clipping"
)]
#![expect(
    clippy::default_numeric_fallback,
    reason = "Renderer literals are fixed SDL pixel coordinates in i32 contexts"
)]

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::Font;
use sdl2::video::Window;

use super::super::model::{
    FogState, GameState, MapPoint, PlayerId, TerrainKind, TileFeature, UnitKind,
};
use super::super::ui_model::{
    CONTROL_PANEL_HEIGHT, CONTROL_PANEL_WIDTH, InGameUiModel, MENU_BAR_HEIGHT,
};
use super::super::{GameplayError, runtime_error};
use super::view::{Camera, TILE_HEIGHT, TILE_WIDTH};
use super::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH};

/// Right-side panel width.
const SIDE_PANEL_WIDTH: u32 = 160;

/// Draws one gameplay frame.
pub(super) fn draw_frame(
    canvas: &mut Canvas<Window>,
    state: &GameState,
    ui: &InGameUiModel,
    camera: &Camera,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    draw_map(canvas, state, camera)?;
    draw_top_menu(canvas, ui, font)?;
    draw_control_panel(canvas, state, font)?;
    draw_side_panel(canvas, state, font)?;
    draw_reachable_labels(canvas, state, camera)?;
    draw_popups(canvas, state, font)?;
    canvas.present();
    Ok(())
}

/// Draws the world map.
fn draw_map(
    canvas: &mut Canvas<Window>,
    state: &GameState,
    camera: &Camera,
) -> Result<(), GameplayError> {
    for y in 0..state.height {
        for x in 0..state.width {
            let point = MapPoint::new(x, y);
            let Some(tile) = state.tile(point) else {
                continue;
            };
            let (screen_x, screen_y) = camera.tile_to_screen(point);
            if screen_x > 1024
                || screen_y > 768
                || screen_x < -TILE_WIDTH
                || screen_y < -TILE_HEIGHT
            {
                continue;
            }
            let mut color = tile_color(tile.terrain);
            if state.filters.fog_of_war {
                color = fogged_color(color, tile.fog);
            }
            canvas.set_draw_color(color);
            canvas
                .fill_rect(Rect::new(
                    screen_x,
                    screen_y,
                    camera.tile_width(),
                    camera.tile_height(),
                ))
                .map_err(runtime_error)?;
            if state.filters.grid {
                canvas.set_draw_color(Color::RGB(20, 20, 20));
                canvas
                    .draw_rect(Rect::new(
                        screen_x,
                        screen_y,
                        camera.tile_width(),
                        camera.tile_height(),
                    ))
                    .map_err(runtime_error)?;
            }
            if tile.feature == Some(TileFeature::Ruin) && tile.fog == FogState::Visible {
                canvas.set_draw_color(Color::RGB(210, 185, 100));
                canvas
                    .fill_rect(Rect::new(screen_x + 8, screen_y + 5, 12, 12))
                    .map_err(runtime_error)?;
            }
        }
    }
    draw_cities(canvas, state, camera)?;
    draw_units(canvas, state, camera)
}

/// Draws cities.
fn draw_cities(
    canvas: &mut Canvas<Window>,
    state: &GameState,
    camera: &Camera,
) -> Result<(), GameplayError> {
    for city in &state.cities {
        let (screen_x, screen_y) = camera.tile_to_screen(city.position);
        canvas.set_draw_color(owner_color(city.owner));
        canvas
            .fill_rect(Rect::new(screen_x + 4, screen_y + 2, 24, 20))
            .map_err(runtime_error)?;
        if state.filters.city_names {
            canvas.set_draw_color(Color::RGB(255, 255, 200));
            canvas
                .draw_rect(Rect::new(screen_x + 2, screen_y, 28, 23))
                .map_err(runtime_error)?;
        }
    }
    Ok(())
}

/// Draws units.
fn draw_units(
    canvas: &mut Canvas<Window>,
    state: &GameState,
    camera: &Camera,
) -> Result<(), GameplayError> {
    for unit in state.units.iter().filter(|unit| unit.alive) {
        if state
            .tile(unit.position)
            .is_some_and(|tile| tile.fog == FogState::Hidden && unit.owner != PlayerId::Human)
        {
            continue;
        }
        let (screen_x, screen_y) = camera.tile_to_screen(unit.position);
        canvas.set_draw_color(unit_color(unit.kind, unit.owner));
        canvas
            .fill_rect(Rect::new(screen_x + 10, screen_y + 4, 12, 18))
            .map_err(runtime_error)?;
        if Some(unit.id) == state.selected_unit {
            canvas.set_draw_color(Color::RGB(255, 255, 255));
            canvas
                .draw_rect(Rect::new(screen_x + 8, screen_y + 2, 16, 22))
                .map_err(runtime_error)?;
        }
    }
    Ok(())
}

/// Draws movement distance overlays.
fn draw_reachable_labels(
    canvas: &mut Canvas<Window>,
    state: &GameState,
    camera: &Camera,
) -> Result<(), GameplayError> {
    for label in state.reachable_tile_labels() {
        let (screen_x, screen_y) = camera.tile_to_screen(label.position);
        let shade = if label.label == "1" { 240 } else { 180 };
        canvas.set_draw_color(Color::RGB(shade, shade, 40));
        canvas
            .fill_rect(Rect::new(screen_x + 2, screen_y + 2, 8, 8))
            .map_err(runtime_error)?;
    }
    Ok(())
}

/// Draws top menu bar and optionally a dropdown.
fn draw_top_menu(
    canvas: &mut Canvas<Window>,
    ui: &InGameUiModel,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    canvas.set_draw_color(Color::RGB(58, 58, 64));
    canvas
        .fill_rect(Rect::new(0, 0, 1024, MENU_BAR_HEIGHT))
        .map_err(runtime_error)?;
    for button in &ui.menu_buttons {
        canvas.set_draw_color(Color::RGB(84, 84, 92));
        canvas
            .fill_rect(Rect::new(
                button.x,
                2,
                button.width,
                MENU_BAR_HEIGHT.saturating_sub(4),
            ))
            .map_err(runtime_error)?;
        draw_text(
            canvas,
            font,
            button.label,
            button.x.saturating_add(6),
            5,
            Color::RGB(230, 224, 190),
        )?;
    }
    if let Some(category) = ui.open_menu {
        canvas.set_draw_color(Color::RGB(43, 43, 48));
        let rows = ui.items_for(category);
        let height = u32::try_from(rows.len()).map_or(0, |count| count.saturating_mul(18));
        let menu_y = i32::try_from(MENU_BAR_HEIGHT).map_or(24, |value| value);
        canvas
            .fill_rect(Rect::new(0, menu_y, 260, height))
            .map_err(runtime_error)?;
        canvas.set_draw_color(Color::RGB(110, 96, 55));
        canvas
            .draw_rect(Rect::new(0, menu_y, 260, height))
            .map_err(runtime_error)?;
        for (index, item) in rows.iter().enumerate() {
            let row_y = menu_y
                .saturating_add(i32::try_from(index).map_or(0, |value| value.saturating_mul(18)));
            let text_color = if item.enabled {
                Color::RGB(235, 230, 205)
            } else {
                Color::RGB(125, 120, 112)
            };
            draw_text(
                canvas,
                font,
                item.label,
                8,
                row_y.saturating_add(2),
                text_color,
            )?;
            if !item.shortcut.is_empty() {
                draw_text(
                    canvas,
                    font,
                    item.shortcut,
                    188,
                    row_y.saturating_add(2),
                    text_color,
                )?;
            }
        }
    }
    Ok(())
}

/// Draws the bottom control panel.
fn draw_control_panel(
    canvas: &mut Canvas<Window>,
    state: &GameState,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    let y = i32::try_from(DEFAULT_WINDOW_HEIGHT.saturating_sub(CONTROL_PANEL_HEIGHT))
        .map_or(584, |value| value);
    canvas.set_draw_color(Color::RGB(40, 43, 48));
    canvas
        .fill_rect(Rect::new(0, y, CONTROL_PANEL_WIDTH, CONTROL_PANEL_HEIGHT))
        .map_err(runtime_error)?;
    draw_text(
        canvas,
        font,
        &format!(
            "Turn {}  Gold {}  Empires {}",
            state.turn,
            state.gold,
            state.contacts.len()
        ),
        16,
        y.saturating_add(12),
        Color::RGB(230, 224, 190),
    )?;
    if let Some(unit) = state.selected_unit() {
        draw_text(
            canvas,
            font,
            &format!("Selected: {:?} MP {}", unit.kind, unit.movement_points),
            16,
            y.saturating_add(34),
            Color::RGB(190, 220, 255),
        )?;
    }
    draw_text(
        canvas,
        font,
        "A/Enter End Turn  B Settle  Space Sentinel",
        16,
        y.saturating_add(58),
        Color::RGB(210, 210, 185),
    )?;
    canvas.set_draw_color(Color::RGB(75, 63, 38));
    canvas
        .fill_rect(Rect::new(940, 590, 70, 150))
        .map_err(runtime_error)?;
    canvas.set_draw_color(Color::RGB(120, 92, 40));
    canvas
        .draw_rect(Rect::new(940, 590, 70, 150))
        .map_err(runtime_error)?;
    draw_text(canvas, font, "END", 960, 648, Color::RGB(255, 235, 140))?;
    draw_text(canvas, font, "TURN", 954, 666, Color::RGB(255, 235, 140))?;
    if state.city_build_menu.is_some() {
        canvas.set_draw_color(Color::RGB(90, 65, 30));
        canvas
            .fill_rect(Rect::new(250, 260, 260, 160))
            .map_err(runtime_error)?;
        canvas.set_draw_color(Color::RGB(220, 180, 80));
        canvas
            .draw_rect(Rect::new(250, 260, 260, 160))
            .map_err(runtime_error)?;
        draw_text(
            canvas,
            font,
            "City Build Queue",
            270,
            282,
            Color::RGB(255, 230, 160),
        )?;
        draw_text(
            canvas,
            font,
            "Warrior   Settler   Granary",
            270,
            314,
            Color::RGB(245, 235, 205),
        )?;
    }
    Ok(())
}

/// Draws minimap/filter side panel.
fn draw_side_panel(
    canvas: &mut Canvas<Window>,
    state: &GameState,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    let x = i32::try_from(DEFAULT_WINDOW_WIDTH.saturating_sub(SIDE_PANEL_WIDTH))
        .map_or(864, |value| value);
    canvas.set_draw_color(Color::RGB(24, 28, 32));
    canvas
        .fill_rect(Rect::new(
            x,
            i32::try_from(MENU_BAR_HEIGHT).map_or(24, |value| value),
            SIDE_PANEL_WIDTH,
            130,
        ))
        .map_err(runtime_error)?;
    draw_text(canvas, font, "Radar", x + 12, 30, Color::RGB(230, 224, 190))?;
    if state.filters.radar {
        canvas.set_draw_color(Color::RGB(55, 85, 55));
        canvas
            .fill_rect(Rect::new(x + 12, 50, 120, 80))
            .map_err(runtime_error)?;
        canvas.set_draw_color(Color::RGB(100, 150, 100));
        canvas
            .draw_rect(Rect::new(x + 12, 50, 120, 80))
            .map_err(runtime_error)?;
    }
    draw_text(
        canvas,
        font,
        "R radar  C names  / fog",
        x + 8,
        138,
        Color::RGB(205, 205, 180),
    )?;
    Ok(())
}

/// Draws pending popups.
fn draw_popups(
    canvas: &mut Canvas<Window>,
    state: &GameState,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    if state.popups.is_empty() {
        return Ok(());
    }
    canvas.set_draw_color(Color::RGB(32, 28, 24));
    canvas
        .fill_rect(Rect::new(260, 180, 500, 170))
        .map_err(runtime_error)?;
    canvas.set_draw_color(Color::RGB(210, 180, 90));
    canvas
        .draw_rect(Rect::new(260, 180, 500, 170))
        .map_err(runtime_error)?;
    if let Some(popup) = state.popups.first() {
        draw_text(
            canvas,
            font,
            &popup.title,
            284,
            205,
            Color::RGB(255, 230, 150),
        )?;
        draw_text(
            canvas,
            font,
            &popup.body,
            284,
            236,
            Color::RGB(235, 232, 210),
        )?;
        draw_text(
            canvas,
            font,
            "Click or Esc to continue",
            284,
            304,
            Color::RGB(185, 180, 160),
        )?;
    }
    Ok(())
}

/// Draws text when a system font is available.
fn draw_text(
    canvas: &mut Canvas<Window>,
    font: Option<&Font<'_, '_>>,
    text: &str,
    x: i32,
    y: i32,
    color: Color,
) -> Result<(), GameplayError> {
    let Some(font) = font else {
        return Ok(());
    };
    let surface = font
        .render(text)
        .blended(color)
        .map_err(|error| runtime_error(error.to_string()))?;
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|error| runtime_error(error.to_string()))?;
    let target = Rect::new(x, y, surface.width(), surface.height());
    canvas.copy(&texture, None, target).map_err(runtime_error)
}

/// Returns fallback color for terrain.
const fn tile_color(terrain: TerrainKind) -> Color {
    let (red, green, blue) = terrain.fallback_color();
    Color::RGB(red, green, blue)
}

/// Applies fog darkening.
const fn fogged_color(color: Color, fog: FogState) -> Color {
    let (divisor, alpha_floor) = match fog {
        FogState::Visible => (1_u8, 0_u8),
        FogState::Explored => (2_u8, 20_u8),
        FogState::Hidden => (6_u8, 5_u8),
    };
    Color::RGB(
        color.r.saturating_div(divisor).saturating_add(alpha_floor),
        color.g.saturating_div(divisor).saturating_add(alpha_floor),
        color.b.saturating_div(divisor).saturating_add(alpha_floor),
    )
}

/// Color for city owner.
const fn owner_color(owner: PlayerId) -> Color {
    match owner {
        PlayerId::Human => Color::RGB(70, 120, 220),
        PlayerId::Empire(_) => Color::RGB(190, 70, 70),
        PlayerId::Barbarians => Color::RGB(120, 50, 50),
    }
}

/// Color for units.
const fn unit_color(kind: UnitKind, owner: PlayerId) -> Color {
    match (kind, owner) {
        (UnitKind::Settler, PlayerId::Human) => Color::RGB(80, 180, 240),
        (UnitKind::Settler, _) => Color::RGB(220, 160, 80),
        (UnitKind::Warrior, PlayerId::Human) => Color::RGB(40, 80, 220),
        (UnitKind::Warrior, _) => Color::RGB(220, 50, 50),
        (UnitKind::Barbarian, _) => Color::RGB(80, 30, 30),
    }
}
