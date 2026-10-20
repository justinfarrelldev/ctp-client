//! Layered lower-right panel from the release `controlpanel.ldl`.

#![expect(
    clippy::default_numeric_fallback,
    reason = "LDL offsets and order slot indices are original fixed i32 pixel values"
)]

use super::actors::draw_stack_shield;
use super::chrome::{draw_art, draw_art_rect};
use super::text::{draw_centered_text, draw_text};
use crate::gameplay::model::{GameState, UnitKind};
use crate::gameplay::runtime::controls::{SHORTCUTS, TABS, order_rect};
use crate::gameplay::runtime::layout::{panel_origin, panel_rect};
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::ui_model::{ControlPanelTab, InGameUiModel};
use crate::gameplay::{GameplayError, runtime_error};
use core::cmp::Ordering;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::Font;
use sdl2::video::Window;

/// Text color inherited from `CTP2_STANDARD_FONT`.
const LABEL: Color = Color::RGB(50, 50, 50);
/// Draws all original chrome without a synthetic opaque gray bounding rectangle.
pub(super) fn draw_panel(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
    ui: &InGameUiModel,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    for (name, x, y) in [
        ("upfg01.rim", 0_i32, 0_i32),
        ("upfg03.rim", 542, 31),
        ("upfg04.rim", 189, 162),
        ("upst01.rim", 126, 167),
    ] {
        panel_art(canvas, textures, name, x, y)?;
    }
    for (name, x, y, _, _) in SHORTCUTS {
        panel_art(canvas, textures, name, x, y)?;
    }
    panel_art(canvas, textures, "uppb01u.rim", 56, 87)?;
    let year = if state.turn == 1_u32 {
        "4000BC".to_owned()
    } else {
        format!("Turn {}", state.turn)
    };
    draw_centered_text(
        canvas,
        font,
        &year,
        panel_rect(56, 87, 75, 22),
        LABEL,
        false,
    )?;
    draw_tabs(canvas, textures, ui, font)?;
    match ui.active_tab {
        ControlPanelTab::Unit => draw_unit_panel(canvas, textures, state, ui, font)?,
        ControlPanelTab::Domestic => draw_empire_panel(canvas, state, font)?,
        ControlPanelTab::City => draw_centered_text(
            canvas,
            font,
            "City",
            panel_rect(210, 60, 300, 24),
            LABEL,
            false,
        )?,
        ControlPanelTab::Messages | ControlPanelTab::TileImprovements => {}
    }
    panel_art(canvas, textures, "upcb14u.rim", 563, 60)?;
    draw_centered_text(
        canvas,
        font,
        "END",
        panel_rect(563, 62, 38, 16),
        LABEL,
        true,
    )?;
    draw_centered_text(
        canvas,
        font,
        "TURN",
        panel_rect(563, 128, 38, 16),
        LABEL,
        true,
    )?;
    let status = if ui.status_text.is_empty() {
        "Unexplored"
    } else {
        &ui.status_text
    };
    draw_centered_text(
        canvas,
        font,
        status,
        panel_rect(126, 167, 468, 17),
        LABEL,
        false,
    )
}

/// Copies one panel-relative image layer without resizing its native art.
fn panel_art(
    canvas: &mut Canvas<Window>,
    textures: &GameplayTextures<'_>,
    name: &str,
    x: i32,
    y: i32,
) -> Result<(), GameplayError> {
    let (origin_x, origin_y) = panel_origin();
    draw_art(
        canvas,
        textures,
        name,
        origin_x.saturating_add(x),
        origin_y.saturating_add(y),
    )
}

