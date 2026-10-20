//! Native SDL manager renders use read-only art and in-memory framebuffer checks.

use super::management::draw_management;
use crate::gameplay::assets::PICTURE_ARCHIVE;
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::assets::rim::load_rim_rgba_entry;
use crate::gameplay::model::GameState;
use crate::gameplay::runtime::test_support::with_canvas;
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::server::ServerProjection;
use crate::gameplay::ui_model::InGameUiModel;
use crate::gameplay::ui_model::management::ManagerIntent;
use crate::gameplay::ui_model::management::{ManagementUi, ManagerKind};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use serde_json::json;
use std::path::Path;

#[test]
fn manager_article_pictures_match_active_catalog_and_original_geometry() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let mut state = GameState::new(24_u16, 18_u16, 1_u8, 42_u64);
        state.viewer = Some(
            serde_json::from_value::<ServerProjection>(
                json!({"revision":17_i64,"turn":3_u32,"research":{"target_id":"ADVANCE_BALLISTICS"}}),
            )
            .map_err(|error| error.to_string())?,
        );
        let mut ui = InGameUiModel {
            management: ManagementUi::new(
                LayoutBook::load(&root).map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?,
            ..InGameUiModel::default()
        };
        for (kind, id, bounds, art) in [
            (
                ManagerKind::Library,
                "TechStillShot",
                Rect::new(298_i32, 102_i32, 160_u32, 120_u32),
                "upap012l.rim",
            ),
            (
                ManagerKind::Science,
                "CurrentResearch.AdvanceProgress.IconBorder.IconButton",
                Rect::new(463_i32, 191_i32, 96_u32, 72_u32),
                "upap012l.rim",
            ),
            (
                ManagerKind::Build,
                "ItemImage.IconBorder.IconButton",
                Rect::new(388_i32, 148_i32, 96_u32, 72_u32),
                "upup002l.rim",
            ),
        ] {
            ui.management.open(kind, &state, None);
            if kind == ManagerKind::Build {
                ui.management.apply_local(ManagerIntent::Select(
                    "buildable".to_owned(),
                    "UNIT_SETTLER".to_owned(),
                ));
            }
            let widget = ui
                .management
                .widgets(&state)
                .into_iter()
                .find(|widget| widget.id == id)
                .ok_or_else(|| format!("missing {id}"))?;
            compare_geometry(widget.rect, bounds, kind)?;
            canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
            canvas.clear();
            let picture = textures
                .dynamic_ui(art)
                .map_err(|error| error.to_string())?
                .ok_or_else(|| "missing original picture".to_owned())?;
            canvas.copy(picture, None, bounds)?;
            let expected = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
            canvas.clear();
            draw_management(canvas, &mut textures, &state, &ui, None)
                .map_err(|error| error.to_string())?;
            let actual = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
            compare_picture_region(&actual, &expected, bounds, kind)?;
        }
        ui.management.open(ManagerKind::Library, &state, None);
        let link = ui
            .management
            .widgets(&state)
            .into_iter()
            .find(|widget| {
                widget.node.field("richtext") == Some("true")
                    && widget.intent == Some(ManagerIntent::Article("UNIT_ARCHER".to_owned()))
            })
            .ok_or_else(|| "source Archer link missing".to_owned())?;
        let x = link.rect.x;
        let y = link.rect.y.saturating_add(7_i32);
        draw_management(canvas, &mut textures, &state, &ui, None)
            .map_err(|error| error.to_string())?;
        let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        let offset = pixel_offset(x, y)?;
        if pixels[offset..offset.saturating_add(3_usize)] != [0_u8, 0_u8, 255_u8] {
            return Err("source hyperlink is not blue and underlined".to_owned());
        }
        let field = 90_usize
            .saturating_mul(800_usize)
            .saturating_add(136_usize)
            .saturating_mul(4_usize);
        if pixels[field..field.saturating_add(3_usize)] != [255_u8; 3_usize] {
            return Err("search edit field is missing its native white background".to_owned());
        }
        Ok(())
    });
    assert!(
        result.is_ok(),
        "original article artwork and hyperlinks: {result:?}"
    );
}

