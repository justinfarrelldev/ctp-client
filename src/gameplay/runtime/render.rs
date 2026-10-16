//! Gameplay frame rendering.

#![expect(
    clippy::arithmetic_side_effects,
    reason = "Renderer uses fixed original CTP2 pixel offsets and bounded SDL rectangles"
)]
#![expect(
    clippy::default_numeric_fallback,
    reason = "Renderer literals are fixed SDL pixel coordinates in i32 contexts"
)]

extern crate alloc;

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Canvas, Texture};
use sdl2::ttf::Font;
use sdl2::video::Window;

use super::super::model::{
    FogState, GameState, MapPoint, PlayerId, TerrainKind, UnitKind,
};
use super::super::ui_model::{
    CONTROL_PANEL_HEIGHT, CONTROL_PANEL_WIDTH, END_TURN_BUTTON_HEIGHT, END_TURN_BUTTON_WIDTH,
    END_TURN_BUTTON_X, END_TURN_BUTTON_Y, InGameUiModel, MENU_BAR_HEIGHT, STATUS_BAR_X,
    STATUS_BAR_Y,
};
use super::super::{GameplayError, runtime_error};
use super::textures::GameplayTextures;
use super::view::{Camera, TILE_HEIGHT, TILE_WIDTH};
use super::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH, MovementInputState, SIDE_PANEL_WIDTH};

/// CTP2 chrome drawing helpers.
mod chrome;
/// Drag-movement overlay drawing helpers.
mod movement;
/// Ground drawing and tile-local fog.
mod terrain;
/// Text drawing helpers.
mod text;

use terrain::draw_map;
use text::{draw_shadowed_text, draw_text};

/// High-contrast text color for top menu button labels over `uptbtest.rim`.
const TOP_MENU_TEXT_COLOR: Color = Color::RGBA(255, 238, 170, 255);
/// Enabled dropdown row text color over original pale menu chrome.
const DROPDOWN_ENABLED_TEXT_COLOR: Color = Color::RGBA(50, 50, 50, 255);
/// Disabled dropdown row text color over original pale menu chrome.
const DROPDOWN_DISABLED_TEXT_COLOR: Color = Color::RGBA(126, 118, 108, 255);

/// Draws one gameplay frame.
pub(super) fn draw_frame(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
    ui: &InGameUiModel,
    camera: &Camera,
    input: &MovementInputState,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    draw_map(canvas, textures, state, camera, input)?;
    movement::draw_planned_path(canvas, input, camera, font)?;
    draw_top_menu(canvas, textures, ui, font)?;
    draw_control_panel(canvas, textures, state, font)?;
    draw_side_panel(canvas, state, font)?;
    draw_popups(canvas, textures, state, font)?;
    canvas.present();
    Ok(())
}

/// Draws an isometric tile diamond.
fn draw_tile_diamond(
    canvas: &mut Canvas<Window>,
    camera: &Camera,
    screen_x: i32,
    screen_y: i32,
    color: Color,
) -> Result<(), GameplayError> {
    let width = i32::try_from(camera.tile_width()).map_or(TILE_WIDTH, |value| value);
    let height = i32::try_from(camera.tile_height()).map_or(TILE_HEIGHT, |value| value);
    let half_width = width.checked_div(2).map_or(0, |value| value);
    let left = Point::new(screen_x, screen_y + height.saturating_sub(24));
    let top = Point::new(screen_x + half_width, screen_y + height.saturating_sub(48));
    let right = Point::new(screen_x + width, screen_y + height.saturating_sub(24));
    let bottom = Point::new(screen_x + half_width, screen_y + height);
    canvas.set_draw_color(color);
    canvas.draw_line(left, top).map_err(runtime_error)?;
    canvas.draw_line(top, right).map_err(runtime_error)?;
    canvas.draw_line(right, bottom).map_err(runtime_error)?;
    canvas.draw_line(bottom, left).map_err(runtime_error)
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
            .fill_rect(Rect::new(screen_x + 36, screen_y + 28, 24, 20))
            .map_err(runtime_error)?;
        if state.filters.city_names {
            canvas.set_draw_color(Color::RGB(255, 255, 200));
            canvas
                .draw_rect(Rect::new(screen_x + 34, screen_y + 26, 28, 23))
                .map_err(runtime_error)?;
        }
    }
    Ok(())
}