/// Overlapping 93px tabs at 65px intervals, with the active tab painted last.
fn draw_tabs(
    canvas: &mut Canvas<Window>,
    textures: &GameplayTextures<'_>,
    ui: &InGameUiModel,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    panel_art(canvas, textures, "upfg05.rim", 189, 45)?;
    let active = TABS
        .iter()
        .position(|&(tab, _)| tab == ui.active_tab)
        .unwrap_or(2_usize);
    for index in (0_usize..5_usize)
        .filter(|index| *index != active)
        .chain(core::iter::once(active))
    {
        let x = 189_i32
            .saturating_add(i32::try_from(index).map_or(0_i32, |v| v.saturating_mul(65_i32)));
        let art = match index.cmp(&active) {
            Ordering::Equal => "upta01u.rim",
            Ordering::Less => "upta01ur.rim",
            Ordering::Greater => "upta01ul.rim",
        };
        panel_art(canvas, textures, art, x, 17)?;
        draw_centered_text(
            canvas,
            font,
            TABS[index].1,
            panel_rect(x, 28, 93, 12),
            LABEL,
            false,
        )?;
    }
    Ok(())
}

/// Unit portrait, source stat boxes, spinner and all twelve order-button cells.
fn draw_unit_panel(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
    ui: &InGameUiModel,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    panel_art(
        canvas,
        textures,
        if ui.stack_expanded {
            "upfg51.rim"
        } else {
            "upfg50.rim"
        },
        189,
        45,
    )?;
    panel_art(canvas, textures, "upfg06.rim", 364, 55)?;
    // Unit-label spinner is a three-part sizable image plus the next-unit arrow.
    for (name, x, width) in [
        ("upsp01bl.rim", 205_i32, 8_u32),
        ("upsp01bm.rim", 213, 106),
        ("upsp01br.rim", 319, 10),
    ] {
        draw_art_rect(canvas, textures, name, panel_rect(x, 50, width, 25))?;
    }
    panel_art(canvas, textures, "upsp01cu.rim", 329, 50)?;
    if ui.stack_expanded {
        super::stack::draw_stack(canvas, textures, state)?;
        return draw_orders(canvas, textures, state);
    }
    if let Some(unit) = state.selected_unit() {
        let art = textures.unit_art(state, unit)?;
        if let Some(art) = art.as_ref() {
            if let Some(portrait) = textures.dynamic_ui(&art.portrait)? {
                canvas
                    .copy(portrait, None, panel_rect(207, 78, 96, 72))
                    .map_err(runtime_error)?;
            }
        }
        let source = state.viewer.as_ref().and_then(|viewer| {
            viewer.units.iter().find(|source| {
                crate::gameplay::model::UnitId::from_server_id(&source.id) == unit.id
            })
        });
        let title = art.map_or_else(
            || {
                source.map_or_else(
                    || unit.kind.label().to_owned(),
                    |source| source.type_id.trim_start_matches("UNIT_").replace('_', " "),
                )
            },
            |art| art.label,
        );
        draw_centered_text(
            canvas,
            font,
            &title,
            panel_rect(205, 50, 124, 24),
            LABEL,
            false,
        )?;
        for (value, x, y) in [
            (unit_stat(source, "attack"), 306_i32, 78_i32),
            (unit_stat(source, "defense"), 325, 78),
            (unit_stat(source, "ranged"), 306, 100),
            (unit_stat(source, "armor"), 325, 100),
            (
                unit.movement_points
                    .max(0_i16)
                    .saturating_add(99_i16)
                    .div_euclid(100_i16)
                    .to_string(),
                306,
                122,
            ),
            (unit_stat(source, "firepower"), 325, 122),
        ] {
            draw_centered_text(canvas, font, &value, panel_rect(x, y, 19, 18), LABEL, true)?;
        }
        draw_health(
            canvas,
            source,
            panel_rect(207, 153, 96, 3),
            !state.authoritative,
        )?;
        let count = state
            .units
            .iter()
            .filter(|candidate| candidate.alive && candidate.position == unit.position)
            .count();
        let shield = panel_rect(283, 80, 16, 20);
        draw_stack_shield(
            canvas,
            textures,
            count,
            shield.x(),
            shield.y(),
            unit.owner,
            false,
        )?;
    }
    draw_orders(canvas, textures, state)
}

