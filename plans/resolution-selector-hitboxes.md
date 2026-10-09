# Resolution Selector and Button Hitbox Fidelity

## Checklist

- [x] Inspect original CTP2 LDL/source for the resolution picker/list box and button/input geometry.
- [x] Add failing tests that capture the original resolution selector chrome/row layout and expected clickable extents.
- [x] Add failing tests that verify button hover/click hitboxes align with rendered art, including tiled/sizable buttons.
- [x] Implement resolution selector model/rendering changes to match original CTP2 geometry and chrome.
- [x] Implement shared hitbox correction so hover and click use the same visible button area.
- [x] Run formatting, lint, tests, coverage check if available, and a build.
  - Coverage script unavailable: no `./scripts/check-coverage-fast.sh` executable present.
- [x] Review changes for fidelity, maintainability, file-size limits, and commit.
