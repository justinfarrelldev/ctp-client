extern crate alloc;

use crate::config::GameInstallation;
use log::{debug, error, info, warn};
use std::path::PathBuf;

use super::VideoResult;
use super::input::SkipEventListener;
use super::process::{run_process_with_skip, spawn_ffplay, spawn_vlc};

/// Plays the intro video for the supplied installation by delegating to external players.
///
/// # Returns
///
/// A [`VideoResult`] describing whether playback completed, was skipped, or failed because an
/// appropriate video file or player executable could not be located.
pub(super) fn play_intro_video(installation: &GameInstallation) -> VideoResult {
    info!("Playing intro video for {}", installation.display_name);

    let Some(video_path) = find_intro_video(installation) else {
        warn!("No intro video found for {}", installation.display_name);
        return VideoResult::Failed("Intro video not found".to_owned());
    };

    info!("Found intro video at: {}", video_path.display());

    let Some(path_str) = video_path.to_str() else {
        error!("Invalid video path encoding");
        return VideoResult::Failed("Invalid video path encoding".to_owned());
    };

    let video_uri = format!("file://{path_str}");
    info!("Playing video: {video_uri}");

    play_video_standalone(&video_uri)
}

/// Attempts to play the supplied video URI with ffplay or VLC, allowing the caller to handle
/// skip events.
fn play_video_standalone(video_uri: &str) -> VideoResult {
    info!("Attempting to play video with FFmpeg: {video_uri}");

    let file_path = video_uri.strip_prefix("file://").unwrap_or(video_uri);
    info!("Playing video with ffplay in separate window. Click or press Space/Esc/Start to skip.");

    let mut skip_listener = match SkipEventListener::start() {
        Ok(listener) => {
            info!(
                "Skip overlay initialized; keyboard, mouse, and controller shortcuts are captured globally."
            );
            Some(listener)
        }
        Err(err) => {
            warn!(
                "Skip listener unavailable ({err}); intro video can only be skipped by closing the player window."
            );
            None
        }
    };

    match spawn_ffplay(file_path, true) {
        Ok(process) => {
            return run_process_with_skip(process, "ffplay (fullscreen)", skip_listener.take());
        }
        Err(err) => {
            error!("Failed to start ffplay: {err}");
        }
    }

    info!("Trying fallback with VLC media player...");
    match spawn_vlc(file_path) {
        Ok(process) => return run_process_with_skip(process, "VLC", skip_listener.take()),
        Err(err) => {
            error!("VLC fallback failed: {err}");
        }
    }

    info!("Trying basic ffplay without fullscreen...");
    spawn_ffplay(file_path, false).map_or_else(
        |_| {
            VideoResult::Failed(
                "No suitable video player found. Please install ffmpeg or vlc.".to_owned(),
            )
        },
        |process| run_process_with_skip(process, "ffplay (windowed)", skip_listener.take()),
    )
}

/// Searches for the intro video within the installation directory using a case-insensitive scan
/// and multiple standard folder locations.
pub(super) fn find_intro_video(installation: &GameInstallation) -> Option<PathBuf> {
    let (data_folder, _) = installation.version.required_folders();

    let video_paths = [
        format!("{data_folder}/default/videos"),
        format!("{data_folder}/english/videos"),
    ];

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
