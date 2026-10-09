//! Original CTP2 listbox chrome rendering.

use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

use crate::main_menu::{MainMenuError, MainMenuModel, MenuRect};

use super::{create_texture_for_ldl_asset, runtime_error, tile_texture, to_sdl_rect};

/// Width reserved by `CTP2_LISTBOX.rangery`.
const SCROLLBAR_WIDTH: u32 = 15;
/// Height of the original listbox increment/decrement buttons.
const SCROLL_BUTTON_HEIGHT: u32 = 15;
/// Native size of the `uptg06*` border-box pieces.
const BORDER_PIECE_SIZE: u32 = 8;
/// Native height of the original `upbt14` scrollbar thumb.
const THUMB_HEIGHT: u32 = 18;
/// Original CTP2 border-box art laid out as a three-by-three grid.
const BORDER_ASSETS: [[&str; 3]; 3] = [
    ["uptg06a.tga", "uptg06b.tga", "uptg06c.tga"],
    ["uptg06d.tga", "uptg06e.tga", "uptg06f.tga"],
    ["uptg06g.tga", "uptg06h.tga", "uptg06i.tga"],
];

/// Draws original CTP2 listbox chrome plus its always-visible vertical ranger.
pub(super) fn draw_listbox_chrome(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    model: &MainMenuModel,
    rect: &MenuRect,
    scroll_offset: usize,
    item_count: usize,
    visible_rows: usize,
) -> Result<(), MainMenuError> {
    if draw_original_listbox(canvas, texture_creator, model, rect).is_err() {
        draw_fallback_listbox(canvas, rect)?;
    }
    if draw_original_scrollbar(
        canvas,
        texture_creator,
        model,
        rect,
        scroll_offset,
        item_count,
        visible_rows,
    )
    .is_err()
    {
        draw_fallback_scrollbar(canvas, rect, scroll_offset, item_count, visible_rows)?;
    }
    Ok(())
}

/// Draws the nine-piece `CTP2_BORDER_BOX` backing used by `CTP2_LISTBOX`.
fn draw_original_listbox(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    model: &MainMenuModel,
    rect: &MenuRect,
) -> Result<(), MainMenuError> {
    let left = rect.x;
    let top = rect.y;
    let middle_width = rect
        .width
        .saturating_sub(BORDER_PIECE_SIZE.saturating_mul(2));
    let middle_height = rect
        .height
        .saturating_sub(BORDER_PIECE_SIZE.saturating_mul(2));
    let middle_x = left.saturating_add(u32_to_i32(BORDER_PIECE_SIZE));
    let middle_y = top.saturating_add(u32_to_i32(BORDER_PIECE_SIZE));
    let right = middle_x.saturating_add(u32_to_i32(middle_width));
    let bottom = middle_y.saturating_add(u32_to_i32(middle_height));

    let destinations = [
        [
            MenuRect::new(left, top, BORDER_PIECE_SIZE, BORDER_PIECE_SIZE),
            MenuRect::new(middle_x, top, middle_width, BORDER_PIECE_SIZE),
            MenuRect::new(right, top, BORDER_PIECE_SIZE, BORDER_PIECE_SIZE),
        ],
        [
            MenuRect::new(left, middle_y, BORDER_PIECE_SIZE, middle_height),
            MenuRect::new(middle_x, middle_y, middle_width, middle_height),
            MenuRect::new(right, middle_y, BORDER_PIECE_SIZE, middle_height),
        ],
        [
            MenuRect::new(left, bottom, BORDER_PIECE_SIZE, BORDER_PIECE_SIZE),
            MenuRect::new(middle_x, bottom, middle_width, BORDER_PIECE_SIZE),
            MenuRect::new(right, bottom, BORDER_PIECE_SIZE, BORDER_PIECE_SIZE),
        ],
    ];

    for (row_index, row) in BORDER_ASSETS.iter().enumerate() {
        for (column_index, asset_name) in row.iter().enumerate() {
            let destination = destinations[row_index][column_index];
            let texture = create_texture_for_ldl_asset(texture_creator, model, asset_name)?;
            if row_index == 1 || column_index == 1 {
                tile_texture(canvas, &texture, destination)?;
            } else {
                copy_texture(canvas, &texture, &destination)?;
            }
        }
    }
    Ok(())
}

/// Draws the original listbox vertical ranger assets.
fn draw_original_scrollbar(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    model: &MainMenuModel,
    rect: &MenuRect,
    scroll_offset: usize,
    item_count: usize,
    visible_rows: usize,
) -> Result<(), MainMenuError> {
    let scrollbar = scrollbar_rect(rect);
    let track = scrollbar_track_rect(&scrollbar);
    let track_texture = create_texture_for_ldl_asset(texture_creator, model, "uptg04e.tga")?;
    tile_texture(canvas, &track_texture, track)?;

    let button_texture = create_texture_for_ldl_asset(texture_creator, model, "upbt13.tga")?;
    copy_texture(
        canvas,
        &button_texture,
        &MenuRect::new(
            scrollbar.x,
            scrollbar.y,
            SCROLLBAR_WIDTH,
            SCROLL_BUTTON_HEIGHT,
        ),
    )?;
    let down_y = scrollbar.y.saturating_add(u32_to_i32(
        scrollbar.height.saturating_sub(SCROLL_BUTTON_HEIGHT),
    ));
    copy_texture(
        canvas,
        &button_texture,
        &MenuRect::new(scrollbar.x, down_y, SCROLLBAR_WIDTH, SCROLL_BUTTON_HEIGHT),
    )?;

    let thumb = create_texture_for_ldl_asset(texture_creator, model, "upbt14.tga")?;
    copy_texture(
        canvas,
        &thumb,
        &MenuRect::new(
            scrollbar.x,
            scrollbar_thumb_y(track, scroll_offset, item_count, visible_rows),
            SCROLLBAR_WIDTH,
            THUMB_HEIGHT.min(track.height),
        ),
    )
}

