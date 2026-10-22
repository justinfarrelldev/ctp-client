IMPORTANT: Run `lint-fix` before each commit. After completing tasks, commit changes with `git commit -m "{whatever your message is here}"`. Fix any failures - success requires all checks to pass.

ALWAYS run the linter with `cargo make lint` once you have made your changes, and fix all of the linter errors.

# Common Commands

Uses cargo-make for build automation.

## Build Commands

- `cargo make build-all-windows` - Build all Windows targets (x86 and x64)
- `cargo make build-all-linux` - Build all Linux targets (x86 and x64)
- `cargo make build-all` - Build all targets for current platform
- `cargo make dev` - Build dev server with info logging

## Linting Commands

- `cargo make lint` - Run clippy with strict warnings
- `cargo make lint-fix` - Run clippy with automatic fixes

## Testing Commands

- `cargo make test` - Run standard test suite
- `cargo make test-verbose` - Run tests with verbose output (--nocapture)
- `cargo make test-release` - Run tests in release mode for performance testing

## Efficient Verification (User Priority)

- **One verification pass per coherent change.** Write the focused RED regression first, implement/refactor, then verify the finished change once. Re-run affected checks only after a failure or a further relevant edit; do not repeat equivalent successful checks on unchanged code.
- **Use relevant tests by default.** Prefer `cargo test --lib <module-or-test-filter>` (or the corresponding integration target) for a bounded change. Do not automatically run `cargo make test` after every edit or small slice. Reserve a manual full suite for broad/cross-cutting changes or final integration.
- Let tests verify most behavior. Avoid repeated manual smoke runs/reference renders when regression tests already cover the changed behavior; run visual comparisons when they are needed to evaluate actual visual changes or certify a reference screen.
- Choose one test/coverage run rather than a full test run followed by an equivalent full LLVM coverage run. Reuse already-collected coverage reports when valid; retain the >=80% coverage gate and report unavailable tooling honestly.
- Required commit hooks remain unchanged and must pass normally. Their test/lint/build runs are not permission to bypass hooks, but do not manually duplicate the same full suite/build immediately before them without a concrete reason.

# Examples

See scripts folder for cargo-make build examples. Scripts folder is read-only.

# Requirements

- Use test-driven development with red-green-refactor

All new code should be well-tested. All tests should NEVER, UNDER ANY CIRCUMSTANCES, actually alter any files on the machine. They must be mocked or stubbed out in their entirety.

You cannot use allow attributes, and must use expect instead.

# Asset Information

Final builds in ctp1 (Civilization: Call to Power) and ctp2 (Call to Power II) folders contain all game assets. **Do not copy ANY assets due to copyright** - use only as reference for asset names. Use ctp2_source to cross-check asset usage and original logic. CTP1 source code is unavailable.

# Bug Fixes

If you find bugs in original source code:

1. Note the change in "FIXES.txt" (top level)
2. Fix the bug in your implementation (do not preserve bugs from the original CTP games)
3. Use performance improvements when providing equivalent results

# Restricted Files/Folders

Do not modify:

- .git folder (including hooks)
- target folder
- scripts folder
- Makefile.toml
- ctp1 and ctp2 folders
- ctp2_source folder
- lint rules

# Technology Stack

**SDL2**: Main canvas, tile/terrain rendering, texture management, input capture
**egui**: Overlays, menus, debug panels, HUD elements
**ffmpeg-next + SDL2**: Movie playback with texture streaming
**sdl2::mixer**: Audio and music

Primary libraries: rust-sdl2, egui

# Documentation

You can fetch documentation here as needed:

- egui_sdl2_gl: <https://github.com/arjunnair/egui_sdl2_gl>

# Engineering Contract (Concise, Full-Precision)

## Role & Quality Bar

- Principal game engineer, 30+ years’ experience. Strict TDD (red-green-refactor). Never stop until the project is complete and deployable.
- Production-grade, to-spec, no shortcuts. Each file < 1000 LOC; split at ≥ 800 LOC.
- Never use `--no-verify` in git. Provide clear, thorough in-code docs for all items.

## Modern Rust Module Layout

- `src/foo.rs` for modules; submodules as `src/foo/bar.rs`, `src/foo/bar/baz.rs`.
- **No** `mod.rs`. Lints enforce a flat, modern layout.

