//! Video playback module for CTP intro videos.
//!
//! This module provides functionality for playing video files, specifically
//! the intro videos from CTP installations. It uses FFmpeg (ffplay) for
//! reliable video playback with broad codec support including IV50.

extern crate alloc;

use crate::config::GameInstallation;
use alloc::string::String;
use log::info;

/// Input capture and skip detection primitives.
mod input;
/// Overlay rendering helpers for skip instructions.
mod overlay;
/// Video playback orchestration functions.
mod player;
/// Process management and monitoring utilities.
mod process;
/// Shared foundational types for video playback internals.
mod types;

/// Result of video playback operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VideoResult {
    /// Video completed successfully.
    Completed,
    /// Video was skipped by user input.
    Skipped,
    /// Video playback failed with an error.
    Failed(String),
}

/// Video player for handling intro video playback.
pub struct VideoPlayer;

impl VideoPlayer {
    /// Creates a new video player instance.
    ///
    /// # Errors
    ///
    /// This constructor currently never fails but returns a [`Result`] to preserve
    /// forward compatibility with future initialization steps.
    pub fn new() -> Result<Self, String> {
        info!("Initializing video player (using external commands)");
        Ok(Self)
    }

    /// Plays the intro video for the given game installation.
    ///
    /// Returns [`VideoResult::Completed`] when playback finishes, [`VideoResult::Skipped`]
    /// when the user interrupts the video, or [`VideoResult::Failed`] if no compatible
    /// player could be launched.
    #[must_use]
    pub fn play_intro_video(&self, installation: &GameInstallation) -> VideoResult {
        player::play_intro_video(installation)
    }
}

#[cfg(test)]
mod tests;
