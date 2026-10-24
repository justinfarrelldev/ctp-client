//! Source Information/OK rendering, usable before any game or server session exists.
/// Source-only controls, pixel isolation and benign modal input regressions.
#[cfg(test)]
mod tests;
use super::{textures::GameplayTextures,fonts::ManagementFonts};
use crate::gameplay::{GameplayError,runtime_error,assets::layout::LayoutBook,ui_model::{MenuRect,management::{ManagementUi,ManagerWidget,confirmation}}};
use sdl2::{event::{Event,WindowEvent},keyboard::Keycode,mouse::MouseButton,pixels::{Color,PixelFormatEnum},rect::Rect,render::{BlendMode,Canvas,Texture,TextureCreator},ttf::Sdl2TtfContext,video::{Window,WindowContext}};
use std::path::Path;

/// Cached original modal resources; no fake world, units or gameplay commands.
pub struct Information<'texture,'ttf> {
    /// Lazy original UI art only, not terrain/sprites or canonical facts.
    textures:GameplayTextures<'texture>,
    /// Fully owned source controls, absent only in labelled resource fallback.
    widgets:Option<Vec<ManagerWidget>>,
    /// Source inherited per-control fonts, with existing platform fallback rules.
    fonts:Option<ManagementFonts<'ttf>>,
    /// Captured safe failure text; never interpolated as asset SLIC.
    message:String,
    /// Transparent800x600 modal overlay uses the existing whole-menu viewport mapping.
    frame:Texture<'texture>,
    /// Same source `RightButton` rect drives semantic pointer input and painting.
    close:MenuRect,
    /// Only a matched explicit press/release may dismiss via pointer.
    pressed:bool,
}
impl<'texture,'ttf> Information<'texture,'ttf> {
    /* Released ctp2_source/ctp2_code/ui/interface/MessageBoxDialog.cpp::
     * Information/constructor/RightButtonActionCallback and messagebox.ldl::
     * MessageBoxDialog320x174, MessageDisplay30,30/260x70, RightButton20,109/80:
     * hide LeftButton/TextField/DontShow, retain only localized MB_OK. This is
     * an acknowledgement, not approval or retry. See loadsavescreen.cpp's
     * CantLoadScenarioData Information failure path; network startup is our
     * explicit server integration difference, not an original retail endpoint.
     * Keep the exact source controls and existing whole800x600 viewport scaling.
     * Missing licensed assets retain a functional labelled fallback, not a claim
     * of art parity or a reason to turn the reported startup error into app exit.
     */
    /// Load once per failure; SDL target errors remain real rendering failures.
    pub(crate) fn new(root:&Path,creator:&'texture TextureCreator<WindowContext>,ttf:Option<&'ttf Sdl2TtfContext>,message:String)->Result<Self,GameplayError> {
        let ui=LayoutBook::load(root).and_then(ManagementUi::new).ok();
        let mut textures=GameplayTextures::information(root,creator);
        let widgets=ui.as_ref().and_then(|ui|confirmation::information(ui,&message)).filter(|_|textures.dynamic_ui("uptg29a.rim").is_ok_and(|image|image.is_some()));
        let fonts=ui.as_ref().and_then(|ui|ttf.and_then(|ttf|ManagementFonts::load(ttf,ui).ok()));
        let close=widgets.as_ref().and_then(|widgets|widgets.iter().find(|widget|widget.node.name=="RightButton")).map_or(MenuRect {x:260_i32,y:322_i32,width:80_u32,height:24_u32},|widget|widget.rect);
        if widgets.is_none(){log::warn!("Original launch information resources unavailable; using nonexact functional fallback");}
        let mut frame=creator.create_texture_target(PixelFormatEnum::RGBA32,800_u32,600_u32).map_err(|error|runtime_error(error.to_string()))?;frame.set_blend_mode(BlendMode::Blend);
        Ok(Self {textures,widgets,fonts,message,frame,close,pressed:false})
    }
    /// Benign acknowledgement owns all modal input; caller never passes it to menu buttons.
    pub(crate) fn dismiss(&mut self,event:&Event,logical:Option<(i32,i32)>)->bool {
        match *event {
            Event::KeyDown {keycode:Some(Keycode::RETURN|Keycode::KP_ENTER|Keycode::SPACE|Keycode::ESCAPE),repeat:false,..}=>true,
            Event::MouseButtonDown {mouse_btn:MouseButton::Left,..}=>{self.pressed=logical.is_some_and(|(x,y)|self.close.contains(x,y));false},
            Event::MouseButtonUp {mouse_btn:MouseButton::Left,..}=>{let pressed=core::mem::take(&mut self.pressed);pressed&&logical.is_some_and(|(x,y)|self.close.contains(x,y))},
            Event::Window {win_event:WindowEvent::FocusLost|WindowEvent::Resized(..)|WindowEvent::SizeChanged(..),..}=>{self.pressed=false;false},
            _=>false,
        }
    }
    /// Overlay after all menu text, preserving outside pixels and current scaling.
    pub(crate) fn draw(&mut self,canvas:&mut Canvas<Window>,viewport:Rect)->Result<(),GameplayError> {
        let mut result=Ok(());
        canvas.with_texture_canvas(&mut self.frame,|target| {
            result=(|| {
                target.set_logical_size(0_u32,0_u32).map_err(|error|runtime_error(error.to_string()))?;target.set_scale(1.0_f32,1.0_f32).map_err(runtime_error)?;target.set_viewport(None);target.set_clip_rect(None);target.set_draw_color(Color::RGBA(0_u8,0_u8,0_u8,0_u8));target.clear();
                super::render::information::draw(target,&mut self.textures,self.widgets.as_deref(),self.fonts.as_ref(),&self.message,self.close)
            })();
        }).map_err(|error|runtime_error(error.to_string()))?;result?;
        canvas.set_viewport(None);canvas.set_clip_rect(None);canvas.set_scale(1.0_f32,1.0_f32).map_err(runtime_error)?;canvas.copy(&self.frame,None,viewport).map_err(runtime_error)
    }
}
