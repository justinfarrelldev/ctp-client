# Original Disabled Button Art Plan

## Goal

Render disabled CTP2 shell buttons with the exact original disabled/inactive `image00`-`image02` art triplet instead of the temporary grey fallback whenever original assets are available.

## Checklist

- [x] Add a render-plan regression test proving disabled blue buttons use only `upbt07[a-c]X` layers.
- [x] Add button-art renderer support for disabled/inactive-only layer drawing.
- [x] Update shell disabled button rendering to use original inactive art and original text placement.
- [x] Keep fallback rendering only for missing assets.
- [x] Update `UI-schisms.md` if needed.
- [x] Run lint, tests, build, and lint-fix.
- [x] Commit.
