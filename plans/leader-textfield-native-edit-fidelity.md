# Native leader text-field editability and style fidelity

- [x] Re-read original CTP2 `ctp2_TextField` / `aui_TextField` rendering and edit-control sizing/focus behavior.
- [x] Add tests for source-derived native edit-control geometry: white inner EDIT area, 2px bevel expansion, 16px text-control height from the original resize path.
- [x] Add tests that focus via the rendered text-field border and edit the leader name through the same runtime event path used by SDL events.
- [x] Update text-field rendering and hit testing to use the original inner edit rectangle and visible bevel rectangle.
- [x] Ensure keyboard/text input remains editable and duplicate-safe after focus.
- [x] Expose focused text-field state to rendering and draw a visible native-style caret in the active leader-name field.
- [x] Run targeted tests, `cargo fmt`, `cargo make lint-fix`, `cargo make lint`, full tests, coverage if available, build, and `git diff --check`.
- [x] Commit the fix.
