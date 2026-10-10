# Single-player In-game Implementation Plan

Branch: `singleplayer-in-game`

## Scope updates from request

- [x] Saving and loading games are out of scope for this pass.
- [x] Top-of-screen drop-downs are in scope and must visually match CTP2, including working shortcuts for in-scope items such as New Game and Quit.
- [x] Save/Load menu rows should be omitted or visibly disabled/no-op with an out-of-scope status, never performing file mutation in tests.

## Research checklist: original rendering and behavior sources

- [x] Main in-game import root: `ctp2_data/english/uidata/layouts/civ3.ldl` imports the gameplay windows.
- [x] Control panel: `controlpanel.ldl` defines `ControlPanelWindow`, `TurnButton`, `ShortcutPad`, tab panels, `ContextMenuItem`, `MainMenu`, resource counters, message panel, and tile-improvement panel.
- [x] Main menu bar: `controlpanel.ldl` `MainMenu:CTP2_MENUBAR` uses pattern `uptb00.rim` from picture ZFS archives in this install; buttons are Civ/Cities/Units/Diplomacy/Info/Options; counters use `uptb02.rim`, `uptb03.rim`, icons `upic33.rim`-`upic36.rim`.
- [x] Menu construction/callbacks: `ctp2_source/ctp2_code/ui/interface/controlpanelwindow.cpp` `BuildMainMenu`, `Build*Menu`, and `*MenuCallback` provide all rows and actions.
- [x] Menu item templates: `ctp_template.ldl` `PlainMenuListItem`/`IconMenuListItem` define text and shortcut columns.
- [x] Keyboard shortcuts: `ctp2_data/default/uidata/keymap.txt` maps movement, settle, end turn, Great Library, New Game, Quit, map filters, zoom, and screen shortcuts.
- [x] End-turn behavior: `EndTurnButton.cpp`, `keypress.cpp`, `TurnCnt.cpp`, and `NewTurnCount.cpp` call `RegisterManualEndTurn` and `AddEndTurn`.
- [x] Unit context/right-click menu: `controlpanelwindow.cpp` `ContextMenuCallback` handles city view/build/rename/disband, tile info, orders, army manager, clear orders, group/ungroup, and transport view.
- [x] Unit order button art and labels: `buttonbank.ldl` defines `SettleButton`, move, go-to, sleep, fortify, pillage, patrol, group/ungroup, cargo, disband, etc.
- [x] Unit manager pad: `unitpad.ldl` defines Army/City/Navy/Air/Space/Special tabs and legal/all-tech toggle.
- [x] City management: `citywindow.ldl`, `citymanager.ldl`, `editqueue.ldl`, `queuewin.ldl`, and `citystatus.ldl` define city view/build-queue windows and government panels.
- [x] Great Library: `greatlibrary.ldl` and `ctp2_data/english/gamedata/Great_Library.txt` define the GL shell/content source.
- [x] Options dialogs reachable in game: `options.ldl`, `graphics.ldl`, `sound.ldl`, `gameplayoptions.ldl`, `profileedit.ldl`, `km_screen.ldl`.
- [x] Government/civilization UI: `civstatus.ldl` contains government comparison/enactment tab; control panel domestic tab shows current government.
- [x] Top info/ranking/wonders: `infoscreen.ldl`, `statswindow.ldl`, `wondermovie.ldl`, `victoryscreen.ldl`.
- [x] Messages/popups: `messagebox.ldl`, `helppopup.ldl`, `tutorialwin.ldl`; startup/tutorial messages are driven by `script.slc`/`tut2_msg.slc`.
- [x] Map/radar/filter controls: `controlpanel.ldl`, keymap entries for city names, trade routes, radar, control panel, all overlays, zoom in/out.
- [x] Terrain/game database: `terrain.txt`, `map.txt`, `goods.txt`, `tileimp.txt`, `terrainicon.txt`, `goodsicon.txt`, `tileimpicon.txt`.
- [x] Settler data: `Units.txt` `UNIT_SETTLER` has `DefaultSprite SPRITE_SETTLER`, land/mountain settle, size 1 city, sounds, 100 move points, 1 vision.
- [x] Sprite IDs: `spriteID.txt` maps `SPRITE_SETTLER` to 2, `SPRITE_WARRIOR` to 3, barb/other units to `GU###.SPR` files.
- [x] Sprite file format: `SpriteFile.h/.cpp`, `Sprite.h`, `FacedSprite.h`, `UnitSpriteGroup.h`, `Anim.h` define SPRF tag, versions, LZW1 compression, action offsets, faced frames, animations, hot points, shield points.
- [x] Sprite loading names: `SpriteGroupList.cpp` tries `GU%03d.SPR` then `GU%02d.SPR`; city `GC%03d.SPR`, goods `GG%03d.SPR`, effects `GX%02d.SPR`.
- [x] Sound database: `sounds.txt` maps unit select/move/ack/death/attack/work/victory names to WAV entries in `sound.zfs`.
- [x] Ruin behavior: `script.slc` messages include ruin city/advance/gold/barbarian outcomes; `risks.txt` defines hut barbarian chance/max barbarians.
- [x] Barbarian risk: `RiskRecord`, `RiskDB`, `risks.txt`, `Wonders.txt` `ProtectFromBarbarians`.
- [x] AI settling reference: `ai/mapanalysis/settlemap.cpp` scores settle targets and invalid cells; `ctpai.cpp` adds settle goals and excludes barbarians from diplomacy.
- [x] Fog of war/exploration: `profileDB.cpp`, world exploration checks in control panel/tile improvement code, and keymap toggles.

