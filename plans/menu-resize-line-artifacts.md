# Investigate menu resize line artifacts

- [x] Reproduce the reported 1920x1080 -> 1280x768 geometry path in tests.
- [x] Identify whether artifacts come from stale backbuffer pixels, viewport clearing, or fractional scaled draw-call seams.
  - Root cause: SDL software rendering did not paint the scaled viewport when relying on renderer scale/viewport state; the frame remained an unscaled 800x600 draw, leaving unpainted black pixels inside the claimed high-resolution viewport after resize.
- [x] Add regression coverage for the exact resize path.
- [x] Implement the minimum renderer change that removes the visible lines while preserving proportional layout and sharp text.
- [x] Run targeted tests, `cargo fmt`, `cargo make lint-fix`, `cargo make lint`, full tests, coverage if available, build, and `git diff --check`.
  - `./scripts/check-coverage-fast.sh` is not present/executable, so coverage could not be run.
- [x] Final review and commit.
