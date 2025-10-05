use std::process::ExitStatus;

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

/// Trigger indicating the user has requested to skip the video.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SkipTrigger {
    /// Skip triggered by a keyboard event.
    Keyboard,
    /// Skip triggered by a mouse event.
    Mouse,
    /// Skip triggered by a controller event.
    Controller,
}

/// Simplified representation of a subprocess exit result for video playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ProcessExit {
    /// Indicates whether the subprocess exited successfully.
    pub(super) success: bool,
    /// Exit status code reported by the subprocess if available.
    pub(super) status_code: Option<i32>,
}

/// Converts an [`ExitStatus`] into a simplified [`ProcessExit`] structure.
///
/// # Returns
///
/// Returns a [`ProcessExit`] capturing success and exit status information.
pub(super) fn to_process_exit(status: ExitStatus) -> ProcessExit {
    let code = status.code().or_else(|| {
        #[cfg(unix)]
        {
            status.signal()
        }
        #[cfg(not(unix))]
        {
            None
        }
    });

    ProcessExit {
        success: status.success(),
        status_code: code,
    }
}
