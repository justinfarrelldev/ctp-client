//! Accessible National navigation preserves selected-city/return identity.
use super::tests::{authority, snapshot, ui};
use crate::gameplay::runtime::authority::state_from_authority_projection;
use crate::gameplay::ui_model::management::{ManagerIntent, ManagerKind};
use serde_json::json;
#[test]
fn source_national_build_city_and_close_cycle_keeps_the_exact_second_city_without_a_session() {
    let mut source = snapshot();
    source["cities"].as_array_mut().expect("own cities").push(json!({"id":"second-city","owner_id":"account-7","tile_id":"b","population":2_u32,"name":"Athens"}));
    let authenticated = authority(9_u16, source);
    let mut state = state_from_authority_projection(
        &authenticated,
        authenticated
            .session
            .projection
            .as_ref()
            .expect("authenticated viewer"),
    );
    state.popups.clear();
    let mut model = ui();
    model.management.open(ManagerKind::National, &state, None);
    super::apply(
        ManagerIntent::Select("city".to_owned(), "second-city".to_owned()),
        &mut state,
        &mut model,
        None,
    );
    for (button, kind) in [
        ("BuildQueueButton", ManagerKind::Build),
        ("CityManagerButton", ManagerKind::City),
    ] {
        let open = model
            .management
            .activate_id(button, &state)
            .expect("exact selected-city source navigation");
        super::apply(open, &mut state, &mut model, None);
        let view = model
            .management
            .view
            .as_ref()
            .expect("related source manager");
        assert_eq!(view.kind, kind, "native destination");
        assert_eq!(
            view.city_id.as_deref(),
            Some("second-city"),
            "not the first owned city"
        );
        let close = model
            .management
            .widgets(&state)
            .into_iter()
            .filter_map(|widget| widget.intent)
            .find(|intent| *intent == ManagerIntent::Close)
            .expect("visible accessible Close control");
        super::apply(close, &mut state, &mut model, None);
        let parent = model
            .management
            .view
            .as_ref()
            .expect("source parent restored");
        assert_eq!(
            parent.kind,
            ManagerKind::National,
            "related Close returns, not exits game"
        );
        assert_eq!(
            parent.selections.get("city").map(String::as_str),
            Some("second-city"),
            "independent parent selection survives"
        );
    }
    assert_eq!(
        state
            .viewer
            .as_ref()
            .expect("unchanged canonical viewer")
            .revision,
        17_i64,
        "navigation sends no command or viewer mutation"
    );
}
