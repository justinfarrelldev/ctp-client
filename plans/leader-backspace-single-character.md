# Leader text-field backspace single-character behavior

- [x] Reproduce the first-focus backspace behavior with tests before changing implementation.
  - [x] Model-level test: focusing the leader-name field and pressing Backspace once removes only the final character.
  - [x] Runtime SDL-event-path test: click/focus the field, send one Backspace keydown, and confirm the leader name is not cleared.
- [x] Update leader-name edit state so first Backspace after focus clears pending replace-on-typing state but still removes only one character.
- [x] Preserve existing first typed character replacement behavior and duplicate-safe text input behavior.
- [x] Run targeted leader-name tests.
- [x] Run `cargo fmt`, `cargo make lint-fix`, `cargo make lint`, `cargo make test`, coverage if available, `cargo build`, and `git diff --check`.
- [ ] Commit the fix.
