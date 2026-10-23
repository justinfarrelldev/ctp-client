//! Reference-sized presentation contracts, independent of server commands.

use super::{ManagementUi, ManagerKind,ManagerIntent};
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::model::GameState;
use crate::gameplay::server::ServerProjection;
use std::path::Path;

/// Installed read-only art/layout with a deliberately empty authoritative viewer.
fn fixture(kind: ManagerKind) -> (ManagementUi, GameState) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
    let mut ui = ManagementUi::new(LayoutBook::load(&root).expect("installed layout"))
        .expect("manager presentation");
    let state = GameState::from_server_projection(&ServerProjection::default(), 1_u8, 1_u64);
    ui.open(kind, &state, None);
    (ui, state)
}

#[test]
fn reference_empire_fills_the_screen_and_separates_food_production_and_split_commerce() {
    let (ui, state) = fixture(ManagerKind::Empire);
    let widgets = ui.widgets(&state);
    let bounds = widgets.first().expect("Empire").rect;
    assert_eq!(
        (bounds.x, bounds.y, bounds.width, bounds.height),
        (0_i32, 28_i32, 800_u32, 558_u32),
        "reference root leaves top menu and status visible"
    );
    for (path, x, width) in [
        ("Food", 40_i32, 196_u32),
        ("Production", 236_i32, 196_u32),
        ("Commerce", 432_i32, 328_u32),
    ] {
        let group = widgets
            .iter()
            .find(|widget| widget.id == format!("TabGroup.Tab1.TabPanel.{path}"))
            .expect("source column");
        assert_eq!(
            (
                group.rect.x,
                group.rect.y,
                group.rect.width,
                group.rect.height
            ),
            (x, 133_i32, width, 414_u32),
            "reference column {path}"
        );
    }
    for path in ["Food", "Production", "Commerce"] {
        let slider = widgets
            .iter()
            .find(|widget| {
                widget.id == format!("TabGroup.Tab1.TabPanel.{path}.Modify.Primary.Slider")
            })
            .expect("policy slider");
        assert_eq!(
            slider.rect.width, 152_u32,
            "reference quantized slider span"
        );
    }
    assert!(
        widgets
            .iter()
            .any(|widget| widget.id
                == "TabGroup.Tab1.TabPanel.Commerce.InformationSummary.GoldTitle"),
        "reference commerce splits into science and gold"
    );
    assert!(
        widgets
            .iter()
            .any(|widget| widget.id == "OptimizeSlidersButton" && widget.intent.is_none()),
        "source optimizer remains visible but not a fake operation"
    );
    assert!(
        !widgets
            .iter()
            .any(|widget| widget.label.starts_with("str_ldl_")),
        "missing retail keys must not leak onto the screen: {:?}",
        widgets
            .iter()
            .filter(|widget| widget.label.starts_with("str_ldl_"))
            .map(|widget| (&widget.id, &widget.label))
            .collect::<Vec<_>>()
    );
}

#[test]
fn reference_graphics_has_two_sliders_and_twelve_labeled_switches_without_fake_commands() {
    let (ui, state) = fixture(ManagerKind::Graphics);
    let widgets = ui.widgets(&state);
    let root = widgets.first().expect("Graphics").rect;
    assert_eq!(
        (root.x, root.y, root.width, root.height),
        (60_i32, 105_i32, 680_u32, 390_u32),
        "six reference rows, not the later seven-row community layout"
    );
    for id in ["UnitSpeedSlider", "CityNameAlphaSlider"] {
        assert!(
            widgets
                .iter()
                .any(|widget| widget.id == format!("{id}/track")),
            "native source groove for {id}"
        );
        assert!(
            widgets
                .iter()
                .any(|widget| widget.id == format!("{id}/thumb")),
            "native thumb for {id}"
        );
    }
    for id in [
        "WalkButton",
        "TradeButton",
        "WonderButton",
        "ArmyNamesButton",
        "CivFlagButton",
        "ShowCityProdButton",
        "PoliticalBordersButton",
        "TradeRoutesButton",
        "CityInflenceButton",
        "GridButton",
        "SmoothButton",
        "GoodsButton",
    ] {
        let switch = widgets
            .iter()
            .find(|widget| widget.id == id)
            .expect("reference switch");
        assert!(
            !switch.label.is_empty(),
            "source stringtable caption for {id}"
        );
        let local_key=match id {"PoliticalBordersButton"=>Some("map-filter:political"),"CityInflenceButton"=>Some("map-filter:influence"),_=>None};
        if let Some(key)=local_key {
            assert_eq!(switch.intent,Some(ManagerIntent::Select(key.to_owned(),"false".to_owned())),"only the two implemented source callbacks may change local overlays");
            assert!(switch.unavailable.is_none()&&switch.selected,"actual source ON flag is available without server authority");
        } else {
            assert!(switch.intent.is_none(),"unimplemented option {id} must remain disabled; no fake command");
        }
    }
}

#[test]
fn sound_sliders_are_atomic_so_thumbs_never_remain_at_the_left_edge() {
    let (ui, state) = fixture(ManagerKind::Sound);
    let widgets = ui.widgets(&state);
    for id in ["SFXSlider", "VoiceSlider", "MusicSlider"] {
        let owner = widgets
            .iter()
            .find(|widget| widget.id == id)
            .expect("slider");
        let thumb = widgets
            .iter()
            .find(|widget| widget.id == format!("{id}/thumb"))
            .expect("atomic thumb");
        assert_eq!(
            thumb.rect.x,
            owner.rect.x.saturating_add(76_i32),
            "source value5 at midpoint"
        );
        assert!(
            !widgets
                .iter()
                .any(|widget| widget.id == format!("{id}.thumb")),
            "unpositioned LDL child must not overpaint real thumb"
        );
    }
}
