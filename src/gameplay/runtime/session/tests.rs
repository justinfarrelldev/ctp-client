//! Native resource scopes must preserve the calling shell's SDL input state.

use super::TextInputScope;
use crate::gameplay::runtime::test_support::with_canvas;
use crate::{config::ctp2_options::{Ctp2GraphicsOptions,Ctp2Toggle},gameplay::{SinglePlayerLaunchConfig,server::ServerConfig}};
use std::path::PathBuf;

#[test]
fn nation_borders_actual_bootstrap_respects_shell_political_and_influence_switches() {
    let config=SinglePlayerLaunchConfig {installation_root:PathBuf::new(),empire_count:2_u8,leader_name:"Tester".to_owned(),seed:1_u64,server:ServerConfig::disabled_for_test(),graphics:Ctp2GraphicsOptions {political_borders:Ctp2Toggle::new(false),city_influence:Ctp2Toggle::new(false),..Ctp2GraphicsOptions::default()}};
    let (authority,state)=super::bootstrap(&config).expect("fileless explicit preview bootstrap");assert!(authority.is_none(),"no service calls in this test");assert!(!state.filters.political_borders&&!state.filters.city_influence,"actual startup must apply shell options before any paint");
}


#[test]
fn unicode_input_scope_enables_events_and_restores_inactive_or_active_shell_state() {
    let result = with_canvas((80_u32, 60_u32), |canvas| {
        let video = canvas.window().subsystem();
        let input = video.text_input();
        let previous = input.is_active();
        input.stop();
        let enabled = {
            let _scope = TextInputScope::new(video);
            input.is_active()
        };
        let stopped = !input.is_active();
        input.start();
        {
            let _scope = TextInputScope::new(video);
        }
        let preserved = input.is_active();
        if !previous {
            input.stop();
        }
        if enabled && stopped && preserved {
            Ok(())
        } else {
            Err("text input was not enabled/restored across both scope lifetimes".to_owned())
        }
    });
    assert!(
        result.is_ok(),
        "SDL Unicode/IME resource lifetime: {result:?}"
    );
}
