//! Same SDL menu/window/input loop with injected gameplay; no live calls or file writes.
use super::super::{RuntimeLoopOptions,with_sdl_test_lock};
use crate::main_menu::{MainMenuAction,MainMenuError,MainMenuModel,MenuRect,shell::{ShellAction,ShellScreenId,screen_for_test_defaults}};
use crate::gameplay::{GameplayOutcome,SinglePlayerLaunchConfig,server::ServerConfig,runtime_error};
use sdl2::{event::Event,keyboard::{Keycode,Mod},mouse::MouseButton};
use std::path::Path;
/// Explicit user behavior after an injected failure, not automatic startup retries.
#[derive(Clone,Copy)]
enum Script {
    /// Acknowledge information then explicitly quit.
    AcknowledgeQuit,
    /// Try the underlying Start without acknowledging; modal must block it.
    BlockedStart,
    /// Acknowledge and deliberately retry; completed New Game returns to the same menu.
    ExplicitRetry,
}
/// Resolve original semantic control identity, never fixed screenshot click coordinates.
fn shell_click(screen:ShellScreenId,action:ShellAction)->Event {
    let source=screen_for_test_defaults(screen);let control=source.controls().iter().find(|control|control.action()==Some(action)).expect("source semantic control");let rect=control.rect();let parent=source.rect();click(MenuRect::new(parent.x.saturating_add(rect.x),parent.y.saturating_add(rect.y),rect.width,rect.height))
}
/// Source descriptor activation in original800x600 window coordinates.
fn click(rect:MenuRect)->Event {Event::MouseButtonDown {timestamp:0_u32,window_id:0_u32,which:0_u32,mouse_btn:MouseButton::Left,clicks:1_u8,x:rect.x.saturating_add(i32::try_from(rect.width.div_euclid(2_u32)).expect("source width")),y:rect.y.saturating_add(i32::try_from(rect.height.div_euclid(2_u32)).expect("source height"))}}
/// Benign Information acknowledgement; cannot also activate the underlying Start.
fn enter()->Event {Event::KeyDown {timestamp:0_u32,window_id:0_u32,keycode:Some(Keycode::RETURN),scancode:None,keymod:Mod::NOMOD,repeat:false}}
/// Run real owner/frames/semantic inputs; fake only the gameplay handoff.
fn exercise(script:Script)->Result<(),MainMenuError> {
    with_sdl_test_lock(|| {
        let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");let model=MainMenuModel::ctp2_landing(root.clone());let landing=model.buttons.iter().find(|button|button.action==MainMenuAction::SinglePlayer).expect("accessible Single Player");let start=shell_click(ShellScreenId::NewGame,ShellAction::LaunchGame);
        let options=RuntimeLoopOptions {initial_events:vec![click(landing.rect),shell_click(ShellScreenId::SinglePlayerHub,ShellAction::Navigate(ShellScreenId::NewGame)),start.clone()],initial_window_size:(800_u32,600_u32),frame_limit:Some(4_u32)};
        let config=SinglePlayerLaunchConfig {installation_root:root,empire_count:4_u8,leader_name:"Tester".to_owned(),seed:1_u64,server:ServerConfig::disabled_for_test(),graphics:crate::config::ctp2_options::Ctp2GraphicsOptions::default()};let mut attempts=0_u8;let mut window=None;let mut violation=None;
        let result=super::run(&model,options,None,Some(&config),|received,canvas,_| {
            attempts=attempts.saturating_add(1_u8);if received.leader_name!=config.leader_name||received.empire_count!=config.empire_count {violation=Some("failed launch discarded captured setup or reduced empires");}
            if let Some(id)=window {if canvas.window().id()!=id {violation=Some("retry/New Game replaced the existing window");}}else{window=Some(canvas.window().id());}
            let events=canvas.window().subsystem().sdl().event().expect("same window events");
            if attempts>1_u8 {events.push_event(Event::Quit {timestamp:0_u32}).expect("explicit later quit");return Ok(GameplayOutcome::NewGame);}
            match script {
                Script::AcknowledgeQuit=>{events.push_event(enter()).expect("acknowledge information");events.push_event(Event::Quit {timestamp:0_u32}).expect("explicit quit");},
                Script::BlockedStart=>events.push_event(start.clone()).expect("underlying Start must be blocked"),
                Script::ExplicitRetry=>{events.push_event(enter()).expect("acknowledge");events.push_event(start.clone()).expect("explicit semantic retry");},
            }
            canvas.window_mut().set_title("Call to Power II - Game").expect("simulate handoff title");Err(runtime_error("private injected startup detail must not be displayed".to_owned()))
        });
        if let Some(message)=violation{return Err(super::super::runtime_error(message.to_owned()));}
        let expected=if matches!(script,Script::ExplicitRetry){2_u8}else{1_u8};if attempts!=expected{return Err(super::super::runtime_error("modal/explicit retry count mismatch".to_owned()));}
        match script {
            Script::BlockedStart=>{if !result.is_err_and(|error|error.to_string().contains("menu loop ended without an action")){return Err(super::super::runtime_error("bounded harness should still be in the menu, not fatal launch propagation".to_owned()));}},
            Script::AcknowledgeQuit|Script::ExplicitRetry=>{if result?!=MainMenuAction::Quit {return Err(super::super::runtime_error("only explicit later Quit may end recovery/New Game".to_owned()));}},
        }
        Ok(())
    })
}
#[test]
fn launch_recovery_menu_diagnostics_keep_safe_setup_context_without_raw_transport_details() {
    let safe=crate::gameplay::GameplayError::Setup {stage:"/game/start_game".to_owned(),status:Some(400_u16),message:"A selected civilization is unavailable.".to_owned()};let shown=super::safe_message(&safe);assert!(shown.contains("/game/start_game")&&shown.contains("400")&&shown.contains("civilization"),"safe actionable endpoint/status/reason");let private=super::safe_message(&runtime_error("http://private:password@host/?session_id=secret".to_owned()));assert!(!private.contains("password")&&!private.contains("session_id")&&!private.contains("secret"),"raw transport credentials never reach the dialog or menu warning");
}
#[test]
fn launch_recovery_real_menu_returns_to_same_window_after_failed_launch_instead_of_exiting() {let result=exercise(Script::AcknowledgeQuit);assert!(result.is_ok(),"actual menu recovery: {result:?}");}
#[test]
fn launch_recovery_modal_blocks_underlying_start_until_explicit_acknowledgement() {let result=exercise(Script::BlockedStart);assert!(result.is_ok(),"no blind retry: {result:?}");}
#[test]
fn launch_recovery_explicit_retry_preserves_setup_window_and_new_game_lifecycle() {let result=exercise(Script::ExplicitRetry);assert!(result.is_ok(),"same owner retry: {result:?}");}
