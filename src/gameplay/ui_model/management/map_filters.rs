//! Original local border switches; display-only, never server commands or file effects.
use super::{ManagementUi,ManagerIntent,ManagerKind};
use crate::{config::ctp2_options::Ctp2GraphicsOptions,gameplay::{assets::layout::{LayoutBook,LayoutNode},model::GameState}};

/* Released ctp2_source/ctp2_code/ui/interface/graphicsscreen.cpp::
 * graphicsscreen_Initialize wires PoliticalBordersButton/CityInflenceButton,
 * seeds states from ProfileDB, and graphicsscreen_checkPress maps their PRESS
 * callbacks (inverted old state) to independent profile flags. Original LDL:
 * ctp2/ctp2_data/english/uidata/layouts/graphics.ldl::{GRAPHICS_CHECKBOX,
 * GraphicsWindow.PoliticalBordersButton,GraphicsWindow.CityInflenceButton};
 * upbt11u0/upbt11u1 and each stringtable's0/1 represent OFF/ON.
 * TiledMap::DrawNationalBorders consumes these flags independently. Preserve
 * current source/reference-variant placement/scaling; fix only actual state,
 * callback, label and checkmark. Native/accessible use the same descriptors.
 * Client difference: session-local changes survive snapshots/Close; existing
 * shell settings initialize gameplay, but general in-game profile persistence
 * is not implemented here. No file write or canonical gameplay command occurs.
 */
/// Exact source path to the presentation-only semantic selection namespace.
fn key(path:&str)->Option<&'static str> {match path {"PoliticalBordersButton"=>Some("map-filter:political"),"CityInflenceButton"=>Some("map-filter:influence"),_=>None}}
/// Source option state, independent of current units, turn phase or server capability.
fn enabled(state:&GameState,path:&str)->Option<bool> {match path {"PoliticalBordersButton"=>Some(state.filters.political_borders),"CityInflenceButton"=>Some(state.filters.city_influence),_=>None}}
/// Only the actual Graphics checkbox owns a local activation, not decorative children.
pub(super) fn binding(kind:ManagerKind,state:&GameState,path:&str)->Option<ManagerIntent> {
    (kind==ManagerKind::Graphics).then_some(())?;Some(ManagerIntent::Select(key(path)?.to_owned(),(!enabled(state,path)?).to_string()))
}
/// Namespace guard keeps display callbacks out of generic row selection.
#[must_use]
pub fn recognizes(key:&str)->bool {matches!(key,"map-filter:political"|"map-filter:influence")}
/// Apply an explicit source switch state only while Graphics owns the control.
#[must_use]
pub fn apply(ui:&ManagementUi,state:&mut GameState,key:&str,value:&str)->bool {
    if ui.view.as_ref().is_none_or(|view|view.kind!=ManagerKind::Graphics){return false;}
    let Ok(value)=value.parse::<bool>()else{return false;};
    match key {"map-filter:political"=>state.filters.political_borders=value,"map-filter:influence"=>state.filters.city_influence=value,_=>return false}true
}
/// Return matching label/check state while preserving inherited control geometry.
pub(super) fn style(kind:ManagerKind,state:&GameState,path:&str,node:&mut LayoutNode,book:&LayoutBook)->Option<(bool,String)> {
    (kind==ManagerKind::Graphics).then_some(())?;let on=enabled(state,path)?;
    node.fields.insert("image0".to_owned(),if on {"upbt11u1.tga"}else{"upbt11u0.tga"}.to_owned());
    let text=match (path,on) {("PoliticalBordersButton",true)=>"str_ldl_ShowPoliticalBorders_On",("PoliticalBordersButton",false)=>"str_ldl_ShowPoliticalBorders_Off",("CityInflenceButton",true)=>"str_ldl_Show_City_Inflence_On",("CityInflenceButton",false)=>"str_ldl_Show_City_Inflence_Off",_=>return None};
    Some((on,book.label(text).to_owned()))
}
/// Carry existing shell choices into both server-backed and explicit preview startup.
pub(in crate::gameplay) const fn initialize(state:&mut GameState,graphics:&Ctp2GraphicsOptions) {state.filters.political_borders=graphics.political_borders.enabled();state.filters.city_influence=graphics.city_influence.enabled();}
