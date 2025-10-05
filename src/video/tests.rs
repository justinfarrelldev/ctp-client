extern crate alloc;

use super::input::{HotplugAction, SkipEventListener};
use super::overlay::SkipOverlayLayout;
use super::player::find_intro_video;
use super::process::{PlaybackProcess, monitor_playback};
use super::types::{ProcessExit, SkipTrigger};
use super::{VideoPlayer, VideoResult};
use crate::config::{GameInstallation, GameVersion};
use alloc::collections::VecDeque;
use core::convert::TryFrom;
use std::path::PathBuf;

#[test]
fn video_result_debug_contains_details() {
    let completed = VideoResult::Completed;
    let skipped = VideoResult::Skipped;
    let failed = VideoResult::Failed("test error".to_owned());

    assert!(format!("{completed:?}").contains("Completed"));
    assert!(format!("{skipped:?}").contains("Skipped"));
    assert!(format!("{failed:?}").contains("Failed"));
    assert!(format!("{failed:?}").contains("test error"));
}

#[test]
fn find_intro_video_logic_handles_missing_installation() {
    let installation = GameInstallation::new(
        PathBuf::from("/nonexistent"),
        GameVersion::CTP2,
        "Test".to_owned(),
        None,
    );

    assert!(VideoPlayer::new().is_ok());
    let result = find_intro_video(&installation);
    assert!(
        result.is_none(),
        "Should not find video in nonexistent path"
    );
}

#[test]
fn video_filename_variations_are_listed() {
    let filenames = ["gm001.avi", "GM001.avi", "GM001.AVI", "gm001.AVI"];

    assert_eq!(filenames.len(), 4);
    assert!(filenames.contains(&"gm001.avi"));
    assert!(filenames.contains(&"GM001.AVI"));
}

#[test]
fn video_path_construction_respects_game_version() {
    let installation = GameInstallation::new(
        PathBuf::from("/test/ctp2"),
        GameVersion::CTP2,
        "Test CTP2".to_owned(),
        None,
    );

    let (data_folder, _) = installation.version.required_folders();
    assert_eq!(data_folder, "ctp2_data");

    let video_path = installation
        .path
        .join(format!("{data_folder}/default/videos"));
    assert_eq!(
        video_path,
        PathBuf::from("/test/ctp2/ctp2_data/default/videos")
    );
}

#[test]
fn video_player_new_returns_ok() {
    let result = VideoPlayer::new();
    assert!(result.is_ok());
}

#[test]
fn cross_platform_path_handling_includes_filename() {
    let base_path = PathBuf::from("/test");
    let video_path = base_path
        .join("ctp2_data")
        .join("default")
        .join("videos")
        .join("gm001.avi");

    assert!(video_path.to_string_lossy().contains("gm001.avi"));
}

/// Mocked playback process used to simulate different completion states.
struct MockProcess {
    /// Queue of predetermined states returned by `try_wait`.
    states: VecDeque<Result<Option<ProcessExit>, String>>,
    /// Tracks whether `terminate` was invoked.
    terminated: bool,
}

impl MockProcess {
    /// Constructs a mock process with a predefined sequence of outcomes.
    fn new(states: Vec<Result<Option<ProcessExit>, String>>) -> Self {
        Self {
            states: states.into_iter().collect(),
            terminated: false,
        }
    }
}

impl PlaybackProcess for MockProcess {
    fn try_wait(&mut self) -> Result<Option<ProcessExit>, String> {
        self.states.pop_front().unwrap_or(Ok(None))
    }

    fn terminate(&mut self) -> Result<(), String> {
        self.terminated = true;
        Ok(())
    }
}

#[test]
fn monitor_playback_skips_on_keyboard_signal() {
    let mut process = MockProcess::new(vec![Ok(None)]);
    let mut poll_calls = usize::default();
    let mut poller = || {
        poll_calls += 1;
        (poll_calls == 1).then_some(SkipTrigger::Keyboard)
    };

    let result = monitor_playback(&mut process, &mut poller).expect("monitor should run");

    assert!(matches!(result, VideoResult::Skipped));
    assert!(process.terminated, "process should be terminated on skip");
}

#[test]
fn monitor_playback_completes_on_success() {
    let exits = vec![Ok(Some(ProcessExit {
        success: true,
        status_code: Some(0_i32),
    }))];
    let mut process = MockProcess::new(exits);
    let mut poller = || None;

    let result = monitor_playback(&mut process, &mut poller).expect("monitor should run");

    assert!(matches!(result, VideoResult::Completed));
    assert!(
        !process.terminated,
        "process should not be terminated on completion"
    );
}

#[test]
fn monitor_playback_reports_failure() {
    let exits = vec![Ok(Some(ProcessExit {
        success: false,
        status_code: Some(1_i32),
    }))];
    let mut process = MockProcess::new(exits);
    let mut poller = || None;

    let result = monitor_playback(&mut process, &mut poller).expect("monitor should run");

    match result {
        VideoResult::Failed(message) => {
            assert!(
                message.contains("codec"),
                "failure message should mention codec issues"
            );
        }
        other => {
            assert!(
                matches!(other, VideoResult::Failed(_)),
                "expected failure result, got {other:?}"
            );
        }
    }
}

