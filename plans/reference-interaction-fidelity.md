# CTP2 reference interaction fidelity

## Contract

Review every image in `images/ctp2_references` using its full filename as context. Work from original CTP2 source and installed art (read-only, never copied). Keep multiplayer/local games server-authoritative. New tests must not write files or contact the real server. Use red → green → refactor, native SDL pixel readback, strict lint, full tests, coverage and a dedicated build. Do not certify pixel parity for unimplemented screens.

## Checklist

- [x] Inspect all 33 reference PNGs individually and read every filename.
- [x] Record a per-reference context inventory and grouped source follow-up map, explicitly separating missing client integration from unknown source behavior.
- [x] Trace original cursor loading/hotspots/hand, edge thresholds/default delay/acceleration, directional mirroring and animation/path timing. Retain existing packed-channel fog rules; reproduce the missing-tile foreign-actor leak.
- [x] Search supplied original UI/source/data for Optimize Sliders and inspect DomesticManagementDialog/CauseAndEffectTab callback registration. No matching working callback is established; see evidence below.
- [x] Add red → green cursor/animation API contracts, reproduce the foreign-actor leak as a failing SDL pixel test, and add in-memory SDL artwork comparisons plus dispatch, latency and option-handoff regressions.
- [x] Implement original-art hand/move/blocked/assault/edge cursors with exact hotspots; suppress existing modal/focus-lost scrolling, preserve fractional ticks and update stationary-pointer pathing after a pan. World wrap/bounds reconciliation remains outstanding.
- [x] Implement directional SPR sampling and a shared travel/frame clock for accepted prefixes of explicitly submitted moves, for the currently supported Settler/Warrior/barbarian families. Never predict a server outcome. Full unit roster and end-turn actor events remain outstanding.
- [ ] Reconcile right-click menus and map information with the referenced click contexts; no immediate hidden command dispatch.
- [ ] Audit fog-safe actors, map details and remembered terrain against source/server contracts; add regressions.
- [ ] Implement and visually reconcile all five control-panel tabs, stack expansion, messages/popups and city/build workflows.
- [ ] Implement and visually reconcile Empire, National, Unit, Trade, Diplomacy, Science, Library, Statistics and in-game Options screens with functional source-verified controls.
- [x] Inspect an actual native SDL motion/cursor capture and add an explicit deterministic motion diagnostic. Document remaining differences without claiming complete parity.
- [x] Run strict lint, full tests, the missing coverage-script command and LLVM coverage, dedicated native build and final review; attempt Windows cross-check and report the environment failure.
- [x] Update `FIXES.txt`, `PLAN.md` and visual-testing documentation; run pre-commit `cargo make lint-fix` and strict lint; commit this partial pass normally, without marking the remaining request complete.

## Reference inventory (all viewed)

