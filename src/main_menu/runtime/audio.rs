//! SDL_mixer-backed CTP2 menu music and option-volume helpers.

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use sdl2::AudioSubsystem;
use sdl2::mixer::{
    self, Chunk, DEFAULT_CHANNELS, DEFAULT_FORMAT, InitFlag, LoaderRWops, Music, Sdl2MixerContext,
};
use sdl2::rwops::RWops;
use unciv::ZfsFile;

use crate::config::ctp2_options::Ctp2Options;
use crate::main_menu::{MainMenuError, MainMenuModel, MainMenuMusicBackend, MenuMusicPolicy};

/// Audio frequency requested from `SDL_mixer` for menu music.
const MIXER_FREQUENCY: i32 = 44_100;
/// Mixer chunk size balancing latency and compatibility.
const MIXER_CHUNK_SIZE: i32 = 1_024;
/// Maximum `SDL_mixer` volume value.
const MIXER_MAX_VOLUME: i32 = 128;
/// Archive containing the original CTP2 UI click sound.
const BUTTON_CLICK_SOUND_ARCHIVE: &str = "ctp2_data/default/sound/sound.zfs";
/// WAV entry used by `SOUND_ID_BUTTONCLICK` / `GAMESOUNDS_BUTTONCLICK` in the original source.
const BUTTON_CLICK_SOUND_ENTRY: &str = "CCLICK1.WAV";

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
    /// Original CTP2 button-click sound, kept alive for repeated UI clicks.
    button_click: Option<Chunk>,
    /// Whether selected music should loop continuously.
    auto_repeat: bool,
    /// Whether playlist ordering should be randomized when playlist support is expanded.
    random_order: bool,
}

impl SdlMenuMusicBackend {
    /// Creates and initializes the `SDL_mixer` backend.
    ///
    /// # Errors
    ///
    /// Returns [`MainMenuError::Music`] when SDL audio or mixer setup fails.
    pub(super) fn new(installation_root: &Path) -> Result<Self, MainMenuError> {
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

        let button_click = match load_button_click_chunk(installation_root) {
            Ok(chunk) => Some(chunk),
            Err(error) => {
                log::warn!("Button click sound is unavailable: {error}");
                None
            }
        };

        Ok(Self {
            _sdl: sdl,
            _audio: audio,
            _mixer_context: mixer_context,
            music: None,
            button_click,
            auto_repeat: false,
            random_order: false,
        })
    }

    /// Applies persisted audio options before the first runtime frame.
    pub(super) fn apply_initial_options(&mut self, model: &MainMenuModel, options: &Ctp2Options) {
        Self::set_channel_volume(options.sound.sound_volume);
        Self::set_music_volume(options.sound.music_volume);
        self.set_auto_repeat(options.music.auto_repeat);
        self.set_random_order(options.music.random_order);

        if !options.music.music_enabled {
            return;
        }

        let result = if options.music.selected_track == 2_usize {
            self.play_music(&model.music, model.music_path())
        } else {
            self.play_track_number(&model.installation_root, options.music.selected_track)
        };
        if let Err(error) = result {
            log::warn!("Main menu music could not be started: {error}");
        }
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
            loops: if self.auto_repeat { -1_i32 } else { 0_i32 },
        };
        self.play_music(&policy, installation_root.join(relative_path))
    }

    /// Plays the original CTP2 button-click sound once.
    pub(super) fn play_button_click(&self) {
        let Some(chunk) = self.button_click.as_ref() else {
            return;
        };
        if let Err(error) = mixer::Channel::all().play(chunk, 0_i32) {
            log::warn!("Button click sound could not be played: {error}");
        }
    }

    /// Applies auto-repeat behavior for subsequently selected tracks.
    pub(super) const fn set_auto_repeat(&mut self, enabled: bool) {
        self.auto_repeat = enabled;
    }

    /// Records random-order behavior for playlist expansion.
    pub(super) const fn set_random_order(&mut self, enabled: bool) {
        self.random_order = enabled;
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

/// Loads the original CTP2 button-click WAV into an SDL chunk.
fn load_button_click_chunk(installation_root: &Path) -> Result<Chunk, MainMenuError> {
    let (_entry_name, bytes) = load_button_click_wav_bytes(installation_root)?;
    let rwops = RWops::from_bytes(bytes.as_slice()).map_err(|message| MainMenuError::Music {
        message: format!("failed to create RWops for {BUTTON_CLICK_SOUND_ENTRY}: {message}"),
    })?;
    rwops.load_wav().map_err(|message| MainMenuError::Music {
        message: format!("failed to decode {BUTTON_CLICK_SOUND_ENTRY}: {message}"),
    })
}

/// Loads the original CTP2 button-click WAV bytes from the immutable game archive.
fn load_button_click_wav_bytes(
    installation_root: &Path,
) -> Result<(String, Vec<u8>), MainMenuError> {
    let archive_path = installation_root.join(BUTTON_CLICK_SOUND_ARCHIVE);
    let file = File::open(&archive_path).map_err(|error| MainMenuError::Music {
        message: format!("failed to open {}: {error}", archive_path.display()),
    })?;
    let mut reader = BufReader::new(file);
    let zfs = ZfsFile::from_stream(&mut reader).map_err(|error| MainMenuError::Music {
        message: format!("failed to parse {}: {error}", archive_path.display()),
    })?;
    let entry = zfs
        .files
        .iter()
        .find(|candidate| {
            candidate
                .name
                .eq_ignore_ascii_case(BUTTON_CLICK_SOUND_ENTRY)
        })
        .cloned()
        .ok_or_else(|| MainMenuError::Music {
            message: format!(
                "{BUTTON_CLICK_SOUND_ENTRY} was not found in {}",
                archive_path.display()
            ),
        })?;
    let bytes = entry
        .read_data(&mut reader)
        .map_err(|error| MainMenuError::Music {
            message: format!(
                "failed to read {} from {}: {error}",
                entry.name,
                archive_path.display()
            ),
        })?;
    Ok((entry.name, bytes))
}

/// Returns a test summary for the original button-click sound asset.
#[cfg(test)]
pub(in crate::main_menu) fn button_click_sound_summary_for_test(
    installation_root: &Path,
) -> Result<(String, String, usize, [u8; 4]), MainMenuError> {
    let (entry_name, bytes) = load_button_click_wav_bytes(installation_root)?;
    let mut header = [0_u8; 4];
    let header_bytes = bytes.get(0..4).ok_or_else(|| MainMenuError::Music {
        message: format!("{BUTTON_CLICK_SOUND_ENTRY} is too short to contain a RIFF header"),
    })?;
    header.copy_from_slice(header_bytes);
    Ok((
        BUTTON_CLICK_SOUND_ARCHIVE.to_owned(),
        entry_name.to_ascii_lowercase(),
        bytes.len(),
        header,
    ))
}
