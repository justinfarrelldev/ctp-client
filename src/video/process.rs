extern crate alloc;

use alloc::string::String;
use log::{debug, info, warn};
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

use super::input::SkipEventListener;
use super::types::{ProcessExit, SkipTrigger, to_process_exit};
use super::VideoResult;

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
) -> VideoResult {
    info!(
        "Playing intro video via {backend_label}. Press Space, click, or controller Start to skip."
    );

    let mut listener_opt = match SkipEventListener::start() {
        Ok(listener) => Some(listener),
        Err(err) => {
            warn!(
                "Skip listener unavailable ({err}); intro video can only be skipped by closing the player window."
            );
            None
        }
    };

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
/// * `fullscreen` - Whether the player should launch in fullscreen mode.
///
/// # Errors
///
/// Returns an error string when the process cannot be launched.
pub(super) fn spawn_ffplay(
    file_path: &str,
    fullscreen: bool,
) -> Result<ExternalPlaybackProcess, String> {
    let mut command = Command::new("ffplay");
    command
        .arg("-autoexit")
        .arg("-noborder")
        .arg("-window_title")
        .arg("CTP Intro Video");

    if fullscreen {
        command.arg("-fs");
    }

    let child = command
        .arg(file_path)
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
///
/// # Errors
///
/// Returns an error string when the process cannot be launched.
pub(super) fn spawn_vlc(file_path: &str) -> Result<ExternalPlaybackProcess, String> {
    let child = Command::new("vlc")
        .arg("--intf=dummy")
        .arg("--play-and-exit")
        .arg("--fullscreen")
        .arg("--quiet")
        .arg(file_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to launch VLC: {e}"))?;

    Ok(ExternalPlaybackProcess::new(child))
}
