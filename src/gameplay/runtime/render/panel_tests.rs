//! Native lower-panel picture/frame pixels and semantic palette interaction.

use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::runtime::snapshot::scene::{PreviewScene, preview_state};
use crate::gameplay::runtime::{
    Camera, MovementInputState, controls, test_support::with_canvas, textures::GameplayTextures,
};
use crate::gameplay::ui_model::{
    ControlPanelTab, InGameUiModel, TileCategory, management::ManagementUi,
};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use std::path::Path;

#[test]
fn city_and_empire_progress_pictures_match_original_art_in_source_panel_coordinates() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let state = preview_state(PreviewScene::Panel {
            tab: ControlPanelTab::City,
            populated: true,
        });
        let mut ui = InGameUiModel {
            management: ManagementUi::new(
                LayoutBook::load(&root).map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?,
            ..InGameUiModel::default()
        };
        for (tab, art) in [
            (ControlPanelTab::City, "upup008l.rim"),
            (ControlPanelTab::Domestic, "upap012l.rim"),
        ] {
            ui.active_tab = tab;
            canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
            canvas.clear();
            let picture = textures
                .dynamic_ui(art)
                .map_err(|error| error.to_string())?
                .ok_or_else(|| "original portrait".to_owned())?;
            canvas.copy(picture, None, Rect::new(400_i32, 489_i32, 96_u32, 72_u32))?;
            let expected = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
            canvas.clear();
            super::panel::draw_panel(canvas, &mut textures, &state, &ui, None)
                .map_err(|error| error.to_string())?;
            let actual = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
            // The source radial button overlays the bottom of the picture.
            for y in 489_usize..546_usize {
                for x in 400_usize..496_usize {
                    let offset = y
                        .saturating_mul(800_usize)
                        .saturating_add(x)
                        .saturating_mul(4_usize);
                    if actual[offset..offset.saturating_add(3_usize)]
                        != expected[offset..offset.saturating_add(3_usize)]
                    {
                        return Err(format!("{tab:?}: original progress picture at {x},{y}"));
                    }
                }
            }
        }
        Ok(())
    });
    assert!(result.is_ok(), "source lower-panel art: {result:?}");
}

#[test]
fn accessible_palette_categories_match_pointer_bounds_and_never_change_the_viewer() {
    let mut state = preview_state(PreviewScene::Panel {
        tab: ControlPanelTab::TileImprovements,
        populated: true,
    });
    let original = state.clone();
    let mut ui = InGameUiModel {
        active_tab: ControlPanelTab::TileImprovements,
        ..InGameUiModel::default()
    };
    let mut camera = Camera::centered_on_selected(&state);
    let mut movement = MovementInputState::default();
    for category in [
        TileCategory::Land,
        TileCategory::Ocean,
        TileCategory::Special,
        TileCategory::TerraformLand,
        TileCategory::TerraformOcean,
    ] {
        let control = controls::panel_controls(&state, ui.active_tab)
            .into_iter()
            .find(|control| control.id == category.control().0)
            .expect("accessible palette");
        let intent = controls::intent_at(
            &state,
            ui.active_tab,
            control.rect.x.saturating_add(1_i32),
            control.rect.y.saturating_add(1_i32),
        )
        .expect("same pointer bounds");
        assert_eq!(intent, control.intent, "pointer and accessible category");
        assert!(
            controls::apply_intent(
                intent,
                &mut state,
                &mut ui,
                &mut camera,
                &mut movement,
                None
            )
            .is_none(),
            "no command outcome"
        );
        assert_eq!(ui.tile_category, category, "local selection");
        assert_eq!(
            state, original,
            "palette switching cannot mutate game facts"
        );
    }
}

#[test]
fn native_footer_paints_imported_advice_description_and_releases_it_on_pointer_leave() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let mut state = preview_state(PreviewScene::PopulatedManager {
            kind: crate::gameplay::ui_model::management::ManagerKind::Unit,
            chart: false,
        });
        state.popups.clear();
        let mut ui = InGameUiModel {
            management: ManagementUi::new(
                LayoutBook::load(&root).map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?,
            ..InGameUiModel::default()
        };
        ui.management.open(
            crate::gameplay::ui_model::management::ManagerKind::Unit,
            &state,
            None,
        );
        let context = sdl2::ttf::init().map_err(|error| error.to_string())?;
        let font = crate::gameplay::runtime::font_loader::load_gameplay_font(&context)
            .ok_or_else(|| "native footer font unavailable".to_owned())?;
        ui.status_text = ui
            .management
            .book
            .label("STATUSBAR_UNITMAN_MAIN_SHOWADV_BUTTON")
            .to_owned();
        canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
        canvas.clear();
        super::panel::draw_panel(canvas, &mut textures, &state, &ui, Some(&font))
            .map_err(|error| error.to_string())?;
        let footer =
            crate::gameplay::runtime::layout::panel_rect(126_i32, 167_i32, 468_u32, 17_u32);
        let expected = canvas.read_pixels(Some(footer), PixelFormatEnum::RGBA32)?;
        let rect = ui
            .management
            .widgets(&state)
            .into_iter()
            .find(|widget| widget.id == "AdviceButton")
            .ok_or_else(|| "accessible advice button".to_owned())?
            .rect;
        "Operational result".clone_into(&mut ui.status_text);
        ui.pointer = Some((rect.x.saturating_add(10_i32), rect.y.saturating_add(10_i32)));
        canvas.clear();
        super::panel::draw_panel(canvas, &mut textures, &state, &ui, Some(&font))
            .map_err(|error| error.to_string())?;
        let actual = canvas.read_pixels(Some(footer), PixelFormatEnum::RGBA32)?;
        if actual != expected {
            return Err(
                "complete native footer differs from independently source-resolved advice caption"
                    .to_owned(),
            );
        }
        ui.pointer = None;
        canvas.clear();
        super::panel::draw_panel(canvas, &mut textures, &state, &ui, Some(&font))
            .map_err(|error| error.to_string())?;
        if canvas.read_pixels(Some(footer), PixelFormatEnum::RGBA32)? == expected {
            return Err(
                "pointer leave did not restore operational text (or font rendered no text)"
                    .to_owned(),
            );
        }
        let outside_before = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        ui.status_text = "Source quote unavailable ".repeat(40_usize);
        canvas.clear();
        super::panel::draw_panel(canvas, &mut textures, &state, &ui, Some(&font))
            .map_err(|error| error.to_string())?;
        let outside_after = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        for y in 0_i32..600_i32 {
            for x in 0_i32..800_i32 {
                if !footer.contains_point((x, y)) {
                    let index = usize::try_from(
                        y.saturating_mul(800_i32)
                            .saturating_add(x)
                            .saturating_mul(4_i32),
                    )
                    .map_err(|error| error.to_string())?;
                    if outside_before[index..index.saturating_add(3_usize)]
                        != outside_after[index..index.saturating_add(3_usize)]
                    {
                        return Err(
                            "long source status overdraws outside its original footer control"
                                .to_owned(),
                        );
                    }
                }
            }
        }
        Ok(())
    });
    assert!(result.is_ok(), "source native footer text: {result:?}");
}
