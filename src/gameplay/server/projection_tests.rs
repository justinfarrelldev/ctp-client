//! Contract regressions for server resync wrappers and Go nil-slice encoding.

use super::test_support::{starting_projection_json, starting_resync_event};
use super::{ServerCommandResponse, ServerProjection};
use serde_json::{Value, json};

#[test]
fn real_resync_envelope_retains_spawned_settler_and_visible_area() {
    let projection = ServerProjection::from_resync_event(&starting_resync_event())
        .expect("real server resync envelope must unwrap snapshot");
    assert_eq!(
        projection.revision, 1_i64,
        "revision comes from snapshot, not a default"
    );
    assert_eq!(projection.turn, 1_u32, "initial turn is retained");
    assert_eq!(
        projection.tiles.len(),
        13_usize,
        "server starter visibility survives decoding"
    );
    assert_eq!(
        projection.units.len(),
        1_usize,
        "actual server settler survives decoding"
    );
    assert_eq!(
        projection.units[0_usize].id, "settler-7-1",
        "spawn identity retained"
    );
    assert_eq!(
        projection.units[0_usize].tile_id, "tile-41-27",
        "spawn tile retained"
    );
    assert_eq!(
        projection.units[0_usize].owner_id, "account-7",
        "authenticated owner retained"
    );
    assert!(
        projection.cities.is_empty(),
        "nil Go city slice means no founded cities yet"
    );
}

#[test]
fn go_nil_lists_and_explicit_empty_lists_are_equivalent_for_resync_and_receipts() {
    for lists in [Value::Null, json!([])] {
        let projection: ServerProjection = serde_json::from_value(json!({"revision": 9_i64, "turn": 3_u32, "tiles": lists, "units": lists, "cities": lists})).expect("Go null/empty viewer slices must decode");
        assert!(
            projection.tiles.is_empty()
                && projection.units.is_empty()
                && projection.cities.is_empty(),
            "no invented data for nil/empty slices"
        );
    }
}

#[test]
fn invalid_resync_envelopes_never_silently_become_empty_games() {
    for data in [
        json!({}),
        json!({"revision": 1_i64, "turn": 1_u32, "tiles": [], "units": [], "cities": []}),
        json!({"current_revision": 1_i64}),
        json!({"current_revision": 1_i64, "snapshot": null}),
        json!({"current_revision": 1_i64, "snapshot": {}}),
        json!({"current_revision": 1_i64, "snapshot": {"turn": 1_u32}}),
        json!({"current_revision": 1_i64, "snapshot": {"revision": 1_i64}}),
        json!({"current_revision": 2_i64, "snapshot": starting_projection_json()}),
        json!({"current_revision": 1_i64, "snapshot": {"revision": 1_i64, "turn": 0_u32}}),
        json!({"current_revision": -1_i64, "snapshot": {"revision": -1_i64, "turn": 1_u32}}),
        json!({"current_revision": 1_i64, "snapshot": {"revision": 1_i64, "turn": 1_u32, "units": "not a list"}}),
    ] {
        assert!(
            ServerProjection::from_resync_event(&format!("event: resync\ndata: {data}\n\n"))
                .is_err(),
            "invalid envelope must not launch an empty map: {data}"
        );
    }
    for body in [
        "",
        "event: other\ndata: {}\n\n",
        "event: resync\ndata: not-json\n\n",
    ] {
        assert!(
            ServerProjection::from_resync_event(body).is_err(),
            "missing/malformed resync must fail clearly"
        );
    }
}

#[test]
fn sse_framing_ignores_unrelated_events_and_accepts_standard_field_spacing() {
    let data = json!({"current_revision": 1_i64, "snapshot": starting_projection_json()});
    let pretty = serde_json::to_string_pretty(&data).expect("fixture serialization");
    let multiline = pretty.lines().fold(String::new(), |mut framed, line| {
        framed.push_str("data: ");
        framed.push_str(line);
        framed.push('\n');
        framed
    });
    for body in [
        format!(": comment\n\nevent: other\ndata: {{}}\n\nevent: resync\ndata: {data}\n\n"),
        format!("event:resync\ndata:{data}\n\n"),
        format!("event: resync\r\ndata: {data}\r\n\r\n"),
        format!("event: resync\n{multiline}\n"),
        format!("event: resync\ndata: {data}"),
    ] {
        let projection =
            ServerProjection::from_resync_event(&body).expect("standard SSE framing decodes");
        assert_eq!(
            projection.units.len(),
            1_usize,
            "framing must not discard the server starter"
        );
    }
}

#[test]
fn command_receipts_share_nullable_lists_but_not_the_sse_wrapper() {
    let response: ServerCommandResponse = serde_json::from_value(json!({"revision":1_i64,"receipt_id":"receipt-1","viewer_projection":starting_projection_json()})).expect("direct viewer projection accepts Go nil cities");
    assert_eq!(
        response
            .viewer_projection
            .expect("viewer projection")
            .units
            .len(),
        1_usize,
        "commands also retain server units"
    );
    assert!(
        serde_json::from_value::<ServerProjection>(json!({})).is_err(),
        "missing metadata never silently defaults during any wire decode"
    );
}

#[test]
fn valid_empty_viewer_snapshot_remains_empty_not_a_local_fallback() {
    let data = json!({"current_revision": 9_i64, "snapshot": {"revision": 9_i64, "turn": 3_u32, "tiles": null, "units": null, "cities": null}});
    let projection =
        ServerProjection::from_resync_event(&format!("event: resync\ndata: {data}\n\n"))
            .expect("legitimate empty viewer snapshot is valid");
    assert_eq!(
        projection.revision, 9_i64,
        "empty is not invalid when metadata is correct"
    );
    assert!(
        projection.tiles.is_empty() && projection.units.is_empty(),
        "no local starting army/geography substituted"
    );
}
