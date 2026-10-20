//! One integer transform for native-frame presentation and window mouse input.
//!
//! SDL's floating logical-event filter can truncate exact positions (450 became
//! 449 at 1.2x). Keep SDL in physical coordinates and round once, explicitly.

use crate::gameplay::{GameplayError, runtime_error};
use sdl2::{event::Event, rect::Rect, render::Canvas, video::Window};

/// Uniform native-frame rectangle; pillar/letterboxing never changes game layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Viewport {
    /// Drawable-pixel destination of the 800x600 frame.
    destination: Rect,
    /// Window-coordinate dimensions, which may differ on high-DPI displays.
    window: (u32, u32),
    /// Renderer drawable-pixel dimensions.
    output: (u32, u32),
}

impl Viewport {
    /// Bounded integer 4:3 fit, with the same rectangle used in both directions.
    fn new(window: (u32, u32), output: (u32, u32)) -> Option<Self> {
        if window.0 == 0_u32 || window.1 == 0_u32 || output.0 == 0_u32 || output.1 == 0_u32 {
            return None;
        }
        let (width, height) = if u64::from(output.0).saturating_mul(600_u64)
            <= u64::from(output.1).saturating_mul(800_u64)
        {
            (
                output.0,
                u32::try_from(
                    u64::from(output.0)
                        .saturating_mul(600_u64)
                        .div_euclid(800_u64),
                )
                .ok()?,
            )
        } else {
            (
                u32::try_from(
                    u64::from(output.1)
                        .saturating_mul(800_u64)
                        .div_euclid(600_u64),
                )
                .ok()?,
                output.1,
            )
        };
        if width == 0_u32 || height == 0_u32 {
            return None;
        }
        Some(Self {
            destination: Rect::new(
                i32::try_from(output.0.saturating_sub(width).div_euclid(2_u32)).ok()?,
                i32::try_from(output.1.saturating_sub(height).div_euclid(2_u32)).ok()?,
                width,
                height,
            ),
            window,
            output,
        })
    }

    /// Round window points exactly once, including high DPI and viewport borders.
    fn point(self, x: i32, y: i32) -> Option<(i32, i32)> {
        let axis =
            |position: i32, window: u32, output: u32, offset: i32, extent: u32, native: i128| {
                let local = i128::from(position)
                    .saturating_mul(i128::from(output))
                    .saturating_sub(i128::from(offset).saturating_mul(i128::from(window)));
                let denominator = i128::from(window).saturating_mul(i128::from(extent));
                if local < 0_i128 || local >= denominator {
                    return None;
                }
                i32::try_from(
                    local
                        .saturating_mul(native)
                        .saturating_add(denominator.div_euclid(2_i128))
                        .checked_div(denominator)?
                        .min(native.saturating_sub(1_i128)),
                )
                .ok()
            };
        Some((
            axis(
                x,
                self.window.0,
                self.output.0,
                self.destination.x(),
                self.destination.width(),
                800_i128,
            )?,
            axis(
                y,
                self.window.1,
                self.output.1,
                self.destination.y(),
                self.destination.height(),
                600_i128,
            )?,
        ))
    }
}

/// Uses the actual renderer drawable size rather than assuming window pixels.
fn current(canvas: &Canvas<Window>) -> Result<Viewport, GameplayError> {
    Viewport::new(
        canvas.window().size(),
        canvas.output_size().map_err(runtime_error)?,
    )
    .ok_or_else(|| runtime_error("gameplay viewport has no drawable area".to_owned()))
}

/// Destination shared with mouse translation; no screenshot-crop compensation.
pub(super) fn destination(canvas: &Canvas<Window>) -> Result<Rect, GameplayError> {
    Ok(current(canvas)?.destination)
}

/// Translate pointer events exactly once; border clicks do not target the map.
pub(super) fn map_event(
    canvas: &Canvas<Window>,
    event: &Event,
) -> Result<Option<Event>, GameplayError> {
    let point = |x, y| current(canvas).map(|viewport| viewport.point(x, y));
    Ok(match event.clone() {
        Event::MouseMotion {
            timestamp,
            window_id,
            which,
            mousestate,
            x,
            y,
            xrel,
            yrel,
        } => point(x, y)?.map(|(x, y)| Event::MouseMotion {
            timestamp,
            window_id,
            which,
            mousestate,
            x,
            y,
            xrel,
            yrel,
        }),
        Event::MouseButtonDown {
            timestamp,
            window_id,
            which,
            mouse_btn,
            clicks,
            x,
            y,
        } => point(x, y)?.map(|(x, y)| Event::MouseButtonDown {
            timestamp,
            window_id,
            which,
            mouse_btn,
            clicks,
            x,
            y,
        }),
        Event::MouseButtonUp {
            timestamp,
            window_id,
            which,
            mouse_btn,
            clicks,
            x,
            y,
        } => point(x, y)?.map(|(x, y)| Event::MouseButtonUp {
            timestamp,
            window_id,
            which,
            mouse_btn,
            clicks,
            x,
            y,
        }),
        other => Some(other),
    })
}

#[cfg(test)]
mod tests {
    use super::Viewport;
    #[test]
    fn logical_pointer_fit_is_exact_for_letterboxing_native_and_high_dpi_outputs() {
        for (window, output, point, expected) in [
            (
                (1280_u32, 720_u32),
                (1280_u32, 720_u32),
                (700_i32, 420_i32),
                (450_i32, 350_i32),
            ),
            (
                (800_u32, 600_u32),
                (1600_u32, 1200_u32),
                (450_i32, 350_i32),
                (450_i32, 350_i32),
            ),
            (
                (640_u32, 480_u32),
                (640_u32, 480_u32),
                (320_i32, 240_i32),
                (400_i32, 300_i32),
            ),
            (
                (800_u32, 1000_u32),
                (800_u32, 1000_u32),
                (400_i32, 500_i32),
                (400_i32, 300_i32),
            ),
        ] {
            let view = Viewport::new(window, output).expect("valid viewport");
            assert_eq!(
                view.point(point.0, point.1),
                Some(expected),
                "one transform for output {output:?}"
            );
        }
        let view = Viewport::new((1280_u32, 720_u32), (1280_u32, 720_u32)).expect("widescreen");
        assert_eq!(
            view.point(159_i32, 300_i32),
            None,
            "left border does not order a unit"
        );
        assert_eq!(
            view.point(1120_i32, 300_i32),
            None,
            "right border does not activate UI"
        );
        assert!(
            Viewport::new((0_u32, 0_u32), (1280_u32, 720_u32)).is_none(),
            "minimized viewport is not divided by zero"
        );
    }
}
