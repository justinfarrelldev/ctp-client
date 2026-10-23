//! Original question art/extent is rendered in memory, never saved as a snapshot.
use crate::gameplay::{assets::{PICTURE_ARCHIVE, layout::LayoutBook, rim::load_rim_rgba_entry}, model::GameState, runtime::{fonts::RenderFonts, test_support::with_canvas, textures::GameplayTextures}, server::ServerProjection, ui_model::{InGameUiModel, management::{ManagementUi, ManagerIntent, ManagerKind}}};
use sdl2::pixels::{Color, PixelFormatEnum};
use serde_json::json;
use std::path::Path;

/*
 * Source: ctp2_source/ctp2_code/ui/interface/MessageBoxDialog.cpp::constructor;
 * ctp2/ctp2_data/english/uidata/layouts/messagebox.ldl::MessageBoxDialog and
 * DialogBackground use centered320x174, actual uptg29a..i, not NameTheCityPopup's
 * input frame. This checks every outside pixel plus independent original corner
 * ink. It is a component regression, not acceptance of any full reference frame.
 */
#[test]
fn build_clear_question_native_source_art_stays_inside_original_bounds_over_visible_build_manager() {
    let result = with_canvas((800_u32,600_u32), |canvas| {
        let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
        let creator=canvas.texture_creator();
        let mut textures=GameplayTextures::load(&root,&creator,true).map_err(|error| error.to_string())?;
        let projection:ServerProjection=serde_json::from_value(json!({"revision":7_i64,"turn":1_u32,"tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"plains"}],"cities":[{"id":"c2","owner_id":"human","tile_id":"a","name":"Rome","queue":[{"entry_id":"c2:1","id":"UNIT_HOPLITE"}],"rush_buy_quote":{"revision":7_i64,"status":"available","cost":30_i64}}],"legal_actions":[{"route":"edit-production","entity_id":"c2","payload":{"city_id":"c2","entries":[]}}]})).map_err(|error|error.to_string())?;
        let state=GameState::from_server_projection(&projection,1_u8,1_u64);
        let mut ui=InGameUiModel { management:ManagementUi::new(LayoutBook::load(&root).map_err(|error|error.to_string())?).map_err(|error|error.to_string())?, ..InGameUiModel::default() };
        ui.management.open(ManagerKind::Build,&state,Some("c2"));
        let dialog=match ui.management.activate_id("QueueGroup.ClearButton",&state) { Some(ManagerIntent::Dialog(dialog))=>Some(*dialog), _=>None }.ok_or_else(|| "source clear query absent".to_owned())?;
        let bounds=dialog.confirmation.first().ok_or_else(||"source root absent".to_owned())?.rect;
        canvas.set_draw_color(Color::RGB(17_u8,31_u8,47_u8));canvas.clear();
        super::management::draw_management(canvas,&mut textures,&state,&ui,None).map_err(|error|error.to_string())?;
        let before=canvas.read_pixels(None,PixelFormatEnum::RGBA32)?;
        ui.operation=Some(dialog);
        super::operations::draw(canvas,&mut textures,&state,&ui,RenderFonts {body:None,management:None}).map_err(|error|error.to_string())?;
        let after=canvas.read_pixels(None,PixelFormatEnum::RGBA32)?;
        let mut changed=0_usize;
        for y in 0_u32..600_u32 { for x in 0_u32..800_u32 {
            let index=usize::try_from(y).map_err(|error|error.to_string())?.saturating_mul(800_usize).saturating_add(usize::try_from(x).map_err(|error|error.to_string())?).saturating_mul(4_usize);
            let rect_x=i32::try_from(x).map_err(|error|error.to_string())?;let rect_y=i32::try_from(y).map_err(|error|error.to_string())?;
            if before[index..index.saturating_add(3_usize)] != after[index..index.saturating_add(3_usize)] {
                if !bounds.contains(rect_x,rect_y) { return Err("question changed parent/outside pixels or used generic input geometry".to_owned()); }
                changed=changed.saturating_add(1_usize);
            }
        }}
        if changed==0_usize { return Err("source question never painted above Build".to_owned()); }
        let corner=load_rim_rgba_entry(&root,PICTURE_ARCHIVE,"uptg29a.rim",Some((248_u8,0_u8,248_u8))).map_err(|error|error.to_string())?;
        let (index,pixel)=corner.data.chunks_exact(4_usize).enumerate().find(|entry| entry.1[3_usize]!=0_u8).ok_or_else(||"original question corner is empty".to_owned())?;
        let width=usize::try_from(corner.width).map_err(|error|error.to_string())?;
        let x=usize::try_from(bounds.x).map_err(|error|error.to_string())?.saturating_add(index.checked_rem(width).ok_or_else(||"zero corner width".to_owned())?);
        let y=usize::try_from(bounds.y).map_err(|error|error.to_string())?.saturating_add(index.checked_div(width).ok_or_else(||"zero corner width".to_owned())?);
        let target=y.saturating_mul(800_usize).saturating_add(x).saturating_mul(4_usize);
        if after[target..target.saturating_add(3_usize)]!=pixel[..3_usize] { return Err("original uptg29 question corner ink/position mismatch".to_owned()); }
        Ok(())
    });
    assert!(result.is_ok(),"native original320x174 question overlay: {result:?}");
}

/* Source: EditQueue.cpp::{LoadModeCallback,DisplayQueueContents,LoadCallback}
 * and editqueue.ldl::{BuildEditorWindow.LoadBox,eq_LoadListItem,eq_ContentsItem}
 * use the original uptg20e2 pane; QueryLoadQueue then uses messagebox.ldl's
 * centered uptg29 controls. This component paints actual source art in memory,
 * not a saved frame or a certified full-reference match.
 */
#[test]
fn build_saved_load_native_browser_art_rows_and_localized_query_render_without_file_writes() {
    let result=with_canvas((800_u32,600_u32),|canvas| {
        let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");let creator=canvas.texture_creator();
        let mut textures=GameplayTextures::load(&root,&creator,true).map_err(|error|error.to_string())?;
        let projection:ServerProjection=serde_json::from_value(json!({"revision":7_i64,"turn":1_u32,"contract":{"version":2_u32,"features":{"atomic_production_queue":true}},"tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"plains"}],"cities":[{"id":"c2","owner_id":"human","tile_id":"a","name":"Rome","queue":[{"entry_id":"head","id":"UNIT_HOPLITE"}],"rush_buy_quote":{"revision":7_i64,"status":"available","cost":30_i64}}],"rules":{"units":[{"id":"UNIT_SETTLER"}]},"legal_actions":[{"route":"edit-production","entity_id":"c2","payload":{"city_id":"c2","entries":[]}}]})).map_err(|error|error.to_string())?;
        let state=GameState::from_server_projection(&projection,1_u8,1_u64);
        let mut ui=InGameUiModel {management:ManagementUi::new(LayoutBook::load(&root).map_err(|error|error.to_string())?).map_err(|error|error.to_string())?,..InGameUiModel::default()};
        ui.management.open(ManagerKind::Build,&state,Some("c2"));
        ui.management.apply_local(ManagerIntent::Select(crate::gameplay::ui_model::management::saved::MODE.to_owned(),"true".to_owned()));
        ui.management.view.as_mut().ok_or_else(||"Build missing".to_owned())?.selections.remove(crate::gameplay::ui_model::management::saved::REFRESH);
        ui.management.saved_queues.files.insert("memory:one".to_owned(),crate::gameplay::ui_model::management::saved::QueueFile::from_bytes("memory:one".to_owned(),"one".to_owned(),b"U UNIT_SETTLER"));
        ui.management.apply_local(ManagerIntent::Select("saved-queue".to_owned(),"memory:one".to_owned()));
        canvas.set_draw_color(Color::BLACK);canvas.clear();
        super::management::draw_management(canvas,&mut textures,&state,&ui,None).map_err(|error|error.to_string())?;
        let pane=ui.management.widgets(&state).into_iter().find(|widget|widget.id=="LoadBox").ok_or_else(||"source load pane absent".to_owned())?.rect;
        let pixels=canvas.read_pixels(Some(sdl2::rect::Rect::new(pane.x,pane.y,pane.width,pane.height)),PixelFormatEnum::RGBA32)?;
        if !pixels.chunks_exact(4_usize).any(|pixel|pixel[..3_usize]!=[0_u8;3_usize]) {return Err("source saved browser art never painted".to_owned());}
        let dialog=match ui.management.activate_id("LoadBox.LoadButton",&state) {Some(ManagerIntent::Dialog(dialog))=>Some(*dialog),_=>None}.ok_or_else(||"source load confirmation absent".to_owned())?;
        ui.operation=Some(dialog);
        super::operations::draw(canvas,&mut textures,&state,&ui,RenderFonts {body:None,management:None}).map_err(|error|error.to_string())?;
        Ok(())
    });
    assert!(result.is_ok(),"source LoadBox/QueryLoadQueue composition: {result:?}");
}

/* Released EditQueue.cpp::SetMode(CUSTOM), editqueue.ldl::BuildEditorWindow.
 * {CustomModeButtons,ItemsBox.ModeLabel,QueueGroup.QueueLabel} provide the actual
 * source-only control geometry/art. This in-memory composition is not a full
 * reference frame or proof of destructive file overwrite/delete functionality.
 */
#[test]
fn build_custom_source_mode_buttons_labels_and_local_queue_art_render_in_memory() {
    let result=with_canvas((800_u32,600_u32),|canvas| {
        let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");let creator=canvas.texture_creator();
        let mut textures=GameplayTextures::load(&root,&creator,true).map_err(|error|error.to_string())?;
        let projection:ServerProjection=serde_json::from_value(json!({"revision":7_i64,"turn":1_u32,"tiles":[],"cities":[],"rules":{"units":[{"id":"UNIT_SETTLER"}]}})).map_err(|error|error.to_string())?;
        let state=GameState::from_server_projection(&projection,1_u8,1_u64);
        let mut ui=InGameUiModel {management:ManagementUi::new(LayoutBook::load(&root).map_err(|error|error.to_string())?).map_err(|error|error.to_string())?,..InGameUiModel::default()};
        ui.management.open(ManagerKind::Build,&state,None);
        if !crate::gameplay::ui_model::management::saved::custom::apply(&mut ui.management,&state,crate::gameplay::ui_model::management::saved::custom::Action::Begin) {return Err("local Custom mode unavailable".to_owned());}
        let save=ui.management.widgets(&state).into_iter().find(|widget|widget.id=="CustomModeButtons.SaveButton").ok_or_else(||"source custom Save control missing".to_owned())?;
        if save.rect.width!=160_u32 || save.label!=ui.management.book.label("str_ldl_EditQueueSave") || !matches!(save.intent,Some(ManagerIntent::Dialog(form)) if form.queue_save.is_some()) {return Err("source Save geometry/label or original filename form binding lost".to_owned());}
        canvas.set_draw_color(Color::BLACK);canvas.clear();
        super::management::draw_management(canvas,&mut textures,&state,&ui,None).map_err(|error|error.to_string())?;
        Ok(())
    });
    assert!(result.is_ok(),"source local Custom presentation: {result:?}");
}

/* Original MessageBoxDialog.cpp::TextQuery and messagebox.ldl source field
 * overlays the unchanged editor; ui/aui_common/aui_textfield.cpp paints white
 * and RGB20 text. Check the entire outside overlay, not a favorable image crop.
 */
#[test]
fn build_save_source_name_field_art_white_background_and_all_outside_pixels_remain_original_parent(){
    let result=with_canvas((800_u32,600_u32),|canvas| {
        let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");let creator=canvas.texture_creator();let mut textures=GameplayTextures::load(&root,&creator,true).map_err(|error|error.to_string())?;
        let projection:ServerProjection=serde_json::from_value(json!({"revision":7_i64,"turn":1_u32,"tiles":[],"cities":[],"rules":{"units":[{"id":"UNIT_SETTLER"}]}})).map_err(|error|error.to_string())?;let state=GameState::from_server_projection(&projection,1_u8,1_u64);
        let mut ui=InGameUiModel {management:ManagementUi::new(LayoutBook::load(&root).map_err(|error|error.to_string())?).map_err(|error|error.to_string())?,..InGameUiModel::default()};ui.management.open(ManagerKind::Build,&state,None);
        if !crate::gameplay::ui_model::management::saved::custom::apply(&mut ui.management,&state,crate::gameplay::ui_model::management::saved::custom::Action::Begin){return Err("Custom unavailable".to_owned());}
        canvas.set_draw_color(Color::BLACK);canvas.clear();super::management::draw_management(canvas,&mut textures,&state,&ui,None).map_err(|error|error.to_string())?;
        let before=canvas.read_pixels(None,PixelFormatEnum::RGBA32)?;
        let mut dialog=crate::gameplay::ui_model::management::saved::save::dialog(&ui.management).ok_or_else(||"source Save unavailable".to_owned())?;dialog.text="Name".to_owned();ui.operation=Some(dialog);
        super::operations::draw(canvas,&mut textures,&state,&ui,RenderFonts {body:None,management:None}).map_err(|error|error.to_string())?;
        let after=canvas.read_pixels(None,PixelFormatEnum::RGBA32)?;
        for (index,pixel) in after.chunks_exact(4_usize).enumerate(){let x=index.checked_rem(800_usize).ok_or_else(||"index".to_owned())?;let y=index.div_euclid(800_usize);if !(240_usize..560_usize).contains(&x)||!(213_usize..387_usize).contains(&y){let start=index.saturating_mul(4_usize);if before.get(start..start.saturating_add(4_usize))!=Some(pixel){return Err("Source Save changed a pixel outside original320x174 overlay".to_owned());}}}
        let probe=canvas.read_pixels(Some(sdl2::rect::Rect::new(529_i32,328_i32,1_u32,1_u32)),PixelFormatEnum::RGBA32)?;
        if probe.get(..3_usize)!=Some(&[255_u8;3_usize]){return Err("Original source TextField white rectangle/overlap not painted".to_owned());}Ok(())
    });assert!(result.is_ok(),"source filename modal pixels: {result:?}");
}

/* Original EditQueue.cpp::DeleteCallback/MessageBoxDialog::Query and
 * messagebox.ldl use the same source opaque question/OK/Cancel, no TextField.
 * This source-art native composition is in memory, not frame certification.
 */
#[test]
fn build_delete_original_question_art_and_stale_affirmative_render_without_parameter_field(){
    let result=with_canvas((800_u32,600_u32),|canvas| {
        let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");let creator=canvas.texture_creator();let mut textures=GameplayTextures::load(&root,&creator,true).map_err(|error|error.to_string())?;
        let projection:ServerProjection=serde_json::from_value(json!({"revision":7_i64,"turn":1_u32,"tiles":[],"cities":[]})).map_err(|error|error.to_string())?;let state=GameState::from_server_projection(&projection,1_u8,1_u64);
        let mut ui=InGameUiModel {management:ManagementUi::new(LayoutBook::load(&root).map_err(|error|error.to_string())?).map_err(|error|error.to_string())?,..InGameUiModel::default()};ui.management.open(ManagerKind::Build,&state,None);
        ui.management.apply_local(ManagerIntent::Select(crate::gameplay::ui_model::management::saved::MODE.to_owned(),"true".to_owned()));ui.management.view.as_mut().ok_or_else(||"view".to_owned())?.selections.remove(crate::gameplay::ui_model::management::saved::REFRESH);
        ui.management.saved_queues.files.insert("profile:exact".to_owned(),crate::gameplay::ui_model::management::saved::QueueFile::from_bytes("profile:exact".to_owned(),"exact".to_owned(),b"# exact version\nU UNIT_SETTLER\n"));ui.management.apply_local(ManagerIntent::Select("saved-queue".to_owned(),"profile:exact".to_owned()));
        let dialog=crate::gameplay::ui_model::management::saved::delete::dialog(&ui.management).ok_or_else(||"source Delete unavailable".to_owned())?;
        if dialog.confirmation.iter().any(|widget|widget.node.name=="TextField")||!dialog.confirmation.iter().any(|widget|widget.node.name=="DontShowButton"&&widget.unavailable.is_some()){return Err("Delete source modality lost: parameter field or destructive DontShow approval".to_owned());}
        ui.operation=Some(dialog);canvas.set_draw_color(Color::BLACK);canvas.clear();super::management::draw_management(canvas,&mut textures,&state,&ui,None).map_err(|error|error.to_string())?;
        super::operations::draw(canvas,&mut textures,&state,&ui,RenderFonts {body:None,management:None}).map_err(|error|error.to_string())?;
        ui.management.saved_queues.files.get_mut("profile:exact").ok_or_else(||"file".to_owned())?.bytes=Some(b"changed".to_vec());
        super::operations::draw(canvas,&mut textures,&state,&ui,RenderFonts {body:None,management:None}).map_err(|error|error.to_string())?;Ok(())
    });assert!(result.is_ok(),"source Delete question and stale paint: {result:?}");
}
