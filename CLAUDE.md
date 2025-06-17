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

Run `lint-fix` before each commit. After completing tasks, commit changes (triggers linter, tests, and build checks). Fix any failures - success requires all checks to pass.

# Examples

See scripts folder for cargo-make build examples. Scripts folder is read-only.

# Requirements

- Use test-driven development
- Maintain minimum 80% code coverage (enforced by pre-commit hook)

# Asset Information

Final builds in ctp1 (Civilization: Call to Power) and ctp2 (Call to Power II) folders contain all game assets. **Do not copy ANY assets due to copyright** - use only as reference for asset names. Use ctp2_source to cross-check asset usage and original logic. CTP1 source code is unavailable.

# Bug Fixes

If you find bugs in original source code:
1. Note the change in "FIXES.txt" (top level)
2. Fix the bug in your implementation
3. Use performance improvements when providing equivalent visual results

# Restricted Files/Folders

Do not modify:
- .git folder (including hooks)
- CLAUDE.md (this file)
- target folder
- scripts folder
- Makefile.toml
- ctp1 and ctp2 folders
- ctp2_source folder
- lint rules

# Technology Stack

**SDL2**: Main canvas, tile/terrain rendering, texture management, input capture
**ImGui**: Overlays, menus, debug panels, HUD elements
**ffmpeg-next + SDL2**: Movie playback with texture streaming
**sdl2::mixer**: Audio and music

Primary libraries: rust-sdl2, imgui-rs, imgui-sdl2, imgui-sdl2-imgui-renderer