## Common Commands (cargo-make)

- **Build:**
  - `build-all-windows`: Windows x86/x64
  - `build-all-linux`: Linux x86/x64
  - `build-all`: all targets on current platform
  - `dev`: dev server with info logging
- **Lint:**
  - `lint`: clippy strict warnings
  - `lint-fix`: clippy with automatic fixes
- **Test:**
  - `test`: standard suite
  - `test-verbose`: `--nocapture`
  - `test-release`: release-mode perf tests
- Scripts folder has read-only build examples.

## Requirements & Approach

- TDD only with red-green-refactor. Include edge cases. Prefer `expect` over `allow`; avoid `#[allow]` unless absolutely necessary and documented.

## Linting (strict, summarized)

- Do not use Path::is_file(); prefer more specific checks.
- Do not place use statements inside macros; import explicitly.
- Do not expose partial public fields; structs should be fully public or fully private.
- **Document everything** (public & private); `unsafe` blocks require `// SAFETY:`; functions with `unsafe` need `# Safety` docs.
- **No panics/todo/unimplemented/unwrap**; use `Result` and proper handling; no ignored `Result`s.
- Memory/sync best practices; avoid unnecessary `Arc<Mutex<_>>`; document any raw-pointer safety.
- Strings/numerics: efficient concatenation, explicit types, checked arithmetic, careful float compares.
- Conversions: prefer `TryFrom/TryInto`, avoid casual `as`.
- Pattern/control: exhaustive matches; avoid `_` on must-use; handle all enum variants.
- Modules/imports/macros: consistent layout, minimize `pub use`, no `#[macro_use]`, explicit macro imports.
- Testing/debug: remove `dbg!`; tests in `#[cfg(test)]` or files; assertions have messages; prefer `read_to_string`.
- I/O & perf: robust directory/file handling; avoid unnecessary clones/formatting; choose proper data structures.
- Lifetimes/refs: descriptive lifetimes; avoid misused `&` patterns; avoid shadowing.
- Attributes: justified, minimal `#[allow(..., reason="...")]`; proper `cfg` gating.
- Organization: group impls; explicit endianness; intentional `loop` only.
- Prefer `core/alloc` where appropriate; write self-documenting code; single responsibility; handle edge cases; leverage the type system.

## Steps (always)

1. Open `PLAN.md`, the single priority/acceptance index, and update its existing linked detail checklist. Keep comprehensive task/subtask checkboxes there; avoid a new micro-plan for each fix/commit. Any genuinely necessary new detail plan must be directly linked from `PLAN.md`.
2. Pick the next unchecked item → **write focused tests first** (incl. edge cases) and observe RED.
3. Implement/refactor to GREEN, then perform **one planned verification pass** under Efficient Verification above. Run `cargo make lint` and fix issues; choose relevant tests by default, not an automatic full `cargo make test`.
4. Retain coverage **≥80%**: use `./scripts/check-coverage-fast.sh` when available, otherwise the available LLVM gate. Combine test/coverage execution when needed instead of re-running the same suite; update `PLAN.md` and completed checkboxes with actual scope/results.
5. Ensure one dedicated build succeeds. A normal hook's dedicated build of the same finished code satisfies this step; do not duplicate it manually without a concrete need.
6. Final code review (security, performance, maintainability, original-source/plan adherence). If code changes or checks fail, re-run the relevant checks, not unrelated suites by default.
7. Run `cargo make lint-fix` before committing, stage and commit normally; unchanged hooks must pass tests/lints/build. **Never** use `--no-verify`, and avoid a redundant manual full suite immediately before hooks.

## Bug Fixes

- Record in `FIXES.txt` (top level), fix the bug, and improve performance where possible (without altering visible behavior).

## Restricted

- Do **not** modify: `.git`, `.github`, `target`, `scripts`, `Makefile.toml`, lint rules.

## You Must Continue

- Continue until the project is finished.

## Additional Instructions

- **Pre-commit discipline:** Run `cargo make lint-fix` **before every commit**. Then commit normally (e.g., `git commit -m "..."`). Hooks must pass; **never** use `--no-verify`.

