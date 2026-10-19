//! Framebuffer assertions use memory only; no snapshot files are ever written.

use super::{HeadlessSdlEnvironment, snapshot};
use crate::gameplay::ui_model::MenuCategory;
use std::path::Path;

#[test]
fn motion_probe_captures_real_window_at_an_injected_animation_phase() -> Result<(), String> {
    let _guard = crate::sdl_test_lock::lock_sdl_for_test()?;
    let environment = HeadlessSdlEnvironment::apply();
    let result = (|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let initial = snapshot::render_preview_scene(
            &root,
            (800, 600),
            None,
            snapshot::scene::PreviewScene::Motion {
                direction: 7,
                quarter: 0,
            },
        )
        .map_err(|error| error.to_string())?;
        let middle = snapshot::render_preview_scene(
            &root,
            (800, 600),
            None,
            snapshot::scene::PreviewScene::Motion {
                direction: 7,
                quarter: 2,
            },
        )
        .map_err(|error| error.to_string())?;
        let repeated = snapshot::render_preview_scene(
            &root,
            (800, 600),
            None,
            snapshot::scene::PreviewScene::Motion {
                direction: 7,
                quarter: 2,
            },
        )
        .map_err(|error| error.to_string())?;
        if initial == middle {
            return Err("movement probe did not advance actual window pixels".to_owned());
        }
        if middle != repeated {
            return Err("injected phase is not deterministic".to_owned());
        }
        Ok(())
    })();
    environment.restore();
    result
}

#[test]
fn deterministic_snapshots_use_actual_renderer_and_requested_output_size() -> Result<(), String> {
    let _guard = crate::sdl_test_lock::lock_sdl_for_test()?;
    let environment = HeadlessSdlEnvironment::apply();
    let result = (|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let frame = snapshot::render_preview(&root, (800_u32, 600_u32), None)
            .map_err(|error| error.to_string())?;
        if (frame.width, frame.height, frame.rgba.len()) != (800_u32, 600_u32, 1_920_000_usize) {
            return Err("native capture must be lossless 800×600 RGBA".to_owned());
        }
        let empty = (100_usize * 800_usize + 700_usize) * 4_usize;
        if frame.rgba[empty..empty.saturating_add(3_usize)] != [0_u8; 3] {
            return Err(
                "undiscovered map must be black, without an upper-right debug panel".to_owned(),
            );
        }
        let dropdown =
            snapshot::render_preview(&root, (640_u32, 480_u32), Some(MenuCategory::Options))
                .map_err(|error| error.to_string())?;
        if dropdown.rgba.len() != 1_228_800_usize || !dropdown.rgba.iter().any(|byte| *byte != 0_u8)
        {
            return Err("scaled snapshot must contain a real 640×480 frame".to_owned());
        }
        Ok(())
    })();
    environment.restore();
    result
}