#[test]
fn monitor_playback_skips_on_controller_signal() {
    let mut process = MockProcess::new(vec![Ok(None), Ok(None)]);
    let mut poll_calls = usize::default();
    let mut poller = || {
        poll_calls += 1;
        (poll_calls == 2).then_some(SkipTrigger::Controller)
    };

    let result = monitor_playback(&mut process, &mut poller).expect("monitor should run");

    assert!(matches!(result, VideoResult::Skipped));
    assert!(
        process.terminated,
        "controller skip should terminate process"
    );
}

#[test]
fn monitor_playback_skips_on_mouse_signal() {
    let mut process = MockProcess::new(vec![Ok(None)]);
    let mut poll_calls = usize::default();
    let mut poller = || {
        poll_calls += 1;
        (poll_calls == 1).then_some(SkipTrigger::Mouse)
    };

    let result = monitor_playback(&mut process, &mut poller).expect("monitor should run");

    assert!(matches!(result, VideoResult::Skipped));
    assert!(process.terminated, "mouse skip should terminate process");
}

#[test]
fn monitor_playback_propagates_process_error() {
    let mut process = MockProcess::new(vec![Err("expected failure".to_owned())]);
    let mut poller = || None;

    let err = monitor_playback(&mut process, &mut poller)
        .expect_err("monitor should surface polling errors");

    assert!(
        err.contains("expected failure"),
        "error message should include original cause"
    );
}

#[test]
fn skip_overlay_layout_exposes_accessible_instructions() {
    let lines = SkipOverlayLayout::instruction_lines();

    assert_eq!(
        lines.len(),
        3,
        "overlay should present three concise instruction lines"
    );
    assert!(
        lines[0].to_ascii_lowercase().contains("space"),
        "keyboard hint should mention Space key"
    );
    assert!(
        lines[1].to_ascii_lowercase().contains("click"),
        "mouse hint should reference clicking"
    );
    assert!(
        lines[2].to_ascii_lowercase().contains("controller"),
        "controller hint should appear on the final line"
    );
}

#[test]
fn skip_overlay_layout_window_title_mentions_all_inputs() {
    let title = SkipOverlayLayout::window_title().to_ascii_lowercase();

    assert!(
        title.contains("press space"),
        "window title should call out keyboard skip shortcut"
    );
    assert!(
        title.contains("click"),
        "window title should reference mouse interaction"
    );
    assert!(
        title.contains("controller start"),
        "window title should mention controller Start button for accessibility"
    );
}

#[test]
fn skip_overlay_layout_bitmap_contains_active_pixels() {
    let glyph =
        SkipOverlayLayout::glyph_bitmap_for_test('A').expect("glyph pattern should exist for A");

    let lit_pixels = glyph
        .iter()
        .map(|row| usize::try_from(row.count_ones()).expect("glyph bitmap count fits in usize"))
        .sum::<usize>();

    assert!(
        lit_pixels > 8,
        "glyph should contain sufficient lit pixels for readability"
    );
}

#[test]
fn skip_overlay_layout_does_not_grab_input() {
    assert!(
        !SkipOverlayLayout::should_grab_input(),
        "skip overlay should not capture input when the video window handles skip gestures"
    );
}

#[test]
fn skip_overlay_layout_window_title_mentions_escape() {
    let title = SkipOverlayLayout::window_title().to_ascii_lowercase();

    assert!(
        title.contains("esc"),
        "window title should mention the Escape key for accessibility"
    );
}

#[test]
fn skip_overlay_instructions_reference_controller_guide_button() {
    let controller_line = SkipOverlayLayout::instruction_lines()[2].to_ascii_uppercase();

    assert!(
        controller_line.contains("GUIDE"),
        "controller instruction line should reference the Guide button"
    );
}

#[test]
fn interpret_hotplug_event_handles_add_and_remove() {
    let add_event = sdl2::event::Event::ControllerDeviceAdded {
        timestamp: 0,
        which: 7,
    };
    let remove_event = sdl2::event::Event::ControllerDeviceRemoved {
        timestamp: 0,
        which: 3,
    };

    assert_eq!(
        SkipEventListener::interpret_hotplug_event(&add_event),
        Some(HotplugAction::Add(7))
    );
    assert_eq!(
        SkipEventListener::interpret_hotplug_event(&remove_event),
        Some(HotplugAction::Remove(3))
    );
}

#[test]
fn derive_skip_trigger_respects_controller_buttons() {
    let event = sdl2::event::Event::ControllerButtonDown {
        timestamp: 0,
        which: 0,
        button: sdl2::controller::Button::Start,
    };

    assert_eq!(
        SkipEventListener::derive_skip_trigger(&event),
        Some(SkipTrigger::Controller)
    );
}
