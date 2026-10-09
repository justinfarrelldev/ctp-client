# Leader name editing and original text-field style fix

- [x] Add failing tests for runtime text replacement from the real SDL key/text paths after focusing the leader field.
- [x] Add tests for the leader text-field visual style: original white fill, black inset text, and smaller original font scale metadata.
- [x] Fix runtime key handling so focused leader fields can be edited through normal keyboard input.
- [x] Render leader text fields with the original white background and appropriately smaller text bounds.
- [x] Run targeted tests, `cargo fmt`, `cargo make lint-fix`, `cargo make lint`, full tests, coverage if available, build, and `git diff --check`.
- [x] Commit the fix.
