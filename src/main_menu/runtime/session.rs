//! Own the SDL menu/window across gameplay handoffs; launch failures are recoverable.
use super::{MainMenuModel,MainMenuAction,MainMenuError,RuntimeLoopOptions,SinglePlayerLaunchConfig,FRAME_DELAY,canvas_output_size,sound_before_menu_event,play_menu_sound,handle_menu_event,update_text_input_focus,apply_shell_runtime_effects,advance_audio_after_frame,output_point_to_menu_point,scaled_font_point_size_for_output_size,BUTTON_FONT_POINT_SIZE,draw_current_menu,load_menu_textures,find_existing_font,runtime_error};
use super::{audio::SdlMenuMusicBackend,shell_runtime::ShellRuntimeState,text::MenuFontCache};
use crate::gameplay::{GameplayError,GameplayOutcome};
use sdl2::{EventPump,event::{Event,WindowEvent},render::Canvas,video::Window};
use crate::gameplay::runtime::information::Information;
use std::thread;

/// Production and injected launches share the same actual menu loop/window owner.
pub(super) fn run(model:&MainMenuModel,options:RuntimeLoopOptions,mut audio_backend:Option<&mut SdlMenuMusicBackend>,gameplay_config:Option<&SinglePlayerLaunchConfig>,mut launch:impl FnMut(&SinglePlayerLaunchConfig,&mut Canvas<Window>,&mut EventPump)->Result<GameplayOutcome,GameplayError>)->Result<MainMenuAction,MainMenuError> {
    let sdl = sdl2::init().map_err(runtime_error)?;
    let video = sdl.video().map_err(runtime_error)?;
    let text_input = video.text_input();
    text_input.start();
    let window = video.window("Call to Power II",options.initial_window_size.0,options.initial_window_size.1).position_centered().build().map_err(|error| runtime_error(error.to_string()))?;
    let mut canvas=window.into_canvas().software().build().map_err(|error|runtime_error(error.to_string()))?;
    let texture_creator=canvas.texture_creator();
    let textures=load_menu_textures(&texture_creator,model);
    let ttf_context=sdl2::ttf::init().ok();
    let font_path=find_existing_font();
    let mut font_cache=MenuFontCache::new(ttf_context.as_ref(),font_path);
    let event_subsystem=sdl.event().map_err(runtime_error)?;
    for event in options.initial_events {event_subsystem.push_event(event).map_err(runtime_error)?;}
    let mut event_pump=sdl.event_pump().map_err(runtime_error)?;
    let mut shell_state=ShellRuntimeState::new();
    let mut selected_action=None;
    let mut suppressed_keydown_text:Option<String>=None;
    let mut rendered_frames=0_u32;
    let mut information:Option<Information<'_, '_>>=None;
    while selected_action.is_none() {
        let event_output_size=canvas_output_size(&canvas);
        for event in event_pump.poll_iter() {
            if let Some(dialog)=information.as_mut() {
                if matches!(event,Event::Quit {..}|Event::Window {win_event:WindowEvent::Close,..}){selected_action=Some(MainMenuAction::Quit);}else{
                    let logical=match event {Event::MouseButtonDown {x,y,..}|Event::MouseButtonUp {x,y,..}=>output_point_to_menu_point(event_output_size.0,event_output_size.1,x,y),_=>None};
                    if dialog.dismiss(&event,logical){information=None;}
                }
                continue;
            }
            if let Some(sound)=sound_before_menu_event(&event,model,&shell_state,event_output_size) {if let Some(backend)=audio_backend.as_deref_mut(){play_menu_sound(backend,sound);}}
            if let Some(sound)=handle_menu_event(&event,model,&mut shell_state,&mut selected_action,event_output_size,&mut suppressed_keydown_text) {if let Some(backend)=audio_backend.as_deref_mut(){play_menu_sound(backend,sound);}}
            update_text_input_focus(&text_input,&shell_state,event_output_size);
        }
        /* Released ctp2_source/ctp2_code/ui/interface/spnewgamescreen.cpp::
         * spnewgamescreen_startPress launches the selected setup;
         * MessageBoxDialog.cpp::Information preserves its owner under modal OK.
         * ctp2/ctp2_data/english/uidata/layouts/{spnewgame,messagebox}.ldl
         * StartButton/MessageBoxDialog establish controls; server startup failures
         * are an explicit client integration difference, not a retail endpoint.
         * Do not propagate a recoverable rejection through GtkUI into Exit, drop
         * the existing menu/window, replay launch or erase/alter a server game.
         */
        if selected_action==Some(MainMenuAction::LaunchSinglePlayer) {
            if let Some(config)=gameplay_config {
                let mut configured=config.clone();configured.graphics=shell_state.gameplay_graphics();
                log::info!("Launching single-player gameplay in the existing SDL menu window");
                match launch(&configured,&mut canvas,&mut event_pump) {
                    Ok(GameplayOutcome::Quit|GameplayOutcome::Closed)=>selected_action=Some(MainMenuAction::Quit),
                    Ok(GameplayOutcome::NewGame)=>selected_action=None,
                    Err(error)=>{
                        let message=safe_message(&error);log::warn!("Single-player launch failed: {message}");
                        information=Some(Information::new(&model.installation_root,&texture_creator,ttf_context.as_ref(),message).map_err(|render_error|runtime_error(render_error.to_string()))?);selected_action=None;
                    }
                }
                canvas.window_mut().set_title("Call to Power II").map_err(|error|runtime_error(error.to_string()))?;
            }
        }
        apply_shell_runtime_effects(&mut canvas,model,audio_backend.as_deref_mut(),&shell_state.take_effects())?;
        advance_audio_after_frame(audio_backend.as_deref_mut(),&model.installation_root);
        let mouse_state=event_pump.mouse_state();let draw_output_size=canvas_output_size(&canvas);
        let (mouse_x,mouse_y)=output_point_to_menu_point(draw_output_size.0,draw_output_size.1,mouse_state.x(),mouse_state.y()).map_or((-1_i32,-1_i32),|point|point);
        let font_point_size=scaled_font_point_size_for_output_size(draw_output_size.0,draw_output_size.1,BUTTON_FONT_POINT_SIZE);
        let frame_font=font_cache.font_for_point_size(font_point_size);
        draw_current_menu(&mut canvas,&texture_creator,&textures,model,&shell_state,mouse_x,mouse_y,frame_font,information.as_mut())?;
        rendered_frames=rendered_frames.saturating_add(1);
        if options.frame_limit.is_some_and(|limit|rendered_frames>=limit){break;}
        thread::sleep(FRAME_DELAY);
    }
    let action=selected_action.ok_or_else(||runtime_error("menu loop ended without an action".to_owned()))?;
    Ok(action)
}
/// Setup exposes only safe bounded diagnostics; unrelated resource/transport text stays private.
#[expect(clippy::pattern_type_mismatch,reason="Borrow safe diagnostics from non-Copy error variants without cloning captured strings")]
fn safe_message(error:&GameplayError)->String {
    match error {
        GameplayError::Setup {..}=>format!("Unable to start game.\n{error}"),
        GameplayError::Rejected {rejection}=>format!("Unable to start game (HTTP {}). {}",rejection.status,rejection.message),
        GameplayError::Runtime {..}|GameplayError::Asset {..}=>"Unable to start game. Check that the server and game installation are available, then try again.".to_owned(),
    }
}

/// Native failure-path regression uses only injected launch operations and SDL memory.
#[cfg(test)]
mod tests;
