# Single-player Launch Crash Diagnosis Plan

## Goal

Diagnose why pressing the bottom Launch button exits/crashes the client with no visible logs, and fix any client-side issue. The in-game screen must reuse the SDL window created for the CTP2 menu instead of closing the menu and creating a second gameplay window.

## Checklist

- [x] Reproduce/trace the launch path from GTK → SDL menu → gameplay runtime.
  - The old path closed the menu loop, returned to `main`/GTK glue, then created a second SDL gameplay window. A local server run also reproduced transient HTTP 429 capacity responses during rapid setup posts, which were previously logged only at hidden default log levels.
- [x] Add a failing regression test proving single-player launch uses the existing menu SDL window/canvas rather than opening a second SDL window.
- [x] Add diagnostics/logging around gameplay bootstrap failures so server/client launch errors are visible in normal debug runs.
- [x] Refactor the SDL menu runtime to hand off its existing SDL context, canvas, and event pump to the gameplay runtime when `LaunchSinglePlayer` is selected.
- [x] Refactor the gameplay runtime to support running on a borrowed/existing SDL canvas and event pump while preserving current headless smoke tests.
- [x] Verify the server bootstrap path still surfaces actionable errors when the server is unavailable or rejects setup.
- [x] Run targeted tests for menu/gameplay launch handoff.
- [x] Run `cargo make lint-fix`, `cargo make lint`, `cargo make test`, and a build.
  - `cargo make lint-fix` ✅
  - `cargo make lint` ✅
  - `cargo make test` ✅
  - `cargo build` ✅
  - Coverage script remains unavailable: `./scripts/check-coverage-fast.sh` returned `No such file or directory`.
- [x] Update this plan with completed steps, review changes, and commit.
  - Reviewed launch flow: gameplay now starts inside the menu-owned SDL runtime and `main.rs`/GTK glue no longer creates a second gameplay window after the menu returns.
  - Reviewed server bootstrap: transient 429 capacity responses now retry with bounded backoff, while persistent server failures still return explicit errors.
  - `git diff --check` ✅
