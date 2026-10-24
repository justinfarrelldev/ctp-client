//! Native Information without a game world; original art is read-only and pixels stay in RAM.
use super::Information;
use crate::gameplay::assets::{PICTURE_ARCHIVE,rim::load_rim_rgba_entry};
use crate::gameplay::runtime::test_support::with_canvas;
use sdl2::{event::{Event,WindowEvent},keyboard::{Keycode,Mod},mouse::MouseButton,pixels::{Color,PixelFormatEnum},rect::Rect};
use std::path::Path;

#[test]
fn launch_recovery_information_uses_original_ok_only_and_never_overpaints_menu_outside_modal() {
    let result=with_canvas((800_u32,600_u32),|canvas| {
        let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");let creator=canvas.texture_creator();let mut info=Information::new(&root,&creator,None,"Unable to start game. Server rejected the chosen civilization.".to_owned()).map_err(|error|error.to_string())?;
        let widgets=info.widgets.as_ref().ok_or_else(||"source information controls".to_owned())?;let ok=widgets.iter().find(|widget|widget.node.name=="RightButton").ok_or_else(||"source OK".to_owned())?;
        if ok.label!="OK"||widgets.iter().any(|widget|widget.id.split('.').any(|part|matches!(part,"LeftButton"|"TextField"|"DontShowButton"))){return Err("Information must have only original benign OK, not confirmation/text/don't-show controls".to_owned());}
        canvas.set_draw_color(Color::RGB(19_u8,23_u8,29_u8));canvas.clear();let before=canvas.read_pixels(None,PixelFormatEnum::RGBA32)?;info.draw(canvas,Rect::new(0_i32,0_i32,800_u32,600_u32)).map_err(|error|error.to_string())?;let after=canvas.read_pixels(None,PixelFormatEnum::RGBA32)?;
        for y in 0_usize..600_usize {for x in 0_usize..800_usize {if (240_usize..560_usize).contains(&x)&&(213_usize..387_usize).contains(&y){continue;}let offset=y.saturating_mul(800_usize).saturating_add(x).saturating_mul(4_usize);if before[offset..offset.saturating_add(3_usize)]!=after[offset..offset.saturating_add(3_usize)]{return Err("source Information modified menu pixels outside its320x174 modal".to_owned());}}}
        let corner=load_rim_rgba_entry(&root,PICTURE_ARCHIVE,"uptg29a.rim",Some((248_u8,0_u8,248_u8))).map_err(|error|error.to_string())?;let (index,pixel)=corner.data.chunks_exact(4_usize).enumerate().find(|entry|entry.1[3_usize]!=0_u8).ok_or_else(||"original information corner".to_owned())?;let width=usize::try_from(corner.width).map_err(|error|error.to_string())?;let x=240_usize.saturating_add(index.checked_rem(width).ok_or_else(||"corner width".to_owned())?);let y=213_usize.saturating_add(index.checked_div(width).ok_or_else(||"corner width".to_owned())?);let offset=y.saturating_mul(800_usize).saturating_add(x).saturating_mul(4_usize);if after[offset..offset.saturating_add(3_usize)]!=pixel[..3_usize]{return Err("Information did not use actual original uptg29 art at source placement".to_owned());}
        Ok(())
    });assert!(result.is_ok(),"source Information pixel isolation: {result:?}");
}
#[test]
fn launch_recovery_missing_assets_still_show_bounded_information_and_matched_benign_input() {
    let result=with_canvas((800_u32,600_u32),|canvas| {
        let creator=canvas.texture_creator();let mut info=Information::new(Path::new(""),&creator,None,"Unable to start. ".repeat(200_usize)).map_err(|error|error.to_string())?;if info.widgets.is_some(){return Err("asset-free fallback fixture unexpectedly has layouts".to_owned());}
        canvas.set_draw_color(Color::RGB(19_u8,23_u8,29_u8));canvas.clear();info.draw(canvas,Rect::new(0_i32,0_i32,800_u32,600_u32)).map_err(|error|error.to_string())?;let pixels=canvas.read_pixels(None,PixelFormatEnum::RGBA32)?;
        for y in 0_usize..600_usize {for x in 0_usize..800_usize {if (240_usize..560_usize).contains(&x)&&(213_usize..387_usize).contains(&y){continue;}let offset=y.saturating_mul(800_usize).saturating_add(x).saturating_mul(4_usize);if pixels[offset..offset.saturating_add(3_usize)]!=[19_u8,23_u8,29_u8]{return Err("fallback message overflowed source modal bounds".to_owned());}}}
        let inside=Some((info.close.x.saturating_add(1_i32),info.close.y.saturating_add(1_i32)));let down=Event::MouseButtonDown {timestamp:0_u32,window_id:0_u32,which:0_u32,mouse_btn:MouseButton::Left,clicks:1_u8,x:0_i32,y:0_i32};let up=Event::MouseButtonUp {timestamp:0_u32,window_id:0_u32,which:0_u32,mouse_btn:MouseButton::Left,clicks:1_u8,x:0_i32,y:0_i32};
        if info.dismiss(&up,inside)||info.dismiss(&down,inside)||info.dismiss(&up,None){return Err("unmatched/outside releases cannot dismiss".to_owned());}
        info.dismiss(&down,inside);info.dismiss(&Event::Window {timestamp:0_u32,window_id:0_u32,win_event:WindowEvent::FocusLost},None);if info.dismiss(&up,inside){return Err("lost focus must release the captured button".to_owned());}
        info.dismiss(&down,inside);if !info.dismiss(&up,inside){return Err("matched source OK dismisses information only".to_owned());}
        let repeat=Event::KeyDown {timestamp:0_u32,window_id:0_u32,keycode:Some(Keycode::RETURN),scancode:None,keymod:Mod::NOMOD,repeat:true};if info.dismiss(&repeat,None){return Err("held launch Enter cannot auto-acknowledge a new failure".to_owned());}
        Ok(())
    });assert!(result.is_ok(),"asset-free information/input recovery: {result:?}");
}
