//! Public committed/observed display facts, never a lookup into hidden authority.
use super::{ServerProjection,ServerTileProjection};
use serde::Deserialize;

/// Artwork geometry observed on the center tile, not discovered neighboring cells.
#[derive(Debug,Clone,Deserialize,PartialEq,Eq)]
pub struct RiverGeometry {
    /// Pinned four-edge interpretation.
    pub version:String,
    /// Original SW/NW/NE/SE bits, in that order.
    pub edges:u8,
    /// Beach-mouth artwork discriminator.
    pub beach:bool,
    /// Last actual observation, not the current hidden-world revision.
    pub observed_revision:i64,
}
impl RiverGeometry {
    /// Original tile-table order: W,NW,N,SW,center,NE,S,SE,E.
    #[must_use]
    pub fn neighborhood(&self)->[bool;9] {
        let mut result=[false;9_usize];result[4_usize]=true;
        for (bit,index) in [3_usize,1_usize,5_usize,7_usize].into_iter().enumerate(){result[index]=self.edges&(1_u8<<bit)!=0_u8;}
        result
    }
}
/// Validated received geometry; unknown/malformed/legacy is never an isolated pool.
#[must_use]
pub(in crate::gameplay) fn river(viewer:&ServerProjection,tile:&ServerTileProjection)->Option<RiverGeometry> {
    if !viewer.interface.supports("observed_river_geometry")||tile.river!=Some(true){return None;}
    let geometry:RiverGeometry=serde_json::from_value(tile.facts.get("river_geometry")?.clone()).ok()?;
    (geometry.version=="ctp2-river-edges-v1"&&geometry.edges<16_u8&&geometry.observed_revision>=0_i64&&geometry.observed_revision<=viewer.revision).then_some(geometry)
}
/// Committed scheduler identity, not real-time resolution progress or a nation color.
#[derive(Debug,Clone,Deserialize,PartialEq,Eq)]
pub struct TurnContext {
    /// Versioned interpretation.
    pub version:String,
    /// Snapshot binding.
    pub revision:i64,
    /// Committed turn binding.
    pub turn:u32,
    /// Server mode.
    pub mode:String,
    /// Acting/planning/conflict/resolving/ended/unavailable.
    pub status:String,
    /// Public scheduler actor only in sequential/long-turn state.
    pub active_player_id:Option<String>,
    /// Explicit false: no continuous progress stream. Missing is unknown.
    pub live_progress:Option<bool>,
}
/* Released ui/interface/TurnYearStatus.cpp::DrawDougsProgress and
 * controlpanel.ldl::ControlPanel.DougsProgressBar require alive-player order and
 * native player color. The new committed context supplies neither color nor a
 * live fraction. These footer sentences explicitly replace unavailable display
 * knowledge, not original source wording or invented hybrid actor progress.
 */
/// Human-readable public scheduler state; never invent private conflict participants.
#[must_use]
pub(in crate::gameplay) fn turn_label(viewer:&ServerProjection)->String {
    let Some(context)=viewer.interface.turn_context.as_ref().filter(|context|viewer.interface.supports("turn_context")&&context.version=="committed-turn-context-v1"&&context.revision==viewer.revision&&context.turn==viewer.turn&&context.live_progress==Some(false)) else {return "Turn context unavailable".to_owned();};
    match context.status.as_str() {
        "acting" if matches!(context.mode.as_str(),"sequential"|"long_turn")=>context.active_player_id.as_deref().filter(|id|!id.is_empty()&&id.len()<=128_usize&&!id.chars().any(char::is_control)).map_or_else(||"Turn context unavailable".to_owned(),|id|format!("Acting player (ID): {id}; committed state, no live progress")),
        "planning"=>"Waiting for planning; no live resolution progress".to_owned(),
        "conflict"=>"Waiting for conflict resolution; participants unavailable".to_owned(),
        "resolving"=>"Resolving; live progress unavailable".to_owned(),
        "ended"=>"Game ended".to_owned(),
        _=>"Turn context unavailable".to_owned(),
    }
}
/* Original ui/interface/controlpanelwindow.cpp::StatsMenuCallback/BuildStatsMenu
 * opens infowindow.cpp::InfoWindow::{SelectRankingTab,SelectScoreTab,SelectWonderTab};
 * controlpanel.ldl::MainMenu.StatsActivator owns the menu and unchanged geometry.
 * Pin/generator text is an explicit client diagnostics addition in the existing
 * status area, not an original game option or permission to regenerate a world.
 */
/// Safe pinned provenance for explicit Info inspection; no inference from IDs/version.
#[must_use]
pub(in crate::gameplay) fn provenance(viewer:&ServerProjection)->String {
    let rules=viewer.interface.rules.as_ref();
    let family=rules.and_then(|rules|rules["family"].as_str()).filter(|family|matches!(*family,"retail"|"synthetic")).unwrap_or("unknown");
    let hash=rules.and_then(|rules|rules["content_hash"].as_str()).filter(|hash|hash.len()<=128_usize&&!hash.chars().any(char::is_control)).unwrap_or("unavailable");
    let world=viewer.interface.world.as_ref();
    let safe=|text:Option<&str>|text.filter(|value|value.len()<=128_usize&&value.bytes().all(|byte|byte.is_ascii_alphanumeric()||matches!(byte,b'-'|b'_'))).unwrap_or("unavailable").to_owned();
    format!("Rules: {family}; pin: {hash}; generator: {}; resources: {}",safe(world.and_then(|world|world.generator_version.as_deref())),safe(world.and_then(|world|world.resource_catalog.as_deref())))
}
