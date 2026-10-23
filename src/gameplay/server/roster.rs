//! Frozen public setup membership used only for client presentation, never fog/authority.
use alloc::collections::{BTreeMap,BTreeSet};
use serde_json::Value;

/// Stable base-game display slots for the already-public accepted setup roster.
#[derive(Debug,Clone,Default,PartialEq,Eq)]
pub struct PlayerRoster {
    /// Exact canonical account owner IDs; no hashing, country or encounter ordering.
    slots:BTreeMap<String,u8>,
    /// Exact authenticated viewer identity, independent of its display slot.
    viewer:Option<String>,
}
impl PlayerRoster {
    /* Released ctp2_source/ctp2_code/gfx/gfx_utils/ColorSet.cpp::GetPlayerColor
     * indexes Colors00 by player number, NOT civilization name or diplomacy.
     * $CTPSERVER/internal/game/player_endpoints.go::startGame returns Game.Players
     * (models.go) via helpers.go::{getGameWithPlayers,listGamePlayersByGameID},
     * whose gamePlayersSelectByGameID query publishes joined_at/id
     * membership order in the authenticated start_game response. The server has
     * no native color-slot field: freeze that public order as CLIENT display
     * slots1..16, reserving0 for barbarians, as an explicit integration rule.
     * Do not claim these are canonical scheduler slots, derive colors from account
     * numbers or sightings, or obtain geography/private knowledge from membership.
     */
    /// Complete bounded setup response; malformed/foreign-game/missing roster is unknown.
    #[must_use]
    pub fn from_game(game:&Value,game_id:i64,account_id:i64)->Option<Self> {
        if game["id"].as_i64()!=Some(game_id)||game_id<=0_i64||account_id<=0_i64{return None;}
        let players=game["players"].as_array().filter(|players|!players.is_empty()&&players.len()<=16_usize)?;
        let mut result=Self::default();let mut memberships=BTreeSet::new();
        for (index,player) in players.iter().enumerate() {
            let member=player["id"].as_i64().filter(|id|*id>0_i64)?;
            let account=player["account_id"].as_i64().filter(|id|*id>0_i64)?;
            if !memberships.insert(member){return None;}
            let owner=format!("account-{account}");
            if result.slots.insert(owner.clone(),u8::try_from(index.checked_add(1_usize)?).ok()?).is_some(){return None;}
            if account==account_id {result.viewer=Some(owner);}
        }
        result.viewer.as_ref()?;Some(result)
    }
    /// Preserve membership only for this exact authenticated viewer; otherwise colors stay unknown.
    #[must_use]
    pub fn for_account(&self,account_id:i64)->Self {
        if account_id<=0_i64 {return Self::default();}
        let viewer=format!("account-{account_id}");
        if self.viewer.as_ref()==Some(&viewer) {self.clone()}else{Self {slots:BTreeMap::new(),viewer:Some(viewer)}}
    }
    /// Exact public membership's display slot; unknown owners never inherit a neighbor.
    #[must_use]
    pub fn slot(&self,owner:&str)->Option<u8>{self.slots.get(owner).copied()}
    /// Authenticated identity retained even when own units/cities are absent.
    #[must_use]
    pub fn viewer(&self)->Option<&str>{self.viewer.as_deref()}
}

#[cfg(test)]
mod tests {
    use super::PlayerRoster;
    use serde_json::json;
    #[test]
    fn player_colors_public_roster_freezes_membership_order_not_account_numbers_or_countries() {
        let game=json!({"id":44_i64,"players":[{"id":90_i64,"account_id":41_i64,"country":"Greeks"},{"id":101_i64,"account_id":7_i64,"country":"Romans"},{"id":120_i64,"account_id":999_i64,"country":"Greeks"}]});
        let roster=PlayerRoster::from_game(&game,44_i64,7_i64).expect("complete public membership");
        assert_eq!((roster.slot("account-41"),roster.slot("account-7"),roster.slot("account-999")),(Some(1_u8),Some(2_u8),Some(3_u8)),"public slot order, not ID suffix/modulo or fixed civilization colors");
        assert_eq!(roster.for_account(7_i64),roster,"same authenticated account preserves display slots");assert_eq!(roster.for_account(999_i64).slot("account-41"),None,"another authenticated viewer cannot inherit cached membership");assert_eq!(PlayerRoster::default().for_account(7_i64).viewer(),Some("account-7"),"self remains exact even when colors are unavailable");assert_eq!(roster.for_account(0_i64),PlayerRoster::default(),"invalid authentication has no identity");
        assert_eq!(roster.viewer(),Some("account-7"),"viewer need not be slot1");assert_eq!(roster.slot("account-8"),None,"unobserved membership is not guessed");
    }
    #[test]
    fn player_colors_missing_duplicate_overflow_and_foreign_game_rosters_stay_unavailable() {
        for game in [json!({"id":44_i64}),json!({"id":45_i64,"players":[{"id":1_i64,"account_id":7_i64}]}),json!({"id":44_i64,"players":[{"id":1_i64,"account_id":8_i64}]}),json!({"id":44_i64,"players":[{"id":1_i64,"account_id":7_i64},{"id":2_i64,"account_id":7_i64}]}),json!({"id":44_i64,"players":[{"id":1_i64,"account_id":7_i64},{"id":1_i64,"account_id":8_i64}]}),json!({"id":44_i64,"players":[{"id":1_i64,"account_id":null}]})] {
            assert!(PlayerRoster::from_game(&game,44_i64,7_i64).is_none(),"invalid metadata grants no color identity");
        }
        let players:Vec<_>=(1_i64..=17_i64).map(|id|json!({"id":id,"account_id":id})).collect();assert!(PlayerRoster::from_game(&json!({"id":44_i64,"players":players}),44_i64,7_i64).is_none(),"no modulo color reuse beyond the original16 civilization slots");
    }
}
