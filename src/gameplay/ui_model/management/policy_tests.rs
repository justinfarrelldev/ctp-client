//! Government recommendations are not exhaustive scalar values; schemas stay strict.

use super::{ManagementUi, ManagerCommand, ManagerIntent, ManagerKind, policy};
use crate::gameplay::assets::layout::LayoutBook;
use crate::gameplay::model::GameState;
use crate::gameplay::server::ServerProjection;
use serde_json::{Value, json};
use std::path::Path;

/// Current government actions advertise field schemas and sample recommendations.
fn fixture() -> GameState {
    let projection:ServerProjection=serde_json::from_value(json!({"revision":12_i64,"turn":4_u32,"diplomacy":{"science_percent":50_i64,"public_works_percent":10_i64,"workday":1_i64,"wages":-1_i64,"rations":0_i64},"legal_actions":[{"route":"set-social-policy","entity_id":"human","fields":["workday","wages","rations"],"payload":{"workday":0_i64,"wages":0_i64,"rations":0_i64}},{"route":"set-science-rate","entity_id":"human","fields":["science_percent"],"payload":{"science_percent":100_i64}},{"route":"set-public-works-rate","entity_id":"human","fields":["public_works_percent"],"payload":{"public_works_percent":20_i64}}]})).expect("current public schema");
    GameState::from_server_projection(&projection, 1_u8, 1_u64)
}
/// An explicit proposed operation, not a mutation or counterfactual evaluation.
fn command(route: &str, payload: Value) -> ManagerCommand {
    ManagerCommand {
        revision: 12_i64,
        route: route.to_owned(),
        payload,
    }
}

#[test]
fn science_limits_require_exact_government_identity_and_cannot_fabricate_current_policy() {
    let mut state = fixture();
    let path = "TabGroup.Tab1.TabPanel.Commerce.Modify.Secondary.Spinner";
    state.viewer.as_mut().expect("viewer").interface.rules =
        Some(json!({"governments":[{"max_science_percent":5_i64}]}));
    assert_eq!(
        policy::spec(&state, path)
            .expect("parameter domain")
            .maximum,
        100_i32,
        "two absent IDs never identify a government"
    );
    let viewer = state.viewer.as_mut().expect("viewer");
    viewer.diplomacy.as_mut().expect("policy").facts["government_id"] = json!("GOVERNMENT_TEST");
    viewer.interface.rules =
        Some(json!({"governments":[{"id":"GOVERNMENT_TEST","max_science_percent":40_i64}]}));
    assert_eq!(
        policy::spec(&state, path)
            .expect("government range")
            .current,
        50_i32,
        "a government receipt can change the cap without rewriting its stored science policy; never pretend a local clamp was received"
    );
    state.viewer.as_mut().expect("viewer").interface.rules =
        Some(json!({"governments":[{"id":"GOVERNMENT_TEST","max_science_percent":"unknown"}]}));
    assert!(
        policy::spec(&state, path).is_none(),
        "malformed disclosed caps are not a valid unrestricted default"
    );
}

#[test]
fn verified_scalar_policy_requests_are_not_limited_to_gateway_recommendation_examples() {
    let state = fixture();
    for operation in [
        command("set-science-rate", json!({"science_percent":55_i64})),
        command(
            "set-public-works-rate",
            json!({"public_works_percent":11_i64}),
        ),
        command(
            "set-social-policy",
            json!({"workday":-1_i64,"wages":-1_i64,"rations":0_i64}),
        ),
    ] {
        assert!(
            operation.valid_for(&state),
            "government.go accepts this typed, bounded explicit request; visibility.go sample payloads are not exhaustive"
        );
    }
}

#[test]
fn source_policy_slider_indices_preserve_companions_and_reverse_workday_before_comparing() {
    let state = fixture();
    let workday = policy::command(
        &state,
        "TabGroup.Tab1.TabPanel.Production.Modify.Primary.Slider",
        3_i32,
    )
    .expect("source reverse workday request");
    assert_eq!(
        workday.payload,
        json!({"workday":-1_i64,"wages":-1_i64,"rations":0_i64}),
        "compare converted workday level, not pre-inversion index-minus-two from the original buggy callback"
    );
    assert!(workday.valid_for(&state), "typed source policy request");
    let rations = policy::command(
        &state,
        "TabGroup.Tab1.TabPanel.Food.Modify.Primary.Slider",
        4_i32,
    )
    .expect("source rations request");
    assert_eq!(
        rations.payload,
        json!({"workday":1_i64,"wages":-1_i64,"rations":2_i64}),
        "changing one slider preserves both other actual current values, not gateway recommendation defaults"
    );
    assert!(
        policy::command(
            &state,
            "TabGroup.Tab1.TabPanel.Food.Modify.Primary.Slider",
            5_i32
        )
        .is_none(),
        "original source exposes only five levels"
    );
    assert!(
        policy::command(
            &state,
            "TabGroup.Tab1.TabPanel.Production.Modify.Primary.Slider",
            1_i32
        )
        .is_none(),
        "source index one is current workday+1 and must not redundantly mutate live state"
    );
    assert!(
        policy::command(&state, "../../set-social-policy", 2_i32).is_none(),
        "only exact original control paths can form policy requests"
    );
}

