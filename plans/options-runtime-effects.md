# Options Runtime Effects Plan

## Scope

Finish wiring shell options and New Game behavior that currently only changes model state or displays placeholder controls.

## Checklist

- [x] Add tests for non-closing Empire selection and Back returning to New Game.
- [x] Remove New Game placeholder/dropdown panel controls that original `SPNewGameWindow` does not instantiate.
- [x] Add tests for runtime effect derivation: resolution resize request, music track path, music enabled toggle, and volume scaling.
- [x] Add shell pending-effects tracking for resolution, sound/music volumes, music enabled, and music track selection.
- [x] Apply pending resolution to the SDL window/canvas.
- [x] Apply pending music effects through `SDL_mixer` backend when audio is available.
- [x] Improve slider rendering with original `upba2513` thumb art.
- [x] Run `cargo fmt`.
- [x] Run `cargo make lint` and fix all issues.
- [x] Run `cargo make test` and fix failures.
- [x] Run `cargo build`.
- [x] Run `./scripts/check-coverage-fast.sh` if present; record if unavailable (`coverage script unavailable: ./scripts/check-coverage-fast.sh`).
- [x] Run `cargo make lint-fix` before commit.
- [x] Commit runtime effects atomically.
