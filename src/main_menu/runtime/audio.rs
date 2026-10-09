//! SDL_mixer-backed CTP2 menu music and option-volume helpers.

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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
/// Mixer chunk size chosen to keep UI sound effects responsive.
const MIXER_CHUNK_SIZE: i32 = 512;
/// Number of mixer channels reserved for overlapping UI SFX.
const MIXER_CHANNEL_COUNT: i32 = 8;
/// Maximum `SDL_mixer` volume value.
const MIXER_MAX_VOLUME: i32 = 128;
/// First one-based CTP2 music track number.
const MUSIC_TRACK_MIN: usize = 1;
/// Last one-based CTP2 music track number.
const MUSIC_TRACK_MAX: usize = 10;
/// Number of available CTP2 menu music tracks.
const MUSIC_TRACK_COUNT_U64: u64 = 10;
/// Archive containing original CTP2 UI sound effects.
const MENU_SOUND_ARCHIVE: &str = "ctp2_data/default/sound/sound.zfs";
/// WAV entry used by `SOUND_ID_BUTTONCLICK` / `GAMESOUNDS_BUTTONCLICK` in the original source.
const BUTTON_CLICK_SOUND_ENTRY: &str = "CCLICK1.WAV";
/// WAV entry used by `SOUND_ID_SLIDERUP` / `GAMESOUNDS_SLIDERUP` in the original source.
const SLIDER_UP_SOUND_ENTRY: &str = "CSLIDEUP.WAV";
/// WAV entry used by `SOUND_ID_SLIDERDOWN` / `GAMESOUNDS_SLIDERDOWN` in the original source.
const SLIDER_DOWN_SOUND_ENTRY: &str = "CSLIDEDN.WAV";

/// Test summary for one original sound asset: archive, entry, byte length, and RIFF header.
#[cfg(test)]
pub(in crate::main_menu) type SoundAssetSummary = (String, String, usize, [u8; 4]);

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
    /// Original CTP2 slider-up sound, kept alive for slider movement.
    slider_up: Option<Chunk>,
    /// Original CTP2 slider-down sound, kept alive for slider release.
    slider_down: Option<Chunk>,
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
        mixer::allocate_channels(MIXER_CHANNEL_COUNT);

        let button_click = load_optional_sound_chunk(
            installation_root,
            BUTTON_CLICK_SOUND_ENTRY,
            "button click sound",
        );
        let slider_up =
            load_optional_sound_chunk(installation_root, SLIDER_UP_SOUND_ENTRY, "slider-up sound");
        let slider_down = load_optional_sound_chunk(
            installation_root,
            SLIDER_DOWN_SOUND_ENTRY,
            "slider-down sound",
        );

        Ok(Self {
            _sdl: sdl,
            _audio: audio,
            _mixer_context: mixer_context,
            music: None,
            button_click,
            slider_up,
            slider_down,
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

        let track_number = initial_music_track_number(options);
        let result = self.play_track_number(&model.installation_root, track_number);
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
        play_loaded_chunk(self.button_click.as_ref(), "Button click sound");
    }

    /// Plays the original CTP2 slider-up sound once.
    pub(super) fn play_slider_up(&self) {
        play_loaded_chunk(self.slider_up.as_ref(), "Slider-up sound");
    }

    /// Plays the original CTP2 slider-down sound once.
    pub(super) fn play_slider_down(&self) {
        play_loaded_chunk(self.slider_down.as_ref(), "Slider-down sound");
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
    let bounded_track = bounded_music_track(track_number);
    PathBuf::from(format!(
        "ctp2_program/ctp/music/Track{bounded_track:02}.ogg"
    ))
}

/// Returns the startup music track for persisted CTP2 music options.
fn initial_music_track_number(options: &Ctp2Options) -> usize {
    initial_music_track_number_from_seed(
        options.music.selected_track,
        options.music.random_order,
        random_music_seed(),
    )
}

/// Returns a deterministic startup music track for a seed.
pub(super) fn initial_music_track_number_from_seed(
    selected_track: usize,
    random_order: bool,
    seed: u64,
) -> usize {
    let selected = bounded_music_track(selected_track);
    if !random_order {
        return selected;
    }

    let slot = seed % MUSIC_TRACK_COUNT_U64;
    let candidate = usize::try_from(slot).map_or(MUSIC_TRACK_MIN, |value| {
        value.saturating_add(MUSIC_TRACK_MIN)
    });
    let bounded_candidate = bounded_music_track(candidate);
    if bounded_candidate == selected {
        next_music_track(bounded_candidate)
    } else {
        bounded_candidate
    }
}

/// Returns a best-effort runtime seed for random music ordering.
fn random_music_seed() -> u64 {
    let time_seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0_u64, |duration| {
            duration.as_secs() ^ u64::from(duration.subsec_nanos())
        });
    time_seed ^ u64::from(std::process::id())
}

