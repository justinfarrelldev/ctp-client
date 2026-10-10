# CTP2 Movement Runtime Repair

## Goal

Fix the live in-game CTP2 movement feel after the previous fidelity pass: clicking the visible settler sprite/tile should reliably activate it, dragging/releasing over visible destination tiles should move it, selected-unit feedback must not use CTP1-style reticles/assets, and gameplay should render sharply with CTP2-native assets rather than a blurry low-resolution stretch.

## Source References

- `ctp2_source/ctp2_code/gfx/tilesys/tiledmap.cpp` — `MousePointToTilePos`, `Click`, `MouseDrag`, `Drop`.
- `ctp2_source/ctp2_code/ui/aui_ctp2/SelItemClick.cpp` — `SelectArmyStartMoveClick`, `MoveDrag`, `MoveDrop`.
- `ctp2_source/ctp2_code/gfx/spritesys/UnitActor.cpp` — `DrawSelectionBrackets` uses CTP2 map bracket icons/color rules, not CTP1 reticles.
- `ctp2_source/ctp2_code/gfx/tilesys/TileHighlight.cpp` — `DrawLegalMove` path line/turn-box feedback.
- `src/gameplay/runtime/view.rs` — gameplay screen/tile conversion.
- `src/gameplay/runtime/render/movement.rs` — selected-unit/path feedback rendering.
- `src/gameplay/runtime/textures.rs` and `src/gameplay/assets/tile.rs` — CTP2 terrain/unit asset loading.

## Checklist

- [x] Add tests proving clicking the visible unit sprite body activates/selects the settler instead of requiring a tile top-left coordinate.
- [x] Add tests proving a sprite-center click-drag to a visible destination tile-center moves the settler in local fallback.
- [x] Add tests proving screen-to-tile conversion returns the intended tile for CTP2 diamond center/feet positions.
- [x] Add tests/guards proving movement feedback uses CTP2 bracket/path primitives and no CTP1 reticle asset names.
- [x] Repair screen-to-tile conversion / hit testing to match the visible CTP2 isometric tile and unit sprite positions.
- [x] Remove or replace any CTP1-style reticle visuals with CTP2 selected-unit brackets/path lines.
- [x] Investigate low-resolution gameplay rendering and make reused/new SDL gameplay output sharp while preserving original 1024x768 layout semantics.
- [x] Run focused tests.
- [x] Run `cargo make lint-fix`, `cargo make lint`, `cargo make test`, `cargo build`, coverage script if present, and `git diff --check`.
- [x] Review file sizes/restricted files/no asset copying, update checklist, and commit.
