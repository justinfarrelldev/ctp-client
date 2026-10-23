//! Released queue-first description priority and explicit non-forced queue deselection.
use super::super::{ManagerKind,ManagerView,ManagerWidget,queue};
use crate::gameplay::{assets::layout::LayoutNode,model::GameState,ui_model::MenuRect};
/*
 * Released ctp2_source/ctp2_code/ui/interface/EditQueue.cpp::{constructor,
 * ListCallback,QueueListCallback,ShowSelectedInfo,SelectChoiceList,Library} and
 * citywindow.cpp::CityWindow::{SetItemDescription,PopulateQueueList}. Queue
 * GetSelectedItem wins; only no selected queue permits visible left choice.
 * ui/aui_common/aui_listbox.cpp::{GetSelectedItem,MouseLGrabInside,DeselectItem,
 * SelectItem} does not consult keyboard focus; blank space clears non-force
 * queue selection. Left lists are forceSelect in source, queue is not.
 * ui/aui_common/aui_control.cpp::SetKeyboardFocus/ReleaseKeyboardFocus does not
 * clear list selection. ui/aui_ctp2/ctp2_listbox.cpp::Draw paints every visual
 * selected row RGB255/179/69, then redraws item text (not focus-only coloring).
 * editqueue.ldl::{ItemsBox lists,QueueGroup.QueueList,ItemDescription,ItemImage,
 * LibraryButton,eq_QueueListItem,eq_UnitItem,eq_BuildingWonderItem} owns geometry.
 * Reference04 alone does not prove focus clears queue: source PopulateQueueList
 * rebuilds clear it, and users can explicitly deselect blank pane. Preserve
 * client's immutable receipt-safe selections and no forced first/neighbor target.
 * Fix source Custom ShowSelectedInfo's inner EditItemInfo *info shadow (outer
 * pointer stayed null) with exact typed local content, not a blank/bogus article.
 * No private buildability/ETA, local city execution or scaling/raster claims.
 */
/// Mirrors released priority while keeping entry and content namespaces distinct.
pub(in crate::gameplay::ui_model::management) fn sync(view:&mut ManagerView){
    if view.kind!=ManagerKind::Build{return;}
    let role=if view.selections.contains_key("queue"){Some("queue")}else if view.selections.contains_key("buildable"){Some("buildable")}else{None};
    if let Some(role)=role {
        if let Some(id)=view.selections.get(role).cloned(){view.selections.insert("description".to_owned(),id);view.selections.insert("description-source".to_owned(),role.to_owned());}
    }else if view.selections.remove("description-source").is_some(){view.selections.remove("description");}
}
/// Missing typed entry is unavailable, never matching another entry's content ID.
pub(in crate::gameplay::ui_model::management) fn selected<'view>(view:&'view ManagerView,state:&'view GameState)->Option<&'view str>{
    if let Some(entry)=view.selections.get("queue"){return queue::content(state,view,entry);}
    if let Some(id)=view.selections.get("buildable"){return Some(id);}
    // Legacy read-only fixture/article views have no typed role; never grant edits.
    if view.selections.contains_key("description-source"){return None;}
    let id=view.selections.get("description")?;Some(queue::content(state,view,id).unwrap_or(id))
}
/// Explicit source empty-pane deselection is scoped to the exact prior selection.
pub(in crate::gameplay::ui_model::management) fn deselect(view:&mut ManagerView,entry:&str)->bool{
    if view.kind!=ManagerKind::Build||view.selections.get("queue").is_none_or(|current|current!=entry){return false;}
    view.selections.remove("queue");sync(view);true
}
/// Semantic blank source pane, not a new painted control or a header/frame hit.
pub(in crate::gameplay::ui_model::management) fn blank(view:&ManagerView,path:&str,rect:MenuRect,used:u32)->Option<ManagerWidget>{
    if view.kind!=ManagerKind::Build||path!="QueueGroup.QueueList"||used>=rect.height{return None;}
    let entry=view.selections.get("queue")?;
    Some(ManagerWidget {id:format!("{path}.EmptyPane"),label:String::new(),rect:MenuRect {x:rect.x,y:rect.y.saturating_add(i32::try_from(used).unwrap_or(i32::MAX)),width:rect.width,height:rect.height.saturating_sub(used)},node:LayoutNode::default(),intent:Some(super::super::ManagerIntent::Select("queue-deselect".to_owned(),entry.clone())),unavailable:None,selected:false})
}
/// Source Custom has no city ETA: `AddChoiceItem(-1)` and `UpdateQueueList` blank it.
pub(in crate::gameplay::ui_model::management) fn time(view:&ManagerView,name:&str)->Option<String>{
    (super::super::saved::custom::active(view)&&matches!(name,"Time"|"Turns")).then(String::new)
}
