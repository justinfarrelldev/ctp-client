# Intro Video Test Coverage and CTP2 Main Menu Plan

## Scope

- Add automated tests proving the intro video path is discovered and delegated to a playback backend without launching real players.
- Add automated tests proving the Play Game action always invokes intro video playback through an accessible, testable activation path.
- After committing the intro-video tests, create branch `main-menu` for the main-menu work.
- Recreate the 800x600 Call to Power II landing menu from the original `InitPlayWindow` LDL/source reference.
- Provide accessible button lookup/click APIs for tests instead of coordinate-only input.
- Verify button geometry, labels, and style asset names against the original LDL definitions.
- Start menu music automatically through an injectable backend that can be asserted in tests.

## Checklist

### Follow-up: original button graphics fidelity

- [x] Add tests that pin the renderer to native original button piece dimensions and tiled center pieces instead of procedural/scaled button fills.
- [x] Add tests that button labels use the original dark text color for readability over the original button art.
- [x] Render original CTP2 three-piece button art at native piece height and tile the center piece, matching `CTP2_BUTTON_SIZABLE`.
- [x] Change menu button label rendering to dark original-style text.
- [x] Run lint, tests, build, lint-fix, and commit the button graphics fidelity fix atomically.
  - `cargo make lint` passed.
  - `cargo make test` passed.
  - `./scripts/check-coverage-fast.sh` is not present in this repository.
  - `cargo build` passed.

### Follow-up: original asset button styling and color fidelity

- [x] Correct SDL texture upload format so original background colors are not interpreted as red.
- [x] Render CTP2 shell buttons from original immutable RIM assets with chroma-key transparency instead of fallback rectangles.
- [x] Add tests proving button asset names resolve to packed RIM entries and both button styles load from the original game copy without modification.
- [x] Add a headless render color test to catch alpha-upload-as-red regressions.
- [x] Run lint, tests, build, lint-fix, and commit the styling fix atomically.
  - `cargo make lint` passed.
  - `cargo make test` passed.
  - `./scripts/check-coverage-fast.sh` is not present in this repository.
  - `cargo build` passed.

### Follow-up: headless runtime smoke coverage

- [x] Add a headless real SDL main-menu smoke test using dummy video/audio drivers and a queued quit event.
- [x] Add a headless Play Game transition smoke test that uses a fake intro result and the real SDL main-menu runtime.
- [x] Add a GTK Play Game callback seam test proving the GTK activation path invokes the game-launch service.
- [x] Ensure runtime smoke tests are part of the default `cargo test` suite so UI/runtime changes cannot skip them.
- [x] Run lint, tests, build, lint-fix, and commit the smoke-test process update atomically.
  - `cargo make lint` passed.
  - `cargo make test` passed.
  - `./scripts/check-coverage-fast.sh` is not present in this repository.
  - `cargo build` passed.

### Follow-up: post-video main menu launch bug

- [x] Add tests proving the Play Game transition launches the main menu after completed, skipped, failed, or unavailable intro playback.
- [x] Wire the GTK Play Game callback to the tested transition so the SDL main menu opens after video playback.
- [x] Make menu display resilient to audio initialization failures so music issues do not suppress the menu window.
- [x] Run lint, tests, build, lint-fix, and commit the fix atomically.
  - `cargo make lint` passed.
  - `cargo make test` passed.
  - `./scripts/check-coverage-fast.sh` is not present in this repository.
  - `cargo build` passed.

### Intro video and Play Game tests

- [x] Inspect current video playback and GTK launch flow.
- [x] Add a mockable video playback backend so tests can assert intro-video delegation without starting `ffplay`/VLC.
- [x] Add tests for intro video success/failure paths using a temporary installation containing `gm001.avi`.
- [x] Add a testable Play Game activation helper and tests that verify it invokes intro playback for the selected installation.
- [x] Run focused tests for the new red/green cycle.
- [x] Run `cargo make lint`, `cargo make test`, and coverage/build commands as far as the environment allows.
  - `cargo make lint` passed.
  - `cargo make test` passed.
  - `./scripts/check-coverage-fast.sh` is not present in this repository.
  - `cargo build` passed.
- [x] Run `cargo make lint-fix` and commit the intro-video test work atomically.

### `main-menu` branch

- [x] Create branch `main-menu` from the post-intro-test commit.
- [x] Capture original menu reference values from `ctp2_data/english/uidata/layouts/spnewgame.ldl` and `ui/interface/initialplaywindow.cpp`.
- [x] Build a `MainMenuModel` with accessible IDs/names and click-by-accessible-name support.
- [x] Add tests for all landing-page button labels, accessible lookup, actions, geometry, and original style asset names.
- [x] Add an injectable menu music policy/backend and tests proving autoplay starts the expected CTP2 music.
- [x] Add SDL rendering/runtime for the 800x600 menu with original background/button assets when available and safe procedural fallback.
- [x] Integrate the menu runtime after intro playback.
- [x] Run lint, tests, coverage/build commands, lint-fix, and commit main-menu work atomically.
  - `cargo make lint` passed.
  - `cargo make test` passed.
  - `./scripts/check-coverage-fast.sh` is not present in this repository.
  - `cargo build` passed.

## Source References

- Original landing menu C++: `ctp2_source/ctp2_code/ui/interface/initialplaywindow.cpp`
- Original landing menu actions: `ctp2_source/ctp2_code/ui/interface/initialplayscreen.cpp`
- Original landing menu LDL: `ctp2/ctp2_data/english/uidata/layouts/spnewgame.ldl`
