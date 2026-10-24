//! Information-only source controls, with explicit nonexact missing-resource fallback.
use super::{management::draw_widget,text::{draw_wrapped_text,draw_centered_text}};
use crate::gameplay::{GameplayError,runtime_error,runtime::{textures::GameplayTextures,fonts::ManagementFonts},ui_model::{MenuRect,management::{ManagerWidget,typography::SourceFont}}};
use sdl2::{pixels::Color,rect::Rect,render::Canvas,video::Window};

/* Released MessageBoxDialog.cpp::constructor owns an opaque popup surface;
 * messagebox.ldl root320x174/MessageDisplay/RightButton are documented at
 * runtime/information.rs. Paint that same bounded surface first, never leak
 * underlying menu text through missing art. Original widgets replace the
 * fallback when available. Fixed neutral/gold fallback is a labelled client
 * error-recovery difference, not copied art or full-frame retail certification.
 */
/// Paint no world/UI command state, only safe captured text and original cached controls.
pub(in crate::gameplay::runtime) fn draw(canvas:&mut Canvas<Window>,textures:&mut GameplayTextures<'_>,widgets:Option<&[ManagerWidget]>,fonts:Option<&ManagementFonts<'_>>,message:&str,close:MenuRect)->Result<(),GameplayError> {
    base(canvas,close)?;
    if let Some(widgets)=widgets {
        let mut failed=false;
        for widget in widgets {
            let font=fonts.and_then(|fonts|fonts.get(&SourceFont::from_node(&widget.node)));
            if draw_widget(canvas,textures,widget,font).is_err(){failed=true;break;}
        }
        if !failed{return Ok(());}
        log::warn!("Original launch information art could not render; using nonexact functional fallback");
        base(canvas,close)?;
    }
    let old=canvas.clip_rect();canvas.set_clip_rect(Rect::new(270_i32,243_i32,260_u32,70_u32));
    let message_result=draw_wrapped_text(canvas,None,message,Rect::new(270_i32,243_i32,260_u32,70_u32),Color::RGB(30_u8,30_u8,30_u8));canvas.set_clip_rect(old);message_result?;
    draw_centered_text(canvas,None,"OK",Rect::new(close.x,close.y,close.width,close.height),Color::RGB(30_u8,30_u8,30_u8),false)
}
/// Source bounds retained even if the installation's popup art is missing or corrupt.
fn base(canvas:&mut Canvas<Window>,close:MenuRect)->Result<(),GameplayError> {
    let bounds=Rect::new(240_i32,213_i32,320_u32,174_u32);canvas.set_draw_color(Color::RGB(190_u8,180_u8,150_u8));canvas.fill_rect(bounds).map_err(runtime_error)?;canvas.set_draw_color(Color::RGB(80_u8,65_u8,35_u8));canvas.draw_rect(bounds).map_err(runtime_error)?;canvas.draw_rect(Rect::new(close.x,close.y,close.width,close.height)).map_err(runtime_error)
}