/// Draws fallback listbox body when original assets are unavailable.
fn draw_fallback_listbox(
    canvas: &mut Canvas<Window>,
    rect: &MenuRect,
) -> Result<(), MainMenuError> {
    canvas.set_draw_color(Color::RGB(150, 137, 103));
    canvas.fill_rect(to_sdl_rect(rect)).map_err(runtime_error)?;
    canvas.set_draw_color(Color::RGB(50, 42, 31));
    canvas.draw_rect(to_sdl_rect(rect)).map_err(runtime_error)
}

/// Draws fallback scrollbar when original assets are unavailable.
fn draw_fallback_scrollbar(
    canvas: &mut Canvas<Window>,
    rect: &MenuRect,
    scroll_offset: usize,
    item_count: usize,
    visible_rows: usize,
) -> Result<(), MainMenuError> {
    let scrollbar = scrollbar_rect(rect);
    canvas.set_draw_color(Color::RGB(88, 78, 58));
    canvas
        .fill_rect(to_sdl_rect(&scrollbar))
        .map_err(runtime_error)?;
    canvas.set_draw_color(Color::RGB(45, 38, 28));
    canvas
        .draw_rect(to_sdl_rect(&scrollbar))
        .map_err(runtime_error)?;
    let track = scrollbar_track_rect(&scrollbar);
    let thumb = MenuRect::new(
        scrollbar.x,
        scrollbar_thumb_y(track, scroll_offset, item_count, visible_rows),
        SCROLLBAR_WIDTH,
        THUMB_HEIGHT.min(track.height),
    );
    canvas.set_draw_color(Color::RGB(180, 165, 125));
    canvas.fill_rect(to_sdl_rect(&thumb)).map_err(runtime_error)
}

/// Copies one texture into a destination rectangle.
fn copy_texture(
    canvas: &mut Canvas<Window>,
    texture: &Texture<'_>,
    destination: &MenuRect,
) -> Result<(), MainMenuError> {
    canvas
        .copy(texture, None, to_sdl_rect(destination))
        .map_err(runtime_error)
}

/// Returns the scrollbar rectangle inside a listbox.
pub(super) fn scrollbar_rect(rect: &MenuRect) -> MenuRect {
    let x = rect
        .x
        .saturating_add(u32_to_i32(rect.width.saturating_sub(SCROLLBAR_WIDTH)));
    MenuRect::new(x, rect.y, SCROLLBAR_WIDTH, rect.height)
}

/// Returns whether a point falls on the listbox scrollbar increment/decrement buttons.
pub(super) fn scrollbar_delta_at(rect: &MenuRect, point_x: i32, point_y: i32) -> Option<i32> {
    let scrollbar = scrollbar_rect(rect);
    if !scrollbar.contains_point(point_x, point_y) {
        return None;
    }
    let down_top = scrollbar.y.saturating_add(u32_to_i32(
        scrollbar.height.saturating_sub(SCROLL_BUTTON_HEIGHT),
    ));
    if point_y < scrollbar.y.saturating_add(u32_to_i32(SCROLL_BUTTON_HEIGHT)) {
        Some(-1_i32)
    } else if point_y >= down_top {
        Some(1_i32)
    } else {
        None
    }
}

/// Returns the track rectangle between scrollbar buttons.
fn scrollbar_track_rect(scrollbar: &MenuRect) -> MenuRect {
    let y = scrollbar.y.saturating_add(u32_to_i32(SCROLL_BUTTON_HEIGHT));
    let height = scrollbar
        .height
        .saturating_sub(SCROLL_BUTTON_HEIGHT.saturating_mul(2));
    MenuRect::new(scrollbar.x, y, scrollbar.width, height)
}

/// Computes the scrollbar thumb y coordinate for the current list offset.
fn scrollbar_thumb_y(
    track: MenuRect,
    scroll_offset: usize,
    item_count: usize,
    visible_rows: usize,
) -> i32 {
    let max_offset = item_count.saturating_sub(visible_rows);
    if max_offset == 0 || track.height <= THUMB_HEIGHT {
        return track.y;
    }
    let available = track.height.saturating_sub(THUMB_HEIGHT);
    let numerator = u64::try_from(scroll_offset)
        .map_or(u64::MAX, |value| value)
        .saturating_mul(u64::from(available));
    let denominator = u64::try_from(max_offset).map_or(1_u64, |value| value.max(1_u64));
    let offset = numerator
        .checked_div(denominator)
        .map_or(0_u64, |value| value);
    track
        .y
        .saturating_add(i32::try_from(offset).map_or(i32::MAX, |value| value))
}

/// Converts a `u32` pixel count into a saturating signed coordinate offset.
fn u32_to_i32(value: u32) -> i32 {
    i32::try_from(value).map_or(i32::MAX, |converted| converted)
}
