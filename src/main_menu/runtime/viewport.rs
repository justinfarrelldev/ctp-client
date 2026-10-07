//! Viewport mapping for crisp 800x600 shell rendering in resized SDL windows.

use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::main_menu::{MainMenuError, MenuRect};

use super::{runtime_error, to_sdl_rect};

/// Computes the crisp, unscaled 800x600 menu viewport for a window size.
fn menu_viewport_for_output_size(output_width: u32, output_height: u32) -> MenuRect {
    let width = super::super::MAIN_MENU_WIDTH.min(output_width);
    let height = super::super::MAIN_MENU_HEIGHT.min(output_height);
    let x = output_width
        .saturating_sub(width)
        .checked_div(2)
        .map_or(0_i32, |value| {
            i32::try_from(value).map_or(0_i32, |converted| converted)
        });
    let y = output_height
        .saturating_sub(height)
        .checked_div(2)
        .map_or(0_i32, |value| {
            i32::try_from(value).map_or(0_i32, |converted| converted)
        });
    MenuRect::new(x, y, width, height)
}

/// Converts a window/output-space point into original 800x600 menu coordinates.
pub(super) fn output_point_to_menu_point(
    output_width: u32,
    output_height: u32,
    point_x: i32,
    point_y: i32,
) -> Option<(i32, i32)> {
    let viewport = menu_viewport_for_output_size(output_width, output_height);
    if !viewport.contains_point(point_x, point_y) {
        return None;
    }

    let local_x = point_x.saturating_sub(viewport.x);
    let local_y = point_y.saturating_sub(viewport.y);
    if viewport.width == super::super::MAIN_MENU_WIDTH
        && viewport.height == super::super::MAIN_MENU_HEIGHT
    {
        return Some((local_x, local_y));
    }

    let scaled_x = scale_viewport_axis(local_x, viewport.width, super::super::MAIN_MENU_WIDTH)?;
    let scaled_y = scale_viewport_axis(local_y, viewport.height, super::super::MAIN_MENU_HEIGHT)?;
    Some((scaled_x, scaled_y))
}

/// Scales one viewport-local coordinate axis into menu coordinates.
fn scale_viewport_axis(
    local_coordinate: i32,
    viewport_extent: u32,
    menu_extent: u32,
) -> Option<i32> {
    if viewport_extent == 0 {
        return None;
    }
    let coordinate = i64::from(local_coordinate);
    let menu_extent_i64 = i64::from(menu_extent);
    let viewport_extent_i64 = i64::from(viewport_extent);
    let scaled = coordinate
        .saturating_mul(menu_extent_i64)
        .checked_div(viewport_extent_i64)?;
    i32::try_from(scaled).ok()
}

/// Prepares a frame by clearing letterbox space and centering the menu viewport.
pub(super) fn prepare_menu_canvas(canvas: &mut Canvas<Window>) -> Result<(), MainMenuError> {
    let (output_width, output_height) = canvas.output_size().map_err(runtime_error)?;
    let viewport = menu_viewport_for_output_size(output_width, output_height);
    canvas.set_viewport(None);
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.set_viewport(Some(to_sdl_rect(&viewport)));
    Ok(())
}

#[cfg(test)]
mod tests {
    //! Unit tests for runtime coordinate transforms.

    use super::*;

    #[test]
    fn output_points_are_translated_from_centered_resolution_viewport() {
        assert_eq!(
            output_point_to_menu_point(1024_u32, 768_u32, 112_i32, 84_i32),
            Some((0_i32, 0_i32)),
            "1024x768 should center the 800x600 menu at a 112,84 offset"
        );
        assert_eq!(
            output_point_to_menu_point(1024_u32, 768_u32, 268_i32, 360_i32),
            Some((156_i32, 276_i32)),
            "post-resize click coordinates should map back to original menu coordinates"
        );
        assert_eq!(
            output_point_to_menu_point(1024_u32, 768_u32, 0_i32, 0_i32),
            None,
            "letterbox space outside the centered menu must not activate controls"
        );
    }
}
