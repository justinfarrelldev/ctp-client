//! Additive repaired contracts: metadata is bounded, observed and never hidden authority.
use super::{ServerProjection, stream::{Change,Invalidations}};
use crate::gameplay::model::GameState;
use serde_json::json;

#[test]
fn repaired_stream_resync_id_is_durable_cursor_not_revision_even_when_snapshot_is_large() {
    let event=format!("id: 900\nevent: resync\ndata: {{\"current_revision\":7,\"event_cursor\":900,\"snapshot\":{{\"padding\":\"{}\"}}}}\n\n","x".repeat(600_000_usize));
    let wire=format!("HTTP/1.1 200 OK\r\n\r\n{event}");
    for size in [1_usize,97_usize,8192_usize] {
        let mut parser=Invalidations::default();let mut changes=Vec::new();
        for chunk in wire.as_bytes().chunks(size){changes.extend(parser.push(chunk).expect("bounded fragmented metadata"));}
        assert_eq!(changes,vec![Change{cursor:Some(900_u64),revision:Some(7_i64)}],"SSE id/resync current revision are independent");
    }
}

#[test]
fn repaired_turn_context_and_provenance_remain_revision_bound_and_do_not_invent_colors_or_progress() {
    let mut viewer:ServerProjection=serde_json::from_value(json!({"revision":7_i64,"turn":3_u32,"contract":{"version":2_u32,"features":{"turn_context":true}},"turn_context":{"version":"committed-turn-context-v1","revision":7_i64,"turn":3_u32,"mode":"sequential","status":"acting","active_player_id":"account-8","live_progress":false},"rules":{"family":"retail","content_hash":"pin"},"world":{"generator_version":"ctp2-original-goods-v4","resource_catalog":"ctp2-original-resources-v1"}})).expect("current contract");
    assert_eq!(super::observations::turn_label(&viewer),"Acting player (ID): account-8; committed state, no live progress","no guessed nation/color/progress fraction");
    assert!(super::observations::provenance(&viewer).contains("retail")&&super::observations::provenance(&viewer).contains("ctp2-original-resources-v1"),"retain received provenance");
    viewer.revision=8_i64;assert_eq!(super::observations::turn_label(&viewer),"Turn context unavailable","stale metadata cannot identify the current actor");
    let state=GameState::from_server_projection(&viewer,1_u8,1_u64);assert_eq!(state.viewer.as_ref().expect("viewer").interface.world.as_ref().expect("world").resource_catalog.as_deref(),Some("ctp2-original-resources-v1"),"keep provenance through render conversion");
}

#[test]
fn repaired_hybrid_context_never_discloses_or_invents_an_acting_private_participant() {
    let viewer:ServerProjection=serde_json::from_value(json!({"revision":7_i64,"turn":3_u32,"contract":{"version":2_u32,"features":{"turn_context":true}},"turn_context":{"version":"committed-turn-context-v1","revision":7_i64,"turn":3_u32,"mode":"hybrid","status":"conflict","active_player_id":"hidden-participant","live_progress":false}})).expect("context");
    let label=super::observations::turn_label(&viewer);assert!(label.contains("participants unavailable")&&!label.contains("hidden-participant"),"committed conflict is not public acting-nation progress");
    assert!(super::observations::provenance(&viewer).contains("Rules: unknown"),"missing family is not guessed retail or synthetic");
}

#[test]
fn repaired_names_and_transformed_goods_keep_received_identity_without_client_migration() {
    let viewer:ServerProjection=serde_json::from_value(json!({"revision":7_i64,"turn":3_u32,"tiles":[{"id":"a","x":0_u16,"y":0_u16,"terrain":"TERRAIN_PLAINS","good_id":"TERRAIN_DESERT_GOOD_TWO"},{"id":"b","x":1_u16,"y":0_u16,"terrain":"TERRAIN_DESERT"}],"cities":[{"id":"technical-one","owner_id":"human","tile_id":"a","name":"City 1"},{"id":"technical-two","owner_id":"human","tile_id":"b","name":"City 1"}]})).expect("received display facts");
    let state=GameState::from_server_projection(&viewer,1_u8,1_u64);
    assert_eq!(state.cities[0_usize].name,"City 1","authoritative name, not identity-derived text");assert_eq!(state.cities[1_usize].name,"City 1","duplicate display names are permitted");assert_ne!(state.cities[0_usize].id,state.cities[1_usize].id,"display names never identify cities");
    assert_eq!(state.tile(crate::gameplay::model::MapPoint::new(0_u16,0_u16)).expect("observed terrain").good_id.as_deref(),Some("TERRAIN_DESERT_GOOD_TWO"),"later terrain changes may retain deposits; never delete or relocate locally");
}

#[test]
fn repaired_geometry_rejects_future_versions_masks_revisions_and_unknown_absence() {
    let mut viewer:ServerProjection=serde_json::from_value(json!({"revision":9_i64,"turn":3_u32,"contract":{"version":2_u32,"features":{"observed_river_geometry":true}},"tiles":[{"id":"a","river":true,"river_geometry":{"version":"ctp2-river-edges-v1","edges":10_u8,"beach":true,"observed_revision":7_i64}}]})).expect("geometry");
    assert!(super::observations::river(&viewer,&viewer.tiles[0_usize]).is_some_and(|geometry|geometry.beach&&geometry.neighborhood()[1_usize]&&geometry.neighborhood()[7_usize]),"beach edges use original NW/SE bits");
    for (field,value) in [("version",json!("future")),("edges",json!(16_u8)),("observed_revision",json!(10_i64)),("beach",json!(null))] {
        let mut changed=viewer.clone();changed.tiles[0_usize].facts["river_geometry"][field]=value;assert!(super::observations::river(&changed,&changed.tiles[0_usize]).is_none(),"unknown {field} never invents geometry");
    }
    viewer.tiles[0_usize].river=None;assert!(super::observations::river(&viewer,&viewer.tiles[0_usize]).is_none(),"missing river presence remains unknown");
}

#[test]
fn repaired_timing_headers_are_bounded_public_metrics_not_raw_response_headers() {
    let response=super::parse_http_response(b"HTTP/1.1 200 OK\r\nServer-Timing: snapshot;dur=12.500\r\nX-Gameplay-Read-Mode: current\r\nAuthorization: private\r\n\r\n{}").expect("framing");
    assert_eq!(response.telemetry.server_timing.as_deref(),Some("snapshot;dur=12.500"),"retain server work separate from client time");
    assert_eq!(response.telemetry.read_mode.as_deref(),Some("current"),"routine current replication is not stream/fault recovery");
    let malformed=super::parse_http_response(b"HTTP/1.1 200 OK\r\nServer-Timing: private;dur=session\r\nX-Gameplay-Read-Mode: credentials\r\n\r\n{}").expect("ordinary response still parsed");
    assert!(malformed.telemetry.server_timing.is_none()&&malformed.telemetry.read_mode.is_none(),"unknown raw headers cannot enter diagnostics");
}

#[test]
fn repaired_legacy_resync_never_turns_ambiguous_id_into_a_durable_cursor_or_revision_hint() {
    let wire=b"HTTP/1.1 200 OK\r\n\r\nid: 201\nevent: resync\ndata: garbage\n\n";
    assert_eq!(Invalidations::default().push(wire).expect("safe fallback"),vec![Change{cursor:None,revision:None}],"ambiguous old/malformed metadata requests one safe current read");
}
