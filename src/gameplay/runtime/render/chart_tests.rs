//! Native chart borders/connectors and semantic navigation remain entirely in memory.

use super::management::draw_management;
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::model::GameState;
use crate::gameplay::runtime::management_input::apply;
use crate::gameplay::runtime::test_support::with_canvas;
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::ui_model::InGameUiModel;
use crate::gameplay::ui_model::management::{ManagementUi, ManagerKind};
use sdl2::pixels::{Color, PixelFormatEnum};
use serde_json::json;
use std::path::Path;

#[test]
fn native_chart_uses_viewer_research_borders_and_local_accessible_dispatch() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let mut state = GameState::new(8_u16, 8_u16, 1_u8, 1_u64);
        state.viewer=Some(serde_json::from_value(json!({"revision":7_i64,"turn":1_u32,
            "research":{"target_id":"ADVANCE_BALLISTICS","advances":["ADVANCE_AGRICULTURE"],"available":["ADVANCE_BALLISTICS"]},
            "rules":{"advances":[{"id":"ADVANCE_AGRICULTURE"},{"id":"ADVANCE_BALLISTICS","prerequisites":["ADVANCE_AGRICULTURE"]},{"id":"ADVANCE_WRITING","prerequisites":["ADVANCE_BALLISTICS"]}]}})).map_err(|error|error.to_string())?);
        let before = state.clone();
        let mut ui = InGameUiModel {
            management: ManagementUi::new(
                LayoutBook::load(&root).map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?,
            ..InGameUiModel::default()
        };
        ui.management.open(ManagerKind::Library, &state, None);
        let tab = ui
            .management
            .activate_id("Tabs.TechTreeTab.TabButton", &state)
            .ok_or_else(|| "chart tab unavailable".to_owned())?;
        apply(tab, &mut state, &mut ui, None);
        canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
        canvas.clear();
        draw_management(canvas, &mut textures, &state, &ui, None)
            .map_err(|error| error.to_string())?;
        let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        for (x, y, expected) in [
            (313_usize, 369_usize, [0_u8, 255_u8, 0_u8]),
            (454_usize, 369_usize, [255_u8, 255_u8, 0_u8]),
            (595_usize, 369_usize, [255_u8, 0_u8, 0_u8]),
            (440_usize, 380_usize, [0_u8; 3_usize]),
        ] {
            let offset = y
                .saturating_mul(800_usize)
                .saturating_add(x)
                .saturating_mul(4_usize);
            if pixels[offset..offset.saturating_add(3_usize)] != expected {
                return Err(format!(
                    "source chart color/connector mismatch at {x},{y}: {:?}",
                    &pixels[offset..offset.saturating_add(3_usize)]
                ));
            }
        }
        let next = ui
            .management
            .activate_id(
                "Tabs.TechTreeTab.TabPanel/successors/ADVANCE_WRITING",
                &state,
            )
            .ok_or_else(|| "successor missing".to_owned())?;
        apply(next, &mut state, &mut ui, None);
        let center = ui
            .management
            .activate_id("Tabs.TechTreeTab.TabPanel/center/ADVANCE_WRITING", &state)
            .ok_or_else(|| "new center missing".to_owned())?;
        apply(center, &mut state, &mut ui, None);
        if ui.management.library.current.as_deref() != Some("ADVANCE_WRITING") || state != before {
            return Err(
                "chart navigation mutated research or failed to open its actual article".to_owned(),
            );
        }
        Ok(())
    });
    assert!(
        result.is_ok(),
        "source chart SDL/dispatch regression: {result:?}"
    );
}
