//! Presentation deltas between installed retail LDL and the supplied CTP2 captures.
//!
//! The references include community-layout additions. Keep the installed book
//! immutable: adapt resolved native controls, never copy replacement game assets.
//! Geometry follows public `default/uidata/layouts/{civstatus,graphics}.ldl`,
//! restricted to the six-row Graphics revision actually shown in the references.

use super::ManagerKind;
use crate::gameplay::GameplayError;
use crate::gameplay::assets::layout::{LayoutBook, LayoutNode};

/// Empire's larger source layout and the captured Graphics revision.
pub(super) fn configure(
    kind: ManagerKind,
    root: &mut LayoutNode,
    book: &LayoutBook,
) -> Result<(), GameplayError> {
    match kind {
        ManagerKind::Empire => empire(root, book)?,
        ManagerKind::Graphics => graphics(root, book)?,
        _ => {}
    }
    Ok(())
}

/// Exact scoped lookup; absent optional source controls are not guessed.
fn node<'layout>(root: &'layout mut LayoutNode, path: &str) -> Option<&'layout mut LayoutNode> {
    let mut current = root;
    for part in path.split('.').filter(|part| !part.is_empty()) {
        current = current
            .children
            .iter_mut()
            .find(|child| child.name == part)?;
    }
    Some(current)
}

/// Mutates only resolved source properties, retaining native style/art metadata.
fn set(root: &mut LayoutNode, path: &str, fields: &[(&str, &str)]) {
    if let Some(control) = node(root, path) {
        for &(key, value) in fields {
            control.fields.insert(key.to_owned(), value.to_owned());
        }
    }
}

/// Source-sized text with explicit bounds prevents inherited parent-sized labels.
fn text(
    book: &LayoutBook,
    name: &str,
    label: &str,
    x: i32,
    y: i32,
    width: i32,
) -> Result<LayoutNode, GameplayError> {
    let mut control = book.resolve("CTP2_STATIC_BACKGROUND")?;
    name.clone_into(&mut control.name);
    for (key, value) in [
        ("text", label.to_owned()),
        ("xpix", x.to_string()),
        ("ypix", y.to_string()),
        ("widthpix", width.to_string()),
        ("heightpix", "16".to_owned()),
        ("just", "left".to_owned()),
    ] {
        control.fields.insert(key.to_owned(), value);
    }
    Ok(control)
}

/// Full-height domestic panel, expanded social ranges and source header spinners.
fn empire(root: &mut LayoutNode, book: &LayoutBook) -> Result<(), GameplayError> {
    set(root, "", &[("widthpix", "800"), ("heightpix", "558")]);
    set(
        root,
        "Background",
        &[
            ("widthpix", "800"),
            ("heightpix", "558"),
            ("imageblttype03", "tile"),
            ("imageblttype05", "tile"),
        ],
    );
    set(root, "CloseButton", &[("xpix", "676"), ("ypix", "528")]);
    set(
        root,
        "DetailsButton",
        &[("xpix", "443"), ("ypix", "528"), ("widthpix", "150")],
    );
    let mut optimize = book.resolve("CTP2_GENERIC_SIZABLE_TEXT_BUTTON")?;
    "OptimizeSlidersButton".clone_into(&mut optimize.name);
    for (key, value) in [
        ("xpix", "200"),
        ("ypix", "528"),
        ("widthpix", "150"),
        ("text", "Optimize Sliders"),
    ] {
        optimize.fields.insert(key.to_owned(), value.to_owned());
    }
    root.children.insert(0_usize, optimize);
    set(
        root,
        "TabGroup",
        &[("widthpix", "730"), ("heightpix", "490")],
    );
    for tab in ["Tab1", "Tab2"] {
        set(
            root,
            &format!("TabGroup.{tab}.TabPanel"),
            &[("widthpix", "730"), ("heightpix", "466")],
        );
        set(
            root,
            &format!("TabGroup.{tab}.TabButton"),
            &[("widthpix", "365"), ("imageblttype", "stretch")],
        );
    }
    set(root, "TabGroup.Tab2.TabButton", &[("xpix", "353")]);
    let Some(panel) = node(root, "TabGroup.Tab1.TabPanel") else {
        return Ok(());
    };
    empire_header(panel, book)?;
    empire_columns(panel, book)
}

