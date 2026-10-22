//! Complete RGB framebuffer comparison, with explicit decoration crops and no masks.
use super::FrameCapture;
use crate::gameplay::{GameplayError, runtime_error};
use serde::Serialize;

/// Capture tolerance and spatial reporting, never exclusions for failing controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    /// Maximum per-channel quantization delta before a pixel counts as different.
    pub channel_tolerance: u8,
    /// Physical diagnostic cell width; partial edge cells remain included.
    pub cell_width: u32,
    /// Physical diagnostic cell height.
    pub cell_height: u32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            channel_tolerance: 8,
            cell_width: 32,
            cell_height: 32,
        }
    }
}
/// One dense spatial cell; sparse controls cannot disappear in a global average.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Cell {
    /// Left physical viewport pixel.
    pub x: u32,
    /// Top physical viewport pixel.
    pub y: u32,
    /// Actual extent, including a partial last column.
    pub width: u32,
    /// Actual extent, including a partial last row.
    pub height: u32,
    /// Every visible RGB pixel in the cell, with no masks.
    pub compared_pixels: u64,
    /// Pixels exceeding the declared per-channel capture tolerance.
    pub changed_pixels: u64,
    /// Unthresholded RGB absolute error; tolerated differences are not discarded.
    pub absolute_rgb_error: u64,
}
/// Complete viewport result, intentionally not an automatic parity certification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    /// Total physical pixels compared.
    pub compared_pixels: u64,
    /// Every pixel exceeding tolerance, including map/chrome/text/edge pixels.
    pub changed_pixels: u64,
    /// Unthresholded sum of all three visible channel differences.
    pub absolute_rgb_error: u64,
    /// Dense complete-coverage spatial diagnostics.
    pub cells: Vec<Cell>,
}
impl Report {
    /// Actual display RGB identity, not merely an average under a loose budget.
    #[must_use]
    pub const fn identical(&self) -> bool {
        self.absolute_rgb_error == 0
    }
}

/// Compares all actual viewport pixels with an explicit crop of the provided image.
/// Alpha is not a screenshot color channel. Nothing inside the viewport is masked.
///
/// # Errors
/// Rejects malformed buffers, empty extents, crop overflow/truncation, zero cell
/// sizes and allocation/metric overflow. No input dimensions can cause indexing panics.
pub fn compare(
    actual: &FrameCapture,
    reference: &FrameCapture,
    origin: (u32, u32),
    settings: Settings,
) -> Result<Report, GameplayError> {
    let compared_pixels = validate(actual)?;
    validate(reference)?;
    if settings.cell_width == 0
        || settings.cell_height == 0
        || origin
            .0
            .checked_add(actual.width)
            .is_none_or(|right| right > reference.width)
        || origin
            .1
            .checked_add(actual.height)
            .is_none_or(|bottom| bottom > reference.height)
    {
        return Err(runtime_error(
            "invalid full-frame crop or diagnostic cell size".to_owned(),
        ));
    }
    let columns = actual.width.div_ceil(settings.cell_width);
    let rows = actual.height.div_ceil(settings.cell_height);
    let count = usize::try_from(u64::from(columns).saturating_mul(u64::from(rows)))
        .map_err(|error| runtime_error(error.to_string()))?;
    let mut cells = Vec::new();
    cells
        .try_reserve_exact(count)
        .map_err(|error| runtime_error(error.to_string()))?;
    for row in 0_u32..rows {
        for column in 0_u32..columns {
            let x = column.saturating_mul(settings.cell_width);
            let y = row.saturating_mul(settings.cell_height);
            let width = actual.width.saturating_sub(x).min(settings.cell_width);
            let height = actual.height.saturating_sub(y).min(settings.cell_height);
            cells.push(Cell {
                x,
                y,
                width,
                height,
                compared_pixels: u64::from(width).saturating_mul(u64::from(height)),
                changed_pixels: 0,
                absolute_rgb_error: 0,
            });
        }
    }
    let mut report = Report {
        compared_pixels,
        changed_pixels: 0,
        absolute_rgb_error: 0,
        cells,
    };
    accumulate(actual, reference, origin, settings, columns, &mut report)?;
    Ok(report)
}
/// Validates bytes before any pixel access or grid allocation.
fn validate(frame: &FrameCapture) -> Result<u64, GameplayError> {
    let count = u64::from(frame.width)
        .checked_mul(u64::from(frame.height))
        .ok_or_else(|| runtime_error("frame pixel-count overflow".to_owned()))?;
    let bytes = count
        .checked_mul(4)
        .and_then(|bytes| usize::try_from(bytes).ok());
    if count == 0 || bytes != Some(frame.rgba.len()) {
        return Err(runtime_error(
            "invalid full-frame dimensions or RGBA byte count".to_owned(),
        ));
    }
    Ok(count)
}
/// Checked physical byte index; crop bounds and buffers have already been validated.
fn offset(width: u32, x: u32, y: u32) -> Result<usize, GameplayError> {
    u64::from(y)
        .checked_mul(u64::from(width))
        .and_then(|row| row.checked_add(u64::from(x)))
        .and_then(|pixel| pixel.checked_mul(4))
        .and_then(|byte| usize::try_from(byte).ok())
        .ok_or_else(|| runtime_error("frame pixel offset overflow".to_owned()))
}
/// Visits every pixel exactly once and retains both raw and tolerance-aware error.
fn accumulate(
    actual: &FrameCapture,
    reference: &FrameCapture,
    origin: (u32, u32),
    settings: Settings,
    columns: u32,
    report: &mut Report,
) -> Result<(), GameplayError> {
    for y in 0_u32..actual.height {
        for x in 0_u32..actual.width {
            let actual_offset = offset(actual.width, x, y)?;
            let expected_offset = offset(
                reference.width,
                x.saturating_add(origin.0),
                y.saturating_add(origin.1),
            )?;
            let mut error = 0_u64;
            let mut changed = false;
            for channel in 0_usize..3_usize {
                let difference = actual.rgba[actual_offset.saturating_add(channel)]
                    .abs_diff(reference.rgba[expected_offset.saturating_add(channel)]);
                error = error.saturating_add(u64::from(difference));
                changed |= difference > settings.channel_tolerance;
            }
            let cell_index = u64::from(y.div_euclid(settings.cell_height))
                .saturating_mul(u64::from(columns))
                .saturating_add(u64::from(x.div_euclid(settings.cell_width)));
            let cell = usize::try_from(cell_index)
                .ok()
                .and_then(|index| report.cells.get_mut(index))
                .ok_or_else(|| runtime_error("invalid comparison grid index".to_owned()))?;
            cell.absolute_rgb_error = cell
                .absolute_rgb_error
                .checked_add(error)
                .ok_or_else(|| runtime_error("cell RGB metric overflow".to_owned()))?;
            cell.changed_pixels = cell.changed_pixels.saturating_add(u64::from(changed));
            report.absolute_rgb_error = report
                .absolute_rgb_error
                .checked_add(error)
                .ok_or_else(|| runtime_error("full-frame RGB metric overflow".to_owned()))?;
            report.changed_pixels = report.changed_pixels.saturating_add(u64::from(changed));
        }
    }
    Ok(())
}