## Implementation checklist

### Launch integration

- [x] Create branch `singleplayer-in-game`.
- [x] Add a gameplay module exported from library and binary.
- [x] After shell `Launch` returns `LaunchSinglePlayer`, start the SDL in-game runtime instead of only logging.
- [x] Keep tests injectable/headless; do not force gameplay runtime in pure Play Game flow unit tests.

### Pure gameplay model (TDD first)

- [x] Add deterministic in-game state: map, terrain, fog, ruins, units, cities, empires, barbarians, turn, selected unit, popups, overlays.
- [x] Ensure barbarians are represented by a non-empire owner and never counted in empire count.
- [x] Generate a CTP2-like starting state with human settlers, other empires, ruins, and barbarian camps/roamers.
- [x] Implement movement/path orders with move-point cost, queued paths across turns, reachable tile labels (`1`, `2`, etc.), exploration, and fog-of-war updates.
- [x] Implement settler settle action that creates a city, consumes/kills the settler, reveals around the city, and opens the city build menu.
- [x] Implement sentinel/sleep behavior for settlers and selected-unit cycling.
- [x] Implement barbarian AI wandering and attacking; settlers can die in combat.
- [x] Implement other-empires encounter visibility/diplomacy placeholder events.
- [x] Implement ruins: gold, knowledge/map reveal, wandering barbarians, and free city/settler outcomes where appropriate.
- [x] Implement end-turn processing and original shortcut equivalents (A, Enter, Space, keypad movement, B settle, Q quit, Ctrl-X new game, Ctrl-5 GL, etc.).
- [x] Implement city build menu model with accessible actions and no file mutation.

### In-game UI model and menus

- [x] Model top menu bar with original labels, ordering, shortcuts, and source references from `controlpanelwindow.cpp`.
- [x] Implement in-scope menu actions: Empire Manager, Trade, Science, City Manager, Build Queue, Unit Manager, Army Manager, Diplomacy placeholder, Great Library, Ranking/Score/Wonders placeholder, Gameplay/Graphics/Sound options, Advanced options placeholder, Restart/New Game/Quit, Zoom in/out.
- [x] Mark Save/Load as out of scope/disabled without file writes.
- [x] Model end-turn button, status bar, resource counters, shortcut pad buttons, domestic/city/unit/message/tileimp tab panel, government display, minimap/radar, and map-filter controls.
- [x] Model right-click context menu for tile/unit/city with accessible rows and keyboard activations.
- [x] Model Great Library popup backed by the original text file.
- [x] Model initial beginning-of-game popup and queue it on launch.

### Asset loading/rendering/audio

- [x] Add SPR loader based on original `SpriteFile` code: header, v1/v2 offsets, LZW1 decompression/copy path, encoded normal/faced sprite frames, animation metadata, hot points, shield points, and RGBA conversion with chroma transparency.
- [x] Load settler/warrior/barbarian sprites from original `GU*.SPR` files and render first decoded frames; full facing/animation advancement remains follow-up.
- [x] Load original UI art from `pic565.zfs`/`pic555.zfs` RIM entries where available; never copy assets into repo.
- [x] Render terrain/isometric map from decoded `gtset565.til` base tiles with fog/explored states, ruins, units, cities, empires, labels, and movement overlays.
- [x] Render control panel, top menu bar/dropdowns, end-turn button, minimap, Great Library/startup popups with original RIM-backed art where decoded; remaining city/goods/improvement sprites remain follow-up.
- [x] Add sound-effect lookup/loading from `sounds.txt` + `sound.zfs`, including UI click, end turn, movement, settle, combat/death, ruin, popup/Great Library.
- [x] Ensure tests/headless mode degrades gracefully without assets, while interactive launch requires original terrain/control-panel assets.

### Verification

- [x] Unit tests: no real file mutation; use mock asset bytes, in-process fake HTTP servers, or read-only optional local CTP2 asset inputs.
- [x] Runtime smoke: headless SDL opens gameplay and exits deterministically.
- [x] Lint: `cargo make lint-fix`, then `cargo make lint`.
- [x] Tests: `cargo make test`.
- [ ] Coverage: `./scripts/check-coverage-fast.sh` (script is absent in this checkout; command returned `No such file or directory`).
- [x] Build: dedicated build step (`cargo build`; `cargo make build-all` still requires cross-target pkg-config/sysroot setup for i686 GTK dependencies).
- [x] Final review for asset copyright, security, performance, Windows paths.
- [ ] Commit all changes normally, no `--no-verify`.
