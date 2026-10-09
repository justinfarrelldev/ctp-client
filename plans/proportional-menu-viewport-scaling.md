# Proportional Menu Viewport Scaling

## Checklist

- [x] Add viewport tests for proportional contain scaling at higher resolutions:
  - [x] 1024x768 fills the whole output instead of leaving black on all sides.
  - [x] 16:9 outputs fill height with pillarbox side bars only.
  - [x] taller outputs fill width with letterbox top/bottom bars only.
- [x] Replace the fixed 800x600 high-resolution viewport with a uniform aspect-preserving viewport that maximizes covered area.
- [x] Ensure pointer hit-testing maps from the scaled viewport back into original 800x600 menu coordinates.
- [x] Ensure SDL rendering scales menu art/buttons/text uniformly inside the viewport while clearing only true letterbox/pillarbox space to black.
- [x] Run targeted viewport/headless tests, `cargo fmt`, `cargo make lint-fix`, `cargo make lint`, full tests, coverage if available, build, and `git diff --check`.
  - `./scripts/check-coverage-fast.sh` is not present/executable, so coverage could not be run.
- [x] Final review and commit.
