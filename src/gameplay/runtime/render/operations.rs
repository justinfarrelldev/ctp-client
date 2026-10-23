//! Native original-art confirmation/input forms above managers and map controls.
use sdl2::{pixels::Color,rect::{Rect,Point},render::Canvas,ttf::Font,video::Window};
use crate::gameplay::{GameplayError,runtime_error};
use crate::gameplay::model::GameState;
use crate::gameplay::ui_model::{InGameUiModel,MenuRect,operations::{OperationDialog,InputKind}};
use super::{super::fonts::RenderFonts,super::textures::GameplayTextures,text::{draw_text,draw_centered_text},context::draw_gold_frame,chrome::draw_art_rect};

/// Shares geometry and enabled states with semantic/keyboard/pointer dispatch.
pub(super) fn draw(canvas:&mut Canvas<Window>,textures:&mut GameplayTextures<'_>,state:&GameState,ui:&InGameUiModel,fonts:RenderFonts<'_, '_, '_>)->Result<(),GameplayError> {
    let Some(dialog)=ui.operation.as_ref() else {return Ok(());};
    /* Source: ctp2_source/ctp2_code/ui/interface/MessageBoxDialog.cpp::constructor
     * and messagebox.ldl::MessageBoxDialog.{DialogBackground,MessageDisplay,
     * LeftButton,RightButton,DontShowButton}. Paint actual cached uptg29 panels,
     * source fonts/bounds above the still-visible manager, not the generic input
     * popup's frame. A stale affirmative is unavailable, never silently rebased.
     */
    if !dialog.confirmation.is_empty() {
        for widget in &dialog.confirmation {
            if widget.node.name == "TextField" && dialog.queue_save.is_some() {
                super::management::draw_widget(canvas,textures,widget,fonts.control(&widget.node))?;
                draw_input(canvas,fonts.control(&widget.node),dialog,widget.rect)?;
            } else if widget.node.name == "LeftButton" && !dialog.queue_save.as_ref().map_or_else(||dialog.choices.first().is_some_and(|choice|choice.command.valid_for(state)),|capture|capture.name.is_none() && capture.current(&ui.management) && crate::gameplay::ui_model::management::saved::save::name_valid(&dialog.text)) {
                let mut disabled = widget.clone();
                disabled.intent = None;
                disabled.unavailable = Some("This confirmation is no longer authorized at its captured revision".to_owned());
                super::management::draw_widget(canvas,textures,&disabled,fonts.control(&disabled.node))?;
            } else {
                super::management::draw_widget(canvas,textures,widget,fonts.control(&widget.node))?;
            }
        }
        return Ok(());
    }
    let font=fonts.body;
    let bounds=OperationDialog::rect();
    draw_gold_frame(canvas,textures,rect(bounds))?;
    draw_text(canvas,font,&dialog.title,220,200,Color::RGB(50,50,50))?;
    if dialog.input.is_some() {
        let field=OperationDialog::field();
        let prompt=if dialog.input == Some(InputKind::Rename) {"City name:"} else {"Content IDs separated by commas; @ID retains an entry."};
        draw_text(canvas,font,prompt,220,237,Color::RGB(50,50,50))?;
        draw_input(canvas,font,dialog,field)?;
        button(canvas,textures,font,"OK",OperationDialog::accept(),dialog.command(state).is_some())?;
    } else {
        for (index,choice) in dialog.choices.iter().enumerate().skip(dialog.start).take(8) {
            if let Some(row)=dialog.row(index) {
                if index == dialog.focused {canvas.set_draw_color(Color::RGB(170,160,125));canvas.fill_rect(rect(row)).map_err(runtime_error)?;}
                let old=canvas.clip_rect();canvas.set_clip_rect(rect(row));
                draw_text(canvas,font,&choice.label,row.x,row.y,if choice.command.valid_for(state) {Color::RGB(30,30,30)} else {Color::RGB(120,120,120)})?;
                canvas.set_clip_rect(old);
            }
        }
        let page=dialog.start.div_euclid(8).saturating_add(1);
        let pages=dialog.choices.len().saturating_add(7).div_euclid(8);
        draw_text(canvas,font,&format!("{page}/{pages} — arrows / mouse wheel"),220,390,Color::RGB(50,50,50))?;
    }
    button(canvas,textures,font,"Cancel",OperationDialog::cancel(),true)
}
/* Released ctp2_source/ctp2_code/ui/aui_common/aui_textfield.cpp::CreateWin/
 * DrawThis sets white background and RGB(20,20,20) text; ui/aui_ctp2/
 * ctp2_textfield.cpp::DrawThis delegates native edit painting. messagebox.ldl::
 * TextField/CTP2_TEXT_FIELD inherits CTP2_STANDARD_FONT; measure/paint same face.
 * Reuse existing client Unicode caret/selection/IME padding/colors as an explicit
 * SDL presentation difference, not a claim of exact Win32 selection raster. Keep
 * source260x24 geometry/overlap, clips and full-frame/scaling acceptance open.
 */
