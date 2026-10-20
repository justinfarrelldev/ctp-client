//! Independent memory-only source alignment and title clipping pixel oracles.

use super::draw;
use crate::gameplay::assets::layout::LayoutNode;
use crate::gameplay::runtime::render::management::draw_widget;
use crate::gameplay::runtime::render::text::draw_text;
use crate::gameplay::runtime::test_support::with_canvas;
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::ui_model::{MenuRect, management::ManagerWidget};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use std::path::Path;

/// A tiny source-caption descriptor with no renderer-specific font side effects.
fn widget(label: &str, rect: MenuRect, fields: &[(&str, &str)]) -> ManagerWidget {
    ManagerWidget {
        id: "Caption".to_owned(),
        label: label.to_owned(),
        rect,
        node: LayoutNode {
            fields: fields
                .iter()
                .copied()
                .map(|(key, value)| (key.to_owned(), value.to_owned()))
                .collect(),
            ..LayoutNode::default()
        },
        intent: None,
        unavailable: None,
        selected: false,
    }
}

#[test]
fn right_justified_source_field_uses_declared_shadow_not_left_row_padding() {
    let result = with_canvas((160_u32, 80_u32), |canvas| {
        let field = widget(
            "RIGHT",
            MenuRect {
                x: 40_i32,
                y: 30_i32,
                width: 80_u32,
                height: 20_u32,
            },
            &[
                ("just", "right"),
                ("shadowfont", "TRUE"),
                ("fontcolorred", "60"),
                ("fontcolorgreen", "60"),
                ("fontcolorblue", "40"),
                ("fontshadowcolorred", "200"),
                ("fontshadowcolorgreen", "200"),
                ("fontshadowcolorblue", "100"),
            ],
        );
        canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
        canvas.clear();
        canvas.set_clip_rect(Rect::new(40_i32, 30_i32, 80_u32, 20_u32));
        // Five six-pixel fallback advances, right aligned at x=120; explicit
        // right justification disables the default vertical-centering flag.
        draw_text(
            canvas,
            None,
            "RIGHT",
            91_i32,
            31_i32,
            Color::RGB(200_u8, 200_u8, 100_u8),
        )
        .map_err(|error| error.to_string())?;
        draw_text(
            canvas,
            None,
            "RIGHT",
            90_i32,
            30_i32,
            Color::RGB(60_u8, 60_u8, 40_u8),
        )
        .map_err(|error| error.to_string())?;
        let expected = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
        canvas.clear();
        draw(canvas, &field, None).map_err(|error| error.to_string())?;
        let actual = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        if actual != expected {
            return Err("source right alignment/shadow pixels differ".to_owned());
        }
        Ok(())
    });
    assert!(result.is_ok(), "native source label alignment: {result:?}");
}

#[test]
fn flattened_legacy_rows_retain_their_explicit_label_inset() {
    let result = with_canvas((160_u32, 80_u32), |canvas| {
        let field = widget(
            "LEFT",
            MenuRect {
                x: 40_i32,
                y: 30_i32,
                width: 80_u32,
                height: 20_u32,
            },
            &[("just", "left"), ("text-inset-left", "3")],
        );
        canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
        canvas.clear();
        draw_text(
            canvas,
            None,
            "LEFT",
            43_i32,
            30_i32,
            Color::RGB(50_u8, 50_u8, 50_u8),
        )
        .map_err(|error| error.to_string())?;
        let expected = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
        canvas.clear();
        draw(canvas, &field, None).map_err(|error| error.to_string())?;
        if canvas.read_pixels(None, PixelFormatEnum::RGBA32)? != expected {
            return Err("row label inset lost while adding source alignment".to_owned());
        }
        Ok(())
    });
    assert!(result.is_ok(), "legacy row text inset: {result:?}");
}

#[test]
fn narrow_original_title_allows_centered_text_overhang_without_losing_caller_clip() {
    let result = with_canvas((240_u32, 80_u32), |canvas| {
        let creator = canvas.texture_creator();
        let mut textures = GameplayTextures::load(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"),
            &creator,
            true,
        )
        .map_err(|error| error.to_string())?;
        let title = widget(
            "ABCDEFGHIJABCDEFGHIJ",
            MenuRect {
                x: 110_i32,
                y: 20_i32,
                width: 20_u32,
                height: 13_u32,
            },
            &[
                ("overflow-title", "true"),
                ("fontcolorred", "50"),
                ("fontcolorgreen", "50"),
                ("fontcolorblue", "50"),
            ],
        );
        canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
        canvas.clear();
        let caller = Rect::new(50_i32, 0_i32, 160_u32, 60_u32);
        canvas.set_clip_rect(caller);
        draw_text(
            canvas,
            None,
            "ABCDEFGHIJABCDEFGHIJ",
            60_i32,
            23_i32,
            Color::RGB(50_u8, 50_u8, 50_u8),
        )
        .map_err(|error| error.to_string())?;
        let expected = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        canvas.set_draw_color(Color::RGB(0_u8, 0_u8, 0_u8));
        canvas.clear();
        draw_widget(canvas, &mut textures, &title, None).map_err(|error| error.to_string())?;
        if canvas.clip_rect() != Some(caller) {
            return Err("title did not restore caller clip".to_owned());
        }
        let actual = canvas.read_pixels(None, PixelFormatEnum::RGBA32)?;
        if actual != expected {
            return Err(
                "long centered source title was clipped to its narrow LDL width".to_owned(),
            );
        }
        Ok(())
    });
    assert!(result.is_ok(), "native source title overhang: {result:?}");
}
