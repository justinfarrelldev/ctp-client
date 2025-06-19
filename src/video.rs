//! Video playback module for CTP intro videos.
//!
//! This module provides functionality for playing video files, specifically
//! the intro videos from CTP installations. It uses FFmpeg (ffplay) for
//! reliable video playback with broad codec support including IV50.

use crate::config::GameInstallation;
use log::{debug, error, info, warn};
use std::path::PathBuf;

/// Result of video playback operations.
#[derive(Debug)]
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
    /// # Returns
    ///
    /// * `Ok(VideoPlayer)` - The initialized video player
    ///
    /// # Errors
    ///
    /// Currently this method never fails, but returns a Result for future compatibility.
    pub fn new() -> Result<Self, String> {
        info!("Initializing video player (using external commands)");
        Ok(Self)
    }

    /// Plays the intro video for the given game installation.
    ///
    /// # Arguments
    ///
    /// * `installation` - The game installation to find the intro video for
    ///
    /// # Returns
    ///
    /// * `VideoResult` - The result of the video playback
    #[must_use]
    pub fn play_intro_video(&self, installation: &GameInstallation) -> VideoResult {
        info!("Playing intro video for {}", installation.display_name);

        // Find the intro video file
        let Some(video_path) = Self::find_intro_video(installation) else {
            warn!("No intro video found for {}", installation.display_name);
            return VideoResult::Failed("Intro video not found".to_owned());
        };

        info!("Found intro video at: {}", video_path.display());

        // Create video URI
        let Some(path_str) = video_path.to_str() else {
            error!("Invalid video path encoding");
            return VideoResult::Failed("Invalid video path encoding".to_owned());
        };

        let video_uri = format!("file://{path_str}");
        info!("Playing video: {video_uri}");

        // Play video using GStreamer
        Self::play_video_standalone(&video_uri)
    }

    /// Plays a video file in a standalone `FFmpeg` window.
    ///
    /// # Arguments
    ///
    /// * `video_uri` - URI of the video file to play
    ///
    /// # Returns
    ///
    /// * `VideoResult` - The result of the video playback
    fn play_video_standalone(video_uri: &str) -> VideoResult {
        info!("Attempting to play video with FFmpeg: {video_uri}");

        let file_path = video_uri.strip_prefix("file://").unwrap_or(video_uri);

        // Try ffplay first (best option for video playback)
        info!("Playing video with ffplay in separate window...");
        info!("Close the video window to continue with the game launch.");

        let ffplay_result = std::process::Command::new("ffplay")
            .arg("-autoexit") // Exit when playback finishes
            .arg("-fs") // Fullscreen mode
            .arg("-noborder") // Remove window border for cleaner look
            .arg("-window_title") // Set window title
            .arg("CTP Intro Video")
            .arg(file_path)
            .status();

        match ffplay_result {
            Ok(status) => {
                if status.success() {
                    info!("Video playback completed successfully");
                    VideoResult::Completed
                } else {
                    warn!("Video playback was interrupted or failed");
                    // Check exit code to determine if it was skipped or failed
                    match status.code() {
                        Some(1) => VideoResult::Failed(
                            "Video file could not be played (codec issue)".to_owned(),
                        ),
                        _ => VideoResult::Skipped, // User closed window or other non-error exit
                    }
                }
            }
            Err(e) => {
                error!("Failed to start ffplay: {e}");

                // Fallback: Try with VLC if available
                info!("Trying fallback with VLC media player...");
                let vlc_result = std::process::Command::new("vlc")
                    .arg("--intf=dummy") // No GUI interface controls
                    .arg("--play-and-exit") // Exit when done
                    .arg("--fullscreen") // Fullscreen
                    .arg(file_path)
                    .status();

                match vlc_result {
                    Ok(status) => {
                        if status.success() {
                            info!("Video played successfully with VLC");
                            VideoResult::Completed
                        } else {
                            VideoResult::Failed("VLC failed to play video".to_owned())
                        }
                    }
                    Err(vlc_err) => {
                        error!("VLC also failed: {vlc_err}");

                        // Final fallback: basic ffplay without special options
                        info!("Trying basic ffplay without special options...");
                        std::process::Command::new("ffplay")
                            .arg("-autoexit")
                            .arg(file_path)
                            .status()
                            .map_or_else(
                                |_| VideoResult::Failed("No suitable video player found. Please install ffmpeg or vlc.".to_owned()),
                                |status| {
                                    if status.success() {
                                        VideoResult::Completed
                                    } else {
                                        VideoResult::Failed("Basic ffplay also failed".to_owned())
                                    }
                                }
                            )
                    }
                }
            }
        }
    }

    /// Finds the intro video file for a game installation.
    ///
    /// # Arguments
    ///
    /// * `installation` - The game installation to search
    ///
    /// # Returns
    ///
    /// * `Some(PathBuf)` - Path to the intro video if found
    /// * `None` - If no intro video was found
    fn find_intro_video(installation: &GameInstallation) -> Option<PathBuf> {
        let (data_folder, _) = installation.version.required_folders();

        // Possible video paths to check (cross-platform)
        let video_paths = [
            // Default videos location
            format!("{data_folder}/default/videos"),
            // English videos location
            format!("{data_folder}/english/videos"),
        ];

        // Possible intro video filenames (case variations)
        let intro_filenames = ["gm001.avi", "GM001.avi", "GM001.AVI", "gm001.AVI"];

        for video_dir in &video_paths {
            let full_video_dir = installation.path.join(video_dir);
            debug!("Checking video directory: {}", full_video_dir.display());

            if full_video_dir.exists() && full_video_dir.is_dir() {
                for filename in &intro_filenames {
                    let video_file = full_video_dir.join(filename);
                    debug!("Checking for video file: {}", video_file.display());

                    if video_file.exists() && video_file.is_file() {
                        info!("Found intro video: {}", video_file.display());
                        return Some(video_file);
                    }
                }
            }
        }

        // Additional fallback: scan directory case-insensitively
        for video_dir in &video_paths {
            let full_video_dir = installation.path.join(video_dir);
            if let Ok(entries) = std::fs::read_dir(&full_video_dir) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    if let Some(name_str) = file_name.to_str() {
                        if name_str.to_lowercase() == "gm001.avi" {
                            let video_file = entry.path();
                            info!(
                                "Found intro video (case-insensitive): {}",
                                video_file.display()
                            );
                            return Some(video_file);
                        }
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GameVersion;
    use std::path::PathBuf;

    #[test]
    fn test_video_result_debug() {
        let completed = VideoResult::Completed;
        let skipped = VideoResult::Skipped;
        let failed = VideoResult::Failed("test error".to_owned());

        assert!(format!("{completed:?}").contains("Completed"));
        assert!(format!("{skipped:?}").contains("Skipped"));
        assert!(format!("{failed:?}").contains("Failed"));
        assert!(format!("{failed:?}").contains("test error"));
    }

    #[test]
    fn test_find_intro_video_logic() {
        // Create a mock installation
        let installation = crate::config::GameInstallation::new(
            PathBuf::from("/nonexistent"),
            GameVersion::CTP2,
            "Test".to_owned(),
            None,
        );

        // This test verifies the logic but won't find actual files
        let _player = VideoPlayer::new();
        let result = VideoPlayer::find_intro_video(&installation);
        assert!(
            result.is_none(),
            "Should not find video in nonexistent path"
        );
    }

    #[test]
    fn test_video_filename_variations() {
        let filenames = ["gm001.avi", "GM001.avi", "GM001.AVI", "gm001.AVI"];

        // Test that all variations are present
        assert_eq!(filenames.len(), 4);
        assert!(filenames.contains(&"gm001.avi"));
        assert!(filenames.contains(&"GM001.AVI"));
    }

    #[test]
    fn test_video_path_construction() {
        let installation = crate::config::GameInstallation::new(
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
    fn test_video_player_new() {
        let result = VideoPlayer::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_cross_platform_path_handling() {
        // Test that PathBuf properly handles cross-platform paths
        let base_path = PathBuf::from("/test");
        let video_path = base_path
            .join("ctp2_data")
            .join("default")
            .join("videos")
            .join("gm001.avi");

        // Should work on both Unix and Windows
        assert!(video_path.to_string_lossy().contains("gm001.avi"));

        // Test Windows-style path construction
        #[cfg(windows)]
        {
            let windows_path = PathBuf::from("C:\\test\\ctp2_data\\default\\videos\\gm001.avi");
            assert!(windows_path.to_string_lossy().contains("gm001.avi"));
        }
    }
}
