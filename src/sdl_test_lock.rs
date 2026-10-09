//! Shared process-wide SDL test serialization.

use std::sync::{Mutex, MutexGuard};

/// Global guard serializing SDL initialization in Rust tests.
static SDL_TEST_LOCK: Mutex<()> = Mutex::new(());

/// Locks all SDL-using tests in this process.
///
/// # Errors
///
/// Returns a message if a previous SDL test panicked while holding the lock.
pub fn lock_sdl_for_test() -> Result<MutexGuard<'static, ()>, String> {
    SDL_TEST_LOCK
        .lock()
        .map_err(|error| format!("SDL test lock was poisoned: {error}"))
}
