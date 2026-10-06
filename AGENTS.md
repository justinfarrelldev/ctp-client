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

1. Open or create a plan file in the plans folder with a comprehensive checklist (all steps/substeps to complete the task at-hand). If the task should be broken into multiple plans, then do that with a reference to the next plan in the chain at the bottom (for example, "Once finished with this plan, see plan2 and finish that").
2. Pick the next unchecked item → **write tests first** (incl. edge cases).
3. Run `cargo make lint` and fix all issues; run tests; ensure coverage **≥ 80%** via `./scripts/check-coverage-fast.sh`.
4. Implement; re-lint; re-test; update PLAN.md and check off completed items.
5. Run a dedicated build step to ensure the app compiles cleanly.
6. Final code review (security, performance, maintainability, plan adherence).
7. Stage and commit; commit hooks must pass tests & lints. **Never** use `--no-verify`.

## Bug Fixes

- Record in `FIXES.txt` (top level), fix the bug, and improve performance where possible (without altering visible behavior).

## Restricted

- Do **not** modify: `.git`, `.github`, `target`, `scripts`, `Makefile.toml`, lint rules.

## You Must Continue

- Continue until the project is finished.

## Additional Instructions

- **Pre-commit discipline:** Run `cargo make lint-fix` **before every commit**. Then commit normally (e.g., `git commit -m "..."`). Hooks must pass; **never** use `--no-verify`.

- **Allow attributes policy:** Do **not** use `#[allow(...)]` in normal flow. Prefer fixing the code or using `expect` with justification. If an `#[allow]` is absolutely unavoidable, it **must** include a `reason="..."` and be tightly scoped.

## End-goal

The end goal is to have near-perfect copies of the original games playable via this client. The server that the client should be integrated with is available if you `cd $CTPSERVER` - that server should be the authoritative single-source-of-truth when playing multiplayer online. The client should expose ALL prerequisite server options for logging in, for playing the games (switching gamemodes, customizations the server exposes, etc) and for allowing people to play multiplayer. Please note that the client should be built around the server's authoritativeness while playing in multiplayer, for now singleplayer should not be a target.

For now, target CTP2! CTP1 can come later. Other documentation mentioning CTP1 should be ignored for now - that is a later target.

The CTP2 assets are available in the ctp2 folder and the source code from the original release is available in the ctp2_source directory. Use these as references - once again, remain server-authoritative, but use the original source as a reference for how things should be laid out in the UI, rendered and how it should work as as whole. Use SDL2 and modern frameworks to render the game - DO NOT hook this into the original game's rendering pipeline, and do not preserve bugs. This version is aiming to be much more stable.

One of the most important parts of this project is testing - the UI should be extensively testable with fake button clicks, the ability to skip the intro video programmatically, etc. End-to-end testing that is based on accessible interactions (not coordinate-based clicking) is paramount.

The game should be buildable primarily on Linux with the understanding that a Windows build is needed for the final product. Do not neglect Windows.
