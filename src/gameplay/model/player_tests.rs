//! Exact server owner identity must survive rendering conversion without color-role guesses.
use super::{GameState,PlayerId};
use crate::gameplay::server::ServerProjection;
use serde_json::json;

#[test]
fn player_colors_projection_preserves_distinct_foreign_owners_and_does_not_guess_self_from_suffixes() {
    let viewer:ServerProjection=serde_json::from_value(json!({"revision":7_i64,"turn":1_u32,"tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"plains"}],"units":[{"id":"u1","owner_id":"account-42","tile_id":"a"},{"id":"u2","owner_id":"account-43","tile_id":"a"},{"id":"u3","owner_id":"account-41","tile_id":"a"},{"id":"u4","owner_id":"not-human","tile_id":"a"},{"id":"u5","owner_id":"barbarian-club","tile_id":"a"}],"cities":[{"id":"c1","owner_id":"account-42","tile_id":"a"},{"id":"c2","owner_id":"account-43","tile_id":"a"}]})).expect("observed owners");
    let state=GameState::from_server_projection(&viewer,3_u8,1_u64);
    assert_ne!(state.units[0_usize].owner,state.units[1_usize].owner,"different foreign identities cannot collapse into one empire");
    assert_ne!(state.units[2_usize].owner,PlayerId::Human,"account ending1 is not authenticated self");
    assert_ne!(state.units[3_usize].owner,PlayerId::Human,"human substring is not authentication");
    assert_ne!(state.units[4_usize].owner,PlayerId::Barbarians,"barbarian substring is not a player kind");
    assert_eq!(state.units[0_usize].owner,state.cities[0_usize].owner,"same exact owner for unit and city");assert_ne!(state.cities[0_usize].owner,state.cities[1_usize].owner,"co-located observations remain distinct by city identity");
}