/// Reference header positions; undeclared economy values remain unknown.
fn empire_header(panel: &mut LayoutNode, book: &LayoutBook) -> Result<(), GameplayError> {
    for name in [
        "CitiesValue",
        "PopulationValue",
        "HappinessValue",
        "HappinessLabel",
        "CurrentGovernmentLabel",
        "CurrentGovernment",
    ] {
        set(panel, name, &[("heightpix", "16"), ("vertcenter", "false")]);
    }
    set(
        panel,
        "HappinessLabel",
        &[("widthpix", "140"), ("just", "left")],
    );
    set(panel, "CurrentGovernmentLabel", &[("just", "center")]);
    for (path, fields) in [
        (
            "CitiesIcon",
            vec![
                ("xpix", "12"),
                ("ypix", "6"),
                ("text", ""),
                ("image00", "upic04.tga"),
            ],
        ),
        (
            "PopulationIcon",
            vec![
                ("xpix", "12"),
                ("ypix", "27"),
                ("text", ""),
                ("image00", "upic03.tga"),
            ],
        ),
        ("CitiesValue", vec![("xpix", "42")]),
        ("PopulationValue", vec![("xpix", "42")]),
        ("HappinessLabel", vec![("xpix", "301")]),
        ("HappinessValue", vec![("xpix", "411")]),
        ("HappinessBar", vec![("xpix", "301")]),
        ("CurrentGovernmentLabel", vec![("xpix", "551")]),
        ("CurrentGovernment", vec![("xpix", "551")]),
    ] {
        set(panel, path, &fields);
    }
    // Move existing source controls rather than creating new policy identities.
    // Retaining their paths keeps revision-bound dispatch and accessible IDs intact.
    set(
        panel,
        "Production.Modify.Secondary.Spinner",
        &[("reference-x", "186"), ("reference-y", "21")],
    );
    set(
        panel,
        "Commerce.Modify.Secondary.Spinner",
        &[("reference-x", "470"), ("reference-y", "21")],
    );
    panel.children.insert(
        0_usize,
        text(
            book,
            "PWTaxLabel",
            "str_ldl_DD_PUBLIC_WORKS_LABEL",
            186_i32,
            9_i32,
            100_i32,
        )?,
    );
    panel.children.insert(
        0_usize,
        text(
            book,
            "ScieTaxLabel",
            "str_ldl_DD_SCIENCE_TAX_LABEL",
            470_i32,
            9_i32,
            100_i32,
        )?,
    );
    Ok(())
}

/// Independent resource columns, including the widened split commerce pane.
fn empire_columns(panel: &mut LayoutNode, book: &LayoutBook) -> Result<(), GameplayError> {
    for (name, x, width) in [
        ("Food", "5", "196"),
        ("Production", "201", "196"),
        ("Commerce", "397", "328"),
    ] {
        let Some(group) = node(panel, name) else {
            continue;
        };
        set(
            group,
            "",
            &[
                ("xpix", x),
                ("widthpix", width),
                ("heightpix", "414"),
                ("pattern", "uptg20e.tga"),
            ],
        );
        let inner = if name == "Commerce" { "324" } else { "192" };
        for path in ["Title", "Separator1", "Modify", "Separator2"] {
            set(group, path, &[("widthpix", inner)]);
        }
        set(group, "Modify", &[("heightpix", "58")]);
        set(group, "Modify.Primary", &[("widthpix", width)]);
        set(
            group,
            "Modify.Primary.Label",
            &[("just", "left"), ("widthpix", "66"), ("heightpix", "16")],
        );
        set(
            group,
            "Modify.Primary.CurValue",
            &[("heightpix", "16"), ("text", "—"), ("just", "left")],
        );
        set(
            group,
            "Modify.Primary.Value",
            &[("heightpix", "16"), ("text", "—")],
        );
        set(
            group,
            "Modify.Primary.Icon",
            &[("xpix", if name == "Commerce" { "284" } else { "152" })],
        );
        set(
            group,
            "Modify.Primary.Value",
            &[("xpix", if name == "Commerce" { "304" } else { "172" })],
        );
        set(
            group,
            "Modify.Primary.Slider",
            &[
                ("widthpix", "152"),
                ("xpix", if name == "Commerce" { "86" } else { "20" }),
            ],
        );
        column_ticks(group, name);
        set(group, "Separator2", &[("heightpix", "0")]);
        if let Some(secondary) = node(group, "Modify.Secondary") {
            secondary.children.retain(|child| child.name == "Spinner");
        }
        for mode in ["InformationSummary", "InformationDetails"] {
            if let Some(info) = node(group, mode) {
                // The captured summary has lower accounting rows and no duplicated Collected.
                info.children.clear();
                info.fields.insert("ypix".to_owned(), "82".to_owned());
                info.fields.insert("widthpix".to_owned(), inner.to_owned());
                info.fields.insert("heightpix".to_owned(), "330".to_owned());
                accounting(info, book, name)?;
            }
        }
    }
    Ok(())
}

