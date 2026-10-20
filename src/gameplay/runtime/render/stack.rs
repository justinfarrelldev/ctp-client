//! Original expanded selection grid, shared with semantic click descriptors.

use crate::gameplay::model::GameState;
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::ui_model::stack::entries;
use crate::gameplay::{GameplayError, runtime_error};
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// Draws installed Icon artwork at each exact `MultipleSelect.UnitN` rectangle.
pub(super) fn draw_stack(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
) -> Result<(), GameplayError> {
    for entry in entries(state) {
        let Some(unit) = state.units.iter().find(|unit| unit.id == entry.unit) else {
            continue;
        };
        let Some(art) = textures.unit_art(state, unit)? else {
            continue;
        };
        let rect = Rect::new(
            entry.rect.x,
            entry.rect.y,
            entry.rect.width,
            entry.rect.height,
        );
        if let Some(icon) = textures.dynamic_ui(&art.icon)? {
            canvas.copy(icon, None, rect).map_err(runtime_error)?;
        }
        let source = state.viewer.as_ref().and_then(|viewer| {
            viewer.units.iter().find(|source| {
                crate::gameplay::model::UnitId::from_server_id(&source.id) == unit.id
            })
        });
        super::panel::draw_health(
            canvas,
            source,
            Rect::new(
                rect.x().saturating_add(2),
                rect.y().saturating_add(22),
                28,
                1,
            ),
            false,
        )?;
    }
    Ok(())
}
