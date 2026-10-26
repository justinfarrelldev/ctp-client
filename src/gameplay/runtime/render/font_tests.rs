//! Original map and Unit-stat Arial exceptions must not inherit generic Times.
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::runtime::fonts::{ManagementFonts, RenderFonts};
use crate::gameplay::runtime::snapshot::scene::{PreviewScene, preview_state};
use crate::gameplay::runtime::{
    layout::panel_rect, test_support::with_canvas, textures::GameplayTextures,
};
use crate::gameplay::ui_model::{
    ControlPanelTab, InGameUiModel,
    management::{ManagementUi, typography::SourceFont},
};
use sdl2::pixels::{Color, PixelFormatEnum};
use std::path::Path;

/// `TiledMap` labels use a runtime string table, not `CTP2_STANDARD_FONT`.
#[test]
fn map_font_uses_original_localized_runtime_table_instead_of_menu_times() {
    let result = with_canvas((80_u32, 60_u32), |_canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let management =
            ManagementUi::new(LayoutBook::load(&root).map_err(|error| error.to_string())?)
                .map_err(|error| error.to_string())?;
        let context = sdl2::ttf::init().map_err(|error| error.to_string())?;
        let fonts =
            ManagementFonts::load(&context, &management).map_err(|error| error.to_string())?;
        let expected = fonts
            .get(&SourceFont {
                name: "arialbd.ttf".to_owned(),
                points: 9_u16,
                bold: false,
                italic: false,
            })
            .ok_or("original TiledMap Arial9 face missing")?;
        let view = RenderFonts {
            body: fonts.get(&SourceFont::default()),
            management: Some(&fonts),
        };
        if !view
            .map()
            .is_some_and(|actual| core::ptr::eq(actual, expected))
        {
            return Err("city labels/path turn annotations still use Times standard instead of original Arial".to_owned());
        }
        Ok(())
    });
    assert!(result.is_ok(), "original map font: {result:?}");
}

/// Compare the entire stat text box with independent original-template painting.
#[test]
fn unit_panel_stat_font_matches_original_arial_value_template_in_native_pixels() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let mut state = preview_state(PreviewScene::Panel {
            tab: ControlPanelTab::Unit,
            populated: true,
        });
        if state.selected_unit().is_none() {
            return Err("missing selected preview unit".to_owned());
        }
        for unit in &mut state.viewer.as_mut().ok_or("viewer")?.units {
            unit.facts["attack"] = serde_json::json!(18_i64);
        }
        let ui = InGameUiModel {
            active_tab: ControlPanelTab::Unit,
            management: ManagementUi::new(
                LayoutBook::load(&root).map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?,
            ..InGameUiModel::default()
        };
        let context = sdl2::ttf::init().map_err(|error| error.to_string())?;
        let fonts =
            ManagementFonts::load(&context, &ui.management).map_err(|error| error.to_string())?;
        let view = RenderFonts {
            body: fonts.get(&SourceFont::default()),
            management: Some(&fonts),
        };
        let key = SourceFont {
            name: "arialbd.ttf".to_owned(),
            points: 10_u16,
            bold: false,
            italic: false,
        };
        let stat = fonts.get(&key).ok_or("source ICON_VALUE_PAIR.Value font")?;
        let bounds = panel_rect(306, 78, 19, 18);
        canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
        canvas.clear();
        super::panel::draw_panel(canvas, &mut textures, &state, &ui, view)
            .map_err(|error| error.to_string())?;
        let actual = canvas.read_pixels(Some(bounds), PixelFormatEnum::RGBA32)?;
        state.selected_unit = None;
        canvas.clear();
        super::panel::draw_panel(canvas, &mut textures, &state, &ui, view)
            .map_err(|error| error.to_string())?;
        super::text::draw_centered_text(
            canvas,
            Some(stat),
            "18",
            bounds,
            Color::RGB(50_u8, 50_u8, 50_u8),
            true,
        )
        .map_err(|error| error.to_string())?;
        if actual != canvas.read_pixels(Some(bounds), PixelFormatEnum::RGBA32)? {
            return Err("unit numeric stat bypasses source Arial font".to_owned());
        }
        Ok(())
    });
    assert!(result.is_ok(), "native Unit stat font: {result:?}");
}