#[test]
fn native_policy_controls_have_source_steps_actual_thumb_and_increment_payloads() {
    let state = fixture();
    let mut ui = ManagementUi::new(
        LayoutBook::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2")).expect("source"),
    )
    .expect("layouts");
    ui.open(ManagerKind::Empire, &state, None);
    let path = "TabGroup.Tab1.TabPanel.Production.Modify.Primary.Slider";
    let widgets = ui.widgets(&state);
    let step = widgets
        .iter()
        .find(|widget| widget.id == format!("{path}/value/3"))
        .expect("quantized source value");
    assert_eq!(
        step.intent,
        Some(ManagerIntent::SetValue(path.to_owned(), 3_i32, 12_i64)),
        "native hit and accessible value use the same path/index"
    );
    let thumb = widgets
        .iter()
        .find(|widget| widget.id == format!("{path}/thumb"))
        .expect("original thumb");
    assert_eq!(
        thumb.node.field("image0"),
        Some("upba2512.tga"),
        "original C3Slider thumb artwork"
    );
    assert_eq!(thumb.rect.width, 9_u32, "source thumb size");
    let science = "TabGroup.Tab1.TabPanel.Commerce.Modify.Secondary.Spinner";
    let increment = widgets
        .iter()
        .find(|widget| widget.id == format!("{science}/increase"))
        .expect("source spinner callback");
    assert_eq!(
        increment.intent,
        Some(ManagerIntent::SetValue(science.to_owned(), 60_i32, 12_i64)),
        "source step increases the actual current fifty percent, not an advertised hundred-percent recommendation"
    );
}

#[test]
fn malformed_advertised_parameter_schema_cannot_grant_new_scalar_permissions() {
    for (fields, payload) in [
        (
            vec!["science_percent", "unknown"],
            json!({"science_percent":100_i64}),
        ),
        (vec!["science_percent"], json!({"science_percent":"bad"})),
        (
            vec!["science_percent"],
            json!({"science_percent":100_i64,"unknown":true}),
        ),
    ] {
        let mut state = fixture();
        let action = state
            .viewer
            .as_mut()
            .expect("viewer")
            .interface
            .legal_actions
            .iter_mut()
            .find(|action| action.route == "set-science-rate")
            .expect("capability");
        action.fields = fields.into_iter().map(str::to_owned).collect();
        action.payload = payload;
        assert!(
            !command("set-science-rate", json!({"science_percent":55_i64})).valid_for(&state),
            "parameterization requires the entire verified current route/field/sample schema"
        );
    }
}

#[test]
fn parameter_changes_cannot_escape_current_revision_advertised_fields_or_engine_bounds() {
    let mut state = fixture();
    for operation in [
        command("set-science-rate", json!({"science_percent":101_i64})),
        command("set-science-rate", json!({"science_percent":-1_i64})),
        command(
            "set-science-rate",
            json!({"science_percent":50_i64,"city_id":"other"}),
        ),
        command(
            "set-public-works-rate",
            json!({"public_works_percent":"50"}),
        ),
        command(
            "set-social-policy",
            json!({"workday":11_i64,"wages":0_i64,"rations":0_i64}),
        ),
        command("set-social-policy", json!({"workday":0_i64})),
    ] {
        assert!(
            !operation.valid_for(&state),
            "malformed/out-of-range/extra-field payload rejected"
        );
    }
    let operation = command("set-science-rate", json!({"science_percent":55_i64}));
    state.viewer.as_mut().expect("viewer").revision = 13_i64;
    assert!(
        !operation.valid_for(&state),
        "receipt invalidates displayed parameter intent"
    );
    state.viewer.as_mut().expect("viewer").revision = 12_i64;
    state
        .viewer
        .as_mut()
        .expect("viewer")
        .interface
        .legal_actions
        .clear();
    assert!(
        !operation.valid_for(&state),
        "known route is not permission to act without a current viewer capability"
    );
}