/// Draws units.
fn draw_units(
    canvas: &mut Canvas<Window>,
    textures: &GameplayTextures<'_>,
    state: &GameState,
    camera: &Camera,
    input: &MovementInputState,
) -> Result<(), GameplayError> {
    for unit in state.units.iter().filter(|unit| unit.alive) {
        if state
            .tile(unit.position)
            .is_some_and(|tile| tile.fog == FogState::Hidden && unit.owner != PlayerId::Human)
        {
            continue;
        }
        let (screen_x, screen_y) = camera.tile_to_screen(unit.position);
        if let Some(texture) = textures.unit_texture(unit.kind) {
            let query = texture.query();
            let target = unit_target_rect_from_size(query.width, query.height, screen_x, screen_y);
            canvas.copy(texture, None, target).map_err(runtime_error)?;
        } else {
            let target = Rect::new(screen_x + 40, screen_y + 20, 12, 18);
            canvas.set_draw_color(unit_color(unit.kind, unit.owner));
            canvas.fill_rect(target).map_err(runtime_error)?;
        }
        if Some(unit.id) == input.active_unit() || Some(unit.id) == state.selected_unit {
            let color = movement::selection_feedback_color(unit.movement_points > 0_i16);
            movement::draw_tile_selection_brackets(canvas, camera, unit.position, color)?;
        }
    }
    Ok(())
}

/// Returns the destination rectangle for a unit sprite anchored to tile bottom-center.
fn unit_target_rect_from_size(width: u32, height: u32, screen_x: i32, screen_y: i32) -> Rect {
    let target_width = width.min(64);
    let target_height = height.min(u32::try_from(TILE_HEIGHT).map_or(72_u32, |value| value));
    let tile_width = u32::try_from(TILE_WIDTH).map_or(94_u32, |value| value);
    let x_offset = tile_width
        .saturating_sub(target_width)
        .checked_div(2_u32)
        .unwrap_or(0_u32);
    let y_offset = u32::try_from(TILE_HEIGHT)
        .map_or(72_u32, |value| value)
        .saturating_sub(target_height);
    Rect::new(
        screen_x.saturating_add(i32::try_from(x_offset).map_or(0_i32, |value| value)),
        screen_y.saturating_add(i32::try_from(y_offset).map_or(0_i32, |value| value)),
        target_width,
        target_height,
    )
}

