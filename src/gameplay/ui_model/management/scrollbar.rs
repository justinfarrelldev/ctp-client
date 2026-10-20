//! Native AUI ranger geometry shared by source lists and hypertext viewports.

use super::{ManagementUi, ManagerIntent, ManagerView, ManagerWidget, articles, data, tree};
use crate::gameplay::assets::layout::LayoutNode;
use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::MenuRect;

/// Uses the exact visible parent/control contract, not a stale global row count.
pub(super) fn maximum(ui: &ManagementUi, state: &GameState, path: &str) -> Option<usize> {
    let view = ui.view.as_ref()?;
    let widget = ui.widgets(state).into_iter().find(|widget| {
        widget.id == path
            && widget.node.field("objecttype").is_some_and(|kind| {
                kind.eq_ignore_ascii_case("ctp2_ListBox")
                    || kind.eq_ignore_ascii_case("ctp2_hypertextbox")
            })
    })?;
    Some(extent(ui, state, view, path, widget.rect).0)
}

/// Source list units are rows; hypertext units are measured native lines.
fn extent(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    path: &str,
    rect: MenuRect,
) -> (usize, usize) {
    if let Some(maximum) = articles::scroll_limit(ui, state, view, path, rect) {
        return (maximum, articles::visible_lines(ui, view, path, rect));
    }
    let page = tree::visible_rows(view.kind, path, rect);
    (
        data::rows(ui, state, view, path).len().saturating_sub(page),
        page,
    )
}

/// Source geometry computed once per descriptor pass, using bounded integer ratios.
struct Geometry {
    /// Original viewport before the outside ranger column.
    rect: MenuRect,
    /// Track excludes the two original arrow buttons.
    track: MenuRect,
    /// Actual proportional, minimum-size thumb.
    thumb: MenuRect,
    /// Arrow-button extent, clamped for degenerate viewports.
    button: u32,
    /// Source logical maximum value.
    maximum: usize,
    /// Source visible-page size.
    page: usize,
    /// Current clamped value.
    offset: usize,
}
impl Geometry {
    /// Reproduces rounded AUI thumb ratios without floating-point drift/overflow.
    fn new(rect: MenuRect, width: u32, maximum: usize, page: usize, offset: usize) -> Option<Self> {
        let button = 15_u32.min(rect.height.div_euclid(2_u32));
        let x = rect
            .x
            .saturating_add(i32::try_from(rect.width).unwrap_or(0_i32));
        let top = rect
            .y
            .saturating_add(i32::try_from(button).unwrap_or(0_i32));
        let height = rect.height.saturating_sub(button.saturating_mul(2_u32));
        if height == 0_u32 || width == 0_u32 {
            return None;
        }
        let track = MenuRect {
            x,
            y: top,
            width,
            height,
        };
        let weight = u128::try_from(page)
            .unwrap_or(u128::MAX)
            .saturating_add(1_u128);
        let denominator = weight
            .saturating_add(u128::try_from(maximum).unwrap_or(u128::MAX))
            .max(1_u128);
        let thumb_height = u32::try_from(
            u128::from(height)
                .saturating_mul(weight)
                .saturating_add(denominator.div_euclid(2_u128))
                .div_euclid(denominator),
        )
        .unwrap_or(height)
        .clamp(20_u32.min(height), height);
        let travel = height.saturating_sub(thumb_height);
        let displacement = if maximum == 0_usize {
            0_u32
        } else {
            u32::try_from(
                u128::from(travel)
                    .saturating_mul(u128::try_from(offset).unwrap_or(u128::MAX))
                    .saturating_add(
                        u128::try_from(maximum)
                            .unwrap_or(u128::MAX)
                            .div_euclid(2_u128),
                    )
                    .div_euclid(u128::try_from(maximum).unwrap_or(u128::MAX)),
            )
            .unwrap_or(travel)
        };
        let thumb = MenuRect {
            x,
            y: top.saturating_add(i32::try_from(displacement).unwrap_or(0_i32)),
            width,
            height: thumb_height,
        };
        Some(Self {
            rect,
            track,
            thumb,
            button,
            maximum,
            page,
            offset,
        })
    }
}

