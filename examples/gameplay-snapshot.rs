//! Lossless original-art preview: no live server, settings writes or intro.
//!
//! `cargo run --example gameplay-snapshot -- ctp2 images/gameplay.png 640 480 [Options|none] [terrain]`

use ctp_client::gameplay::runtime::snapshot::render_preview_scene;
use ctp_client::gameplay::runtime::snapshot::scene::PreviewScene;
use ctp_client::gameplay::ui_model::MenuCategory;
use std::error::Error;
use std::path::Path;

/// Captures the real renderer at a requested size and optional accessible menu title.
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1_usize).collect();
    let root = args.first().map_or("ctp2", String::as_str);
    let output = args
        .get(1_usize)
        .map_or("images/gameplay.png", String::as_str);
    let width = args
        .get(2_usize)
        .map_or(Ok(640_u32), |value| value.parse::<u32>())?;
    let height = args
        .get(3_usize)
        .map_or(Ok(480_u32), |value| value.parse::<u32>())?;
    let menu = args
        .get(4_usize)
        .filter(|label| !label.eq_ignore_ascii_case("none"))
        .map(|label| {
            [
                MenuCategory::Civilization,
                MenuCategory::Cities,
                MenuCategory::Units,
                MenuCategory::Diplomacy,
                MenuCategory::Info,
                MenuCategory::Options,
            ]
            .into_iter()
            .find(|category| category.title().eq_ignore_ascii_case(label))
            .ok_or("unknown menu title; use Empire, Cities, Units, Diplomacy, Stats or Options")
        })
        .transpose()?;
    let scene = match args.get(5_usize).map(String::as_str) {
        Some("motion") => PreviewScene::Motion {
            direction: args
                .get(6_usize)
                .map_or(Ok(3_u8), |value| value.parse::<u8>())?,
            quarter: args
                .get(7_usize)
                .map_or(Ok(2_u8), |value| value.parse::<u8>())?,
        },
        Some("terrain") => PreviewScene::Terrain,
        _ => PreviewScene::Generated,
    };
    let mut frame = render_preview_scene(Path::new(root), (width, height), menu, scene)?;
    frame.save_png(Path::new(output))?;
    println!("Saved {output} ({width}×{height}); deterministic preview, not a live game");
    Ok(())
}