/// Static scale captions are installed presentation text, not policy conversions.
fn column_ticks(group: &mut LayoutNode, column: &str) {
    for (index, tick) in [22_i32, 57_i32, 90_i32, 125_i32, 160_i32]
        .into_iter()
        .enumerate()
    {
        let tick = if column == "Commerce" {
            tick.saturating_add(66_i32)
        } else {
            tick
        };
        set(
            group,
            &format!("Modify.Primary.SlideLabel{}", index.saturating_add(1_usize)),
            &[
                ("xpix", &tick.to_string()),
                ("widthpix", "24"),
                ("heightpix", "16"),
                ("vertcenter", "false"),
            ],
        );
    }
}

/// Declarative source accounting label, placement and negative-value style.
type AccountingRow = (&'static str, &'static str, i32, i32, bool);

/// Only source field labels are static; economic values are not simulated.
fn accounting_rows(column: &str) -> Vec<AccountingRow> {
    let mut rows = vec![
        ("Collected", "Subtotal:", 211_i32, 0_i32, false),
        ("Crime", "Crime:", 268_i32, 0_i32, true),
    ];
    match column {
        "Food" => rows.extend([
            ("Consumed", "Consumed:", 251_i32, 0_i32, true),
            ("Remainder", "Stored:", 301_i32, 0_i32, false),
        ]),
        "Production" => rows.extend([
            ("Enemy", "Enemy:", 234_i32, 0_i32, true),
            ("Upkeep", "Unit Upkeep:", 251_i32, 0_i32, true),
            ("CityUse", "City Use:", 293_i32, 0_i32, false),
            ("PublicWorks", "Public Works:", 310_i32, 0_i32, false),
        ]),
        _ => {
            rows = vec![
                ("Collected", "Subtotal:", 53_i32, 0_i32, false),
                ("ScieSubTotal", "Subtotal:", 211_i32, 0_i32, false),
                ("GoldSubTotal", "Subtotal:", 211_i32, 162_i32, false),
                ("Crime", "Crime:", 268_i32, 0_i32, true),
                ("Enemy", "Enemy:", 234_i32, 162_i32, true),
                ("Upkeep", "Unit Support:", 251_i32, 162_i32, true),
                ("GoldCrime", "Crime:", 268_i32, 162_i32, true),
                ("Science", "Science:", 301_i32, 0_i32, false),
                ("Savings", "Savings:", 293_i32, 162_i32, false),
                ("Conversion", "Conversion:", 310_i32, 162_i32, false),
            ];
        }
    }
    rows
}

/// Original split-commerce and lower accounting rows, never fabricated totals.
fn accounting(info: &mut LayoutNode, book: &LayoutBook, column: &str) -> Result<(), GameplayError> {
    let mut rows = accounting_rows(column);
    if column == "Commerce" {
        for (name, label, x) in [
            ("ScienceTitle", "SCIENCE", 0_i32),
            ("GoldTitle", "GOLD", 162_i32),
        ] {
            let mut title = text(book, name, label, x, 73_i32, 162_i32)?;
            title.fields.insert("just".to_owned(), "center".to_owned());
            title
                .fields
                .insert("pattern".to_owned(), "uptg20e3.tga".to_owned());
            info.children.push(title);
        }
    }
    if info.name == "InformationDetails" {
        for (name, label, y) in [
            ("Terrain", "Terrain:", 2_i32),
            ("Buildings", "Buildings:", 19_i32),
            ("Specialists", "Specialists:", 36_i32),
        ] {
            rows.push((name, label, y, 0_i32, false));
        }
    }
    let split = column == "Commerce";
    for (name, label, y, x, negative) in rows {
        let mut row = book.resolve("DD_CAUSE_AND_EFFECT_LABEL")?;
        name.clone_into(&mut row.name);
        for (key, value) in [
            ("xpix", x.to_string()),
            ("ypix", y.to_string()),
            ("widthpix", if split { "162" } else { "192" }.to_owned()),
            ("heightpix", "17".to_owned()),
        ] {
            row.fields.insert(key.to_owned(), value);
        }
        set(
            &mut row,
            "Label",
            &[
                ("text", label),
                ("just", "left"),
                ("widthpix", "120"),
                ("heightpix", "16"),
            ],
        );
        set(
            &mut row,
            "Value",
            &[
                ("text", "—"),
                ("xpix", if split { "110" } else { "140" }),
                ("widthpix", "46"),
                ("ypix", "2"),
                ("heightpix", "16"),
                ("just", "right"),
                ("fontcolorred", if negative { "245" } else { "50" }),
            ],
        );
        info.children.push(row);
    }
    // Lower horizontal rules and commerce's science/gold divider.
    for (name, x, y, width, height) in [
        (
            "Rule1",
            0_i32,
            229_i32,
            if split { 324_i32 } else { 192_i32 },
            1_i32,
        ),
        (
            "Rule2",
            0_i32,
            285_i32,
            if split { 324_i32 } else { 192_i32 },
            1_i32,
        ),
    ] {
        let mut rule = text(book, name, "", x, y, width)?;
        rule.fields
            .insert("heightpix".to_owned(), height.to_string());
        rule.fields
            .insert("pattern".to_owned(), "uptg20e3.tga".to_owned());
        info.children.push(rule);
    }
    if split {
        let mut divider = text(book, "Divider", "", 162_i32, 73_i32, 1_i32)?;
        divider
            .fields
            .insert("heightpix".to_owned(), "256".to_owned());
        divider
            .fields
            .insert("pattern".to_owned(), "uptg20e3.tga".to_owned());
        info.children.push(divider);
    }
    Ok(())
}