/// Checks coordinates before indexing a fixed 800x600 RGBA test framebuffer.
fn pixel_offset(x: i32, y: i32) -> Result<usize, String> {
    Ok(usize::try_from(y)
        .map_err(|error| error.to_string())?
        .saturating_mul(800_usize)
        .saturating_add(usize::try_from(x).map_err(|error| error.to_string())?)
        .saturating_mul(4_usize))
}

/// Independently specified original control coordinates, not a render-only offset.
fn compare_geometry(
    actual: crate::gameplay::ui_model::MenuRect,
    expected: Rect,
    kind: ManagerKind,
) -> Result<(), String> {
    if (actual.x, actual.y, actual.width, actual.height)
        == (
            expected.x(),
            expected.y(),
            expected.width(),
            expected.height(),
        )
    {
        Ok(())
    } else {
        Err(format!(
            "{kind:?} original picture geometry differs: {actual:?}"
        ))
    }
}

/// Independent original-picture comparison in the fixed 800x600 SDL fixture.
fn compare_picture_region(
    actual: &[u8],
    expected: &[u8],
    bounds: Rect,
    kind: ManagerKind,
) -> Result<(), String> {
    // Science's original radial-progress control overlays the lower portrait.
    let inset = if kind == ManagerKind::Science {
        16_i32
    } else {
        4_i32
    };
    for y in bounds.y().saturating_add(4_i32)..bounds.bottom().saturating_sub(inset) {
        for x in bounds.x().saturating_add(4_i32)..bounds.right().saturating_sub(4_i32) {
            let offset = pixel_offset(x, y)?;
            if actual[offset..offset.saturating_add(3_usize)]
                != expected[offset..offset.saturating_add(3_usize)]
            {
                return Err(format!("{kind:?} reference picture differs at {x},{y}"));
            }
        }
    }
    Ok(())
}

#[test]
fn every_manager_renders_without_touching_files_and_uses_its_original_corner() {
    let result = with_canvas((800_u32, 600_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let state = GameState::new(24, 18, 1, 42);
        let mut ui = InGameUiModel {
            management: ManagementUi::new(
                LayoutBook::load(&root).map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?,
            ..InGameUiModel::default()
        };
        for kind in ManagerKind::ALL {
            ui.management.open(kind, &state, None);
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            draw_management(canvas, &mut textures, &state, &ui, None)
                .map_err(|error| format!("{kind:?}: {error}"))?;
            let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
            if pixels.chunks_exact(4).all(|pixel| pixel[..3] == [0_u8; 3]) {
                return Err(format!("{kind:?} is blank"));
            }
        }
        for (kind, art, x) in [
            (ManagerKind::Build, "uptg25a.rim", 0_usize),
            (ManagerKind::Science, "uptg24a.rim", 224),
        ] {
            ui.management.open(kind, &state, None);
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            draw_management(canvas, &mut textures, &state, &ui, None)
                .map_err(|error| error.to_string())?;
            let pixels = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
            let corner = load_rim_rgba_entry(&root, PICTURE_ARCHIVE, art, Some((248, 0, 248)))
                .map_err(|error| error.to_string())?;
            let mut compared = 0_usize;
            for row in 0_usize..10 {
                for column in 0_usize..40 {
                    let source = row
                        .saturating_mul(corner.pitch)
                        .saturating_add(column.saturating_mul(4));
                    if corner.data[source.saturating_add(3)] != 255 {
                        continue;
                    }
                    let destination = row
                        .saturating_add(108)
                        .saturating_mul(800)
                        .saturating_add(x.saturating_add(column))
                        .saturating_mul(4);
                    if pixels[destination..destination.saturating_add(3)]
                        != corner.data[source..source.saturating_add(3)]
                    {
                        return Err(format!("{kind:?} source corner mismatch at {column},{row}"));
                    }
                    compared = compared.saturating_add(1);
                }
            }
            if compared == 0 {
                return Err(format!("no source pixels checked for {kind:?}"));
            }
        }
        Ok(())
    });
    assert!(result.is_ok(), "native source manager art: {result:?}");
}
