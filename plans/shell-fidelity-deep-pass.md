# Shell Fidelity Deep Pass Plan

## Scope

Perform an extensive CTP2 base-game fidelity pass for the single-player/New Game/Options shell. Fix concrete regressions reported in option controls, resolution handling, picker styling/layering, leader-name behavior, and spacing. Record additional mismatches in top-level `UI-schisms.md`.

## Checklist

### Reference review

- [x] Re-read original LDL for `spnewgame.ldl`, `spnewgamepopups.ldl`, `options.ldl`, `graphics.ldl`, `sound.ldl`, and `musicscreen.ldl` around affected controls.
- [x] Re-read original source callbacks for `spnewgamewindow`, `spnewgametribescreen`, `optionswindow`, `graphicsscreen`, `soundscreen`, and `musicscreen`.
- [x] Compare current shell builders/runtime against original positions, control types, styling, and layering.
- [x] Record all newly found mismatches in `UI-schisms.md`.

### Tests first

- [x] Add tests for New Game label/button spacing so headers are not cut off.
- [x] Add tests that Empire selection updates both nation and default leader name, remains in picker, and returns via styled Back.
- [x] Add tests for editable/styled leader-name field model behavior.
- [x] Add tests for Graphics switches toggling state.
- [x] Add tests for resolution clicks preserving hit testing / not desynchronizing coordinates after resize.
- [x] Add tests for Accept/Back controls using unsmooshed original-art/fallback sizing.
- [x] Add tests for Difficulty Back layering/placement after the option radio controls.
- [x] Add tests for slider dragging/value updates, not just click-to-set.

### Implementation

- [x] Correct New Game label/button spacing and dimensions using original LDL/source.
- [x] Add leader-name defaults per civilization and update leader name when the Empire picker changes nation.
- [x] Add text input/edit behavior for leader-name text fields with base-game-like styling.
- [x] Style all picker Back/Accept controls with original button art and ensure they are top-layered/clickable.
- [x] Make Graphics switches toggleable and persist state in `ShellOptionsState`.
- [x] Improve Unit Speed slider geometry/rendering and support mouse-drag updates.
- [x] Fix resolution mode changes so mouse hit testing remains aligned and visual scaling does not degrade the 800x600 shell art.
- [x] Re-check Options root navigation/back affordance and add/stylize it according to original popup behavior.

### Verification and delivery

- [x] Run `cargo fmt`.
- [x] Run `cargo make lint` and fix all issues.
- [x] Run `cargo make test` and fix failures.
- [x] Run `cargo build`.
- [x] Run `./scripts/check-coverage-fast.sh` if present; record if unavailable (`coverage script unavailable: ./scripts/check-coverage-fast.sh`).
- [x] Run `cargo make lint-fix` before commit.
- [x] Commit the completed fidelity pass atomically.
