//! Independent national territory and city influence from observed server assignments.
use crate::gameplay::{GameplayError,runtime_error,model::{GameState,MapPoint},runtime::{Camera,textures::GameplayTextures}};
use sdl2::{pixels::Color,rect::Rect,render::Canvas,video::Window};

/// Original four diamond sides mapped to the server's documented cardinal topology.
#[derive(Clone,Copy)]
enum Side {
    /// Server positive X, lower right diamond side.
    SouthEast,
    /// Server positive Y, lower left diamond side.
    SouthWest,
    /// Server negative X, upper left diamond side.
    NorthWest,
    /// Server negative Y, upper right diamond side.
    NorthEast,
}
impl Side {
    /// Preserve the existing server-cardinal to screen-diamond neighbor mapping.
    const fn delta(self)->(i16,i16) {match self {Self::SouthEast=>(1_i16,0_i16),Self::SouthWest=>(0_i16,1_i16),Self::NorthWest=>(-1_i16,0_i16),Self::NorthEast=>(0_i16,-1_i16)}}
    /// Northern sides occupy the first24 rows below tile headroom.
    const fn north(self)->bool {matches!(self,Self::NorthWest|Self::NorthEast)}
    /// Western sides paint inside the left hit-mask edge.
    const fn west(self)->bool {matches!(self,Self::NorthWest|Self::SouthWest)}
}
/* Released ctp2_source/ctp2_code/gfx/tilesys/tiledraw.cpp::TiledMap::
 * DrawNationalBorders/GetVisibleCellOwner compare national owners for solid
 * GetPlayerColor edges, then CityOwner assignments for dashed COLOR_WHITE
 * edges under independent ProfileDB flags. City influence requires the city's
 * visibility bit, not selection/settler preview. These are runtime map pixels,
 * no LDL node: graphics.ldl::{PoliticalBordersButton,CityInflenceButton} owns
 * only their switches. Server docs/city-development-contract.md supplies exact
 * observed owner_id/influence_city_id, empty versus unknown, fog memory and
 * cardinal topology; do NOT copy native radius/nearest-city logic here.
 * Original GetVisibleCityOwner reads canonical hidden state: intentionally use
 * only received assignments and currently visible city/center observations.
 * Explicit projected national ownership is display permission, not a fabricated
 * native HasSeen/diplomatic-contact flag. Known influence can stand alone even
 * when a legacy observation lacks national ownership; the two toggles differ.
 * Missing/null neighbors are unknown, not neutral. Remembered assignments may
 * be rendered, but cannot fetch/expose a hidden city's current state.
 */
/// Draw both permitted systems; no local ownership, influence or diplomatic inference.
pub(super) fn draw(canvas:&mut Canvas<Window>,textures:&GameplayTextures<'_>,state:&GameState,camera:&Camera,point:MapPoint)->Result<(),GameplayError> {
    let Some(viewer)=state.viewer.as_ref() else{return Ok(());};
    let Some(tile)=textures.observed_tile(viewer,point) else{return Ok(());};
    let owner=tile.owner_id.as_deref().filter(|owner|!owner.is_empty());
    let city=tile.influence_city_id.as_deref().filter(|city|!city.is_empty()&&textures.influence_city_visible(city));
    for side in [Side::SouthEast,Side::SouthWest,Side::NorthWest,Side::NorthEast] {
        let (dx,dy)=side.delta();
        let Some(neighbor)=point.offset(dx,dy,state.width,state.height).and_then(|point|textures.observed_tile(viewer,point))else{continue;};
        if state.filters.political_borders {
            if let Some(owner)=owner.filter(|owner|neighbor.owner_id.as_deref().is_some_and(|other|other!=*owner)) {
                edge(canvas,camera,point,side,textures.observed_owner_color(state,owner),false)?;
            }
        }
        if state.filters.city_influence&&city.is_some_and(|city|neighbor.influence_city_id.as_deref().is_some_and(|other|city!=other)) {
            edge(canvas,camera,point,side,Color::RGB(255_u8,255_u8,255_u8),true)?;
        }
    }
    Ok(())
}
/* Released ctp2_source/ctp2_code/gfx/tilesys/tiledmap.cpp::GenerateHitMask
 * and tiledraw.cpp::DrawColoredBorderEdge:94x72 tile,24 headroom,48 ground
 * rows; upper mask starts46/47 and expands2 pixels each row, lower reverses.
 * Paint THREE pixels inward; dashed mode suppresses row&2 (two on/two off).
 * Use disjoint24-row halves instead of the original post-increment overlap.
 * Fix original east '*pDestPixel = 0; selectColorPixel;' typo (all3 pixels
 * must carry the chosen color) and whole-tile/offscreen/raw-surface rejection:
 * SDL clips safe rectangles, so partly visible borders remain functional.
 * Existing integer zoom is preserved; this is not a scaling/parity-fitting pass.
 */
/// Exact native hit-mask row raster, safely clipped and multiplied by current zoom.
fn edge(canvas:&mut Canvas<Window>,camera:&Camera,point:MapPoint,side:Side,color:Color,dashed:bool)->Result<(),GameplayError> {
    let (x,y)=camera.tile_to_screen(point);let zoom=camera.tile_width().div_euclid(94_u32).max(1_u32);let scale=i32::try_from(zoom).unwrap_or(1_i32);
    canvas.set_draw_color(color);
    for row in 0_i32..48_i32 {
        if side.north()!=(row<24_i32)||(dashed&&row&2_i32!=0_i32){continue;}
        let inset=if row<24_i32 {46_i32.saturating_sub(row.saturating_mul(2_i32))}else{row.saturating_sub(24_i32).saturating_mul(2_i32)};
        let column=if side.west(){inset}else{91_i32.saturating_sub(inset)};
        canvas.fill_rect(Rect::new(x.saturating_add(column.saturating_mul(scale)),y.saturating_add(24_i32.saturating_add(row).saturating_mul(scale)),3_u32.saturating_mul(zoom),zoom)).map_err(runtime_error)?;
    }
    Ok(())
}
