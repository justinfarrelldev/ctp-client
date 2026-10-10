# In-game Dropdown Parity Plan

## Goal

Fix the top in-game dropdowns first, using the CTP2 full release source/assets as the reference. The menu bar must look and behave like the base game, including the top-level button names, dropdown row ordering, disabled rows, and nested submenu indicators. This is the first slice of the broader in-game interface parity work.

## Checklist

Reference note: CTP2 `MainMenu` top dropdowns are flat `ctp2_Menu` lists; no nested submenu metadata was found in `controlpanelwindow.cpp` or the `CTP2_MENU`/`CTP2_MENUBAR` LDL templates.

Verification note: `./scripts/check-coverage-fast.sh` is still absent in this checkout (`No such file or directory`).

- [x] Crawl the CTP2 source and layout assets for in-game top menu definitions, button positions, dropdown row labels, disabled items, and submenu behavior.
- [x] Add failing model tests for the original CTP2 top-menu categories, row labels, disabled save/load rows, and submenu metadata.
- [x] Add failing runtime/render tests or pure geometry tests for original top-menu hit boxes and dropdown row rectangles.
- [x] Implement an original-parity in-game top-menu model with exact categories and rows found in source.
- [x] Update top-menu click handling so terrain/end-turn clicks cannot accidentally toggle a stray dropdown/box.
- [x] Update the top-menu renderer to draw original-style chrome/text/disabled/submenu indicators rather than the current placeholder panel.
- [x] Run targeted tests after each red/green/refactor slice.
- [x] Run `cargo make lint-fix`, `cargo make lint`, `cargo make test`, and `cargo build`.
- [x] Attempt coverage script and record the known absence if still unavailable.
- [x] Final review, update plan, and commit.
