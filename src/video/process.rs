extern crate alloc;

use alloc::string::String;
use log::{debug, info};
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

use super::VideoResult;
use super::input::SkipEventListener;
use super::types::{ProcessExit, SkipTrigger, to_process_exit};

/// Window size requested for external intro-video players.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PlaybackWindowSize {
    /// Requested video window width in pixels.
    width: u32,
    /// Requested video window height in pixels.
    height: u32,
}

impl PlaybackWindowSize {
    /// Creates a playback window size.
    pub(super) const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Returns the width clamped to an external-player-safe non-zero value.
    const fn width(self) -> u32 {
        if self.width == 0 { 1 } else { self.width }
    }

    /// Returns the height clamped to an external-player-safe non-zero value.
    const fn height(self) -> u32 {
        if self.height == 0 { 1 } else { self.height }
    }
}

/// Abstraction over a launched external video player process.
pub(super) trait PlaybackProcess {
    /// Attempts to poll the playback process for completion without blocking.
    fn try_wait(&mut self) -> Result<Option<ProcessExit>, String>;

    /// Requests the playback process to terminate.
    fn terminate(&mut self) -> Result<(), String>;
}

/// Wrapper around an externally spawned video playback process.
pub(super) struct ExternalPlaybackProcess {
    /// Handle to the spawned child process.
    child: Child,
}

impl ExternalPlaybackProcess {
    /// Creates a new [`ExternalPlaybackProcess`] using the provided child handle.
    pub(super) const fn new(child: Child) -> Self {
        Self { child }
    }
}

impl PlaybackProcess for ExternalPlaybackProcess {
    fn try_wait(&mut self) -> Result<Option<ProcessExit>, String> {
        match self.child.try_wait() {
            Ok(Some(status)) => Ok(Some(to_process_exit(status))),
            Ok(None) => Ok(None),
            Err(e) => Err(format!("Failed to poll video player status: {e}")),
        }
    }

    fn terminate(&mut self) -> Result<(), String> {
        if let Some(stdin) = self.child.stdin.as_mut() {
            // Attempt graceful shutdown via ffplay quit command.
            if let Err(e) = stdin.write_all(b"q\n") {
                debug!("Failed to signal video player via stdin: {e}");
            }
            if let Err(e) = stdin.flush() {
                debug!("Failed to flush video player stdin: {e}");
            }
            thread::sleep(Duration::from_millis(150));
        }

        match self.child.try_wait() {
            Ok(Some(_)) => Ok(()),
            Ok(None) => {
                if let Err(e) = self.child.kill() {
                    return Err(format!("Failed to terminate video player: {e}"));
                }
                self.child
                    .wait()
                    .map(|_| ())
                    .map_err(|e| format!("Failed to wait for video player termination: {e}"))
            }
            Err(e) => Err(format!("Failed to query video player after skip: {e}")),
        }
    }
}

/// Monitors playback state while observing skip triggers.
///
/// # Parameters
///
/// * `process` - The playback process under observation.
/// * `poll_skip` - Closure providing skip triggers, if any.
///
/// # Returns
///
/// Returns a [`VideoResult`] describing how playback concluded when successful.
///
/// # Errors
///
/// Returns an error if polling or terminating the playback process fails.
pub(super) fn monitor_playback<P, F>(
    process: &mut P,
    poll_skip: &mut F,
) -> Result<VideoResult, String>
where
    P: PlaybackProcess,
    F: FnMut() -> Option<SkipTrigger>,
{
    let sleep_interval = Duration::from_millis(50);

    loop {
        if let Some(exit) = process.try_wait()? {
            if exit.success {
                return Ok(VideoResult::Completed);
            }

            if exit.status_code == Some(1) {
                return Ok(VideoResult::Failed(
                    "Video file could not be played (codec issue)".to_owned(),
                ));
            }

            info!(
                "Video playback exited early with status code {:?}; treating as skip",
                exit.status_code
            );
            return Ok(VideoResult::Skipped);
        }

        if let Some(trigger) = poll_skip() {
            let trigger_label = match trigger {
                SkipTrigger::Keyboard => "keyboard",
                SkipTrigger::Mouse => "mouse",
                SkipTrigger::Controller => "controller",
            };
            info!("Intro video skip requested via {trigger_label} input");
            process.terminate()?;
            return Ok(VideoResult::Skipped);
        }

        thread::sleep(sleep_interval);
    }
}

