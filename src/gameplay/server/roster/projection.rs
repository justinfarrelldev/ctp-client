//! Repaired server public identity metadata, never encounter/account-order colors.
use super::PlayerRoster;
use crate::gameplay::server::interface::ServerInterface;
use alloc::collections::BTreeSet;
use serde_json::Value;

/// Bound opaque identifiers without renaming/normalizing or deriving country identities.
fn id(value:&Value)->Option<&str> {value.as_str().filter(|id|!id.is_empty()&&id.len()<=256_usize)}
/* Released ctp2_source/ctp2_code/gfx/gfx_utils/ColorSet.cpp::GetPlayerColor
 * indexes installed Colors00.txt by player number; slot0 is barbarian,1..16
 * are participants. Server docs/city-development-contract.md and
 * internal/gameplay/visibility/player_identity.go define color_index as a
 * persisted ZERO-based participant index, not that native barbarian slot.
 * Add1 exactly once, never modulo/hash/reorder; absent/null legacy-save colors
 * stay unknown even for self. Presence of malformed new metadata is NOT an
 * invitation to use the old public setup-order assignment. Identity/slots
 * reveal no city names/positions/private knowledge and authorize no commands.
 */
/// None means wholly legacy schema; Some(empty) means explicitly unavailable metadata.
pub(super) fn read(interface:&ServerInterface)->Option<PlayerRoster> {
    if interface.viewer_player_id.is_none()&&interface.players.is_none(){return None;}
    let mut roster=PlayerRoster {authoritative:true,viewer:interface.viewer_player_id.as_ref().and_then(id).map(str::to_owned),..PlayerRoster::default()};
    let Some(players)=interface.players.as_ref().and_then(Value::as_array).filter(|rows|!rows.is_empty()&&rows.len()<=16_usize)else{return Some(roster);};
    let mut ids=BTreeSet::new();let mut colors=BTreeSet::new();
    for player in players {
        let Some(owner)=id(&player["id"]).filter(|_|id(&player["nation_id"]).is_some())else{roster.slots.clear();return Some(roster);};
        if !ids.insert(owner){roster.slots.clear();return Some(roster);}
        if let Some(index)=player.get("color_index").filter(|index|!index.is_null()) {
            let Some(slot)=index.as_u64().and_then(|index|u8::try_from(index).ok()).filter(|index|*index<16_u8).and_then(|index|index.checked_add(1_u8)).filter(|slot|colors.insert(*slot)) else {roster.slots.clear();return Some(roster);};
            roster.slots.insert(owner.to_owned(),slot);
        }
    }
    if roster.viewer.as_deref().is_none_or(|viewer|!ids.contains(viewer)){roster.slots.clear();}
    Some(roster)
}