/// Appends actual source-art arrows/track/thumb, including always-visible empty lists.
pub(super) fn append(
    ui: &ManagementUi,
    state: &GameState,
    view: &ManagerView,
    node: &LayoutNode,
    path: &str,
    rect: MenuRect,
    widgets: &mut Vec<ManagerWidget>,
) {
    let Some(ranger) = node.child("rangery") else {
        return;
    };
    let (maximum, page) = extent(ui, state, view, path, rect);
    if maximum == 0_usize
        && !node
            .field("alwaysranger")
            .is_some_and(|value| value.eq_ignore_ascii_case("true"))
    {
        return;
    }
    let width = ranger
        .integer("widthpix")
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(15_u32);
    let offset = view
        .scroll
        .get(path)
        .copied()
        .unwrap_or(0_usize)
        .min(maximum);
    let Some(geometry) = Geometry::new(rect, width, maximum, page, offset) else {
        return;
    };
    let mut style = LayoutNode::default();
    style.fields.insert(
        "pattern".to_owned(),
        ranger.field("pattern").unwrap_or("uptg04e.tga").to_owned(),
    );
    widgets.push(widget(
        format!("{path}/ranger/track"),
        geometry.track,
        style,
        None,
        None,
    ));
    thumb(ranger, path, &geometry, widgets);
    arrows(ui, ranger, path, &geometry, widgets);
    pages(path, &geometry, widgets);
}

/// A real range control retains exact current maximum/page metadata for keyboard use.
fn thumb(ranger: &LayoutNode, path: &str, geometry: &Geometry, widgets: &mut Vec<ManagerWidget>) {
    let mut style = LayoutNode::default();
    style.fields.insert(
        "pattern".to_owned(),
        ranger
            .child("thumb")
            .and_then(|part| part.field("pattern"))
            .unwrap_or("upbt14.tga")
            .to_owned(),
    );
    for (key, value) in [
        ("scroll-maximum", geometry.maximum),
        ("scroll-page", geometry.page),
    ] {
        style.fields.insert(key.to_owned(), value.to_string());
    }
    style
        .fields
        .insert("scroll-owner".to_owned(), path.to_owned());
    style
        .fields
        .insert("aria-label".to_owned(), "Scroll position".to_owned());
    widgets.push(widget(
        format!("{path}/ranger/thumb"),
        geometry.thumb,
        style,
        (geometry.maximum > 0_usize).then(|| ManagerIntent::BeginScroll(path.to_owned())),
        None,
    ));
}

/// C3 ranger arrows are centered installed 9x9 images, not font-dependent glyphs.
fn arrows(
    ui: &ManagementUi,
    ranger: &LayoutNode,
    path: &str,
    geometry: &Geometry,
    widgets: &mut Vec<ManagerWidget>,
) {
    for (suffix, arrow, y, delta, enabled) in [
        (
            "up",
            "RangerUp",
            geometry.rect.y,
            -1_i32,
            geometry.offset > 0_usize,
        ),
        (
            "down",
            "RangerDown",
            geometry.rect.y.saturating_add(
                i32::try_from(geometry.rect.height.saturating_sub(geometry.button))
                    .unwrap_or(0_i32),
            ),
            1_i32,
            geometry.offset < geometry.maximum,
        ),
    ] {
        let mut style = LayoutNode::default();
        let key = if suffix == "up" { "decy" } else { "incy" };
        style.fields.insert(
            "pattern".to_owned(),
            ranger
                .child(key)
                .and_then(|part| part.field("pattern"))
                .unwrap_or("upbt13.tga")
                .to_owned(),
        );
        style
            .fields
            .insert("aria-label".to_owned(), format!("Scroll {suffix}"));
        widgets.push(widget(
            format!("{path}/{suffix}"),
            MenuRect {
                x: geometry.track.x,
                y,
                width: geometry.track.width,
                height: geometry.button,
            },
            style,
            enabled.then(|| ManagerIntent::Scroll(path.to_owned(), delta)),
            (!enabled).then(|| "Scroll boundary".to_owned()),
        ));
        if let Ok(mut image) = ui.book.resolve(arrow) {
            image.children.clear();
            widgets.push(widget(
                format!("{path}/{suffix}/icon"),
                MenuRect {
                    x: geometry.track.x.saturating_add(3_i32),
                    y: y.saturating_add(3_i32),
                    width: 9_u32,
                    height: 9_u32,
                },
                image,
                None,
                None,
            ));
        }
    }
}

