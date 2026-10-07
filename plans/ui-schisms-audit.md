# UI Schisms Audit Plan

## Goal

Audit every currently modeled UI element against the CTP2 base game, document all mismatches in `UI-schisms.md`, and remove or quarantine UI that cannot be mapped to an original game element while preserving required non-game bootstrap functionality.

## Checklist

- [x] Inventory all UI surfaces in the app.
  - [x] GTK launcher, settings, add-installation, extraction dialogs, and file picker bridge.
  - [x] Intro video skip/player UI.
  - [x] SDL landing menu.
  - [x] Single-player hub.
  - [x] New-game screen.
  - [x] Options root popup.
  - [x] Graphics options popup.
  - [x] Sound options popup.
  - [x] Music options popup.
  - [x] Resolution and music-track pickers.
  - [x] New-game setup picker popups.
- [x] Locate original CTP2 references for each mappable element.
  - [x] LDL layout geometry and object names.
  - [x] Original source wiring and generated controls.
  - [x] Button/switch/field/pattern/background art and text styling references.
- [x] Write failing verification tests for removed/changed UI where needed.
- [x] Remove current in-game UI elements that cannot be mapped to base CTP2.
- [x] Update `UI-schisms.md` with every audited element, mismatch, styling note, and unresolved schism.
- [x] Run `cargo make lint` and fix lints.
- [x] Run tests and coverage check. (`cargo make test` passed; `./scripts/check-coverage-fast.sh` is absent; `cargo llvm-cov --summary-only` completed but reports current project line coverage at 74.91%.)
- [x] Run a build step. (`cargo build` passed; `cargo make build-all` cannot cross-build i686 GTK without a configured pkg-config/sysroot in this environment.)
- [x] Final review, run `cargo make lint-fix`, stage, and commit.