/// Clamps a one-based CTP2 track number to the available track range.
const fn bounded_music_track(track_number: usize) -> usize {
    if track_number < MUSIC_TRACK_MIN {
        MUSIC_TRACK_MIN
    } else if track_number > MUSIC_TRACK_MAX {
        MUSIC_TRACK_MAX
    } else {
        track_number
    }
}

/// Returns the next one-based CTP2 track number, wrapping at the end.
const fn next_music_track(track_number: usize) -> usize {
    if track_number >= MUSIC_TRACK_MAX {
        MUSIC_TRACK_MIN
    } else {
        track_number.saturating_add(1_usize)
    }
}

/// Loads an optional original CTP2 UI sound chunk.
fn load_optional_sound_chunk(
    installation_root: &Path,
    entry_name: &str,
    label: &str,
) -> Option<Chunk> {
    match load_sound_chunk(installation_root, entry_name) {
        Ok(chunk) => Some(chunk),
        Err(error) => {
            log::warn!("{label} is unavailable: {error}");
            None
        }
    }
}

/// Plays a loaded chunk once on the first available channel.
fn play_loaded_chunk(chunk: Option<&Chunk>, label: &str) {
    let Some(chunk) = chunk else {
        return;
    };
    if let Err(error) = mixer::Channel::all().play(chunk, 0_i32) {
        log::warn!("{label} could not be played: {error}");
    }
}

/// Loads one original CTP2 WAV into an SDL chunk.
fn load_sound_chunk(
    installation_root: &Path,
    requested_entry: &str,
) -> Result<Chunk, MainMenuError> {
    let (_entry_name, bytes) = load_sound_wav_bytes(installation_root, requested_entry)?;
    let rwops = RWops::from_bytes(bytes.as_slice()).map_err(|message| MainMenuError::Music {
        message: format!("failed to create RWops for {requested_entry}: {message}"),
    })?;
    rwops.load_wav().map_err(|message| MainMenuError::Music {
        message: format!("failed to decode {requested_entry}: {message}"),
    })
}

/// Loads original CTP2 WAV bytes from the immutable game archive.
fn load_sound_wav_bytes(
    installation_root: &Path,
    requested_entry: &str,
) -> Result<(String, Vec<u8>), MainMenuError> {
    let archive_path = installation_root.join(MENU_SOUND_ARCHIVE);
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
        .find(|candidate| candidate.name.eq_ignore_ascii_case(requested_entry))
        .cloned()
        .ok_or_else(|| MainMenuError::Music {
            message: format!(
                "{requested_entry} was not found in {}",
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
) -> Result<SoundAssetSummary, MainMenuError> {
    sound_summary_for_test(installation_root, BUTTON_CLICK_SOUND_ENTRY)
}

/// Returns test summaries for the original slider sound assets.
#[cfg(test)]
pub(in crate::main_menu) fn slider_sound_summaries_for_test(
    installation_root: &Path,
) -> Result<Vec<SoundAssetSummary>, MainMenuError> {
    Ok(vec![
        sound_summary_for_test(installation_root, SLIDER_UP_SOUND_ENTRY)?,
        sound_summary_for_test(installation_root, SLIDER_DOWN_SOUND_ENTRY)?,
    ])
}

/// Returns a test summary for one original UI sound asset.
#[cfg(test)]
fn sound_summary_for_test(
    installation_root: &Path,
    requested_entry: &str,
) -> Result<SoundAssetSummary, MainMenuError> {
    let (entry_name, bytes) = load_sound_wav_bytes(installation_root, requested_entry)?;
    let mut header = [0_u8; 4];
    let header_bytes = bytes.get(0..4).ok_or_else(|| MainMenuError::Music {
        message: format!("{requested_entry} is too short to contain a RIFF header"),
    })?;
    header.copy_from_slice(header_bytes);
    Ok((
        MENU_SOUND_ARCHIVE.to_owned(),
        entry_name.to_ascii_lowercase(),
        bytes.len(),
        header,
    ))
}

/// Returns the mixer chunk size used for tests.
#[cfg(test)]
pub(in crate::main_menu) const fn mixer_chunk_size_for_test() -> i32 {
    MIXER_CHUNK_SIZE
}

/// Returns the allocated mixer channel count used for tests.
#[cfg(test)]
pub(in crate::main_menu) const fn mixer_allocated_channels_for_test() -> i32 {
    MIXER_CHANNEL_COUNT
}
