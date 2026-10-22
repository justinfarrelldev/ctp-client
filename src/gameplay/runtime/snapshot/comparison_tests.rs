//! Full-frame comparison has no gameplay masks, file sinks or network lookups.
use super::{
    FrameCapture,
    comparison::{Settings, compare},
};

/// Solid in-memory RGBA only; this helper cannot write a capture.
fn solid(width: u32, height: u32, color: [u8; 4]) -> FrameCapture {
    FrameCapture {
        width,
        height,
        rgba: color.repeat(usize::try_from(width.saturating_mul(height)).expect("small fixture")),
    }
}
#[test]
fn complete_viewport_is_compared_after_explicit_os_decoration_crop_including_partial_cells() {
    let actual = solid(7, 5, [10, 20, 30, 255]);
    let mut decorated = solid(11, 10, [255, 0, 255, 255]);
    for y in 0_usize..5_usize {
        for x in 0_usize..7_usize {
            let offset = y
                .saturating_add(3)
                .saturating_mul(11)
                .saturating_add(x.saturating_add(2))
                .saturating_mul(4);
            decorated.rgba[offset..offset.saturating_add(4)].copy_from_slice(&[10, 20, 30, 255]);
        }
    }
    let report = compare(
        &actual,
        &decorated,
        (2, 3),
        Settings {
            cell_width: 4,
            cell_height: 4,
            channel_tolerance: 0,
        },
    )
    .expect("explicit complete game viewport");
    assert_eq!(
        report.compared_pixels, 35,
        "no title/panel/map pixel is masked out"
    );
    assert_eq!(report.changed_pixels, 0, "OS frame alone is excluded");
    assert_eq!(
        report
            .cells
            .iter()
            .map(|cell| cell.compared_pixels)
            .sum::<u64>(),
        35,
        "partial right/bottom cells still count"
    );
    assert_eq!(
        report.cells.len(),
        4,
        "bounded spatial diagnostics, not a single diluted full-frame average"
    );
    assert!(report.identical(), "actual game pixels match exactly");
}
#[test]
fn a_tiny_missing_control_and_truncated_crop_cannot_hide_in_black_background() {
    let reference = solid(96, 64, [0, 0, 0, 255]);
    let mut actual = reference.clone();
    let offset = 63_usize
        .saturating_mul(96)
        .saturating_add(95)
        .saturating_mul(4);
    actual.rgba[offset] = 255;
    let report = compare(&actual, &reference, (0, 0), Settings::default()).expect("complete frame");
    assert_eq!(
        report.changed_pixels, 1,
        "even one last-edge pixel is examined"
    );
    assert_eq!(
        report.cells.last().expect("edge cell").changed_pixels,
        1,
        "localized failure cannot be averaged away by empty map"
    );
    assert!(
        !report.identical(),
        "no success merely because most of the screen is black"
    );
    let shifted = compare(&actual, &reference, (1, 0), Settings::default());
    assert!(
        shifted.is_err(),
        "a shifted crop may not silently truncate the viewport"
    );
}
#[test]
fn one_pixel_interior_displacement_fails_in_both_adjacent_spatial_cells() {
    let mut actual = solid(96, 64, [0, 0, 0, 255]);
    let mut reference = actual.clone();
    for y in 10_usize..26_usize {
        for (frame, x) in [(&mut actual, 32_usize), (&mut reference, 31_usize)] {
            let index = y.saturating_mul(96).saturating_add(x).saturating_mul(4);
            frame.rgba[index..index.saturating_add(3)].copy_from_slice(&[255_u8; 3]);
        }
    }
    let report = compare(&actual, &reference, (0, 0), Settings::default())
        .expect("unshifted complete viewport");
    assert_eq!(
        report.changed_pixels, 32,
        "one-pixel misplaced rule is not registered away"
    );
    assert_eq!(
        report.cells[0].changed_pixels, 16,
        "old position is missing"
    );
    assert_eq!(
        report.cells[1].changed_pixels, 16,
        "new position is misplaced"
    );
}
#[test]
fn capture_quantization_is_reported_separately_from_exact_identity_and_alpha() {
    let actual = solid(2, 2, [10, 20, 30, 255]);
    let expected = solid(2, 2, [15, 23, 31, 0]);
    let report = compare(
        &actual,
        &expected,
        (0, 0),
        Settings {
            channel_tolerance: 5,
            ..Settings::default()
        },
    )
    .expect("declared per-channel capture tolerance");
    assert_eq!(
        report.changed_pixels, 0,
        "small RGB capture quantization is bounded explicitly"
    );
    assert_eq!(
        report.absolute_rgb_error, 36,
        "all RGB differences remain visible in the report"
    );
    assert!(
        !report.identical(),
        "tolerated is not identical; alpha is not a visible framebuffer color"
    );
}
#[test]
fn malformed_frame_extents_bytes_and_zero_cell_sizes_fail_without_panicking() {
    let valid = solid(2, 2, [0, 0, 0, 255]);
    let impossible = FrameCapture {
        width: u32::MAX,
        height: u32::MAX,
        rgba: Vec::new(),
    };
    assert!(
        compare(&impossible, &valid, (0, 0), Settings::default()).is_err(),
        "header multiplication cannot wrap into a valid buffer"
    );
    let mut malformed = valid.clone();
    malformed.rgba.pop();
    assert!(
        compare(&malformed, &valid, (0, 0), Settings::default()).is_err(),
        "bad actual buffer"
    );
    assert!(
        compare(&valid, &malformed, (0, 0), Settings::default()).is_err(),
        "bad reference buffer"
    );
    assert!(
        compare(&valid, &valid, (u32::MAX, 0), Settings::default()).is_err(),
        "crop overflow"
    );
    assert!(
        compare(
            &valid,
            &valid,
            (0, 0),
            Settings {
                cell_width: 0,
                ..Settings::default()
            }
        )
        .is_err(),
        "zero cell width"
    );
    assert!(
        compare(
            &solid(0, 2, [0, 0, 0, 255]),
            &valid,
            (0, 0),
            Settings::default()
        )
        .is_err(),
        "empty frame is never a successful acceptance"
    );
}
