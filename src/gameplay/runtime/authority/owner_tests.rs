//! Public display membership and exact observed owner IDs survive every receipt.
use super::{RuntimeAuthority,state_from_authority_projection,replace_state_from_projection};
use crate::gameplay::{model::{CityId,PlayerId},server::{ServerClient,ServerConfig,ServerGameplaySession,ServerProjection,roster::PlayerRoster}};
use serde_json::json;

#[test]
fn nation_borders_receipts_keep_persisted_colors_and_filters_after_last_unit_and_reject_stale_identity() {
    let (mut authority,mut viewer)=fixture();
    viewer.interface.viewer_player_id=Some(json!("account-7"));viewer.interface.players=Some(json!([{"id":"account-43","nation_id":"n43","color_index":1_u8},{"id":"account-7","nation_id":"n7","color_index":4_u8},{"id":"account-42","nation_id":"n42","color_index":0_u8}]));
    authority.session.apply_projection(viewer.clone());let mut state=state_from_authority_projection(&authority,&viewer);state.filters.political_borders=false;state.filters.city_influence=false;
    assert_eq!(state.roster.slot("account-7"),Some(5_u8),"persisted slot supersedes setup position2");
    let old=viewer.clone();viewer.revision=8_i64;viewer.units.clear();viewer.interface.players.as_mut().and_then(serde_json::Value::as_array_mut).expect("public identities").reverse();replace_state_from_projection(&authority,&mut state,&viewer);authority.session.apply_projection(viewer.clone());
    assert_eq!(state.roster.slot("account-7"),Some(5_u8),"last settler consumption/reordered roster cannot turn self into foreign or change color");assert!(!state.filters.political_borders&&!state.filters.city_influence,"received state preserves independent local display settings");
    assert_eq!(state.cities.iter().find(|city|city.id==CityId::from_server_id("c7")).expect("founded city").owner,PlayerId::Human,"self identity does not depend on units");
    authority.session.apply_projection(old.clone());replace_state_from_projection(&authority,&mut state,&old);assert_eq!(state.viewer.as_ref().expect("current").revision,8_i64,"old founding receipt cannot roll back assignments or identity");assert_eq!(authority.session.roster.slot("account-7"),Some(5_u8),"session retains published static identities on stale replay");
    viewer.revision=9_i64;viewer.interface.players=Some(json!([{"id":"account-7","nation_id":"n7"}]));replace_state_from_projection(&authority,&mut state,&viewer);assert_eq!(state.roster.slot("account-7"),None,"explicit legacy-save unknown index cannot revive stale/setup colors");
}

#[test]
fn nation_borders_explicit_viewer_identity_is_opaque_not_rederived_from_account_or_units() {
    let (mut authority,mut viewer)=fixture();viewer.interface.viewer_player_id=Some(json!("opaque-viewer"));viewer.interface.players=Some(json!([{"id":"opaque-viewer","nation_id":"nation-x","color_index":3_u8}]));
    for unit in &mut viewer.units {if unit.owner_id=="account-7"{"opaque-viewer".clone_into(&mut unit.owner_id);}}
    for city in &mut viewer.cities {if city.owner_id=="account-7"{"opaque-viewer".clone_into(&mut city.owner_id);}}
    authority.session.apply_projection(viewer.clone());let state=state_from_authority_projection(&authority,&viewer);assert_eq!(state.roster.viewer(),Some("opaque-viewer"),"authenticated response defines the exact viewer ID");
    let own=state.units.iter().find(|unit|unit.owner==PlayerId::Human).expect("exact explicit self");assert_eq!(super::server_unit_id_for_unit(&authority,own.id),"u7","wire selection uses the same opaque viewer identity");assert_eq!(PlayerId::Human.color_slot(&state.roster),Some(4_u8),"explicit persisted color, not inferred account order");
}