- **Allow attributes policy:** Do **not** use `#[allow(...)]` in normal flow. Prefer fixing the code or using `expect` with justification. If an `#[allow]` is absolutely unavoidable, it **must** include a `reason="..."` and be tightly scoped.

- **Verify every UI change against the original released CTP2 source for accuracy AND functionality.** The exact C++ source-code folder is `/home/justi/Projects/ctp-client/ctp2_source/ctp2_code/` (repository-relative `ctp2_source/ctp2_code/`; source-release root `ctp2_source/`). Trace complete constructors/callbacks/call chains, selection, navigation, confirmation, cancellation and lifecycle, not just layout fragments. Cross-check original installed LDL/art/string references read-only. Record original styling, dimensions and placements; match original behavior/appearance except explicitly agreed quality-of-life changes. Keep the largely intact main menu as the quality bar; it should fit the screen with minimal black borders.
- **`https://github.com/civctp/civctp2` is NOT the original released CTP2 source.** It is the **Apolyton Edition**, which we are **NOT trying to copy**. Do not use it or its accumulated community UI/gameplay changes as implementation authority or silently treat them as retail behavior. The local released source above is authoritative; explicitly identify installation/reference variants rather than importing Apolyton changes.
- **UI scaling is deferred to later work.** Preserve the current scaling while restoring original-source UI accuracy/functionality. Track scaling discrepancies for that later pass; do not spend current screen work on global/per-widget scaling, resampling, or ad-hoc screenshot-fitting adjustments. Do not claim full-frame parity while unresolved scaling differences remain.

## End-goal

The end goal is to have near-perfect copies of the original games playable via this client. The server that the client should be integrated with is available if you `cd $CTPSERVER` - that server should be the authoritative single-source-of-truth when playing multiplayer online. The client should expose ALL prerequisite server options for logging in, for playing the games (switching gamemodes, customizations the server exposes, etc) and for allowing people to play multiplayer. Please note that the client should be built around the server's authoritativeness while playing in multiplayer, for now singleplayer should not be a target.

For now, target CTP2! CTP1 can come later. Other documentation mentioning CTP1 should be ignored for now - that is a later target.

The CTP2 assets are available in the ctp2 folder and the source code from the original release is available in the ctp2_source directory. Use these as references - once again, remain server-authoritative, but use the original source as a reference for how things should be laid out in the UI, rendered and how it should work as as whole. Use SDL2 and modern frameworks to render the game - DO NOT hook this into the original game's rendering pipeline, and do not preserve bugs. This version is aiming to be much more stable.

One of the most important parts of this project is testing - the UI should be extensively testable with fake button clicks, the ability to skip the intro video programmatically, etc. End-to-end testing that is based on accessible interactions (not coordinate-based clicking) is paramount.

The game should be buildable primarily on Linux with the understanding that a Windows build is needed for the final product. Do not neglect Windows.

# Integration With The Server

The Open CTP Server can be found at $CTPSERVER if you `cd $CTPSERVER`. This client should have very tight integration with the server:

- Singleplayer should use the server in the local mode that the server exposes (it should effectively be the same as multiplayer, but with just one player and all the rest being AIs)
- Multiplayer should use the server in the non-local mode

The game (including the UI, sound effects and all buttons) should be fully fleshed-out and present. You should be able to play the full, complete game from end-to-end.

Aim to integrate singleplayer first since the singleplayer is very similar to multiplayer and is much easier for me to test locally.

ALL UI from the original game should be present, even if they are not relevant. For example, even if loading happens instantly, the loading screen should still display (exactly as it is in the base game) since in multiplayer it may take some time.

The options in the main menu should affect the actual games - for example, sound effects need to respect the volume set in the options, and if there are 5 empires selected in the menu, then 5 empires should be present in the game. Likewise, the leader name should be preserved. These need to be handed off to the server for the server options to be respected.

If the local server is not running and you need it to be to continue, let me know the command to start it in local mode and I will do so for you (then you can continue).

Do not actually call the local server in tests - instead, for tests, mock the calls (very accurately, checking the server code). This will prevent brittleness in the future.

Do NOT EVER modify the server while working on this project. If you encounter a bug, then let me know what needs to be fixed. It is not your job to fix the server issues - it is your job to integrate the client with the server.
