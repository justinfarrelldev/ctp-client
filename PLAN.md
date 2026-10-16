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

## 2025-10-07 Asset Extraction Pipeline

- [x] Audit existing installation and Play Game workflow to determine integration points for archive extraction.
- [x] Select cross-platform user data directory (AppData equivalent) and document rationale in code comments.
- [x] Enumerate all ZFS archives across bundled installations and verify availability before extraction.
- [x] Document archive purposes and hierarchy in `ZFS_ASSET_NOTES.md` alongside this plan.
- [x] Design and land initial test coverage for validation, extraction success, and missing archive errors.
- [x] Expand tests for manifest persistence, disk space failures, and progress callbacks.
- [x] Finalize extraction pipeline refactor (remove legacy backend trait usages, stabilise summary struct consumers).
- [x] Integrate extraction invocation into Play Game action with sensible progress reporting and error messaging.
- [x] Run lint, tests, coverage (or document missing script), and platform build targets.
  - `cargo make lint` passed.
  - `cargo make test` passed.
  - `./scripts/check-coverage-fast.sh` is not present in this repository.
  - `cargo build` passed; `cargo make build-all` was attempted and is blocked by missing i686 GLib/pkg-config cross-compilation sysroot configuration on this host.

## CTP2 Terrain Pixel Seam Repair

See [the detailed plan and red/green evidence](plans/terrain-pixel-seams.md).

- [x] Reproduce tile-edge damage with actual SDL pixel readback on deterministic application-generated maps using installed original art (read-only).
- [x] Fix rectangular fog spill, zoomed edge culling, and undersized fallback ground; keep mountain/forest overhang transparency outside the flat-ground contract.
- [x] Test all four grassland/plains/water edges, full ground interiors, zoom 1-3, generated and mixed fog, and single-pixel negative controls. No snapshots, copied assets, live server, or file-mutating new tests.
- [x] Run strict lint, full tests, dedicated build, coverage measurement and final review. Terrain renderer line coverage: 93.88%; overall coverage: 76.56% (below the repository-wide 80% target, principally unrelated GTK/video paths). Required fast coverage script remains absent.
- [x] Conduct final review, update documentation, and prepare commit after running `cargo make lint-fix`.

## CTP2 Original-Art Gameplay Fidelity Pass

See [the detailed checklist and red/green evidence](plans/pixel-perfect-gameplay.md) and [snapshot usage and remaining limitations](GAMEPLAY_SNAPSHOTS.md).

- [x] Compose the original 800×600 logical frame, full terrain transitions/hats/rivers, native actors/reticles, layered controls, dropdowns and radar chrome; leave the main menu intact.
- [x] Provide offline previews and F12 capture with an isolated file sink; all new screenshot/PNG tests stay in memory and game-server calls remain mocked.
- [x] Fix stacked-unit identity/ownership, hidden order hit boxes, no-selection order appearance, empty authoritative projections and local fog revealing unknown geography.
- [x] Accept optional authoritative resource IDs and render original resource SPR actors with visibility-safe remembered shading; never invent missing server metadata.
- [x] Add source-verified river/resource/ownership omissions and visibility-safe recommendations only to the explicitly authorized server report; no server implementation changes.
- [x] Inspect 640×480, native 800×600 and larger 1280×960 captures. Document synthetic sightlines and remaining radar/dialog/stat/workflow limitations without claiming all-screen pixel parity.
- [x] `cargo make lint-fix`, strict lint, full tests and dedicated native build pass. LLVM line coverage: **80.24%**. Required fast coverage script is absent; Windows binary validation remains environment-dependent.
- [x] Review restricted paths, asset copyright, file sizes and authority boundaries; prepare normal commit with verification enabled.