/// Shared bounded Unicode field; no font search or screenshot fitting per frame.
fn draw_input(canvas:&mut Canvas<Window>,font:Option<&Font<'_, '_>>,dialog:&OperationDialog,field:MenuRect)->Result<(),GameplayError>{
    canvas.set_draw_color(Color::RGB(255,255,255));canvas.fill_rect(rect(field)).map_err(runtime_error)?;
    let old=canvas.clip_rect();canvas.set_clip_rect(rect(field));
    let caret=dialog.editor.caret_offset(&dialog.text);let scroll=dialog.metrics.scroll(&dialog.text,caret,field.width);
    let x=field.x.saturating_add(3).saturating_sub(i32::try_from(scroll).unwrap_or(i32::MAX));let selected=dialog.editor.range(&dialog.text);
    if !selected.is_empty(){let left=dialog.metrics.width(&dialog.text,selected.start);let right=dialog.metrics.width(&dialog.text,selected.end);canvas.set_draw_color(Color::RGB(140,180,230));canvas.fill_rect(Rect::new(x.saturating_add(i32::try_from(left).unwrap_or(i32::MAX)),field.y,right.saturating_sub(left),field.height)).map_err(runtime_error)?;}
    draw_text(canvas,font,&dialog.text,x,field.y,Color::RGB(20,20,20))?;
    if dialog.queue_save.as_ref().is_none_or(|capture|capture.name.is_none()) && (dialog.queue_save.is_none() || dialog.confirmation_focus.as_deref()==Some(crate::gameplay::ui_model::management::saved::save::FIELD)) {
        let cursor=x.saturating_add(i32::try_from(dialog.metrics.width(&dialog.text,caret)).unwrap_or(i32::MAX));canvas.set_draw_color(Color::RGB(0,0,0));canvas.draw_line(Point::new(cursor,field.y),Point::new(cursor,field.y.saturating_add(17))).map_err(runtime_error)?;draw_text(canvas,font,&dialog.editor.composition,cursor,field.y,Color::RGB(30,60,150))?;
    }
    canvas.set_clip_rect(old);Ok(())
}
/// Source c3_PopupClose-sized original button, with actual disabled labeling.
fn button(canvas:&mut Canvas<Window>,textures:&GameplayTextures<'_>,font:Option<&Font<'_, '_>>,label:&str,bounds:MenuRect,enabled:bool)->Result<(),GameplayError> {
    draw_art_rect(canvas,textures,"upbt02u.rim",rect(bounds))?;
    draw_centered_text(canvas,font,label,rect(bounds),if enabled {Color::RGB(50,50,50)} else {Color::RGB(130,130,130)},false)
}
/// Logical native rectangle conversion used only at the renderer boundary.
fn rect(bounds:MenuRect)->Rect {Rect::new(bounds.x,bounds.y,bounds.width,bounds.height)}
