# New Game Header and World Shape Text Alignment Follow-up

## Checklist

- [x] Reproduce the remaining alignment defects with tests:
  - [x] World Shape side-description labels should render from their LDL left/top anchor instead of centered in a tall label box.
  - [x] New Game setup headers should have a visual upward nudge so their text clears the native-height button art below.
- [x] Add reusable label layout metadata for centered, left/top, and visually raised labels without changing original LDL control coordinates.
- [x] Apply left/top layout to World Shape side text while preserving original popup row positions.
- [x] Apply raised centered layout to New Game setup headers above buttons.
- [x] Run targeted tests, `cargo fmt`, `cargo make lint-fix`, `cargo make lint`, full tests, coverage if available, build, and `git diff --check`.
  - `./scripts/check-coverage-fast.sh` is not present/executable, so coverage could not be run.
- [x] Final review and commit.
