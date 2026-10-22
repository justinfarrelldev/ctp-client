//! Framebuffer assertions use memory only; no snapshot files are ever written.

use super::{HeadlessSdlEnvironment, snapshot};
use crate::gameplay::ui_model::{MenuCategory, management::ManagerKind};
use std::path::Path;

#[test]
fn unit_reference_diagnostic_is_two_owned_settlers_not_the_unrelated_city_preview() {
    let scene = snapshot::scene::PreviewScene::PopulatedManager {
        kind: ManagerKind::Unit,
        chart: false,
    };
    let state = snapshot::scene::preview_state(scene);
    assert_eq!(
        state
            .units
            .iter()
            .filter(|unit| unit.owner == crate::gameplay::model::PlayerId::Human)
            .count(),
        2,
        "reference's actual own type count is explicit fixture data"
    );
    assert!(
        state.cities.is_empty(),
        "do not borrow the Build diagnostic's city/queue"
    );
    assert_eq!(
        state
            .viewer
            .as_ref()
            .expect("viewer")
            .diplomacy
            .as_ref()
            .expect("nation")
            .facts["readiness"],
        serde_json::json!("war"),
        "reference's known current readiness"
    );
    assert!(
        state
            .viewer
            .as_ref()
            .expect("viewer")
            .units
            .iter()
            .all(|unit| unit.type_id == "UNIT_SETTLER"),
        "reference scenario, not inferred local starters"
    );
}

#[test]
fn trade_and_national_reference_fixtures_are_explicit_no_city_starts_not_builds_queue() {
    for kind in [ManagerKind::Trade, ManagerKind::National] {
        let state =
            snapshot::scene::preview_state(snapshot::scene::PreviewScene::PopulatedManager {
                kind,
                chart: false,
            });
        assert!(
            state.cities.is_empty(),
            "original empty manager reference has not founded a city"
        );
        assert_eq!(
            state.units.len(),
            2,
            "same explicit two-settler starting profile"
        );
        assert_eq!(
            state
                .viewer
                .as_ref()
                .and_then(|viewer| viewer.interface.contract.as_ref())
                .map(|contract| contract.version),
            Some(2_u32),
            "complete v2 facts justify empty own-city accounting sums"
        );
    }
}

#[test]
fn motion_probe_captures_real_window_at_an_injected_animation_phase() -> Result<(), String> {
    let _guard = crate::sdl_test_lock::lock_sdl_for_test()?;
    let environment = HeadlessSdlEnvironment::apply();
    let result = (|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let initial = snapshot::render_preview_scene(
            &root,
            (800, 600),
            None,
            snapshot::scene::PreviewScene::Motion {
                direction: 7,
                quarter: 0,
            },
        )
        .map_err(|error| error.to_string())?;
        let middle = snapshot::render_preview_scene(
            &root,
            (800, 600),
            None,
            snapshot::scene::PreviewScene::Motion {
                direction: 7,
                quarter: 2,
            },
        )
        .map_err(|error| error.to_string())?;
        let repeated = snapshot::render_preview_scene(
            &root,
            (800, 600),
            None,
            snapshot::scene::PreviewScene::Motion {
                direction: 7,
                quarter: 2,
            },
        )
        .map_err(|error| error.to_string())?;
        if initial == middle {
            return Err("movement probe did not advance actual window pixels".to_owned());
        }
        if middle != repeated {
            return Err("injected phase is not deterministic".to_owned());
        }
        Ok(())
    })();
    environment.restore();
    result
}

#[test]
fn populated_manager_diagnostics_use_explicit_offline_projection_fixtures() {
    let state = snapshot::scene::preview_state(snapshot::scene::PreviewScene::PopulatedManager {
        kind: ManagerKind::Science,
        chart: false,
    });
    let viewer = state.viewer.as_ref().expect("explicit fixture viewer");
    assert_eq!(
        viewer
            .interface
            .research
            .as_ref()
            .expect("fixture research")
            .target_id,
        "ADVANCE_BALLISTICS",
        "populated research is labeled fixture data, never invented in the empty preview"
    );
    assert_eq!(
        viewer.cities.len(),
        1_usize,
        "a deterministic selectable city"
    );
    assert_eq!(
        viewer.cities[0].facts["queue"].as_array().map(Vec::len),
        Some(2_usize),
        "populated queues must use the actual public queue field, not an invented alias"
    );
    assert!(
        viewer.interface.rules.is_some(),
        "descriptions use runtime metadata"
    );
    assert!(
        snapshot::scene::preview_state(snapshot::scene::PreviewScene::Manager(
            ManagerKind::Science
        ))
        .viewer
        .is_none(),
        "existing empty diagnostics must remain empty"
    );
}

