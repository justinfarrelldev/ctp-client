//! SDL_mixer-backed CTP2 menu music and option-volume helpers.

use std::path::{Path, PathBuf};

use sdl2::AudioSubsystem;
use sdl2::mixer::{self, DEFAULT_CHANNELS, DEFAULT_FORMAT, InitFlag, Music, Sdl2MixerContext};

use crate::main_menu::{MainMenuError, MainMenuMusicBackend, MenuMusicPolicy};

/// Audio frequency requested from `SDL_mixer` for menu music.
const MIXER_FREQUENCY: i32 = 44_100;
/// Mixer chunk size balancing latency and compatibility.
const MIXER_CHUNK_SIZE: i32 = 1_024;
/// Maximum `SDL_mixer` volume value.
const MIXER_MAX_VOLUME: i32 = 128;

/// SDL_mixer-backed menu music player.
pub(super) struct SdlMenuMusicBackend {
    /// SDL context kept alive for the audio subsystem.
    _sdl: sdl2::Sdl,
    /// Audio subsystem lifetime guard.
    _audio: AudioSubsystem,
    /// Mixer context lifetime guard.
    _mixer_context: Sdl2MixerContext,
    /// Currently loaded music, kept alive while playback continues.
    music: Option<Music<'static>>,
}

impl SdlMenuMusicBackend {
    /// Creates and initializes the `SDL_mixer` backend.
    ///
    /// # Errors
    ///
    /// Returns [`MainMenuError::Music`] when SDL audio or mixer setup fails.
    pub(super) fn new() -> Result<Self, MainMenuError> {
        let sdl = sdl2::init().map_err(|message| MainMenuError::Music { message })?;
        let audio = sdl
            .audio()
            .map_err(|message| MainMenuError::Music { message })?;
        let mixer_context =
            mixer::init(InitFlag::OGG).map_err(|message| MainMenuError::Music { message })?;
        mixer::open_audio(
            MIXER_FREQUENCY,
            DEFAULT_FORMAT,
            DEFAULT_CHANNELS,
            MIXER_CHUNK_SIZE,
        )
        .map_err(|message| MainMenuError::Music { message })?;
        mixer::allocate_channels(4);

        Ok(Self {
            _sdl: sdl,
            _audio: audio,
            _mixer_context: mixer_context,
            music: None,
        })
    }

    /// Plays a numbered CTP2 music track from the installation.
    ///
    /// # Errors
    ///
    /// Returns [`MainMenuError::Music`] when loading or playback fails.
    pub(super) fn play_track_number(
        &mut self,
        installation_root: &Path,
        track_number: usize,
    ) -> Result<(), MainMenuError> {
        let relative_path = music_track_relative_path(track_number);
        let policy = MenuMusicPolicy {
            autoplay: true,
            relative_path: relative_path.clone(),
            loops: -1_i32,
        };
        self.play_music(&policy, installation_root.join(relative_path))
    }

    /// Pauses or resumes current music playback.
    pub(super) fn set_music_enabled(enabled: bool) {
        if enabled {
            Music::resume();
        } else {
            Music::pause();
        }
    }

    /// Applies music volume using the original 0-10 option scale.
    pub(super) fn set_music_volume(value: i32) {
        Music::set_volume(option_volume_to_mixer_volume(value));
    }

    /// Applies SFX/voice channel volume using the original 0-10 option scale.
    pub(super) fn set_channel_volume(value: i32) {
        mixer::Channel::all().set_volume(option_volume_to_mixer_volume(value));
    }
}

impl MainMenuMusicBackend for SdlMenuMusicBackend {
    fn play_music(
        &mut self,
        policy: &MenuMusicPolicy,
        absolute_path: PathBuf,
    ) -> Result<(), MainMenuError> {
        let music = Music::from_file(&absolute_path).map_err(|message| MainMenuError::Music {
            message: format!("failed to load {}: {message}", absolute_path.display()),
        })?;
        music
            .play(policy.loops)
            .map_err(|message| MainMenuError::Music { message })?;
        self.music = Some(music);
        Ok(())
    }
}

impl Drop for SdlMenuMusicBackend {
    fn drop(&mut self) {
        Music::halt();
        mixer::close_audio();
    }
}

/// Converts a 0-10 shell option volume to `SDL_mixer`'s 0-128 scale.
pub(super) fn option_volume_to_mixer_volume(value: i32) -> i32 {
    value
        .clamp(0_i32, 10_i32)
        .saturating_mul(MIXER_MAX_VOLUME)
        .checked_div(10_i32)
        .map_or(0_i32, |scaled| scaled)
}

/// Returns the relative path for a one-based CTP2 music track number.
pub(super) fn music_track_relative_path(track_number: usize) -> PathBuf {
    let bounded_track = track_number.clamp(1_usize, 10_usize);
    PathBuf::from(format!(
        "ctp2_program/ctp/music/Track{bounded_track:02}.ogg"
    ))
}