/// Reference six-row options; later Windowed/Run-in-background rows are absent.
fn graphics(root: &mut LayoutNode, book: &LayoutBook) -> Result<(), GameplayError> {
    set(root, "", &[("heightpix", "390")]);
    set(root, "UnitSpeedSlider", &[("ypix", "45")]);
    let mut alpha = root.child("UnitSpeedSlider").cloned().unwrap_or_default();
    "CityNameAlphaSlider".clone_into(&mut alpha.name);
    for (key, value) in [("ypix", "85"), ("maxxval", "4"), ("xval", "4")] {
        alpha.fields.insert(key.to_owned(), value.to_owned());
    }
    root.children.insert(0_usize, alpha);
    let mut alpha_label = text(
        book,
        "CityNameAlphaName",
        "Show City Names",
        30_i32,
        95_i32,
        140_i32,
    )?;
    alpha_label
        .fields
        .insert("just".to_owned(), "right".to_owned());
    root.children.insert(0_usize, alpha_label);
    root.children
        .retain(|child| child.name != "CityNamesButton");
    for (id, x, y, label) in [
        ("WalkButton", 30_i32, 130_i32, "Unit Animations On"),
        ("TradeButton", 30_i32, 160_i32, "Animated Trade Routes On"),
        ("WonderButton", 30_i32, 190_i32, "Wonder Movies On"),
        ("ArmyNamesButton", 30_i32, 220_i32, "Hide Army Names"),
        ("CivFlagButton", 30_i32, 250_i32, "Show Nation Flags"),
        (
            "ShowCityProdButton",
            30_i32,
            280_i32,
            "Show City Production",
        ),
        (
            "PoliticalBordersButton",
            350_i32,
            130_i32,
            "Show Political Borders",
        ),
        ("TradeRoutesButton", 350_i32, 160_i32, "Show Trade Routes"),
        (
            "CityInflenceButton",
            350_i32,
            190_i32,
            "Show City Influence",
        ),
        ("GridButton", 350_i32, 220_i32, "Map Grid Lines Off"),
        ("SmoothButton", 350_i32, 250_i32, "Thin Pixel Borders"),
        ("GoodsButton", 350_i32, 280_i32, "Good Animations Off"),
    ] {
        let existing = root.children.iter().position(|child| child.name == id);
        let mut switch = existing.map_or_else(
            || book.resolve("GRAPHICS_CHECKBOX"),
            |index| Ok(root.children.remove(index)),
        )?;
        id.clone_into(&mut switch.name);
        switch.children.clear();
        for (key, value) in [
            ("xpix", x.to_string()),
            ("ypix", y.to_string()),
            ("text", label.to_owned()),
            (
                "image0",
                if matches!(
                    id,
                    "ArmyNamesButton" | "GridButton" | "SmoothButton" | "GoodsButton"
                ) {
                    "upbt11u0.tga"
                } else {
                    "upbt11u1.tga"
                }
                .to_owned(),
            ),
        ] {
            switch.fields.insert(key.to_owned(), value);
        }
        root.children.insert(0_usize, switch);
    }
    set(
        root,
        "ResolutionButton",
        &[("ypix", "325"), ("heightpix", "39")],
    );
    Ok(())
}
