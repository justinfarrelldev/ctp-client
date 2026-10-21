//! Original parchment message rows and 530x188 gold-framed semantic windows.
use super::{
    super::{message_input, textures::GameplayTextures},
    chrome::draw_art_rect,
    context::draw_gold_frame,
    text::{draw_centered_text, draw_text, draw_wrapped_text},
};
use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::{ControlPanelTab, InGameUiModel, MenuRect, messages};
use crate::gameplay::{GameplayError, runtime_error};
use sdl2::{pixels::Color, rect::Rect, render::Canvas, ttf::Font, video::Window};

/// Draws only retained own results; historical eye links cannot disclose new cells.
pub(super) fn draw(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
    ui: &InGameUiModel,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    if ui.active_tab == ControlPanelTab::Messages {
        for (index, notice) in ui
            .messages
            .items
            .iter()
            .enumerate()
            .skip(ui.messages.start)
            .take(5)
        {
            if let Some(row) = ui.messages.row(index) {
                let old = canvas.clip_rect();
                canvas.set_clip_rect(rect(row));
                let color = if ui.messages.read.contains(&notice.id) {
                    Color::RGB(50, 50, 50)
                } else {
                    Color::RGB(150, 25, 15)
                };
                let result = draw_text(
                    canvas,
                    font,
                    &messages::text(notice, state, &ui.management.book),
                    row.x,
                    row.y,
                    color,
                );
                canvas.set_clip_rect(old);
                result?;
            }
        }
    }
    let Some(notice) = ui.messages.current() else {
        return Ok(());
    };
    let window = ui.messages.window();
    draw_gold_frame(canvas, textures, rect(window))?;
    let date = ui
        .messages
        .dates
        .get(&notice.turn)
        .cloned()
        .or_else(|| {
            state
                .viewer
                .as_ref()
                .and_then(|viewer| viewer.interface.calendar.as_ref())
                .filter(|date| date.turn == notice.turn)
                .map(crate::gameplay::server::v2::Calendar::label)
        })
        .unwrap_or_else(|| format!("Turn {}", notice.turn));
    draw_text(
        canvas,
        font,
        &date,
        window.x.saturating_add(20),
        window.y.saturating_add(16),
        Color::RGB(50, 50, 50),
    )?;
    let bounds = Rect::new(
        window.x.saturating_add(22),
        window.y.saturating_add(40),
        window.width.saturating_sub(44),
        92,
    );
    let old = canvas.clip_rect();
    canvas.set_clip_rect(bounds);
    let result = draw_wrapped_text(
        canvas,
        font,
        &messages::text(notice, state, &ui.management.book),
        bounds,
        Color::RGB(35, 35, 35),
    );
    canvas.set_clip_rect(old);
    result?;
    let close = message_input::close_rect(&ui.messages);
    draw_art_rect(canvas, textures, "upbt02u.rim", rect(close))?;
    draw_centered_text(
        canvas,
        font,
        "Close",
        rect(close),
        Color::RGB(50, 50, 50),
        false,
    )?;
    if ui.messages.location(state).is_some() {
        let eye = message_input::eye_rect(&ui.messages);
        if let Some(texture) = textures.dynamic_ui("ups006.rim")? {
            canvas
                .copy(texture, None, rect(eye))
                .map_err(runtime_error)?;
        }
    }
    Ok(())
}
/// Converts presentation geometry at the SDL boundary without one-pixel phantoms.
fn rect(bounds: MenuRect) -> Rect {
    Rect::new(bounds.x, bounds.y, bounds.width, bounds.height)
}