1. `building-complete-message-in-msgs-clicked-to-show-popup.png`: message click opens top-left gold-framed 3800BC text/eye popup.
2. `building-complete-message-in-msgs.png`: completed Hoplite message row in parchment Msgs panel; red unread indication.
3. `build-manager-multiple-items-added-granary-selected.png`: Buildings tab, selected queue entry describes food/storage effects; queue reordering/removal.
4. `build-manager-multiple-items-added-settler-selected-on-left.png`: Units tab, selected buildable Settler enables Add/Insert; queue remains separately selected.
5. `city-founded-then-build-manager-pops-up.png`: automatic Build Manager, empty queue, disabled action controls until selection.
6. `city-tab-no-city.png`: empty city portrait/name/mayor, no fabricated city.
7. `city-tab-with-city.png`: Hoplite production portrait, Rome selector, mayor and rush-buy cost.
8. `diplomacy-manager-opened.png`: right-anchored contact table; no contacts means disabled diplomatic actions.
9. `empire-manager-opened.png`: domestic/government, rations/workday/wages sliders, science/public works spinners, Optimize Sliders/Details.
10. `empire-tab.png`: Ballistics research portrait, cities/population/PW/government/science/pollution summary.
11. `fortified-warrior.png`: stationary fortified unit on map while Settler selected; radius preview and order availability.
12. `graphics-overlay-ingame-from-dropdown.png`: gold-frame Graphics dialog, unit speed/city-name sliders and 12 switches.
13. `great-library-opened.png`: category rail/search, Ballistics article/link navigation, tabs and Set Goal.
14. `hoplite-built-in-rome.png`: new unit selected in city, production label/queue and Msgs unread state.
15. `messages-tab-no-messages.png`: empty parchment list with scrollbar, not placeholder labels.
16. `moving-unit-line.png`: green/yellow numbered turn boxes along thin path.
17. `moving-unit-to-city-line-notice-the-numbers-for-num-of-turns.png`: terrain-aware multi-turn path into friendly city.
18. `national-manager-opened.png`: Resources/Status/Specialists list and selection-gated city actions.
19. `ranking-overlay-from-stats-dropdown-ingame.png`: Statistics Ranking graph, nation legend, rank selector and Zero Sum Graph.
20. `right-click-terrain-with-friendly-city-and-fortified-unit-in-city.png`: city context takes priority; City/Build/Rename/Disband/Mayor/Map Info.
21. `right-click-terrain-with-friendly-unit-on.png`: Army/Move/Attack/Fortify/Sentinel/Pillage/Pirate/Expel/Reform/Board/Disband/Map Info menu.
22. `right-click-terrain-with-no-unit-on.png`: gold-frame terrain information dialog, tile art and authoritative yields/cost.
23. `ruin-information-popup.png`: gold-frame authoritative ruin result, Close button.
24. `science-manager-opened.png`: current Ballistics, costs/turns, available advances, Change Research.
25. `settler-shows-green-tile-outlines-around-where-city-bounds-will-be.png`: green city-radius overlay only around selected settler.
26. `sound-overlay-from-dropdown-ingame.png`: compact centered gold-frame Sound with three 0–10 sliders and Close.
27. `tile-tab-not-enough-pw-for-farm.png`: five terrain categories, farm art, remaining grid empty/disabled.
28. `trade-manager-opened.png`: Market/Export/Import, city filter/goods count, advisor and disabled route creation.
29. `unit-manager-opened.png`: type/count/statistics, advisor, military readiness slider and disband control.
30. `unit-tab-multiple-units-in-capitol-rome-clicked-on-arrow-to-expand-units.png`: arrow toggles portrait grid; selected stack persists.
31. `unit-tab-multiple-units-in-capitol-rome.png`: top Hoplite portrait with blue stack-count herald and expansion arrow.
32. `unit-tab.png`: two-settler herald, settler-only order availability and green selection brackets.
33. `yellow-select-archer-can-no-longer-move.png`: exhausted Archer has yellow (not green) selection brackets.

## Source evidence and unresolved behavior

- `ctp2/ctp2_data/english/uidata/layouts/civ3.ldl:428–506`: UC001 pointing hand, UC002/003 movement, UC004 assault, UC010–013 edge cursors and their non-centered hotspots. `cursormanager.h/.cpp`, `aui_cursor.cpp`, `aui_mouse.cpp` establish the variant/index and software cursor path. Installed TGA art is loaded read-only, with original RGB565 quantization and black chroma key.
- `ctp2_source/ctp2_code/ctp/civ3_main.cpp:594–881`: edge thresholds x<4/x>width-4, y<=2/y>=height-2; vertical corner priority; delay in 100ms units and 50ms acceleration. `gs/database/profileDB.cpp:187` sets the default ScrollDelay to **3**, not 2. The new bounded 16ms timestep intentionally avoids render-FPS-dependent travel and suspended-frame camera jumps; full wrap/boundary rules are still not reconciled.
- `gfx/spritesys/spriteutils.cpp:779`, `FacedSprite.cpp:167–190`, `ActorPath.cpp:87–119`, `action.cpp`, `Anim.cpp`, `UnitActor.cpp:522`, `director.cpp:3089–3141`: eight direction mapping, mirrored western images/hotspots, linear path legs, sequence frame/delta/alpha metadata, IDLE→MOVE fallback and 10-speed profile. The presenter uses SPR playback metadata with a bounded fallback and the profile scaling for both travel and frames. Original `ActorPath` converts negative floating displacement to unsigned; signed integer interpolation avoids that non-portable conversion.
- **Optimize Sliders remains undeduced.** Case-insensitive searches of supplied C++/headers/LDL/string data for Optimize/Optimise Sliders found no corresponding working UI callback. `ui/interface/DomesticManagementDialog.cpp` constructs `CauseAndEffectTab`; `CauseAndEffectTab.cpp:194–210` registers Details, happiness, rations, workday, public works, wages, science tax and tab callbacks, not an optimizer. `ai/CityManagement/governor.cpp` has production/gold/food optimization logic but is not proof of this button's intended behavior. This may be a release/source mismatch. Do not guess an algorithm or label that AI code as the verified button implementation.
- The server already exposes social-policy, tax/PW rates, government, readiness, research, disband, fortify, army and production routes (`$CTPSERVER/internal/gameplay/http/http.go`). Their absence from the current UI is a **client integration gap**, not evidence they are impossible. No server files were changed, and no new test contacts the running server.

