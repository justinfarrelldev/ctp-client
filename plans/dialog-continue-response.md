# Plan: Ensure extraction Continue responses launch work

## Goal

Fix the extraction confirmation dialog so pressing **Continue** is reliably treated as an affirmative response and does not fall back to the launcher screen as if the user canceled.

## Checklist

- [x] Audit the custom GTK dialog loop for response ordering issues.
- [x] Add/extend response-mapping tests for affirmative and dismissal responses.
- [x] Replace the hand-rolled nested loop with GTK's response future so button clicks resolve from the dialog response signal.
- [x] Run focused tests for dialog helpers.
  - `cargo test dialogs::tests` passed.
- [x] Run `cargo make lint` and fix all issues.
- [x] Run the full test suite and coverage check, documenting unavailable tooling.
  - `cargo make test` passed.
  - `./scripts/check-coverage-fast.sh` is not present in this repository.
- [x] Run a build check.
  - `cargo build` passed.
- [x] Run `cargo make lint-fix`, stage, and commit the fix.