/// Deliberately put self second, use unrelated account suffixes, and co-locate cities.
fn fixture()->(RuntimeAuthority,ServerProjection) {
    let roster=PlayerRoster::from_game(&json!({"id":44_i64,"players":[{"id":90_i64,"account_id":42_i64},{"id":91_i64,"account_id":7_i64},{"id":92_i64,"account_id":43_i64}]}),44_i64,7_i64).expect("public accepted roster");
    let viewer:ServerProjection=serde_json::from_value(json!({"revision":7_i64,"turn":1_u32,"tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"plains"}],"units":[{"id":"u42","owner_id":"account-42","tile_id":"a"},{"id":"u7","owner_id":"account-7","tile_id":"a"},{"id":"u43","owner_id":"account-43","tile_id":"a"}],"cities":[{"id":"c42","owner_id":"account-42","tile_id":"a"},{"id":"c7","owner_id":"account-7","tile_id":"a"},{"id":"c43","owner_id":"account-43","tile_id":"a"}]})).expect("observations");
    let authority=RuntimeAuthority {client:ServerClient::from_config(&ServerConfig {base_url:"http://127.0.0.1:1".to_owned(),required:true}).expect("unused endpoint"),session:ServerGameplaySession {game_id:44_i64,session_id:71_i64,account_id:7_i64,projection:Some(viewer.clone()),roster},empire_count:3_u8,seed:1_u64,io:None};(authority,viewer)
}
#[test]
fn player_colors_authenticated_mapping_uses_exact_city_and_player_ids_not_coordinates_or_one_foreign_bucket() {
    let (authority,viewer)=fixture();let state=state_from_authority_projection(&authority,&viewer);
    assert_eq!(state.units[0_usize].owner,PlayerId::Server("account-42".to_owned()),"exact first foreign identity");assert_eq!(state.units[1_usize].owner,PlayerId::Human,"exact authenticated self");assert_eq!(state.units[2_usize].owner,PlayerId::Server("account-43".to_owned()),"exact other foreign identity");
    for (id,owner,slot) in [("c42",PlayerId::Server("account-42".to_owned()),1_u8),("c7",PlayerId::Human,2_u8),("c43",PlayerId::Server("account-43".to_owned()),3_u8)] {
        let city=state.cities.iter().find(|city|city.id==CityId::from_server_id(id)).expect("exact city");assert_eq!(city.owner,owner,"co-located observations cannot rebind another city's owner");assert_eq!(city.owner.color_slot(&state.roster),Some(slot),"public display slot independent of self/foreign role");
    }
}
#[test]
fn player_colors_snapshot_reorder_hidden_departure_reveal_and_capture_never_reassign_existing_display_slots() {
    let (authority,mut viewer)=fixture();let mut state=state_from_authority_projection(&authority,&viewer);let roster=state.roster.clone();
    viewer.units.reverse();viewer.cities.reverse();viewer.revision=8_i64;viewer.units.retain(|unit|unit.owner_id!="account-43");viewer.cities.retain(|city|city.owner_id!="account-43");replace_state_from_projection(&authority,&mut state,&viewer);
    assert_eq!(state.roster,roster,"hidden departure does not free/reassign presentation slots");
    let (_,mut revealed)=fixture();revealed.revision=9_i64;revealed.units.reverse();revealed.cities.reverse();"account-43".clone_into(&mut revealed.cities.iter_mut().find(|city|city.id=="c42").expect("captured city").owner_id);replace_state_from_projection(&authority,&mut state,&revealed);
    assert_eq!(state.roster,roster,"reveal/order/capture cannot rotate colors");
    let captured=state.cities.iter().find(|city|city.id==CityId::from_server_id("c42")).expect("city");assert_eq!(captured.owner,PlayerId::Server("account-43".to_owned()),"actual received capture changes owner, not an adjacent identity");assert_eq!(captured.owner.color_slot(&state.roster),Some(3_u8),"new owner's original display palette");
}
