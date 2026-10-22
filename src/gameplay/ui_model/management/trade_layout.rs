//! Resolved-property adapters for the source three-tab Trade reference revision.
//! Geometry follows public trademanager.ldl; actual templates/art stay installed.
use super::fidelity::{node, set};
use crate::gameplay::{
    GameplayError, asset_error,
    assets::layout::{LayoutBook, LayoutNode},
};

/// Restores Market/Export/Import from original installed three-tab art, not a
/// stretched two-tab bitmap or a copied replacement asset.
pub(super) fn main(root: &mut LayoutNode, book: &LayoutBook) -> Result<(), GameplayError> {
    let standard = book.resolve("CTP2_3TAB_DIALOG_WINDOW")?;
    set(root, "", &[("widthpix", "591")]);
    set(root, "Background", &[("widthpix", "591")]);
    let tabs = node(root, "TradeTabs")
        .ok_or_else(|| asset_error("missing source TradeTabs".to_owned()))?;
    if tabs.child("Import").is_none() {
        let mut imported = tabs
            .child("Summary")
            .cloned()
            .ok_or_else(|| asset_error("missing source Trade Summary".to_owned()))?;
        "Import".clone_into(&mut imported.name);
        tabs.children.push(imported);
    }
    for (name, template, x, key, fallback) in [
        ("Market", "Tab1", "7", "str_ldl_TradeMarket", "Market"),
        ("Summary", "Tab2", "151", "str_ldl_Export", "Export"),
        ("Import", "Tab3", "322", "str_ldl_TradeImport", "Import"),
    ] {
        let mut button = standard
            .child("TabGroup")
            .and_then(|group| group.child(template))
            .and_then(|tab| tab.child("TabButton"))
            .cloned()
            .ok_or_else(|| asset_error("missing original three-tab template".to_owned()))?;
        let target = node(root, &format!("TradeTabs.{name}.TabButton"))
            .ok_or_else(|| asset_error("missing Trade tab button".to_owned()))?;
        if let Some(status) = target.field("statustext") {
            button
                .fields
                .insert("statustext".to_owned(), status.to_owned());
        }
        button.fields.insert("xpix".to_owned(), x.to_owned());
        button
            .fields
            .insert("text".to_owned(), caption(book, key, fallback));
        *target = button;
        let list = format!("TradeTabs.{name}.TabPanel.List");
        set(
            root,
            &list,
            &[
                ("xpix", "20"),
                ("ypix", if name == "Market" { "58" } else { "38" }),
                ("widthpix", "466"),
                ("heightpix", if name == "Market" { "200" } else { "220" }),
            ],
        );
        let price = if name == "Market" { 5_u8 } else { 6_u8 };
        let caravans = price.saturating_add(1);
        set(
            root,
            &format!("{list}.header.switch{price}"),
            &[("widthpix", "35")],
        );
        set(
            root,
            &format!("{list}.header.switch{caravans}"),
            &[("widthpix", "60")],
        );
        set(
            root,
            &format!(
                "TradeTabs.{name}.TabPanel.{}",
                if name == "Market" {
                    "CreateRouteButton"
                } else {
                    "BreakRouteButton"
                }
            ),
            &[("xpix", "177")],
        );
    }
    filters(root);
    set(
        root,
        "TradeTabs.Market.TabPanel.List.header.switch0",
        &[("text", "str_ldl_Source")],
    );
    set(
        root,
        "TradeTabs.Import.TabPanel.List.header.switch7",
        &[("text", &caption(book, "str_ldl_TradeBonus", "Bonus"))],
    );
    set(root, "ShowAdviceButton", &[("xpix", "56")]);
    Ok(())
}
/// Public source revision offsets the filter group without changing its controls.
fn filters(root: &mut LayoutNode) {
    set(root, "TradeTabs.Market.TabPanel", &[("widthpix", "511")]);
    set(
        root,
        "TradeTabs.Market.TabPanel.Filters",
        &[("xpix", "9"), ("ypix", "3"), ("widthpix", "485")],
    );
    for (name, x) in [
        ("CitiesPerGoodLabel", "277"),
        ("CitiesSlider", "377"),
        ("NumCities", "463"),
    ] {
        set(
            root,
            &format!("TradeTabs.Market.TabPanel.Filters.{name}"),
            &[("xpix", x)],
        );
    }
}
/// Six 13-pixel source accounting rows replace the old four 20-pixel rows.
pub(super) fn advice(root: &mut LayoutNode, book: &LayoutBook) {
    for (name, key, fallback) in [
        (
            "Transit",
            "str_ldl_TotalTransitProfit",
            "Total Transit Profit:",
        ),
        ("Piracy", "str_ldl_TotalPiracyLoss", "Expected Piracy Loss:"),
    ] {
        for suffix in ["Label", ""] {
            let target = format!("{name}{suffix}");
            if root.child(&target).is_none() {
                if let Some(mut cloned) = root.child(&format!("Profit{suffix}")).cloned() {
                    cloned.name.clone_from(&target);
                    root.children.insert(0, cloned);
                }
            }
            if suffix == "Label" {
                set(root, &target, &[("text", &caption(book, key, fallback))]);
            }
        }
    }
    for (name, y) in [
        ("Available", "229"),
        ("InUse", "242"),
        ("Routes", "255"),
        ("Profit", "268"),
        ("Transit", "281"),
        ("Piracy", "294"),
    ] {
        for suffix in ["Label", ""] {
            set(
                root,
                &format!("{name}{suffix}"),
                &[("ypix", y), ("heightpix", "13")],
            );
        }
    }
}
/// Installed captions win; only missing revision-specific short UI labels fall back.
fn caption(book: &LayoutBook, key: &str, fallback: &str) -> String {
    let label = book.label(key);
    if label == key {
        fallback.to_owned()
    } else {
        label.to_owned()
    }
}
