# Disable Options Buttons and Prune Hub Plan

## Goal

Make OptionsWindow right-side game-management buttons visibly disabled, and remove Map Editor and Scenario from the single-player hub screen.

## Checklist

- [x] Add failing tests for disabled visual state on OptionsWindow right-side buttons.
- [x] Add failing tests that SPWindow omits Scenario and Map Editor controls.
- [x] Add enabled/disabled metadata to shell controls and builders.
- [x] Render disabled buttons distinctly.
- [x] Remove Scenario and Map Editor from SPWindow builder.
- [x] Update `UI-schisms.md`.
- [x] Run lint, tests, build, and lint-fix.
- [x] Commit.
