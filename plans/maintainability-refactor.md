# Maintainability Refactor Plan

Goal: refactor duplicated shell/menu code without changing rendered output, hitboxes, labels, sounds, actions, or asset references. Preserve original CTP2 source/LDL references wherever intent is unclear.

## Checklist

- [x] Baseline review
  - [x] Inspect duplicated button/control construction across shell screens.
  - [x] Inspect duplicated button rendering/text-rectangle logic across landing and shell runtime paths.
  - [x] Identify a small, behavior-preserving refactor surface with strong existing coverage.
- [x] Tests first
  - [x] Add tests that lock down options/root button metadata while permitting maintainable builders.
  - [x] Add tests for any new pure button-render planning helper, including enabled, disabled, hovered, and fallback-text cases.
- [x] Refactor shell model builders
  - [x] Introduce reusable shell button specs/build helpers for repeated option buttons.
  - [x] Use data-driven specs for root options action/disabled button groups.
  - [x] Use reusable specs/helpers for graphics toggles, sound slider groups, and music toggles where safe.
  - [x] Keep LDL/source references beside specs to preserve original intent.
- [x] Refactor runtime button rendering
  - [x] Extract shared button render planning/text-rectangle selection so landing and shell code cannot diverge.
  - [x] Preserve exact draw order and fallback rendering behavior.
- [ ] Verification
  - [x] Run `cargo make lint` and fix issues.
  - [x] Run relevant tests, then full `cargo make test`.
  - [ ] Run `./scripts/check-coverage-fast.sh` and ensure coverage remains at least 80%.
  - [x] Run a dedicated build step.
  - [x] Run `cargo make lint-fix` immediately before commit.
  - [x] Stage and commit changes normally (no `--no-verify`).

## Notes

- No visual output changes are permitted; tests should compare existing LDL-derived labels, coordinates, styles, actions, and source references.
- CTP2 reference names already used in code (`options.ldl`, `graphics.ldl`, `sound.ldl`, `musicscreen.ldl`, `c3_popupwindow.cpp`) must remain visible in docs/source strings.
- `./scripts/check-coverage-fast.sh` is not present in this checkout (`scripts/` only contains `example-build.sh`). Fallback checks were attempted with `cargo llvm-cov --all-targets --all-features --workspace --summary-only` (76.19% line coverage), `cargo llvm-cov --lib --summary-only` (75.93% line coverage), and `cargo make coverage-check`/tarpaulin (66.26%, below its existing 80% threshold). These failures are repository-wide baseline coverage shortfalls concentrated in UI/video/runtime rendering code, not introduced by this refactor.
- `cargo make build-all` was attempted but the 32-bit Linux cross-build failed because `pkg-config` is not configured with an i686 sysroot for GTK/Pango/Cairo. Native `cargo make build` succeeds.
