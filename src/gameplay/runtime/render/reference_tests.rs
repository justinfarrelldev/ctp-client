//! Supplied release captures, cropped independently of model control geometry.
//!
//! These small artwork masks check placement despite original RGB565 capture and
//! scaling differences. They are NOT full-screen pixel-parity acceptance tests.

use crate::gameplay::runtime::snapshot::{FrameCapture, render_preview_scene, scene::PreviewScene};
use crate::gameplay::runtime::test_support::HeadlessSdlEnvironment;
use crate::gameplay::ui_model::management::ManagerKind;
use sdl2::image::LoadSurface;
use sdl2::pixels::PixelFormatEnum;
use sdl2::surface::Surface;
use std::path::Path;

#[test]
fn gold_popup_corners_align_with_supplied_graphics_and_sound_release_captures() {
    let guard = crate::sdl_test_lock::lock_sdl_for_test().expect("SDL lock");
    let environment = HeadlessSdlEnvironment::apply();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for (kind, filename, origin, corner) in [
        (
            ManagerKind::Sound,
            "sound-overlay-from-dropdown-ingame.png",
            (7_usize, 20_usize),
            (180_usize, 136_usize),
        ),
        (
            ManagerKind::Graphics,
            "graphics-overlay-ingame-from-dropdown.png",
            (3_usize, 23_usize),
            (48_usize, 84_usize),
        ),
    ] {
        let frame = render_preview_scene(
            &root.join("ctp2"),
            (640_u32, 480_u32),
            None,
            PreviewScene::Manager(kind),
        )
        .expect("real SDL frame");
        let reference = Surface::from_file(root.join("images/ctp2_references").join(filename))
            .expect("provided reference")
            .convert_format(PixelFormatEnum::RGBA32)
            .expect("RGBA reference");
        let error = corner_error(&frame, &reference, origin, corner);
        assert!(
            error <= 45_u64,
            "{filename}: corner RGB mean-sum error {error} exceeds capture/quantization tolerance"
        );
        let displaced = corner_error(
            &frame,
            &reference,
            (
                origin.0.saturating_add(4_usize),
                origin.1.saturating_add(4_usize),
            ),
            corner,
        );
        assert!(
            displaced > 45_u64,
            "oracle must reject a displaced gold frame, not merely assert rendering succeeded"
        );
    }
    drop(environment);
    drop(guard);
}

/// Mean RGB absolute error for the 14x14 reference corner, excluding scene data.
fn corner_error(
    frame: &FrameCapture,
    reference: &Surface<'_>,
    origin: (usize, usize),
    corner: (usize, usize),
) -> u64 {
    let mut error = 0_u64;
    reference.with_lock(|pixels| {
        for y in 0_usize..14_usize {
            for x in 0_usize..14_usize {
                let actual = corner
                    .1
                    .saturating_add(y)
                    .saturating_mul(640_usize)
                    .saturating_add(corner.0.saturating_add(x))
                    .saturating_mul(4_usize);
                let expected = corner
                    .1
                    .saturating_add(y)
                    .saturating_add(origin.1)
                    .saturating_mul(usize::try_from(reference.pitch()).unwrap_or(0_usize))
                    .saturating_add(
                        corner
                            .0
                            .saturating_add(x)
                            .saturating_add(origin.0)
                            .saturating_mul(4_usize),
                    );
                for channel in 0_usize..3_usize {
                    error = error.saturating_add(u64::from(
                        frame.rgba[actual.saturating_add(channel)]
                            .abs_diff(pixels[expected.saturating_add(channel)]),
                    ));
                }
            }
        }
    });
    error.div_euclid(196_u64)
}