/// The two track regions have separate semantic page actions, matching pointer hits.
fn pages(path: &str, geometry: &Geometry, widgets: &mut Vec<ManagerWidget>) {
    let displacement =
        u32::try_from(geometry.thumb.y.saturating_sub(geometry.track.y)).unwrap_or(0_u32);
    for (suffix, area, delta) in [
        (
            "page-up",
            MenuRect {
                x: geometry.track.x,
                y: geometry.track.y,
                width: geometry.track.width,
                height: displacement,
            },
            i32::try_from(geometry.page)
                .unwrap_or(i32::MAX)
                .saturating_neg(),
        ),
        (
            "page-down",
            MenuRect {
                x: geometry.track.x,
                y: geometry
                    .thumb
                    .y
                    .saturating_add(i32::try_from(geometry.thumb.height).unwrap_or(0_i32)),
                width: geometry.track.width,
                height: geometry
                    .track
                    .height
                    .saturating_sub(displacement)
                    .saturating_sub(geometry.thumb.height),
            },
            i32::try_from(geometry.page).unwrap_or(i32::MAX),
        ),
    ] {
        if area.height > 0_u32 {
            widgets.push(widget(
                format!("{path}/ranger/{suffix}"),
                area,
                LayoutNode::default(),
                Some(ManagerIntent::Scroll(path.to_owned(), delta)),
                None,
            ));
        }
    }
}

/// Starts at the actual grab offset rather than jumping the thumb on first motion.
pub(super) fn begin(
    ui: &mut ManagementUi,
    path: &str,
    pointer_y: Option<i32>,
    state: &GameState,
) -> bool {
    let id = format!("{path}/ranger/thumb");
    let Some(thumb) = ui
        .widgets(state)
        .into_iter()
        .find(|widget| widget.id == id && widget.intent.is_some())
    else {
        return false;
    };
    let offset = pointer_y.map_or_else(
        || i32::try_from(thumb.rect.height.div_euclid(2_u32)).unwrap_or(0_i32),
        |y| y.saturating_sub(thumb.rect.y),
    );
    ui.scroll_drag = Some((path.to_owned(), offset));
    true
}

/// Converts pointer motion to a bounded current value without float drift.
pub(super) fn drag(ui: &mut ManagementUi, y: i32, state: &GameState) -> bool {
    let Some((path, grab)) = ui.scroll_drag.clone() else {
        return false;
    };
    let widgets = ui.widgets(state);
    let track = widgets
        .iter()
        .find(|widget| widget.id == format!("{path}/ranger/track"));
    let thumb = widgets
        .iter()
        .find(|widget| widget.id == format!("{path}/ranger/thumb"));
    let (Some(track), Some(thumb)) = (track, thumb) else {
        ui.scroll_drag = None;
        return false;
    };
    let maximum = thumb
        .node
        .field("scroll-maximum")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0_usize);
    let travel = track.rect.height.saturating_sub(thumb.rect.height);
    let position = u32::try_from(y.saturating_sub(track.rect.y).saturating_sub(grab))
        .unwrap_or(0_u32)
        .min(travel);
    let value = if travel == 0_u32 {
        0_usize
    } else {
        usize::try_from(
            u128::from(position)
                .saturating_mul(u128::try_from(maximum).unwrap_or(u128::MAX))
                .saturating_add(u128::from(travel.div_euclid(2_u32)))
                .div_euclid(u128::from(travel)),
        )
        .unwrap_or(maximum)
    };
    ui.scroll_to(&path, value, state)
}

/// Draw-only parts never intercept adjacent semantic controls.
const fn widget(
    id: String,
    rect: MenuRect,
    node: LayoutNode,
    intent: Option<ManagerIntent>,
    unavailable: Option<String>,
) -> ManagerWidget {
    ManagerWidget {
        id,
        label: String::new(),
        rect,
        node,
        intent,
        unavailable,
        selected: false,
    }
}