## Remaining UI source map

Reference numbers refer to the complete viewed inventory above. These are follow-up entry points, not claims that every callback has already been traced:

| References | Original source entry points | Remaining client work |
| --- | --- | --- |
| 1, 2, 15, 23 | `MessageControlPanel.cpp`, `messagelist.cpp`, `messagewindow.cpp`, `messageeyepoint.cpp` | Read/unread rows, date/eye popups, authoritative completion/ruin messages |
| 3–7, 14, 18, 20 | `EditQueue.cpp`, `CityControlPanel.cpp`, `citymanager.cpp`, `NationalManagementDialog.cpp` | City selection, queue editing/details, build-completion workflow, mayor/rush buy, city context priority |
| 8 | `diplomacywindow.cpp`, `DiplomacyDetails.cpp`, diplomacy LDLs | Contact data, selection-gated actions, proposals and intelligence |
| 9, 10 | `DomesticControlPanel.cpp`, `DomesticManagementDialog.cpp`, `CauseAndEffectTab.cpp`, `GovernmentTab.cpp` | Empire summary/dialog, slider calculations and authoritative commands; Optimize unresolved |
| 11, 13, 16, 17, 21, 24–27, 29–33 | `UnitControlPanel.cpp`, `unitmanager.cpp`, `armymanagerwindow.cpp`, `controlpanelwindow.cpp`, `TilesControlPanel.cpp`, `ScienceManagementDialog.cpp`, `greatlibrary.cpp` | Full unit metadata/art roster, fortification, semantic stack expansion, real context menus and terrain info, cost-based path turn labels, research/library, improvement legality/PW costs |
| 12, 19, 22, 28 | `graphics.ldl`, `sound.ldl`, `infowindow.cpp`, `trademanager.cpp`, `controlpanelwindow.cpp` | Functional in-game options, statistics, terrain info and trade manager with source placement/art |

Foreign actors now require explicitly visible tiles; hidden/stale/missing tile pixel tests pass even with local fog disabled. This is a targeted safety repair, **not full sightline/remembered-city/fortification parity certification**. Existing server river/resource/ownership limitations are documented in `GAMEPLAY_SNAPSHOTS.md`; no new server implementation issue was demonstrated here.

## Validation evidence

- Red: missing cursor/animation/probe APIs; fog pixel test failed with `foreign actor with missing tile leaked`. Green: native SDL pixels match source cursor art and source MOVE frame selection for both supported art families, eight directions × four phases; behavioral tests cover accepted prefixes, rejected motion, speed, frame tables/deltas/alpha/delay, response latency, focus, modal scrolling, frame-rate independence and shell handoff.
- All new rendering/screenshot tests retain RGBA/PNG data in memory. The opt-in example alone writes diagnostic screenshots. `images/reference-motion-nw.png` was generated and visually inspected, not committed; it is synthetic and plainly differs from the complete reference scenarios.
- `cargo make lint`: passed. `cargo make test`: **326 library + 344 binary + 19 integration** tests passed. `cargo build`: passed. LLVM line coverage: **80.88%** overall; animation presenter **96.70%**, cursor policy **92.52%**, directional texture sampler **88.57%**. `./scripts/check-coverage-fast.sh` was attempted and is absent.
- `cargo check --target x86_64-pc-windows-msvc` was attempted and stopped in `gio-sys`: the host has no configured GLib/GIO cross-compiling pkg-config/sysroot. The new presentation code uses portable SDL/Rust APIs, but Windows binaries have not been validated.
- Final review: no original art copied/committed; restricted paths and server files untouched; no new file-mutating/live-server tests; new modules below 800 LOC; no new allow attributes; selection/authority remain model-owned. Full client completeness is explicitly not asserted.

This is an explicitly **partial first implementation pass**. The unchecked work above is still required to satisfy the full user request. No all-screen pixel-perfect, all-buttons-functional or all-units-animated claim is made.
