//! Exact owner identity is independent of authentication, display color and civilization name.
use super::PlayerId;
use crate::gameplay::server::roster::PlayerRoster;

impl PlayerId {
    /// Returns whether this owner is a normal empire, including an exact foreign ID.
    #[must_use]
    pub const fn is_empire(&self)->bool {matches!(*self,Self::Human|Self::Empire(_)|Self::Server(_))}
    /// Returns whether this owner is explicitly barbarian, not an ID substring.
    #[must_use]
    pub const fn is_barbarian(&self)->bool {matches!(*self,Self::Barbarians)}
    /// Legacy fileless fixtures name their human explicitly; account IDs are never guessed.
    #[must_use]
    pub(in crate::gameplay) fn from_observed(owner:&str)->Self {
        if matches!(owner,"human"|"player-1") {Self::Human}else{Self::foreign(owner)}
    }
    /// Production self is only the exact authenticated owner, never an ID substring/suffix.
    #[must_use]
    pub(in crate::gameplay) fn authenticated(owner:&str,viewer:&str)->Self {
        if !viewer.is_empty()&&owner==viewer {Self::Human}else{Self::foreign(owner)}
    }
    /// Preserve opaque identity; only explicit legacy barbarian owner IDs imply that role.
    fn foreign(owner:&str)->Self {
        if matches!(owner,"barbarian"|"barbarians"){Self::Barbarians}else{Self::Server(owner.to_owned())}
    }
    /* Released ctp2_source/ctp2_code/gfx/gfx_utils/ColorSet.cpp::GetPlayerColor
     * uses native player index.
     * Repaired sessions use persisted participant indices (server/roster.rs);
     * only wholly legacy projections keep frozen public setup-order slots.
     * No guessed country/encounter/hash colors; source barbarian0 is reserved.
     * Offline fixture Empire(1) is first AI/slot2; legacy self-only blue fallback
     * remains slot1 only for legacy schemas, not missing repaired server slots.
     * Unknown participant colors
     * remain unavailable rather than masquerading as another player's palette row.
     */
    /// Borrowed territorial lookup avoids allocating owner strings for every painted tile.
    #[must_use]
    pub(in crate::gameplay) fn observed_color_slot(owner:&str,roster:&PlayerRoster)->Option<u8> {
        roster.slot(owner).or_else(|| {
            if matches!(owner,"barbarian"|"barbarians") {Some(0_u8)}else if !roster.uses_server_slots()&&(roster.viewer()==Some(owner)||(roster.viewer().is_none()&&matches!(owner,"human"|"player-1"))){Some(1_u8)}else{None}
        })
    }
    /// Exact original palette slot, without authorizing actions or hidden information.
    #[must_use]
    #[expect(clippy::pattern_type_mismatch,reason="Borrow exact opaque owner strings without cloning retained identities")]
    pub(in crate::gameplay) fn color_slot(&self,roster:&PlayerRoster)->Option<u8> {
        match self {
            Self::Human=>roster.viewer().and_then(|id|roster.slot(id)).or_else(||(!roster.uses_server_slots()).then_some(1_u8)),
            Self::Empire(index)=>index.checked_add(1_u8).filter(|slot|*slot<=16_u8),
            Self::Server(id)=>roster.slot(id),
            Self::Barbarians=>Some(0_u8),
        }
    }
}
