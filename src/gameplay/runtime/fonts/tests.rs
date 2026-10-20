//! Read-only native source-font selection and measured hyperlink geometry.

use super::{ManagementFonts, RenderFonts};
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::model::GameState;
use crate::gameplay::runtime::test_support::with_canvas;
use crate::gameplay::ui_model::management::{ManagementUi, ManagerKind, typography::SourceFont};
use serde_json::json;
use std::path::Path;

#[test]
fn actual_source_faces_sizes_and_word_metrics_agree_without_loading_fonts_per_frame() {
    let result = with_canvas((80_u32, 60_u32), |_canvas| {
        let context = sdl2::ttf::init().map_err(|error| error.to_string())?;
        let mut ui = ManagementUi::new(
            LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"))
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        let mut state = GameState::new(8_u16, 8_u16, 1_u8, 1_u64);
        state.viewer=Some(serde_json::from_value(json!({"revision":1_i64,"turn":1_u32,"research":{"target_id":"ADVANCE_BALLISTICS"}})).map_err(|error|error.to_string())?);
        ui.open(ManagerKind::Library, &state, None);
        let fonts =
            ManagementFonts::load_measured(&context, &mut ui).map_err(|error| error.to_string())?;
        let node = ui
            .layouts
            .get(&ManagerKind::Library)
            .and_then(|root| root.child("RequirementsText"))
            .ok_or_else(|| "requirements source box missing".to_owned())?;
        let key = SourceFont::from_node(node);
        if key.name != "times.ttf" || key.points != 8_u16 {
            return Err(format!("first-template source header style lost: {key:?}"));
        }
        let actual = fonts
            .get(&key)
            .ok_or_else(|| "header face missing".to_owned())?;
        let word = ui
            .widgets(&state)
            .into_iter()
            .find(|widget| widget.id.starts_with("RequirementsText/word-"))
            .ok_or_else(|| "source requirements word missing".to_owned())?;
        let width = actual
            .size_of(&word.label)
            .map_err(|error| error.to_string())?
            .0;
        if word.rect.width != width {
            return Err(
                "semantic link geometry does not use its actual source font metrics".to_owned(),
            );
        }
        let view = RenderFonts {
            body: None,
            management: Some(&fonts),
        };
        let chosen = view
            .control(&word.node)
            .ok_or_else(|| "painted style missing".to_owned())?;
        if !core::ptr::eq(actual, chosen) {
            return Err("painted font and measured source font differ".to_owned());
        }
        Ok(())
    });
    assert!(result.is_ok(), "native per-control typography: {result:?}");
}
