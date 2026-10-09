# Random Music Startup, Click Latency, and Slider Sounds Plan

## Checklist

- [x] Document original CTP2 slider sound mappings from source (`GAMESOUNDS_SLIDERUP` -> `CSLIDEUP.WAV`, `GAMESOUNDS_SLIDERDOWN` -> `CSLIDEDN.WAV`).
- [x] Add red tests proving random-order startup chooses a randomized music track instead of always reusing the persisted selected track.
- [x] Add red tests proving slider sound assets resolve from the original `sound.zfs` archive.
- [x] Add red tests proving slider movement queues the slider-up sound and slider release queues the slider-down sound.
- [x] Add red tests or helper assertions for low-latency mixer settings used by UI SFX.
- [x] Implement deterministic/testable initial random-track selection and use it on startup when Random Order is enabled.
- [x] Play button-click audio before action handling mutates screens to remove perceived click lag.
- [x] Reduce SDL_mixer SFX buffer latency and allocate enough channels for overlapping UI SFX.
- [x] Load `CSLIDEUP.WAV` and `CSLIDEDN.WAV` as original slider chunks.
- [x] Play slider-up on value-changing slider moves and slider-down on slider release.
- [x] Ensure sound/music volume settings still apply to all UI SFX.
- [x] Run `cargo fmt`.
- [x] Run `cargo make lint` and fix issues.
- [x] Run `cargo make test` and fix failures.
- [x] Run coverage script if present and record unavailability if missing. Coverage note: `./scripts/check-coverage-fast.sh` is not present/executable in this checkout.
- [x] Run dedicated build.
- [x] Run `cargo make lint-fix` before commit.
- [x] Update this plan checklist.
- [x] Commit changes.
