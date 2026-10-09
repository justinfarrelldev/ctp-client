# Sharp scaled main-menu viewport

- [x] Add regression tests proving high-resolution menu clicks use physical output coordinates after the first render frame.
- [x] Add regression tests proving high-resolution menu text uses a scaled font size instead of SDL logical-size upscaling.
- [x] Replace SDL logical-size rendering with explicit viewport + renderer scale so SDL does not rewrite mouse coordinates.
- [x] Render text at physical output resolution while keeping menu element positions proportional.
- [x] Keep resolution changes on the actual window size with no persistent logical-size scaling.
- [x] Run targeted tests, `cargo fmt`, `cargo make lint-fix`, `cargo make lint`, full tests, coverage if available, build, and `git diff --check`.
  - `./scripts/check-coverage-fast.sh` is not present/executable, so coverage could not be run.
- [x] Final review and commit.
