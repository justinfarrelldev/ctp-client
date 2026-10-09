//! Original CTP2 listbox chrome rendering.

use sdl2::pixels::Color;
use sdl2::rect::Rect as SdlRect;
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
/// Visible height of the arrow glyph drawn over original scrollbar buttons.
const ARROW_GLYPH_HEIGHT: u32 = 5;
/// Visible width of the arrow glyph drawn over original scrollbar buttons.
const ARROW_GLYPH_WIDTH: u32 = 9;
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

    draw_scrollbar_arrow_glyphs(canvas, &scrollbar)?;

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
    canvas
        .fill_rect(to_sdl_rect(&thumb))
        .map_err(runtime_error)?;
    draw_scrollbar_arrow_glyphs(canvas, &scrollbar)
}

/// Draws visible directional glyphs over the original blank scroller buttons.
fn draw_scrollbar_arrow_glyphs(
    canvas: &mut Canvas<Window>,
    scrollbar: &MenuRect,
) -> Result<(), MainMenuError> {
    canvas.set_draw_color(Color::RGB(22, 18, 12));
    for (direction, rect) in arrow_glyph_plan_from_scrollbar(scrollbar) {
        draw_arrow_glyph(canvas, direction, &rect)?;
    }
    Ok(())
}

/// Draws one arrow glyph as a small filled triangle.
fn draw_arrow_glyph(
    canvas: &mut Canvas<Window>,
    direction: ArrowDirection,
    rect: &MenuRect,
) -> Result<(), MainMenuError> {
    let height = i32::try_from(rect.height).map_or(0_i32, |value| value);
    let center_x = rect.x.saturating_add(
        i32::try_from(rect.width.checked_div(2_u32).map_or(0_u32, |value| value))
            .map_or(0_i32, |value| value),
    );
    for row in 0_i32..height {
        let half_width = match direction {
            ArrowDirection::Up => row,
            ArrowDirection::Down => height.saturating_sub(1_i32).saturating_sub(row),
        };
        let width = u32::try_from(half_width.saturating_mul(2_i32).saturating_add(1_i32))
            .map_or(1_u32, |value| value);
        let x = center_x.saturating_sub(half_width);
        let y = rect.y.saturating_add(row);
        canvas
            .fill_rect(SdlRect::new(x, y, width, 1_u32))
            .map_err(runtime_error)?;
    }
    Ok(())
}

/// Returns arrow glyph rectangles inside a scrollbar.
fn arrow_glyph_plan_from_scrollbar(scrollbar: &MenuRect) -> [(ArrowDirection, MenuRect); 2] {
    let x = scrollbar.x.saturating_add(
        u32_to_i32(scrollbar.width.saturating_sub(ARROW_GLYPH_WIDTH))
            .checked_div(2_i32)
            .map_or(0_i32, |value| value),
    );
    let top_y = scrollbar.y.saturating_add(
        u32_to_i32(SCROLL_BUTTON_HEIGHT.saturating_sub(ARROW_GLYPH_HEIGHT))
            .checked_div(2_i32)
            .map_or(0_i32, |value| value),
    );
    let down_button_y = scrollbar.y.saturating_add(u32_to_i32(
        scrollbar.height.saturating_sub(SCROLL_BUTTON_HEIGHT),
    ));
    let bottom_y = down_button_y.saturating_add(
        u32_to_i32(SCROLL_BUTTON_HEIGHT.saturating_sub(ARROW_GLYPH_HEIGHT))
            .checked_div(2_i32)
            .map_or(0_i32, |value| value),
    );
    [
        (
            ArrowDirection::Up,
            MenuRect::new(x, top_y, ARROW_GLYPH_WIDTH, ARROW_GLYPH_HEIGHT),
        ),
        (
            ArrowDirection::Down,
            MenuRect::new(x, bottom_y, ARROW_GLYPH_WIDTH, ARROW_GLYPH_HEIGHT),
        ),
    ]
}

/// Direction for a scrollbar arrow glyph.
#[derive(Clone, Copy)]
enum ArrowDirection {
    /// Up/increment-toward-start glyph.
    Up,
    /// Down/increment-toward-end glyph.
    Down,
}

impl ArrowDirection {
    /// Returns a stable test label.
    #[cfg(test)]
    const fn label(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
        }
    }
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

/// Returns whether a point falls inside the listbox scrollbar.
pub(super) fn scrollbar_contains_point(rect: &MenuRect, point_x: i32, point_y: i32) -> bool {
    scrollbar_rect(rect).contains_point(point_x, point_y)
}

/// Maps a pointer y coordinate to the first visible row for thumb dragging.
pub(super) fn scrollbar_drag_offset_at(
    rect: &MenuRect,
    point_y: i32,
    item_count: usize,
    visible_rows: usize,
) -> usize {
    let scrollbar = scrollbar_rect(rect);
    let track = scrollbar_track_rect(&scrollbar);
    let max_offset = item_count.saturating_sub(visible_rows);
    if max_offset == 0 || track.height <= THUMB_HEIGHT {
        return 0_usize;
    }
    let thumb_height = THUMB_HEIGHT.min(track.height);
    let available = track.height.saturating_sub(thumb_height);
    if available == 0 {
        return 0_usize;
    }
    let half_thumb = u32_to_i32(thumb_height.checked_div(2_u32).map_or(0_u32, |value| value));
    let centered_y = point_y.saturating_sub(track.y).saturating_sub(half_thumb);
    let clamped_position = centered_y.clamp(0_i32, u32_to_i32(available));
    let numerator = u64::try_from(clamped_position)
        .map_or(0_u64, |value| value)
        .saturating_mul(u64::try_from(max_offset).map_or(u64::MAX, |value| value))
        .saturating_add(u64::from(
            available.checked_div(2_u32).map_or(0_u32, |value| value),
        ));
    let offset = numerator
        .checked_div(u64::from(available))
        .map_or(0_u64, |value| value);
    usize::try_from(offset).map_or(max_offset, |value| value.min(max_offset))
}

/// Returns the visible scrollbar arrow overlay plan for tests.
#[cfg(test)]
pub(in crate::main_menu::runtime) fn arrow_glyph_plan_for_test(
    listbox_rect: MenuRect,
) -> Vec<(&'static str, MenuRect)> {
    arrow_glyph_plan_from_scrollbar(&scrollbar_rect(&listbox_rect))
        .into_iter()
        .map(|(direction, rect)| (direction.label(), rect))
        .collect()
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
