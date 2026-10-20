//! Declared-height strip and iconic-overlay scaling, checked with SDL readback.

use super::management::draw_widget;
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::runtime::{test_support::with_canvas, textures::GameplayTextures};
use crate::gameplay::ui_model::{MenuRect, management::ManagerWidget};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use std::path::Path;

#[test]
fn short_source_tile_radio_keeps_the_bottom_of_its_endcap_instead_of_clipping_it_off() {
    let result = with_canvas((180_u32, 60_u32), |canvas| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator = canvas.texture_creator();
        let mut textures =
            GameplayTextures::load(&root, &creator, true).map_err(|error| error.to_string())?;
        let source = textures
            .dynamic_ui("upbt12ax.rim")
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "source cap".to_owned())?;
        let width = source.query().width;
        canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
        canvas.clear();
        canvas.copy(source, None, Rect::new(10_i32, 10_i32, width, 15_u32))?;
        let expected = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        canvas.clear();
        let node = LayoutBook::load(&root)
            .and_then(|book| book.resolve("CTP2_TILE_RADIO_BUTTON"))
            .map_err(|error| error.to_string())?;
        let widget = ManagerWidget {
            id: "short-radio".to_owned(),
            label: String::new(),
            rect: MenuRect {
                x: 10_i32,
                y: 10_i32,
                width: 145_u32,
                height: 15_u32,
            },
            node,
            intent: None,
            unavailable: None,
            selected: false,
        };
        draw_widget(canvas, &mut textures, &widget, None).map_err(|error| error.to_string())?;
        let actual = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        for y in 10_u32..25_u32 {
            for x in 10_u32..10_u32.saturating_add(width) {
                let offset = usize::try_from(
                    y.saturating_mul(180_u32)
                        .saturating_add(x)
                        .saturating_mul(4_u32),
                )
                .map_err(|error| error.to_string())?;
                if actual[offset..offset.saturating_add(3_usize)]
                    != expected[offset..offset.saturating_add(3_usize)]
                {
                    return Err(format!("source endcap clipped at {x},{y}"));
                }
            }
        }
        Ok(())
    });
    assert!(result.is_ok(), "declared strip height: {result:?}");
}