/// Source health bar, shared with expanded portraits; unknown live health stays
/// black rather than falsely reporting a full green bar.
pub(super) fn draw_health(
    canvas: &mut Canvas<Window>,
    source: Option<&crate::gameplay::server::ServerUnitProjection>,
    rect: Rect,
    explicit_preview: bool,
) -> Result<(), GameplayError> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.fill_rect(rect).map_err(runtime_error)?;
    let bar = crate::gameplay::ui_model::stack::health_bar(source, rect.width())
        .or_else(|| explicit_preview.then_some((rect.width(), (0_u8, 255_u8, 0_u8))));
    if let Some((width, (red, green, blue))) = bar.filter(|bar| bar.0 > 0_u32) {
        canvas.set_draw_color(Color::RGB(red, green, blue));
        canvas
            .fill_rect(Rect::new(rect.x(), rect.y(), width, rect.height()))
            .map_err(runtime_error)?;
    }
    Ok(())
}

/// Reads one authoritative combat value; absent contracts are not fabricated.
fn unit_stat(
    source: Option<&crate::gameplay::server::ServerUnitProjection>,
    field: &str,
) -> String {
    source.map_or_else(
        || "—".to_owned(),
        |source| source.facts[field].as_i64().unwrap_or(0).to_string(),
    )
}

/// Exact 4×3 grid and original command icons; blank slots stay disabled gray.
fn draw_orders(
    canvas: &mut Canvas<Window>,
    textures: &GameplayTextures<'_>,
    state: &GameState,
) -> Result<(), GameplayError> {
    let selected = state.selected_unit();
    let settler = selected.is_some_and(|unit| unit.kind == UnitKind::Settler);
    let controls = crate::gameplay::runtime::controls::panel_controls(state, ControlPanelTab::Unit);
    for index in 0_i32..12_i32 {
        let rect = order_rect(index);
        let enabled = controls
            .iter()
            .find(|control| control.rect == rect)
            .is_some_and(|control| control.enabled);
        let icon = if selected.is_none() {
            None
        } else {
            match index {
                0 => Some("upsi26.rim"),
                3 => Some("upsi29.rim"),
                4 | 8 if settler => Some("upsi04.rim"),
                1 if !settler => Some("upsi27.rim"),
                2 if !settler => Some("upsi28.rim"),
                9 => Some("upsi02.rim"),
                11 => Some("upsi30.rim"),
                _ => None,
            }
        };
        draw_art_rect(
            canvas,
            textures,
            if icon.is_some() && enabled {
                "upbt02u.rim"
            } else {
                "upbt02x.rim"
            },
            Rect::new(rect.x, rect.y, rect.width, rect.height),
        )?;
        if let Some(icon) = icon {
            draw_art_rect(
                canvas,
                textures,
                icon,
                Rect::new(
                    rect.x.saturating_add(7_i32),
                    rect.y.saturating_add(2_i32),
                    26,
                    26,
                ),
            )?;
        }
        if index == 8_i32 && settler {
            // Retail Add to City reuses the settlement silhouette with a plus.
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas
                .fill_rect(Rect::new(
                    rect.x.saturating_add(27_i32),
                    rect.y.saturating_add(4_i32),
                    2,
                    9,
                ))
                .map_err(runtime_error)?;
            canvas
                .fill_rect(Rect::new(
                    rect.x.saturating_add(24_i32),
                    rect.y.saturating_add(7_i32),
                    8,
                    2,
                ))
                .map_err(runtime_error)?;
        }
    }
    Ok(())
}

/// Resource values are drawn from the current state, never screenshot constants.
fn draw_empire_panel(
    canvas: &mut Canvas<Window>,
    state: &GameState,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    for (index, (label, value)) in [
        ("Cities", state.cities.len().to_string()),
        ("Population", "0".to_owned()),
        ("Public Works", state.public_works.to_string()),
        ("Government", String::new()),
        ("Science", String::new()),
    ]
    .into_iter()
    .enumerate()
    {
        let y =
            55_i32.saturating_add(i32::try_from(index).map_or(0_i32, |v| v.saturating_mul(16_i32)));
        let rect = panel_rect(319, y, 150, 16);
        draw_text(canvas, font, label, rect.x(), rect.y(), LABEL)?;
        draw_centered_text(
            canvas,
            font,
            &value,
            panel_rect(469, y, 55, 16),
            LABEL,
            false,
        )?;
    }
    Ok(())
}
