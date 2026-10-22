//! Read-only complete-frame comparison: renders in memory, never saves PNG/assets.
use ctp_client::gameplay::runtime::snapshot::{
    FrameCapture,
    comparison::{Settings, compare},
    render_preview_scene,
    scene::PreviewScene,
};
use ctp_client::gameplay::ui_model::management::ManagerKind;
use sdl2::{image::LoadSurface, pixels::PixelFormatEnum, surface::Surface};
use std::{error::Error, path::Path};

/// Explicit decoration crop is independent of manager/control geometry.
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let installation = args.first().map_or("ctp2", String::as_str);
    let reference = args
        .get(1)
        .ok_or("provide reference PNG, diagnostic name, game-origin x/y")?;
    let name = args.get(2).map_or("unit-populated", String::as_str);
    let mut origin = (
        args.get(3).map_or(Ok(0_u32), |value| value.parse())?,
        args.get(4).map_or(Ok(20_u32), |value| value.parse())?,
    );
    let scene = if let Some(kind) = name
        .strip_suffix("-populated")
        .and_then(ManagerKind::from_name)
    {
        PreviewScene::PopulatedManager { kind, chart: false }
    } else {
        PreviewScene::Manager(
            ManagerKind::from_name(name).ok_or("unknown explicit diagnostic manager")?,
        )
    };
    let actual = render_preview_scene(Path::new(installation), (640, 480), None, scene)?;
    let surface =
        Surface::from_file(Path::new(reference))?.convert_format(PixelFormatEnum::RGBA32)?;
    let expected = FrameCapture {
        width: surface.width(),
        height: surface.height(),
        rgba: surface.with_lock(<[u8]>::to_vec),
    };
    if args.get(5).is_some_and(|flag| flag == "scan") {
        origin = menu_decoration_origin(&actual, &expected)?;
    }
    println!("Explicit full viewport origin: {},{}", origin.0, origin.1);
    let report = compare(&actual, &expected, origin, Settings::default())?;
    println!(
        "Compared every viewport pixel: {}. Differing pixels (RGB channel delta >8): {}. Raw RGB absolute error: {}. Identical: {}.",
        report.compared_pixels,
        report.changed_pixels,
        report.absolute_rgb_error,
        report.identical()
    );
    let mut worst: Vec<_> = report.cells.iter().collect();
    worst.sort_by_key(|cell| core::cmp::Reverse(cell.absolute_rgb_error));
    for cell in worst.into_iter().take(20) {
        println!(
            "cell {},{} {}x{}: {} / {} changed; RGB error {}",
            cell.x,
            cell.y,
            cell.width,
            cell.height,
            cell.changed_pixels,
            cell.compared_pixels,
            cell.absolute_rgb_error
        );
    }
    println!(
        "Diagnostic comparison only. Synthetic fixture data/capture/font differences must be reconciled; this tool does not certify parity."
    );
    Ok(())
}

/// Optional decoration calibration uses only the unchanged top menu, never
/// manager/panel/map interiors. The final report still compares every pixel.
fn menu_decoration_origin(
    actual: &FrameCapture,
    reference: &FrameCapture,
) -> Result<(u32, u32), Box<dyn Error>> {
    let margin_x = reference
        .width
        .checked_sub(actual.width)
        .ok_or("reference narrower than viewport")?;
    let margin_y = reference
        .height
        .checked_sub(actual.height)
        .ok_or("reference shorter than viewport")?;
    if margin_x > 32 || margin_y > 80 {
        return Err("decoration scan is bounded; provide an explicit crop".into());
    }
    let height = actual.height.min(20);
    let size = usize::try_from(
        u64::from(actual.width)
            .saturating_mul(u64::from(height))
            .saturating_mul(4),
    )?;
    let header = FrameCapture {
        width: actual.width,
        height,
        rgba: actual
            .rgba
            .get(..size)
            .ok_or("invalid header buffer")?
            .to_vec(),
    };
    let mut best = None;
    for y in 0_u32..=margin_y {
        for x in 0_u32..=margin_x {
            let error = compare(
                &header,
                reference,
                (x, y),
                Settings {
                    channel_tolerance: 0,
                    cell_width: actual.width,
                    cell_height: height,
                },
            )?
            .absolute_rgb_error;
            if best.is_none_or(|(_origin, previous)| error < previous) {
                best = Some(((x, y), error));
            }
        }
    }
    best.map(|(origin, _error)| origin)
        .ok_or_else(|| "no valid decoration crop".into())
}

#[cfg(test)]
mod tests {
    use super::{FrameCapture, Settings, compare, menu_decoration_origin};
    #[test]
    fn menu_anchor_finds_real_decoration_without_fitting_a_failing_manager() {
        let mut actual = FrameCapture {
            width: 4,
            height: 32,
            rgba: vec![0_u8; 512],
        };
        let mut reference = FrameCapture {
            width: 7,
            height: 35,
            rgba: vec![255_u8; 980],
        };
        for y in 0_usize..32_usize {
            for x in 0_usize..4_usize {
                let index = y.saturating_mul(4).saturating_add(x).saturating_mul(4);
                actual.rgba[index] =
                    u8::try_from(x.saturating_mul(20).saturating_add(y)).expect("small pattern");
                let target = y
                    .saturating_add(2)
                    .saturating_mul(7)
                    .saturating_add(x.saturating_add(1))
                    .saturating_mul(4);
                reference.rgba[target..target.saturating_add(4)]
                    .copy_from_slice(&actual.rgba[index..index.saturating_add(4)]);
            }
        }
        actual.rgba[392] = 255; // Interior control at (2,24), below the top-menu anchor.
        assert_eq!(
            compare(&actual, &reference, (1, 2), Settings::default())
                .expect("whole frame")
                .changed_pixels,
            1,
            "interior defect remains a full-frame failure"
        );
        assert_eq!(
            menu_decoration_origin(&actual, &reference).expect("menu-only crop"),
            (1_u32, 2_u32),
            "no control-relative translation or truncated viewport"
        );
    }
}
