# Options Right-Side Disabled Only Plan

## Goal

Correct the OptionsWindow disabled visual state so only the intended right-side game-management buttons are visibly disabled, while Game Play, Keyboard, and Cheat Mode are not incorrectly dimmed by virtue of being unimplemented.

## Checklist

- [x] Add/update tests proving only Save Game, Load Game, Restart, and New Game are visibly disabled.
- [x] Decouple `ShellAction::Disabled` from visual disabled state.
- [x] Add an explicit disabled-button builder for controls that should render disabled.
- [x] Update OptionsWindow builder to use explicit disabled visuals only for right-side buttons.
- [x] Update `UI-schisms.md`.
- [x] Run lint, tests, build, and lint-fix.
- [x] Commit.
