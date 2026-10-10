# In-game Interface Repair Plan

## Goal

Continue the CTP2 in-game interface repair from the current `singleplayer-in-game` branch. Fix visible text, correct the settler sprite source, remove stray popup/dropdown behavior from map/control clicks, and improve the lower UI/control-panel layout using original CTP2 layout/source/assets. Saving/loading remains out of scope and must stay disabled/no-op.

## Source References To Re-check

- `ctp2_source/ctp2_code/ui/interface/controlpanelwindow.cpp` — menu callbacks, top menu rows, control-panel setup.
- `ctp2_source/ctp2_code/ui/aui_ctp2/ctp2_Menu.cpp` — dropdown sizing, modal cancel behavior.
- `ctp2_source/ctp2_code/gfx/tilesys/tileset.h` — tile dimensions/headroom.
- `ctp2_source/ctp2_code/gfx/tilesys/tiledmap.cpp` — tile/sprite placement and hit testing.
- `ctp2_source/ctp2_code/gfx/spritesys/spritefile.cpp` — SPR load behavior.
- `ctp2/ctp2_data/default/gamedata/spriteID.txt` and `Units.txt` — CTP2 sprite IDs.
- `ctp2/ctp2_data/english/uidata/layouts/controlpanel.ldl` — lower panel, end-turn button, status bar, menu button positions.
- `ctp2/ctp2_data/english/uidata/layouts/ctp_template.ldl` — dropdown/listbox chrome and row metrics.
- `ctp2/ctp2_data/default/graphics/pictures/pic565.zfs` — RIM dimensions for UI chrome.
- `ctp2/ctp2_data/default/graphics/sprites/Gu002.spr`, `GU003.SPR` — CTP2 unit art.

## Checklist

### A. Reproduce and Protect Visible Text

- [x] Add a failing/headless-safe renderer test or pure bitmap-text test proving menu/dropdown text is still visible when no system TTF font is available.
- [x] Add coverage for high-contrast top-menu button/dropdown foreground colors so text cannot blend into original chrome.
- [x] Implement a no-TTF fallback text renderer for gameplay UI labels using deterministic bitmap glyphs.
- [x] Update top dropdown/button text colors and optional outlines/shadows for reliable contrast over `uptbtest` and `uptg05*` assets.
- [x] Verify text appears for top buttons, dropdown rows, disabled Save/Load rows, popup body text, control-panel status, and end-turn labels.

### B. Fix CTP2 Settler/Unit Sprite Loading

- [x] Add a failing asset test that CTP2 three-digit mixed-case sprite filenames (`Gu002.spr`) are preferred over legacy two-digit uppercase filenames (`GU02.SPR`).
- [x] Update sprite candidate ordering to prefer exact CTP2 source/install naming and keep legacy variants only as fallback.
- [x] Change gameplay texture loading to resolve unit sprites by sprite ID through the shared loader instead of hard-coded `GU02.SPR`/`GU03.SPR` file names.
- [x] Add/adjust local-asset test to decode the actual CTP2 settler sprite candidate when present.
- [x] Review unit destination rect/origin so settler feet anchor near the tile center/bottom rather than floating or offset.
- [x] Ensure failed sprite decode falls back visibly but does not silently use CTP1-looking art when CTP2 art exists.

### C. Fix Popup/Dropdown Click Containment

- [x] Add tests proving clicks outside an open dropdown cancel only the dropdown and cannot trigger end-turn, terrain movement, or bottom-panel actions.
- [x] Add tests proving clicks on the lower control panel/status/button area do not move/select terrain tiles.
- [x] Add tests proving clicks on the side/radar panel do not move/select terrain tiles.
- [x] Ensure modal gameplay popups consume clicks only to dismiss and never leak through to map/end-turn/menu actions in the same event.
- [x] Keep Save/Load menu rows disabled/no-op and ensure clicking them never creates/mutates files.

### D. Repair Lower Control Panel / Rest of Visible UI Slice

- [x] Use `controlpanel.ldl` layer positions for `upfg01`, `upfg03`, and `upfg04` instead of drawing only the base fragment.
- [x] Move the end-turn button to original panel-relative coordinates (`x=563`, `y=60`) and use the original `upcb14*` dimensions.
- [x] Update end-turn hit testing to match the original visible button rectangle.
- [x] Render `upst01` status-bar art when available and put status/help text in that original region.
- [x] Stop drawing placeholder end-turn/control-panel blocks outside the original panel unless assets are missing.
- [x] Keep the side/radar panel visually separate from the original lower control panel and prevent it from intercepting/overlapping original controls incorrectly.
- [x] Add tests for original control-panel/end-turn geometry and map-click exclusion.

### E. Terrain/Map Alignment Follow-up

- [x] Assert tile constants match CTP2 source (`94x72` grid, `94x48` pixel diamond, `24` headroom, `47x24` step).
- [x] Review terrain draw destination rects to avoid scaling decoded tiles away from their native dimensions at zoom 1.
- [x] Review unit anchor and selection diamond to use tile headroom/center consistently.
- [x] Add pure camera tests for tile-to-screen and screen-to-tile round trips that include tile headroom.

### F. Verification and Commit

- [x] Run targeted tests after each slice.
- [x] Run `cargo make lint-fix` before commit.
- [x] Run `cargo make lint` and fix all warnings/errors.
- [x] Run `cargo make test`.
- [x] Run `cargo build`.
- [x] Attempt `./scripts/check-coverage-fast.sh`; record absence if still missing.
- [x] Run `git diff --check`.
- [x] Final review for file sizes, restricted-file compliance, no asset copying, and no file-mutating tests.
- [x] Commit with a clear message.