#[test]
fn city_chooser_diagnostic_has_explicit_two_city_projection_without_changing_other_fixtures() {
    let state = snapshot::scene::preview_state(snapshot::scene::PreviewScene::CityChooser);
    assert_eq!(
        state.viewer.as_ref().expect("viewer").cities.len(),
        2_usize,
        "popup diagnostic has explicit synthetic choices"
    );
    let normal = snapshot::scene::preview_state(snapshot::scene::PreviewScene::PopulatedManager {
        kind: ManagerKind::Build,
        chart: false,
    });
    assert_eq!(
        normal.viewer.as_ref().expect("viewer").cities.len(),
        1_usize,
        "do not silently alter existing preview contexts"
    );
}

#[test]
fn populated_empire_fixture_explicitly_discloses_policy_values_without_changing_existing_scenes() {
    let scene = snapshot::scene::PreviewScene::PopulatedManager {
        kind: ManagerKind::Empire,
        chart: false,
    };
    let state = snapshot::scene::preview_state(scene);
    let viewer = state.viewer.as_ref().expect("explicit public fixture");
    assert_eq!(
        viewer.diplomacy.as_ref().expect("policies").facts["science_percent"],
        serde_json::json!(50_i64),
        "explicit current policy, not a live request or inferred recommendation"
    );
    let original =
        snapshot::scene::preview_state(snapshot::scene::PreviewScene::PopulatedManager {
            kind: ManagerKind::Build,
            chart: false,
        });
    assert!(
        original
            .viewer
            .as_ref()
            .expect("original projection")
            .diplomacy
            .as_ref()
            .expect("original diplomacy")
            .facts
            .get("science_percent")
            .is_none(),
        "existing diagnostic fixtures remain unchanged"
    );
}

#[test]
fn populated_chart_scene_renders_projected_research_borders_with_native_source_fonts()
-> Result<(), String> {
    let _guard = crate::sdl_test_lock::lock_sdl_for_test()?;
    let environment = HeadlessSdlEnvironment::apply();
    let result = (|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let chart = snapshot::render_preview_scene(
            &root,
            (800_u32, 600_u32),
            None,
            snapshot::scene::PreviewScene::PopulatedManager {
                kind: ManagerKind::Library,
                chart: true,
            },
        )
        .map_err(|error| error.to_string())?;
        let offset = 348_usize
            .saturating_mul(800_usize)
            .saturating_add(313_usize)
            .saturating_mul(4_usize);
        if chart.rgba[offset..offset.saturating_add(3_usize)] != [0_u8, 255_u8, 0_u8] {
            return Err(
                "native chart did not render the known Agriculture research-state border"
                    .to_owned(),
            );
        }
        Ok(())
    })();
    environment.restore();
    result
}

#[test]
fn deterministic_snapshots_use_actual_renderer_and_requested_output_size() -> Result<(), String> {
    let _guard = crate::sdl_test_lock::lock_sdl_for_test()?;
    let environment = HeadlessSdlEnvironment::apply();
    let result = (|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let frame = snapshot::render_preview(&root, (800_u32, 600_u32), None)
            .map_err(|error| error.to_string())?;
        if (frame.width, frame.height, frame.rgba.len()) != (800_u32, 600_u32, 1_920_000_usize) {
            return Err("native capture must be lossless 800×600 RGBA".to_owned());
        }
        let empty = (100_usize * 800_usize + 700_usize) * 4_usize;
        if frame.rgba[empty..empty.saturating_add(3_usize)] != [0_u8; 3] {
            return Err(
                "undiscovered map must be black, without an upper-right debug panel".to_owned(),
            );
        }
        let dropdown =
            snapshot::render_preview(&root, (640_u32, 480_u32), Some(MenuCategory::Options))
                .map_err(|error| error.to_string())?;
        if dropdown.rgba.len() != 1_228_800_usize || !dropdown.rgba.iter().any(|byte| *byte != 0_u8)
        {
            return Err("scaled snapshot must contain a real 640×480 frame".to_owned());
        }
        Ok(())
    })();
    environment.restore();
    result
}
