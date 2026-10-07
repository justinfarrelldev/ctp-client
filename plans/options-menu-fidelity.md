# Options Menu Fidelity Plan

## Goal

Align the modeled CTP2 `OptionsWindow` with base-game behavior: no visible quit button in the options popup, right-side game-management buttons remain disabled, and editor-launch labels render as user-facing text rather than LDL tokens.

## Checklist

- [x] Add/update tests for absent options quit button and disabled right-side actions.
- [x] Remove the options popup quit control from the modeled UI.
- [x] Ensure right-side options actions are disabled, including New Game.
- [x] Add/update tests for editor-launch user-facing text.
- [x] Replace editor-launch LDL token labels with real English text.
- [x] Update `UI-schisms.md` to document the corrected mapping.
- [x] Run lint, tests, build, and lint-fix.
- [x] Commit.
