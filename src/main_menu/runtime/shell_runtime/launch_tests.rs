//! Launch handoff observes in-memory options without file writes or SDL setup.

use super::{ShellRuntimeEffects, ShellRuntimeState};
use crate::config::ctp2_options::Ctp2Options;
use crate::main_menu::shell::{ShellOptionField, SinglePlayerShellModel};

#[test]
fn gameplay_launch_preserves_current_unit_speed_and_animation_toggle() {
    let mut runtime = ShellRuntimeState {
        model: SinglePlayerShellModel::with_persisted_options(&Ctp2Options::default()),
        pending_effects: ShellRuntimeEffects::default(),
        active_slider: None,
        active_listbox_scroll: None,
    };
    assert!(
        runtime
            .model
            .options_mut()
            .set_numeric(ShellOptionField::UnitSpeed, 9),
        "speed change accepted"
    );
    assert!(
        runtime
            .model
            .options_mut()
            .toggle(ShellOptionField::UnitAnimation),
        "animation toggle accepted"
    );
    let graphics = runtime.gameplay_graphics();
    assert_eq!(
        graphics.unit_speed, 9_i32,
        "launch uses current speed, not constructor defaults"
    );
    assert!(
        !graphics.unit_animation.enabled(),
        "launch uses current animation preference"
    );
    assert_eq!(
        runtime.pending_effects,
        ShellRuntimeEffects::default(),
        "reading launch settings has no persistence side effect"
    );
}