/// Draws top menu bar and optionally a dropdown.
fn draw_top_menu(
    canvas: &mut Canvas<Window>,
    textures: &GameplayTextures<'_>,
    ui: &InGameUiModel,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    if let Some(texture) = textures.ui_texture("uptb00.rim") {
        tile_texture_horizontally(canvas, texture, 0, 0, DEFAULT_WINDOW_WIDTH)?;
    } else {
        canvas.set_draw_color(Color::RGB(58, 58, 64));
        canvas
            .fill_rect(Rect::new(0, 0, DEFAULT_WINDOW_WIDTH, MENU_BAR_HEIGHT))
            .map_err(runtime_error)?;
    }
    for button in &ui.menu_buttons {
        if let Some(texture) = textures.ui_texture("uptbtest.rim") {
            canvas
                .copy(
                    texture,
                    None,
                    Rect::new(button.x, 0, button.width, MENU_BAR_HEIGHT),
                )
                .map_err(runtime_error)?;
        } else {
            canvas.set_draw_color(Color::RGB(126, 113, 81));
            canvas
                .fill_rect(Rect::new(button.x, 0, button.width, MENU_BAR_HEIGHT))
                .map_err(runtime_error)?;
            canvas.set_draw_color(Color::RGB(64, 52, 38));
            canvas
                .draw_rect(Rect::new(button.x, 0, button.width, MENU_BAR_HEIGHT))
                .map_err(runtime_error)?;
        }
        if ui.open_menu == Some(button.category) {
            canvas.set_blend_mode(BlendMode::Blend);
            canvas.set_draw_color(Color::RGBA(40, 30, 12, 60));
            canvas
                .fill_rect(Rect::new(button.x, 0, button.width, MENU_BAR_HEIGHT))
                .map_err(runtime_error)?;
            canvas.set_blend_mode(BlendMode::None);
        }
        draw_shadowed_text(
            canvas,
            font,
            button.label,
            button.x.saturating_add(8),
            5,
            TOP_MENU_TEXT_COLOR,
        )?;
    }
    if let Some(category) = ui.open_menu {
        if let Some(rect) = ui.dropdown_rect(category) {
            chrome::draw_dropdown_chrome(
                canvas,
                textures,
                rect.x,
                rect.y,
                rect.width,
                rect.height,
            )?;
            let shortcut_x = ui
                .dropdown_shortcut_x(category)
                .unwrap_or_else(|| rect.x.saturating_add(90));
            for (index, item) in ui.items_for(category).iter().enumerate() {
                if let Some(row) = ui.dropdown_row_rect(category, index) {
                    let text_color = if item.enabled {
                        DROPDOWN_ENABLED_TEXT_COLOR
                    } else {
                        DROPDOWN_DISABLED_TEXT_COLOR
                    };
                    let text_x = row.x.saturating_add(7);
                    let text_y = row.y.saturating_add(1);
                    draw_text(canvas, font, item.label, text_x, text_y, text_color)?;
                    if !item.shortcut.is_empty() {
                        draw_text(canvas, font, item.shortcut, shortcut_x, text_y, text_color)?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Draws the bottom control panel.
fn draw_control_panel(
    canvas: &mut Canvas<Window>,
    textures: &GameplayTextures<'_>,
    state: &GameState,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    let y = i32::try_from(DEFAULT_WINDOW_HEIGHT.saturating_sub(CONTROL_PANEL_HEIGHT))
        .map_or(584, |value| value);
    draw_control_panel_chrome(canvas, textures, y)?;
    draw_resource_counter(canvas, textures, state, font)?;
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
    draw_end_turn_button(canvas, textures, font, y)?;
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

/// Draws the original layered lower control-panel chrome.
fn draw_control_panel_chrome(
    canvas: &mut Canvas<Window>,
    textures: &GameplayTextures<'_>,
    y: i32,
) -> Result<(), GameplayError> {
    canvas.set_draw_color(Color::RGB(40, 43, 48));
    canvas
        .fill_rect(Rect::new(0, y, CONTROL_PANEL_WIDTH, CONTROL_PANEL_HEIGHT))
        .map_err(runtime_error)?;
    for (name, x_offset, y_offset) in [
        ("upfg01.rim", 0_i32, 0_i32),
        ("upfg03.rim", 542_i32, 31_i32),
        ("upfg04.rim", 189_i32, 162_i32),
    ] {
        if let Some(texture) = textures.ui_texture(name) {
            let query = texture.query();
            canvas
                .copy(
                    texture,
                    None,
                    Rect::new(
                        x_offset,
                        y.saturating_add(y_offset),
                        query.width,
                        query.height,
                    ),
                )
                .map_err(runtime_error)?;
        }
    }
    if let Some(texture) = textures.ui_texture("upst01.rim") {
        let query = texture.query();
        canvas
            .copy(
                texture,
                None,
                Rect::new(
                    STATUS_BAR_X,
                    y.saturating_add(STATUS_BAR_Y),
                    query.width,
                    query.height,
                ),
            )
            .map_err(runtime_error)?;
    }
    Ok(())
}

/// Draws the original end-turn button on the lower control panel.
fn draw_end_turn_button(
    canvas: &mut Canvas<Window>,
    textures: &GameplayTextures<'_>,
    font: Option<&Font<'_, '_>>,
    panel_y: i32,
) -> Result<(), GameplayError> {
    let x = END_TURN_BUTTON_X;
    let y = panel_y.saturating_add(END_TURN_BUTTON_Y);
    if let Some(texture) = textures.ui_texture("upcb14u.rim") {
        let query = texture.query();
        canvas
            .copy(texture, None, Rect::new(x, y, query.width, query.height))
            .map_err(runtime_error)?;
    } else {
        canvas.set_draw_color(Color::RGB(75, 63, 38));
        canvas
            .fill_rect(Rect::new(
                x,
                y,
                END_TURN_BUTTON_WIDTH,
                END_TURN_BUTTON_HEIGHT,
            ))
            .map_err(runtime_error)?;
        canvas.set_draw_color(Color::RGB(120, 92, 40));
        canvas
            .draw_rect(Rect::new(
                x,
                y,
                END_TURN_BUTTON_WIDTH,
                END_TURN_BUTTON_HEIGHT,
            ))
            .map_err(runtime_error)?;
    }
    draw_text(
        canvas,
        font,
        "END",
        x.saturating_add(9),
        y.saturating_add(2),
        Color::RGB(255, 235, 140),
    )?;
    draw_text(
        canvas,
        font,
        "TURN",
        x.saturating_add(5),
        y.saturating_add(68),
        Color::RGB(255, 235, 140),
    )
}

/// Draws the top resource counter row on the control panel/menu bar.
fn draw_resource_counter(
    canvas: &mut Canvas<Window>,
    textures: &GameplayTextures<'_>,
    state: &GameState,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    for (index, (icon, label)) in [
        ("upic34.rim", format!("{}", state.gold)),
        ("upic33.rim", format!("{}", state.public_works)),
        ("upic35.rim", "72".to_owned()),
        ("upic36.rim", "0".to_owned()),
    ]
    .into_iter()
    .enumerate()
    {
        let x = 516_i32
            .saturating_add(i32::try_from(index).map_or(0, |value| value.saturating_mul(74)));
        if let Some(texture) = textures.ui_texture(icon) {
            let query = texture.query();
            canvas
                .copy(texture, None, Rect::new(x, 3, query.width, query.height))
                .map_err(runtime_error)?;
        }
        draw_text(canvas, font, &label, x + 24, 5, Color::RGB(230, 224, 190))?;
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
        canvas.set_draw_color(Color::RGB(12, 18, 12));
        canvas
            .fill_rect(Rect::new(x + 12, 50, 120, 80))
            .map_err(runtime_error)?;
        draw_minimap(canvas, state, x + 12, 50, 120, 80)?;
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

/// Draws an original-style mini map using real terrain colors.
fn draw_minimap(
    canvas: &mut Canvas<Window>,
    state: &GameState,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(), GameplayError> {
    let scale_x = width.max(1);
    let scale_y = height.max(1);
    for tile_y in 0..state.height {
        for tile_x in 0..state.width {
            let Some(tile) = state.tile(MapPoint::new(tile_x, tile_y)) else {
                continue;
            };
            let px = x.saturating_add(
                i32::from(tile_x)
                    .saturating_mul(i32::try_from(scale_x).map_or(1, |value| value))
                    .checked_div(i32::from(state.width).max(1))
                    .unwrap_or(0),
            );
            let py = y.saturating_add(
                i32::from(tile_y)
                    .saturating_mul(i32::try_from(scale_y).map_or(1, |value| value))
                    .checked_div(i32::from(state.height).max(1))
                    .unwrap_or(0),
            );
            canvas.set_draw_color(fogged_color(tile_color(tile.terrain), tile.fog));
            canvas
                .draw_point(Point::new(px, py))
                .map_err(runtime_error)?;
        }
    }
    Ok(())
}

/// Draws pending popups.
fn draw_popups(
    canvas: &mut Canvas<Window>,
    textures: &GameplayTextures<'_>,
    state: &GameState,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    if state.popups.is_empty() {
        return Ok(());
    }
    if let Some(texture) = textures.ui_texture("uptg20e.rim") {
        let query = texture.query();
        canvas
            .copy(
                texture,
                None,
                Rect::new(250, 160, query.width, query.height),
            )
            .map_err(runtime_error)?;
    } else {
        canvas.set_draw_color(Color::RGB(32, 28, 24));
        canvas
            .fill_rect(Rect::new(260, 180, 500, 170))
            .map_err(runtime_error)?;
        canvas.set_draw_color(Color::RGB(210, 180, 90));
        canvas
            .draw_rect(Rect::new(260, 180, 500, 170))
            .map_err(runtime_error)?;
    }
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

/// Tiles a texture horizontally.
fn tile_texture_horizontally(
    canvas: &mut Canvas<Window>,
    texture: &Texture<'_>,
    x: i32,
    y: i32,
    width: u32,
) -> Result<(), GameplayError> {
    let query = texture.query();
    let tile_width = query.width.max(1);
    let mut drawn = 0_u32;
    while drawn < width {
        let copy_width = tile_width.min(width.saturating_sub(drawn));
        canvas
            .copy(
                texture,
                Rect::new(0, 0, copy_width, query.height),
                Rect::new(
                    x.saturating_add(i32::try_from(drawn).map_or(0, |value| value)),
                    y,
                    copy_width,
                    query.height,
                ),
            )
            .map_err(runtime_error)?;
        drawn = drawn.saturating_add(copy_width);
    }
    Ok(())
}

/// Returns fallback color for terrain.
const fn tile_color(terrain: TerrainKind) -> Color {
    let (red, green, blue) = terrain.fallback_color();
    Color::RGB(red, green, blue)
}

/// Applies fog darkening for the minimap.
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

#[cfg(test)]
mod tests;
#[cfg(test)]
mod pixel_tests;
