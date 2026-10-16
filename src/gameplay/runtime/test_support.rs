//! Restorable dummy SDL drivers for in-memory framebuffer tests.

use sdl2::render::Canvas;
use sdl2::video::Window;
use std::ffi::{OsStr, OsString};

/// Scoped environment changes; callers must hold the shared SDL test lock.
pub(super) struct HeadlessSdlEnvironment {
    /// Previous video driver.
    previous_video_driver: Option<OsString>,
    /// Previous audio driver.
    previous_audio_driver: Option<OsString>,
}

impl HeadlessSdlEnvironment {
    /// Applies dummy drivers while the caller holds the SDL test lock.
    pub(super) fn apply() -> Self {
        let environment = Self {
            previous_video_driver: std::env::var_os("SDL_VIDEODRIVER"),
            previous_audio_driver: std::env::var_os("SDL_AUDIODRIVER"),
        };
        // SAFETY: Every caller holds the shared SDL test lock throughout this
        // scope; Drop restores both variables before that lock is released.
        unsafe {
            std::env::set_var("SDL_VIDEODRIVER", "dummy");
            std::env::set_var("SDL_AUDIODRIVER", "dummy");
        }
        environment
    }

    /// Restores immediately, including when a test returns an error.
    pub(super) fn restore(self) {
        drop(self);
    }
}

impl Drop for HeadlessSdlEnvironment {
    fn drop(&mut self) {
        restore_env_var("SDL_VIDEODRIVER", self.previous_video_driver.as_deref());
        restore_env_var("SDL_AUDIODRIVER", self.previous_audio_driver.as_deref());
    }
}

/// Runs readback/texture assertions on a dummy renderer without file I/O.
pub(super) fn with_canvas(
    size: (u32, u32),
    check: impl FnOnce(&mut Canvas<Window>) -> Result<(), String>,
) -> Result<(), String> {
    let _guard = crate::sdl_test_lock::lock_sdl_for_test()?;
    let _environment = HeadlessSdlEnvironment::apply();
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let window = video
        .window("in-memory gameplay regression", size.0, size.1)
        .hidden()
        .build()
        .map_err(|error| error.to_string())?;
    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|error| error.to_string())?;
    check(&mut canvas)
}

/// Restores one variable under the same shared SDL test lock.
fn restore_env_var(key: &str, value: Option<&OsStr>) {
    // SAFETY: The shared SDL test lock is still held by the caller's scope.
    unsafe {
        match value {
            Some(previous) => std::env::set_var(key, previous),
            None => std::env::remove_var(key),
        }
    }
}
