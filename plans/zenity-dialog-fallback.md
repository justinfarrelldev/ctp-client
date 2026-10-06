# Plan: Remove zenity dependency from play-time dialogs

## Goal

Fix the Play Game workflow so it no longer fails when the host Linux system does not have `zenity` installed.

## Checklist

- [x] Reproduce and isolate the `rfd`/zenity dialog path used when starting a game.
- [x] Add tests for dialog copy, extraction error formatting, and affirmative/cancel response handling without opening real dialogs.
- [x] Implement GTK-native modal dialogs for extraction confirmation and extraction errors.
- [x] Replace `rfd::MessageDialog` usage in the GTK Play Game path with GTK-native dialogs.
- [x] Replace the non-test extraction prompt/error helpers in `main.rs` so no play-time dialog requires zenity.
- [x] Run `cargo make lint` and fix all issues.
- [x] Run the test suite and coverage check, documenting any unavailable tooling.
  - `cargo make test` passed.
  - `./scripts/check-coverage-fast.sh` is not present in this repository.
- [x] Run a build check.
  - `cargo build` passed.
- [x] Run `cargo make lint-fix`, stage, and commit the fix.
