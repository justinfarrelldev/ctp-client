//! Center-tile observed artwork: fog neighbors never become known dry cells.
use crate::gameplay::{assets::tile::TileSet,model::{GameState,MapPoint,FogState},server::observations};

/* Released ctp2_source/ctp2_code/gfx/tilesys/tiledmap.cpp::TryRiver and
 * PostProcessTile select SW/NW/NE/SE plus beach-mouth art.
 * gs/world/UnseenCell.cpp::UnseenCell(MapPoint)/UnseenCell(UnseenCell*) copies
 * TileInfo and Serialize retains it, not today's hidden neighboring geography.
 * Server docs/server-contract-repairs.md::Observed river geometry projects only
 * that center artwork. Bits map SW->3,NW->1,NE->5,SE->7 in the tile-table order.
 * No neighboring entity/terrain/good is inferred. Missing geometry is unknown,
 * not an isolated/source pool. Local non-authoritative fixture fallback alone
 * retains the old complete local neighborhood; no scaling/art substitution.
 */
/// Select the installed original overlay from received observation, not guessed edges.
pub(super) fn piece(tiles:&TileSet,state:&GameState,point:MapPoint,index:&super::observed::ObservedTiles)->Option<usize> {
    let tile=state.tile(point).filter(|tile|tile.fog!=FogState::Hidden)?;
    if !tile.river {return None;}
    if let Some(viewer)=state.viewer.as_ref() {
        let source=index.get(viewer,point)?;
        let geometry=observations::river(viewer,source)?;
        return tiles.river_piece(geometry.neighborhood(),geometry.beach);
    }
    if state.authoritative {return None;}
    let neighbors=super::NEIGHBOR_OFFSETS.map(|(dx,dy)|point.offset(dx,dy,state.width,state.height).and_then(|neighbor|state.tile(neighbor)).is_some_and(|neighbor|neighbor.river&&neighbor.fog!=FogState::Hidden));
    tiles.river_piece(neighbors,tile.art_base_type==Some(13_u8))
}

#[cfg(test)]
mod tests {
    use super::piece;
    use crate::gameplay::{assets::tile::TileSet,model::{GameState,MapPoint},server::ServerProjection};
    use serde_json::json;
    use std::path::Path;
    #[test]
    fn repaired_river_all_masks_use_received_edges_at_frontier_and_under_fog_without_neighbors() {
        let tiles=TileSet::load_from_installation(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2")).expect("read-only original art");
        for mask in 0_u8..16_u8 {for stale in [false,true] {for beach in [false,true] {
            if beach&&mask==0_u8 {continue;}
            let viewer:ServerProjection=serde_json::from_value(json!({"revision":9_i64,"turn":3_u32,"contract":{"version":2_u32,"features":{"observed_river_geometry":true}},"tiles":[{"id":"center","x":2_u16,"y":2_u16,"terrain":"plains","river":true,"stale":stale,"river_geometry":{"version":"ctp2-river-edges-v1","edges":mask,"beach":beach,"observed_revision":7_i64}}]})).expect("observed center only");
            let state=GameState::from_server_projection(&viewer,1_u8,1_u64);let mut lookup=super::super::observed::ObservedTiles::default();lookup.update(&viewer);let index=piece(&tiles,&state,MapPoint::new(2_u16,2_u16),&lookup).expect("all installed river shapes");let pattern=tiles.rivers[index].pattern;
            for (bit,edge) in [3_usize,1_usize,5_usize,7_usize].into_iter().enumerate(){assert_eq!(pattern[edge]!=0_i16,mask&(1_u8<<bit)!=0_u8,"received original edge; mask{mask}, remembered={stale}");}
            assert_eq!(pattern[4_usize],if beach {2_i16}else{1_i16},"original distinct beach-mouth discriminator");
            assert_eq!(viewer.tiles.len(),1_usize,"no hidden neighbor backfill");
        }}}
    }
    #[test]
    fn repaired_river_missing_legacy_geometry_is_not_zero_connectivity_pool() {
        let tiles=TileSet::load_from_installation(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2")).expect("read-only art");
        let viewer:ServerProjection=serde_json::from_value(json!({"revision":9_i64,"turn":3_u32,"contract":{"version":2_u32,"features":{"observed_river_geometry":true}},"tiles":[{"id":"center","x":2_u16,"y":2_u16,"terrain":"plains","river":true,"stale":true}]})).expect("legacy observation");
        let state=GameState::from_server_projection(&viewer,1_u8,1_u64);
        let mut lookup=super::super::observed::ObservedTiles::default();lookup.update(&viewer);
        assert_eq!(piece(&tiles,&state,MapPoint::new(2_u16,2_u16),&lookup),None,"unknown edges cannot render a fabricated pool");
    }
}
