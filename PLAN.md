# Plan for Intro Video Skip Feature

## Objectives
- Ensure players can skip intro videos using keyboard, mouse, or controller input.
- Maintain accessibility by providing immediate, low-effort skip controls.
- Preserve compatibility with existing video playback pipeline and error handling.

## Checklist
- [x] Analyze current video playback implementation and process spawning behavior.
- [x] Design input handling architecture that works alongside external video player processes.
- [x] Define precise skip triggers for keyboard (Space/Enter/Escape), mouse (primary click), and controller (A/B/Start) interactions.
- [x] Establish clear communication channel between input handler and video process (e.g., signals or IPC) for clean termination.
- [x] Draft automated tests covering keyboard, mouse, and controller skip scenarios, plus non-skip completion path.
- [x] Implement mockable abstractions for process management and input polling to enable TDD.
- [x] Implement runtime input handling loop that monitors user input while video plays.
- [x] Integrate controller support via SDL2 with robust initialization/teardown and error reporting.
- [x] Ensure accessibility messaging/logging for skip functionality.
- [x] Run `cargo make lint` and address all findings.
- [x] Run `cargo make test` and confirm all tests pass.
- [x] Run `./scripts/check-coverage-fast.sh` to verify coverage threshold (script missing in repo; documented in status report).
- [x] Perform final code review for performance and maintainability.
- [x] Update documentation/comments reflecting new skip controls.
- [x] Prepare commit after running `cargo make lint-fix`.


## Post-review follow-up

- [x] Split `src/video.rs` into focused submodules to keep each file under 1000 LOC.
- [x] Expand overlay tests to cover controller instructions in the window title and bitmap rendering integrity.
- [x] Resolve outstanding review issues: controller skip pattern, missing error string, joystick subsystem documentation, startup controller removal handling, layout constant documentation, and error context naming consistency.

## 2025-10-05 Update

- [x] Pre-initialize the SDL skip overlay before spawning video players to eliminate `BadAccess` failures on fullscreen backends.
- [x] Enable keyboard and mouse grabs so Space/Esc/click/controller Guide skip shortcuts operate while the external player window has focus.
- [x] Verified intro playback orchestration cleanly hands control back to the main menu workflow after skips or completions.

## 2025-10-06 Global Input Tracking

- [x] Prototype replacement for overlay-dependent skip handling with a global input tracker abstraction.
- [x] Implement device-query backed provider and unit tests for keyboard, mouse, and steady-state behavior.
- [x] Validate SkipEventListener wiring with global tracker via focused tests.
- [x] Refresh accessibility and behavior documentation to note direct video window interaction.
- [x] Re-run formatting, lint, tests, coverage, and build targets before finalizing changes.
