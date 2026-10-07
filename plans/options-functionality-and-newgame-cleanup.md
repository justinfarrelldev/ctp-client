# Options Functionality and New Game Cleanup Plan

## Scope

Fix the CTP2 single-player/new-game/options shell behavior and visual mismatches reported after the picker fidelity pass. Validate against original LDL/source references, keep original assets immutable, and make modeled interactions functional where they currently only navigate or display.

## Findings to Verify Against Original References

- Empire/tribe picker should behave like original modal radio group: selecting a tribe changes the selection but does not close the popup; Back/close returns to New Game.
- New Game screen currently contains placeholder/list controls that are visually wrong and likely not present in the top shell view; compare with `SPNewGameWindow` and remove or restyle incorrect controls.
- Scenario and Map Editor placement must be validated between `SPWindow` and `SPNewGameWindow` in original LDL/source before changing.
- Options root currently contains unsupported/odd controls (`Quit To Shell`, central `Quit To Windows`) that need original validation and either exact styling/placement or removal/disable treatment.
- Graphics `Accept` is an unstyled decoration, unit speed slider is a placeholder, and resolution picker selection only updates model text rather than the SDL window mode.
- Music options should update the active track/state; sound/voice/music volume sliders should affect runtime mixer state where possible.

## Checklist

### Reference review and tests first

- [x] Read original `spnewgame.ldl`, `options.ldl`, `graphics.ldl`, `sound.ldl`, `musicscreen.ldl`, and matching C++ callbacks for reported controls.
- [x] Add/adjust model tests proving tribe selection stays in `NationPicker` until Back.
- [x] Add/adjust model tests proving New Game screen omits/removes placeholder controls that are not on the original top-level New Game shell.
- [x] Add tests for Options/Graphics/Sound/Music controls and callback-like outcomes for resolution, volume, music track, and toggles.
- [x] Add headless SDL tests for selecting a nation, backing out, applying a resolution, changing music track, and returning via Accept.

### Implementation

- [x] Add non-closing setup selection action for tribe/radio-style pickers and wire Empire picker to it.
- [x] Clean New Game screen controls to match original shell: remove/reposition incorrect placeholder buttons and ensure scenario/map-editor placement matches original.
- [x] Correct Options root controls: remove nonsensical unsupported entries or match original placement/style exactly, and style Accept/Back controls consistently.
- [x] Add original slider/button art metadata for graphics/sound sliders and popup accept/image buttons.
- [x] Implement runtime option effects: resolution resize/window mode, music track changes, volume state application, and music toggle behavior.
- [x] Keep all asset reads immutable from original archives.

### Verification and delivery

- [x] Run `cargo fmt`.
- [x] Run `cargo make lint` and fix all issues.
- [x] Run `cargo make test` and fix failures.
- [x] Run `cargo build`.
- [x] Run `./scripts/check-coverage-fast.sh` if present; record if unavailable (`coverage script unavailable: ./scripts/check-coverage-fast.sh`).
- [x] Run `cargo make lint-fix` before commit.
- [x] Commit the completed correction atomically.
