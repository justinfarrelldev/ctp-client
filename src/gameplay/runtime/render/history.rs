//! Original Statistics graph pane populated exclusively by actual own samples.
use super::text::draw_text;
use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::{InGameUiModel, history, management::ManagerKind};
use crate::gameplay::{GameplayError, runtime_error};
use sdl2::{
    pixels::Color,
    rect::{Point, Rect},
    render::Canvas,
    ttf::Font,
    video::Window,
};

/// Uses the source InfoGraph/InfoPlayerList coordinates, not an invented overlay.
pub(super) fn draw(
    canvas: &mut Canvas<Window>,
    state: &GameState,
    ui: &InGameUiModel,
    font: Option<&Font<'_, '_>>,
) -> Result<(), GameplayError> {
    let Some(view) = ui
        .management
        .view
        .as_ref()
        .filter(|view| view.kind == ManagerKind::Statistics)
    else {
        return Ok(());
    };
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(());
    };
    let widgets = ui.management.widgets(state);
    let Some(graph) = widgets
        .iter()
        .find(|widget| widget.node.name == "InfoGraph")
    else {
        return Ok(());
    };
    let bounds = graph.rect;
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas
        .fill_rect(Rect::new(bounds.x, bounds.y, bounds.width, bounds.height))
        .map_err(runtime_error)?;
    let metric = view
        .selections
        .get("history-metric")
        .map_or("score", String::as_str);
    let points = history::points(&viewer.interface.statistics, metric, bounds);
    canvas.set_draw_color(Color::RGB(40, 90, 255));
    for pair in points.windows(2) {
        canvas
            .draw_line(
                Point::new(pair[0].0, pair[0].1),
                Point::new(pair[1].0, pair[1].1),
            )
            .map_err(runtime_error)?;
    }
    if points.len() == 1 {
        canvas
            .draw_point(Point::new(points[0].0, points[0].1))
            .map_err(runtime_error)?;
    }
    if points.is_empty() {
        draw_text(
            canvas,
            font,
            "History unavailable",
            bounds.x.saturating_add(10),
            bounds.y.saturating_add(10),
            Color::RGB(220, 220, 220),
        )?;
    }
    if let Some(legend) = widgets
        .iter()
        .find(|widget| widget.node.name == "InfoPlayerList")
    {
        draw_text(
            canvas,
            font,
            &format!("Own {metric}"),
            legend.rect.x.saturating_add(6),
            legend.rect.y.saturating_add(6),
            Color::RGB(30, 60, 180),
        )?;
        draw_text(
            canvas,
            font,
            "Foreign history private",
            legend.rect.x.saturating_add(6),
            legend.rect.y.saturating_add(25),
            Color::RGB(80, 70, 60),
        )?;
    }
    Ok(())
}
