//! Viewport mapping for proportional 800x600 shell rendering in resized SDL windows.

use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::main_menu::{MainMenuError, MenuRect};

use super::runtime_error;

/// Computes the largest proportional 800x600 menu viewport for a window size.
fn menu_viewport_for_output_size(output_width: u32, output_height: u32) -> MenuRect {
    let menu_width = super::super::MAIN_MENU_WIDTH;
    let menu_height = super::super::MAIN_MENU_HEIGHT;
    if output_width == 0 || output_height == 0 {
        return MenuRect::new(0, 0, 0, 0);
    }

    let output_scaled_by_height = u64::from(output_width).saturating_mul(u64::from(menu_height));
    let output_scaled_by_width = u64::from(output_height).saturating_mul(u64::from(menu_width));
    let (width, height) = if output_scaled_by_height <= output_scaled_by_width {
        let computed_height = u64::from(output_width)
            .saturating_mul(u64::from(menu_height))
            .checked_div(u64::from(menu_width))
            .and_then(|value| u32::try_from(value).ok())
            .map_or(output_height, |value| value.min(output_height));
        (output_width, computed_height)
    } else {
        let computed_width = u64::from(output_height)
            .saturating_mul(u64::from(menu_width))
            .checked_div(u64::from(menu_height))
            .and_then(|value| u32::try_from(value).ok())
            .map_or(output_width, |value| value.min(output_width));
        (computed_width, output_height)
    };
    let x = centered_offset(output_width, width);
    let y = centered_offset(output_height, height);
    MenuRect::new(x, y, width, height)
}

/// Returns the centered offset for an extent inside an output dimension.
fn centered_offset(output_extent: u32, viewport_extent: u32) -> i32 {
    output_extent
        .saturating_sub(viewport_extent)
        .checked_div(2)
        .and_then(|value| i32::try_from(value).ok())
        .map_or(0_i32, |value| value)
}

/// Converts a window/output-space point into original 800x600 menu coordinates.
pub(in crate::main_menu::runtime) fn output_point_to_menu_point(
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

/// Prepares a frame by clearing letterbox space and enabling proportional menu scaling.
pub(super) fn prepare_menu_canvas(canvas: &mut Canvas<Window>) -> Result<(), MainMenuError> {
    canvas
        .set_logical_size(0, 0)
        .map_err(|error| runtime_error(error.to_string()))?;
    canvas.set_scale(1.0, 1.0).map_err(runtime_error)?;
    canvas.set_viewport(None);
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas
        .set_logical_size(
            super::super::MAIN_MENU_WIDTH,
            super::super::MAIN_MENU_HEIGHT,
        )
        .map_err(|error| runtime_error(error.to_string()))
}

#[cfg(test)]
mod tests {
    //! Unit tests for runtime coordinate transforms.

    use super::*;

    #[test]
    fn output_points_are_translated_from_scaled_resolution_viewport() {
        assert_eq!(
            menu_viewport_for_output_size(1024_u32, 768_u32),
            MenuRect::new(0, 0, 1024, 768),
            "4:3 outputs should fill the entire window instead of leaving black on all sides"
        );
        assert_eq!(
            output_point_to_menu_point(1024_u32, 768_u32, 0_i32, 0_i32),
            Some((0_i32, 0_i32)),
            "the scaled viewport origin should map to original menu coordinates"
        );
        assert_eq!(
            output_point_to_menu_point(1024_u32, 768_u32, 512_i32, 384_i32),
            Some((400_i32, 300_i32)),
            "scaled center clicks should map back to the original 800x600 center"
        );
    }

    #[test]
    fn widescreen_outputs_use_side_pillarbox_only() {
        assert_eq!(
            menu_viewport_for_output_size(1920_u32, 1080_u32),
            MenuRect::new(240, 0, 1440, 1080),
            "16:9 outputs should fill height and leave black bars only on the sides"
        );
        assert_eq!(
            output_point_to_menu_point(1920_u32, 1080_u32, 240_i32, 0_i32),
            Some((0_i32, 0_i32)),
            "left edge of the scaled viewport should be active"
        );
        assert_eq!(
            output_point_to_menu_point(1920_u32, 1080_u32, 0_i32, 540_i32),
            None,
            "pillarbox space should not activate controls"
        );
    }

    #[test]
    fn tall_outputs_use_top_bottom_letterbox_only() {
        assert_eq!(
            menu_viewport_for_output_size(1280_u32, 1024_u32),
            MenuRect::new(0, 32, 1280, 960),
            "tall outputs should fill width and leave black bars only at top/bottom"
        );
        assert_eq!(
            output_point_to_menu_point(1280_u32, 1024_u32, 0_i32, 32_i32),
            Some((0_i32, 0_i32)),
            "top edge of the scaled viewport should be active"
        );
        assert_eq!(
            output_point_to_menu_point(1280_u32, 1024_u32, 640_i32, 0_i32),
            None,
            "letterbox space should not activate controls"
        );
    }
}
