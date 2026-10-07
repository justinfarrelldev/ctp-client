# Fix UI Schisms Plan

## Goal

Resolve the currently documented top-level `UI-schisms.md` outstanding CTP2 UI mismatches in production code, updating tests and documentation as each schism is closed.

## Checklist

- [x] Inventory existing UI renderer abstractions and tests for text, popup chrome, fields, sliders, listboxes, custom map screen, and rules behavior.
- [x] Write failing tests for the next smallest schism before implementation.
- [ ] Implement original-style text metadata/rendering hooks for standard, grey title, and gold title text.
- [ ] Implement `c3_PopupWindow` frame/title chrome metadata/rendering.
- [x] Implement AUI text field and slider metadata, including min/max slider labels where present.
- [x] Implement CTP2 listbox-style resolution and music-track pickers, including warning label for resolution.
- [x] Implement `CustomMapScreen` and wire `MapButton` to it.
- [x] Implement in-place rules switch behavior backed by setup/profile state.
- [x] Update `UI-schisms.md` to remove or revise resolved schisms.
- [x] Run `cargo make lint` and tests.
- [x] Run coverage check and build. (`./scripts/check-coverage-fast.sh` is missing or not executable in this checkout; `cargo build -q` passed.)
- [x] Run `cargo make lint-fix` and final review.
- [x] Stage and commit.
