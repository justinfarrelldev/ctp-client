# Resolution Scrollbar Drag and Immediate-Apply Cleanup

## Checklist

- [x] Add tests proving the resolution picker no longer shows the obsolete restart warning.
- [x] Add tests proving scrollbar track/thumb dragging changes the visible resolution rows and can select later modes.
- [x] Inspect original/listbox arrow asset behavior and current renderer so arrow buttons render visibly.
- [x] Implement listbox scroll capture/drag mapping in runtime state.
- [x] Render up/down scrollbar arrows visibly while preserving original listbox chrome.
- [x] Remove the restart warning control/text from the resolution picker.
- [x] Run formatting, lint, tests, coverage check if available, and a build.
  - Coverage script unavailable: no `./scripts/check-coverage-fast.sh` executable present.
- [x] Final review and commit.
