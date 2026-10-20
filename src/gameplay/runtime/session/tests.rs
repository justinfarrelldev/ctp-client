//! Native resource scopes must preserve the calling shell's SDL input state.

use super::TextInputScope;
use crate::gameplay::runtime::test_support::with_canvas;

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