/// Runs a playback process while monitoring for skip events.
///
/// # Parameters
///
/// * `process` - The externally spawned playback process to monitor.
/// * `backend_label` - Human-readable description of the backend for logging.
///
/// # Returns
///
/// Returns a [`VideoResult`] describing how playback concluded.
pub(super) fn run_process_with_skip(
    mut process: ExternalPlaybackProcess,
    backend_label: &str,
    skip_listener: Option<SkipEventListener>,
) -> VideoResult {
    let mut listener_opt = skip_listener;

    if listener_opt.is_some() {
        info!(
            "Playing intro video via {backend_label}. Press Space/Esc, click, or controller Start/Guide to skip."
        );
    } else {
        info!(
            "Playing intro video via {backend_label}. Close the video window to continue if skip shortcuts are unavailable."
        );
    }

    let mut poller = || {
        listener_opt
            .as_mut()
            .and_then(SkipEventListener::poll_event)
    };

    let result = monitor_playback(&mut process, &mut poller);

    drop(listener_opt);

    result.unwrap_or_else(VideoResult::Failed)
}

/// Spawns an `ffplay` process configured for intro playback.
///
/// # Parameters
///
/// * `file_path` - Path to the video file to play.
/// * `window_size` - Requested player window size matching the application window.
///
/// # Errors
///
/// Returns an error string when the process cannot be launched.
pub(super) fn spawn_ffplay(
    file_path: &str,
    window_size: PlaybackWindowSize,
) -> Result<ExternalPlaybackProcess, String> {
    let child = Command::new("ffplay")
        .args(ffplay_arguments(file_path, window_size))
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to launch ffplay: {e}"))?;

    Ok(ExternalPlaybackProcess::new(child))
}

/// Spawns a VLC process configured for intro playback.
///
/// # Parameters
///
/// * `file_path` - Path to the video file to play.
/// * `window_size` - Requested player window size matching the application window.
///
/// # Errors
///
/// Returns an error string when the process cannot be launched.
pub(super) fn spawn_vlc(
    file_path: &str,
    window_size: PlaybackWindowSize,
) -> Result<ExternalPlaybackProcess, String> {
    let child = Command::new("vlc")
        .args(vlc_arguments(file_path, window_size))
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to launch VLC: {e}"))?;

    Ok(ExternalPlaybackProcess::new(child))
}

/// Builds `ffplay` arguments for an application-sized intro window.
fn ffplay_arguments(file_path: &str, window_size: PlaybackWindowSize) -> Vec<String> {
    vec![
        "-autoexit".to_owned(),
        "-noborder".to_owned(),
        "-window_title".to_owned(),
        "CTP Intro Video".to_owned(),
        "-x".to_owned(),
        window_size.width().to_string(),
        "-y".to_owned(),
        window_size.height().to_string(),
        file_path.to_owned(),
    ]
}

/// Builds VLC arguments for an application-sized intro window.
fn vlc_arguments(file_path: &str, window_size: PlaybackWindowSize) -> Vec<String> {
    vec![
        "--intf=dummy".to_owned(),
        "--play-and-exit".to_owned(),
        "--no-video-deco".to_owned(),
        "--width".to_owned(),
        window_size.width().to_string(),
        "--height".to_owned(),
        window_size.height().to_string(),
        "--quiet".to_owned(),
        file_path.to_owned(),
    ]
}

/// Returns `ffplay` arguments for sizing tests.
#[cfg(test)]
pub(super) fn ffplay_arguments_for_test(
    file_path: &str,
    window_size: PlaybackWindowSize,
) -> Vec<String> {
    ffplay_arguments(file_path, window_size)
}

/// Returns VLC arguments for sizing tests.
#[cfg(test)]
pub(super) fn vlc_arguments_for_test(
    file_path: &str,
    window_size: PlaybackWindowSize,
) -> Vec<String> {
    vlc_arguments(file_path, window_size)
}
