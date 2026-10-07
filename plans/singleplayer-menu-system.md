# Single-player Shell Menu Recreation Plan

## Scope

Recreate the CTP2 single-player shell flow under the landing page `Single Player` button with immutable reads of original assets only. The implementation must be testable through accessible controls and original-reference validation, not coordinate-only tests.

## Source References

- `ctp2/ctp2_data/english/uidata/layouts/spnewgame.ldl`
- `ctp2/ctp2_data/english/uidata/layouts/spnewgamepopups.ldl`
- `ctp2/ctp2_data/english/uidata/layouts/options.ldl`
- `ctp2/ctp2_data/english/uidata/layouts/graphics.ldl`
- `ctp2/ctp2_data/english/uidata/layouts/sound.ldl`
- `ctp2/ctp2_data/english/uidata/layouts/musicscreen.ldl`
- `ctp2_source/ctp2_code/ui/interface/spscreen.cpp`
- `ctp2_source/ctp2_code/ui/interface/spwindow.cpp`
- `ctp2_source/ctp2_code/ui/interface/spnewgamescreen.cpp`
- `ctp2_source/ctp2_code/ui/interface/spnewgamewindow.cpp`
- `ctp2_source/ctp2_code/ui/interface/optionsscreen.cpp`
- `ctp2_source/ctp2_code/ui/interface/optionswindow.cpp`
- `ctp2_source/ctp2_code/ui/interface/graphicsscreen.cpp`
- `ctp2_source/ctp2_code/ui/interface/soundscreen.cpp`
- `ctp2_source/ctp2_code/ui/interface/musicscreen.cpp`

## Checklist

### Model and validation

- [x] Add a reusable shell screen model with original asset references, labels, controls, accessibility metadata, and validation helpers.
- [x] Model the `SPWindow` single-player hub with every visible and hidden LDL control, original background `upsg002`, and exact geometry/styles.
- [x] Model `SPNewGameWindow` with all controls up to launch, original labels, fields, tab headings, decorative/error placeholders, background, and exact geometry/styles.
- [x] Model options entry screens used before launch: `OptionsWindow`, `GraphicsWindow`, `SoundWindow`, and `MusicScreen`/track picker metadata.
- [x] Model functional option state for SFX/music/voice volumes, music on/off/style/track, screen resolution, and Windows-resolution flag.
- [x] Add validation tests for every modeled element against the original LDL-derived values and referenced source behavior.

### Navigation and interaction

- [x] Add accessible activation for `Single Player` that transitions to `SPWindow` without leaving the menu runtime.
- [x] Add accessible activation for `New Game`, `Options`, `Back`, `Quit`, and `Launch`/Play flow.
- [x] Add option-control interactions for volume sliders, resolution selection, and music toggles/track selection.
- [x] Add sound-effect policy metadata for click/hover behavior and tests proving every actionable control has original button sound semantics.

### Rendering

- [x] Generalize SDL rendering so shell screens render original backgrounds, buttons, static labels, text fields, sliders/switches, and list items using immutable original assets.
- [x] Ensure original backgrounds are visible for landing, single-player hub, new-game, and options screens.
- [x] Render hover and button layers with original `CTP2_BUTTON_SIZABLE` layering for every shell button style.
- [x] Add headless smoke tests for landing -> single-player -> new game -> options -> back -> launch/quit flows.

### Verification and delivery

- [x] Run `cargo fmt`.
- [x] Run `cargo make lint` and fix all issues.
- [x] Run `cargo make test` and fix failures.
- [x] Run `cargo build`.
- [x] Run `./scripts/check-coverage-fast.sh` if present; record if unavailable (`coverage script unavailable: ./scripts/check-coverage-fast.sh`).
- [x] Run `cargo make lint-fix` before commit.
- [x] Commit the completed single-player shell recreation atomically.
