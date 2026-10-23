# CTP2 reference interaction fidelity

## Contract

Review every image in `images/ctp2_references` using its full filename as context. Work from original CTP2 source and installed art (read-only, never copied). Keep multiplayer/local games server-authoritative. New tests must not write files or contact the real server. Use red → green → refactor, native SDL pixel readback, strict lint, full tests, coverage and a dedicated build. Do not certify pixel parity for unimplemented screens.

## Active full-screen continuation

The user explicitly requested completion of every remaining checklist item after `ea98904`. Execute [full-screen-reference-parity.md](full-screen-reference-parity.md) alongside this parent, using TDD and atomic commits. No prior component/protocol milestone certifies full-frame parity. Do not stop at another partial implementation report or check off unverified/blocked acceptance.

## Server-backed nation/city-influence borders

- [x] Audit server9dfe6ef `docs/city-development-contract.md`, public identity/fog projection and released `TiledMap::{DrawNationalBorders,DrawColoredBorderEdge,GenerateHitMask}`, Graphics construction/callbacks/default profile and original LDL. Three causal REDs reproduce ignored persisted slots, absent influence and disabled display callbacks.
- [x] Consume exact opaque viewer/public persisted indices and nullable observed `influence_city_id`; supersede old setup-order colors whenever new metadata is present. Missing indices are unknown, not self-blue/index0. Solid national and independent dashed white influence use received assignments only, explicit empty versus unknown, matching currently visible city IDs and revision-indexed observation caches. No local region/nearest-city computation, hidden city request or settlement-preview substitute.
- [x] Restore original three-pixel/two-row dash stencil, safe partially clipped raster and existing integer zoom; fix original east black-pixel typo/post-increment overlap. Actual Graphics source checkboxes now change their local flags/check art/localized captions; shell choices initialize them, receipts and Close preserve them, Pending never becomes acknowledgement. Native/model/runtime regressions cover fields/identity/fog/visibility/all four sides/known unowned/cached growth-shrink-capture-removal/zoom and semantic input; isolated bootstrap SSE retains the new metadata.
- [ ] General in-game option persistence, complete border/caption/other-map styling and all33 full frames/Windows/live gameplay remain unaccepted. This integration changes no server implementation, installed data or scaling; see [current authority evidence](reference-parity-contract-gaps.md#repaired-national-territory-city-influence-and-persisted-identities-server9dfe6ef).

## Distinct owner/player-color repair (historical; persisted server slots now preferred)

- [x] Reproduce collapsed foreign owners as a causal RED; retain exact opaque owner strings, exact authenticated self and city identity rather than suffix/substring/coordinate guesses. Cover separate peers, misleading IDs, co-located cities, reorder, hidden/reveal and received capture.
- [x] Audit released `ColorSet.cpp`, `colorset.h`, `pixelutils.h`, `UnitActor::{DrawHerald,DrawStackingIndicator}`, `TiledMap::{DrawCityNames,DrawNationalBorders,GetVisibleCellOwner}` and original `Colors00.txt`. Read the installed table once, never copy/embed assets; share packed-format normal/dark colors across map actors/flags, panel herald, cities and observed borders. Original remembered borders stay normal, city colors darken.
- [x] Keep the already-returned authenticated start-game public membership order as frozen **client display** slots1–16, barbarian0; absent/invalid/overflow stays unknown/neutral. No ID hash/modulo, country/encounter ordering, canonical slots, hidden knowledge or extra HTTP reads. Nine pure/native/mock cases cover the workflow; native readback checks three foreign actor colors, city/fog boxes and known-only political edges. See [authority/color evidence](reference-parity-contract-gaps.md#distinct-observed-owners-and-original-player-palette--client-repair).
- [ ] Native canonical slot/Science and other manager-header knowledge integration, alive-player progress, complete caption/extra-map-flag/HUD-herald styling and all33 full frames remain unaccepted. This color repair does not certify those unrelated screens or Windows/live gameplay.

## Current server contract integration

The server has independently implemented contract v2, including retained destinations, ruin rewards and city production governors. The completed client-only contract integration is in [PLAN.md's implementation ledger](../PLAN.md#completed-implementation-ledger); current outstanding scope/priorities are owned by that main plan. Earlier blocker notes below are historical, not the current server feature status. Do not modify the server.

The v2 client integration now implements typed optional observations/capabilities, phase-aware closeout, stable-entry atomic queues, explicit city/governor/goal/target controls, asynchronous viewer-only synchronization, retained journeys and rollback, cached display facts, semantic messages and actual own-history graphs. Six coherent slices are tracked in the child plan; complete-reference screenshots, unintegrated advisors/managers and unavailable culture/style/optimizer contracts remain separate. Current verification: 1065 test executions and 84.27% LLVM line coverage; native build passes. Windows validation is blocked before client compilation by the missing GNU Rust target and GTK/GLib pkg-config cross sysroot. No live server or server file was changed.

## Historical server-first request and instruction blocker

The user subsequently authorized server work and asked to fix `client-discovered-server-issue.md` first. Both that untracked singular report and the tracked plural report, plus the server's complete `AGENTS.md`, were inspected read-only. The developer-provided client project instructions still explicitly prohibit **any server modifications**. No server source, tests, plans, reports, hooks or configuration were changed; its pre-existing untracked singular report is untouched. Server implementation cannot proceed under this session's governing instructions. Do not mark affected checkboxes complete or claim that client defaults repair missing authoritative facts.

The singular report requires a versioned generated-world river/good/ownership handoff, visible tile projection, viewer-specific last-observed metadata and legacy-unknown handling. Its in-memory regressions cover visible/stale/re-observed facts, hidden canonical changes, player isolation, serialization/replay/idempotency and the world-start handoff. The plural report additionally requires canonical city names/type, registered legal routes and safe queue contracts. These are source-confirmed server work, not reasons to stop all unblocked client repairs.

## Current reported gameplay regressions (all required; no partial completion)

- [x] Inspect `images/singleplayer-starting-again-cut-off-bottom.png` against the original city, movement and terrain-info references.
- [x] Replace blue city rectangles with installed original city sprites, source anchoring, viewer-safe names/population/production/ownership labels; test owned, remembered and hidden cities in native SDL. Observed age/capital/production forecasts are now consumed; unprovided architectural culture remains explicit fallback styling, not inferred state.
- [x] Reconcile movement-line anchors, stroke and numbered turn boxes with terrain diamonds at all zoom levels and scaled viewport input; add visual/input regressions. Native line pixels and zoom1–3 geometry now use the actual diamond centre; terrain-aware retained-order acceptance remains separately unchecked.
- [x] Cache and show actual server-projected tile yields, movement, goods and remembered facts without lookup requests or fabricated zero values; precisely document missing server fields/contracts. General inspection costs remain distinct from planning authorization; omitted legacy values remain unknown.
- [x] Make Enter/keypad Enter and the original End Turn button issue the exact authoritative command, apply receipts and handle errors/phase synchronization with mocked HTTP tests. Waiting/locked actors cannot close a turn; queued hybrid acknowledgements are not movement completion.
- [x] Hide the OS cursor throughout gameplay, maintain original software cursor hotspot alignment across resizing/letterboxing/high DPI and restore inherited cursor state on exit; add scaled SDL pointer tests. Actual SDL injection reproduced 450→449 truncation, fixed by shared integer presentation/input mapping; pure tests cover native, pillarboxed, portrait and high-DPI transforms. Reassert hiding each frame.

City repair evidence: native SDL now compares actual GC028 building pixels and original hotspot placement against an independently loaded idle frame, including packed-channel remembered shading. Installed city style/age/population recipes and every referenced GC group decode read-only; `SpriteGroupList.cpp` stores these in UnitSpriteGroup containers, not the unused city-reader stub. Foreign fresh-on-stale metadata is rejected even with local fog disabled. Projected names/queue heads no longer become machine IDs/hardcoded Warriors. Contract v2 now supplies permitted name/age/capital/production forecasts and those are consumed. Architectural culture/style are still unavailable; explicit default styling is not a claim about a city's civilization. This repair is **not** accepted as complete reference parity.

The user confirmed the cut-off bottom in this capture was caused by their image capture, **not a client defect**. Do not change viewport layout to compensate for the crop.
- [ ] Run lint, full tests, ≥80% coverage, native build, Windows validation/environment check and atomic verified commits; continue every remaining unchecked item below.

## Checklist

- [x] Inspect all 33 reference PNGs individually and read every filename.
- [x] Record a per-reference context inventory and grouped source follow-up map, explicitly separating missing client integration from unknown source behavior.
- [x] Trace original cursor loading/hotspots/hand, edge thresholds/default delay/acceleration, directional mirroring and animation/path timing. Retain existing packed-channel fog rules; reproduce the missing-tile foreign-actor leak.
- [x] Search supplied original UI/source/data for Optimize Sliders, then locate the Apolyton callback and Governor evaluator. Distinguish the original-release/source mismatch from the remaining authoritative evaluation contract gap.
- [x] Add red → green cursor/animation API contracts, reproduce the foreign-actor leak as a failing SDL pixel test, and add in-memory SDL artwork comparisons plus dispatch, latency and option-handoff regressions.
- [x] Implement original-art hand/move/blocked/assault/edge cursors with exact hotspots; suppress existing modal/focus-lost scrolling, preserve fractional ticks and update stationary-pointer pathing after a pan. Public non-wrapped bounds now constrain camera/input without discovery; wrapping remains unsupported by the current server.
- [x] Implement directional SPR sampling and a shared travel/frame clock for accepted prefixes of explicitly submitted moves, for the currently supported Settler/Warrior/barbarian families. Never predict a server outcome. Full unit roster and end-turn actor events remain outstanding.
- [ ] Reconcile right-click menus and map information with the referenced click contexts; no immediate hidden command dispatch.
- [ ] Audit fog-safe actors, map details and remembered terrain against source/server contracts; add regressions.
- [ ] Implement and visually reconcile all five control-panel tabs, stack expansion, messages/popups and city/build workflows.
- [ ] Implement and visually reconcile Empire, National, Unit, Trade, Diplomacy, Science, Library, Statistics and in-game Options screens with functional source-verified controls.
- [x] Inspect an actual native SDL motion/cursor capture and add an explicit deterministic motion diagnostic. Document remaining differences without claiming complete parity.
- [x] Run strict lint, full tests, the missing coverage-script command and LLVM coverage, dedicated native build and final review; attempt Windows cross-check and report the environment failure.
- [x] Update `FIXES.txt`, `PLAN.md` and visual-testing documentation; run pre-commit `cargo make lint-fix` and strict lint; commit this partial pass normally, without marking the remaining request complete.

## Completion checklist (continuation; every item requires source, implementation and tests)

### Authoritative data and commands
- [ ] Consume viewer-safe legal actions, tile movement planning, runtime rule definitions, research, diplomacy, production/queues, city reports, trade routes and unit health/stats/fortification without inventing missing facts.
- [ ] Preserve exact server identities and selected city/unit across receipts; add mocked command-dispatch tests that verify route, revision and payload.
- [x] Separate Fortify, Sentinel/Sleep and other orders; never silently submit a different order for an unsupported action.
- [ ] Document individually any base-game operation with no authoritative route; do not mutate local authoritative state as a substitute.

### Right-click workflows and fog
- [x] City-over-unit context priority, source-order menu labels, explicit selection/cancel, keyboard Escape and semantic accessible activation.
- [x] Friendly-unit context menu with correct targeted unit identity, legal/illegal actions, no command on menu open.
- [ ] Empty-terrain Map Info with original frame/art and visibility-safe yields/movement/goods; unknown geography never disclosed.
- [ ] City/unit/terrain context-menu native framebuffer and interaction regressions.
- [ ] Remembered city/terrain/overlay state, visibility transitions and all metadata consumers audited with adversarial fog tests.

### Units, movement and camera
- [x] All installed retail unit/art mappings (including Hoplite and Archer); unknown canonical types never impersonate Warriors.
- [ ] Direction/action/idle/attack/death/fortification art and metadata; missing action fallbacks checked against source.
- [ ] End-turn/queued accepted movement animation and moving-stack hit testing.
- [ ] Authoritative terrain-cost-aware path previews and numbered turn boundaries, including depleted movement and movement into cities.
- [ ] Bounded/wrapped camera and cursor feedback at screen/world edges and zoom levels.

### Control panel and notifications
- [ ] Empire summary: real population, cities, PW, government, science/research and pollution.
- [ ] Empty/selected City tab, portrait, production, selector, mayor and rush buy.
- [ ] Unit stats/health/stack herald, expanded stack portrait grid and exact semantic selection.
- [ ] Empty/nonempty Msgs list, unread state, completion/ruin popups, date and eye/location navigation.
- [ ] Tile categories, improvements, affordability/legality and feedback.
- [ ] Original city sprites/labels/production/ownership rendering and settlement radius preview.

### Managers and dialogs

Implementation substeps (do not equate a framework or disabled button with completed acceptance):
- [x] Build source-layout inheritance/geometry, original nine-piece manager frames and localized control labels with read-only installed-data and in-memory parser tests.
- [x] Share manager semantic descriptors between native SDL painting, pointer/keyboard events and accessible test activation; prevent modal click/shortcut/scroll-through.
- [ ] Reconcile original modeless versus strongly-modal manager behavior, related-window return navigation, source-created advisor panes, dropdowns, list headers/rangers and runtime control resizing for every manager.
  - [x] Separate original list frames from content/header/ranger geometry; bound complete-row capacity and zero extents. Render cached original Build/Science/Research item templates with clipped numeric columns and explicit unavailable values.
  - [x] Integrate source city selector previous/next cycling, collapsed caption/five-part artwork, popup selection/scrolling/keyboard/cancellation and exact current owned-city identity; test selection/command handoff across receipts without mutating game state.
- [ ] Preserve independent city, buildable-item, queue-entry, unit, advance, trade-target and contact selections through authoritative receipts; invalidate stale legal payloads.
- [x] Exercise implemented Add, Rush Buy, Change Research and founding handoff against isolated mocked HTTP/revision/payload contracts and viewer-safe receipts. This does not certify unimplemented manager operations.
- [ ] Add equivalent transport/receipt regressions for each remaining newly integrated manager operation.
- [ ] Render real empty/populated management fixtures and inspect captures; retain per-scenario differences rather than certifying framework coverage as pixel parity.
- [ ] Bind source Graphics/Sound controls to live state/animation/audio and test option handoff without touching files.

- [ ] Build Manager: automatic opening after settlement, Units/Buildings/Wonders, independent list/queue selections, descriptions, Add/Insert/Remove/Clear/Move Up/Down, Suggest/Load/custom queues, rush buy and library links.
- [ ] Empire Manager: policies, tax/PW, happiness, government, Details and source-verified Optimize Sliders (or precise demonstrated source/contract blocker).
  - [x] Bind source rations/workday/wages sliders and science/PW spinners to revision-checked typed parameter commands, with native range geometry/input, untouched companion values, mocked receipt tests and no live counterfactual policy probing.
  - [x] Restore source Empire Details/Summary visibility, keep Details inside Empire rather than substituting National Manager, and preserve the mode across tabs.
  - [ ] Integrate government comparison/eligible enactment and source-valued displays without inventing unavailable economy totals.
- [ ] National Manager: Resources/Status/Specialists and selection-gated city/build/disband controls.
- [ ] Unit/Army Manager: stats/tactical/advice/readiness/disband/army composition.
- [ ] Trade Manager: Market/Export/Import, advisor, filters, transport accounting and route creation.
- [ ] Diplomacy Manager: contacts/intelligence/proposals/war/embargo and disabled empty-contact states.
- [ ] Science Manager: current research/progress/cost/available advances and Change Research.
  - [x] Correct the discovery table to display viewer-known advances, not currently available research candidates. Keep the picker separate; deduplicate exact identities and sort both from projected costs. Draw only verified own-player knowledge/header blocks, never infer foreign research from embassy identities.
- [ ] Great Library: categories/search/articles/links/tabs/history/navigation/research goals. Component checklist and exact remaining chart/editor/style work: [great-library-workflows.md](great-library-workflows.md).
  - [x] Installed sections/art associations; explicit global search, measured internal hyperlinks, Gameplay/Historical content, bounded Back/Forward, source descriptions and parent-window return.
  - [x] Bound stored list/article offsets and test recovery from extreme scrolling, without duplicate variable evaluation or gameplay input leakage.
  - [x] Projected advance dependency chart/recentering, overflow pages and a functional center article callback, with native research-border tests.
  - [x] Source sort/index/ranger controls, bounded native keyboard/pointer/clipboard/IME editing and verified numeric mappings; no guessed movement/fuel units.
  - [x] Session-owned per-control font styles/metrics, original LDL precedence/draw order, right alignment/shadows and long title overhang.
  - [x] Persistent authoritative research goals: contractv2 plus original item enabling-advance callback, typed exact advertised choices and queued-state/native source keyboard/HTTP regressions; see [main completed goal-callback ledger](../PLAN.md#completed-implementation-ledger),3c19f55.
  - [ ] Complete per-reference typography/layout/art acceptance; do not treat the implemented mechanisms as pixel certification.
- [ ] Statistics: Score/Wonders/Ranking, selectors/graph/legend/Zero Sum Graph.
- [ ] In-game Graphics/Sound dialogs: original frame/controls/positions, live option effects and unit-speed synchronization.

### Additional dependencies discovered during continuation
- [x] Correct distinct Empire/National/Trade entry points and Score/Ranking/Wonders shortcuts/tab selection; test that manager entry is not just a status label.
- [x] Use exact server-recommended founding identity/name instead of repeating `New City`; automatically open the accepted new city's Build Manager once, without selecting the first old city.
- [x] Bind unambiguous Board Transport to its exact legal embark payload; keep multiple-target cases unavailable until a chooser exists. Disable fake-enabled Add to City/Board controls.
- [x] Render authoritative single/expanded unit health with original 0.333/0.666 color thresholds, zero/unknown health handling and overflow-safe ratios; stop hardcoding firepower 1.
- [x] Implement explicit transport/special-order target choosers for ambiguous legal candidates. Both native and semantic forms use exact current legal payloads; no first-candidate dispatch.
- [ ] Reconcile Apolyton/reference layout additions absent from the installed original-release LDL: Graphics' second slider/twelve switches, Build Suggest/National links, National Specialists and other changed controls. Public layouts were located under `ctp2_data/default/uidata/layouts`, not `english`.
- [ ] Replace remaining unbound manager controls with source-verified implementations, not enabled no-ops or guessed server commands.
- [ ] Consume authoritative city labels/type, calendar/date, world bounds/wrap, permitted history/graph data, notifications and ruin results where provided; establish precise contract gaps for absent data. Static artwork is not an authoritative gameplay fact.
- [ ] Remove hardcoded/nonfunctional panel body values and connect all map display options to actual rendering/audio effects.
- [x] Audit received/queued multi-turn movement against engine atomic path rejection and ZOC; a cost-aware preview is not implementation of retained multi-turn orders. Retained destinations/cancellation use server-owned orders, support exhausted units and reconnect restoration, and never replay locally. Provisional pixels roll back on disagreement; confirmed animation requires a submitted/retained route.
- [x] Audit real-time viewer synchronization and notification/animation event handoff; no canonical hidden information may be inferred from omitted fields. Streaming invalidations trigger complete safe current reads; revision guards, stable lost-response retries, queued-versus-executed state and actual command/transport rejection recovery have isolated regressions. Historical notice display is immutable/no transport and cannot reopen recovery or overwrite Pending; [main sync-ownership ledger](../PLAN.md#completed-implementation-ledger),3df0793,supersedes initial display-triggered semantic resync.

### Acceptance
- [ ] Match each of the 33 reference scenarios with deterministic viewer-safe fixtures and real SDL output, not just component tests.
- [ ] Semantic end-to-end clicks cover every visible enabled control, dialog navigation, selection and disabled states.
- [ ] Visually inspect all reference-scenario outputs at native and reference sizes; reconcile fonts/radar/frame scaling.
- [ ] Run strict lint, all tests, ≥80% coverage, dedicated Linux build, Windows validation or exact environment blocker, final review and normal verified commit.

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
20. `right-click-terrain-with-friendly-city-and-fortified-unit-in-city.png`: unselected explicitly sheltered garrison leaves city context (City/Build/Rename/Disband/Mayor/Map Info). This is not unconditional city priority: complete original BuildList appends current selected/eligible army rows before city; [main call-chain ledger](../PLAN.md#completed-implementation-ledger),fbeb093,corrects the earlier blanket rule.
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
- **Optimize Sliders source is now established (supersedes the first-pass uncertainty).** The supplied original release predates the button. Public Apolyton `ctp2_code/ui/interface/CauseAndEffectTab.cpp`, `OptimizeSlidersButtonActionCallback`, invokes `Governor::OptimizeSliders`, applies `Governor::SetSliders`, and refreshes resource displays; modification notes date the addition to April 2005. `ctp2_code/ai/CityManagement/governor.cpp` evaluates trial policies using specialist reassignment, happiness and complete city economic/resource calculations. A `set-social-policy` mutation route alone is not a verified equivalent counterfactual evaluator; do not substitute guessed arithmetic or repeatedly mutate a live game to search settings. See `$CTPSERVER/client-discovered-server-issues.md` (server documentation commit `5ed6237`) for evidence, distinct defect/feature-gap classifications and proposed in-memory regressions.
- The server already exposes social-policy, tax/PW rates, government, readiness, research, disband, fortify, army and production routes (`$CTPSERVER/internal/gameplay/http/http.go`). Their absence from the current UI is a **client integration gap**, not evidence they are impossible. With explicit user authorization, only the server issue report was added in server commit `5ed6237`; server implementation remains untouched and no new test contacts the running server.

## Remaining UI source map

Reference numbers refer to the complete viewed inventory above. These are follow-up entry points, not claims that every callback has already been traced:

| References | Original source entry points | Remaining client work |
| --- | --- | --- |
| 1, 2, 15, 23 | `MessageControlPanel.cpp`, `messagelist.cpp`, `messagewindow.cpp`, `messageeyepoint.cpp` | Read/unread rows, date/eye popups, authoritative completion/ruin messages |
| 3–7, 14, 18, 20 | `EditQueue.cpp`, `CityControlPanel.cpp`, `citymanager.cpp`, `NationalManagementDialog.cpp` | City selection, queue editing/details, build-completion workflow, mayor/rush buy, selection/stance-dependent city/garrison context |
| 8 | `diplomacywindow.cpp`, `DiplomacyDetails.cpp`, diplomacy LDLs | Contact data, selection-gated actions, proposals and intelligence |
| 9, 10 | `DomesticControlPanel.cpp`, `DomesticManagementDialog.cpp`, `CauseAndEffectTab.cpp`, `GovernmentTab.cpp` | Empire summary/dialog, slider calculations and authoritative commands; Optimize callback located, authoritative trial-evaluation contract unresolved |
| 11, 13, 16, 17, 21, 24–27, 29–33 | `UnitControlPanel.cpp`, `unitmanager.cpp`, `armymanagerwindow.cpp`, `controlpanelwindow.cpp`, `TilesControlPanel.cpp`, `ScienceManagementDialog.cpp`, `greatlibrary.cpp` | Full unit metadata/art roster, fortification, semantic stack expansion, real context menus and terrain info, cost-based path turn labels, research/library, improvement legality/PW costs |
| 12, 19, 22, 28 | `graphics.ldl`, `sound.ldl`, `infowindow.cpp`, `trademanager.cpp`, `controlpanelwindow.cpp` | Functional in-game options, statistics, terrain info and trade manager with source placement/art |

Foreign actors now require explicitly visible tiles; hidden/stale/missing tile pixel tests pass even with local fog disabled. This is a targeted safety repair, **not full sightline/remembered-city/fortification parity certification**. Server river/resource/ownership and interaction contract findings are documented in `$CTPSERVER/client-discovered-server-issues.md`, including 18 advertised-but-unregistered HTTP actions, non-head cancellation progress loss, atomic/repeated queue limitations and missing city-name projection. Client-side windows and already-supported operation bindings remain client responsibilities.

## Validation evidence (original committed cursor/motion pass)

- Red: missing cursor/animation/probe APIs; fog pixel test failed with `foreign actor with missing tile leaked`. Green: native SDL pixels match source cursor art and source MOVE frame selection for both supported art families, eight directions × four phases; behavioral tests cover accepted prefixes, rejected motion, speed, frame tables/deltas/alpha/delay, response latency, focus, modal scrolling, frame-rate independence and shell handoff.
- All new rendering/screenshot tests retain RGBA/PNG data in memory. The opt-in example alone writes diagnostic screenshots. `images/reference-motion-nw.png` was generated and visually inspected, not committed; it is synthetic and plainly differs from the complete reference scenarios.
- `cargo make lint`: passed. `cargo make test`: **326 library + 344 binary + 19 integration** tests passed. `cargo build`: passed. LLVM line coverage: **80.88%** overall; animation presenter **96.70%**, cursor policy **92.52%**, directional texture sampler **88.57%**. `./scripts/check-coverage-fast.sh` was attempted and is absent.
- `cargo check --target x86_64-pc-windows-msvc` was attempted and stopped in `gio-sys`: the host has no configured GLib/GIO cross-compiling pkg-config/sysroot. The new presentation code uses portable SDL/Rust APIs, but Windows binaries have not been validated.
- Final review: no original art copied/committed; restricted paths and server files untouched; no new file-mutating/live-server tests; new modules below 800 LOC; no new allow attributes; selection/authority remain model-owned. Full client completeness is explicitly not asserted.

## Continuation implementation evidence (not full acceptance)

- Retain viewer legal actions, research/rules/reports, projected unit/city/tile facts and movement planning. Installed art resolution now covers all 72 retail unit records using `newsprite.txt`, including distinct Hoplite/Archer art. Source zero-frame fallback and Windows-1252 database text are tested.
- Context open/select/cancel uses real semantic rows rather than issuing a hidden first command. Native SDL tests cover terrain gold-frame artwork, selected context rows and hidden information suppression. Selected-versus-sheltered city/garrison composition and exact actor/target identity have pure interaction tests; [main call-chain ledger](../PLAN.md#completed-implementation-ledger),fbeb093,corrects the earlier blanket city priority and panel-only/no-popup audit. Full city-context framebuffer parity remains unchecked.
- Fourteen source-layout roots resolve and render with native SDL. Inheritance merges child overrides without changing draw order; original main-window x≥224 centering and image-derived unspecified dimensions are tested. Source wordwrap and health-bar pixels have regressions. Frame-corner comparisons are component evidence, not complete screenshot matches.
- Buildable and queue selections are independent. Add/Rush Buy/Research submit exact revision-bound payloads through isolated loopback HTTP stubs, not the real server. Founding uses the viewer-provided unique city identity/name; its accepted receipt opens the correct new city once. Unsafe queue rewriting, duplicate queues and progress-destroying cancellation are not disguised as functional controls.
- Explicit manager previews are available through `gameplay-snapshot` (`build`, `science`, `research`, `empire`, `national`, `city`, `unit`, `army`, `trade`, `diplomacy`, `library`, `statistics`, `graphics`, `sound`). `context`, `stack` and `info` are separate component diagnostics. The inspected context/stack/info and native Build/Science captures still differ from complete references; no 33-scenario acceptance is claimed.
- Many policy, specialist, diplomacy, trade, statistics and options controls remain unbound/disabled; Library browsing/search/history and source description bindings are now functional, but the dependency chart, sort/ranger controls, editor completeness, remaining numeric mappings and precise typography/layout remain unfinished. Panel/message/city rendering and attack/death/end-turn animation still require implementation. The remaining client work is **not** all server-blocked. Exact server defects/gaps remain linked above and cannot be fixed by a client-local simulation.

Continuation milestone checks: `cargo make lint-fix`, `cargo make lint`, `cargo make test` (**364 library + 382 binary + 19 integration**), `cargo build` and LLVM coverage passed; total line coverage **81.55%**. The requested fast script is absent. Repeated Windows x86_64 MSVC check failed in `gobject-sys` before client compilation: no configured GLib/GObject pkg-config cross sysroot. Source/image files remain read-only; tests create no screenshot/configuration files and only isolated HTTP stubs are contacted.

## Great Library continuation evidence

See [the component work plan](great-library-workflows.md). Installed articles remain read-only; links resolve exact catalog topics, not paths/URLs, and SLIC expressions never execute. Search is explicitly executed across original article labels/text and cached rather than rescanned per frame. Production input tests exercise search, exact results, Cities links, Back/Forward and return to the selected Build window without changing the authoritative snapshot. Unsupported future research goals are not replaced with immediate research mutations.

Independent SDL comparisons cover the original Ballistics Library/Science and selected Settler Build pictures at declared coordinates, blue underlined hyperlinks and native white search input. They reproduced missing portrait stretch and oversized title art erasing siblings; clipping and declared overlay scaling now pass. The obsolete negative-position search caption, missing uppercase title key and random joke search initialization are not preserved. Native/reference-sized Library captures were inspected; font/layout/runtime-created-control differences remain, so the reference scenario is not accepted yet. The unimplemented dependency chart is explicitly unavailable, not a fake-enabled blank tab.

Library milestone validation: `cargo make lint-fix`, strict lint, full tests (**381 library + 399 binary + 19 integration executions**), native `cargo build` and LLVM coverage pass; overall line coverage **82.09%**. Required fast coverage script is absent. Repeated Windows x86_64 MSVC checking stops before client compilation in GTK native dependencies (Pango/Cairo/GdkPixbuf) because cross-compiling pkg-config/sysroots are not configured. No new test writes files or contacts a running server; no installed assets, restricted paths, main-menu code or server implementation were changed.

## Chart/editor/source-typography continuation

- Implemented current-rule advance dependencies, known/available/unavailable borders, local recentering and article navigation. Overflow is paginated rather than silently dropping neighbors beyond the original four. Missing rule data disables the chart; it no longer represents unimplemented client work.
- Added complete bounded local editor mechanics, measured pointer/selection/IME rendering, explicit injected clipboard operations and AltGr safety. Scrollbars use original track/thumb/arrow artwork with arrow/page/drag/keyboard interaction. Sort by Age uses projected enabling/age associations and labels missing associations unavailable.
- Source review exposed a client LDL resolver bug missed by earlier single-inheritance tests: `ldl_datablock::CopyAttributesFrom` fills absent attributes, so first template wins, inherited child attributes precede child fallback templates, and `aui_Region::DrawChildren` paints tail-to-head. New pure/native regressions cover the corrected behavior.
- Source font face/point identities, 96-DPI sizing, measured hyperlink/editor geometry, declared shadows/right justification and narrow-title horizontal overhang now share the actual native font set. Fonts are loaded once per session. This host lacks Times fonts; explicit Liberation fallback does not certify identical text pixels.
- Inspect populated diagnostic projections separately from empty scenes. Ignored native/reference-sized Library, Build, Science and chart captures show actual output; picture framing/content, fonts, advisor/control/data columns and full workflows still differ. No 33-scenario acceptance is claimed.
- Historical persistent-goal contract gap is superseded by contractv2 and the verified original Library item-enabler integration. Original Player::SetResearchGoal maps units/buildings/wonders/governments to enabling advances; client uses only current projected prerequisite/enable_advance_id facts and advertised persistent-goal actions, never immediate set-research or local recursive planning. Server implementation remains untouched; other incomplete manager work remains open.

Chart/editor continuation validation: `cargo make lint-fix`, strict lint, full tests (**410 library + 428 binary + 19 integration = 857 executions**), dedicated native build and LLVM coverage pass; **82.91%** line coverage (38,212 lines, 6,529 missed). Required fast script is still absent. Windows checking fails before client compilation in GTK/GLib native dependencies because the cross-compiling pkg-config/sysroots are not configured. Native glyph/ranger/chart/clipboard tests are in-memory; the opt-in diagnostic alone writes ignored captures. No server implementation, restricted paths, installed artwork or main-menu code were changed; changed Rust files remain below 800 lines.

## Runtime list/row checkpoint (continue the main plan afterward)

Completed source lifecycle/cached item-template evidence is retained in [PLAN.md's implementation ledger](../PLAN.md#completed-implementation-ledger) and commitb3fbbd8 (retired manager-runtime-geometry component). New pure/native regressions reproduce and repair border/content conflation, shifted scroll arrows, phantom zero-sized rows/SDL pixels and missing Build stat columns. Science now distinguishes known discoveries from research candidates and paints original own-player knowledge/header blocks without disclosing unknown foreign facts. Item children clip to their content panes; pointer/semantic activation remains bound to the exact parent identity.

Verified missing economy facts remain explicit: the current `CityReport` does not expose complete yields, research projection exposes no rate, and runtime unit movement has no universal retail 100-point scale. Missing production/research turn estimates remain unavailable. This is not a local economy simulator or a server implementation change.

Lint-fix/strict lint, **419 library + 437 binary + 19 integration = 875 executions**, dedicated native build and **83.15%** LLVM line coverage pass. Native/reference-sized populated Build/Science and Library captures were inspected; selector/popup lifecycle, fonts, picture content/framing, advisors and remaining workflows still differ. Windows remains blocked before client compilation by GLib/GTK cross-compiling pkg-config setup; the required fast coverage script remains absent. Continue next with the main plan's source-created city selectors/dropdowns and remaining authoritative manager operations, not a final completion report.

## City selector checkpoint (continue the main plan afterward)

- Source `ctp2_DropDown` constructs a detached popup; it must not be painted beneath a closed selector. A failing regression found the city caption drawn seven times. Caption/button/popup now follow the original lifecycle, with `cw_CityItem` source rows, five-part native button artwork and explicit normal/disabled arrow layer flags (not hover art as the normal state).
- Build `SingleCityChooser` and City Manager `CityList` previous/next callbacks cycle exact currently owned IDs in source order. Selection invalidates only dependent city-specific queue/buildable/description state. Keyboard/pointer selection, complete-row scrolling, outside/Escape/focus cancellation and stale-highlight rejection do not mutate authoritative state. Dismissing a popup cannot activate the underlying Previous City control.
- An isolated HTTP test verifies that selecting city-8 and Add sends city-8's exact 440-point payload, not city-7's 330-point payload, with current session/revision and receipt identity retention. No running server is contacted.
- `city-chooser` is a separate explicit two-city preview; existing previews stay unchanged. Native/reference-size open and native closed captures were inspected. Full Build/City screen parity still needs the remaining controls, art/font reconciliation and authoritative missing data.
- Lint-fix/strict lint, **432 library + 450 binary + 19 integration = 901 executions**, native build and LLVM coverage **83.43%** pass (40,192 lines; 6,660 missed). Windows/fast-script environment blockers are unchanged. Continue with remaining authoritative manager operations; these commits are checkpoints, not task completion.

## Empire policy checkpoint (continue government/layout and the main plan)

- Original `CauseAndEffectTab` maps rations/wages to index-minus-two and workday to two-minus-index. Fix its pre-inversion comparison bug instead of preserving suppressed valid workday changes. Retain both untouched current companions, source five-level geometry, native 9x20 thumb/pattern/groove, spinner arrows/current display and ten-percent increments.
- Parameterize only the registered science/public-works/social-policy routes after validating their entire advertised field/sample schema, bounded integer payload and captured revision. Sample recommendations do not enumerate their domain. No live trial-policy optimizer or local economic simulation is used.
- Native/accessible value buttons, thumb capture/release and keyboard carry the displayed revision; newer receipts cancel rather than silently recomputing another receipt's companions. Dragging/focus loss/Escape affect presentation only. Isolated HTTP tests verify all three exact parameter envelopes and receipt-owned display (requested science60, mock receipt57, displayed57).
- Verify `RuntimeGovernmentDefinition.max_science_percent` against the server schema; never identify governments by two missing IDs, accept malformed caps as unrestricted, or rewrite the received current policy with a local clamp. Missing economy/physical-policy conversion facts are not fabricated.
- `empire-populated` is a separate explicit policy fixture. Inspect ignored native/reference-sized `images/policy-empire-{native,reference}.png`; installed slider/arrow pixels pass independent memory-only source oracles, but Empire summary/detail labels still overlap, some installed text keys are absent, and the retail root/column/spinner placement differs substantially from the reference's Apolyton layout. Those are outstanding client layout/lifecycle work, not server blockers or accepted screenshot matches. Public community `ctp2_data/default/uidata/layouts/civstatus.ldl` establishes the 800x558 root, 152-pixel social sliders, top-level `PWSpinner`/`ScieTaxSpinner`, expanded columns and Optimize button.

Policy checkpoint validation: lint-fix/strict lint, **445 library + 463 binary + 19 integration = 927 executions**, dedicated native build and **83.79%** LLVM line coverage pass (41,530 lines; 6,731 missed). Repeated required fast-script command exits127 because the script is absent. Windows checking still stops before client compilation in GTK/GLib cross-compiling pkg-config/sysroot setup. No server implementation, installed art or restricted paths changed; tests do not write files or contact a running server. Continue with government/layout and the main plan.

## Launch blocker reported during UI continuation

- [x] Reproduce non-UTF-8 original `Units.txt` through an injected, fileless installation reader; cover every unit-art input, UTF-8 mods and contextual I/O failures.
- [x] Decode original database bytes with the shared Windows-1252/UTF-8 decoder, and audit the remaining gameplay text loaders. Never edit original assets or source databases.
- [x] Run strict lint, full tests, coverage and native build; update documentation and commit the isolated launch fix normally.

Encoding blocker evidence: the injected production reader first failed on `Units.txt: invalid utf-8 sequence`, and a second test reproduced lost `gl_str.txt` I/O context. All four unit-art metadata reads now accept original ANSI comments; CP1252 punctuation/accents and UTF-8 mod labels survive unchanged. Audit repaired the same strict-UTF-8 assumption in goods/audio. No original file is rewritten or re-encoded, and tests create no files or server calls. Strict lint, **468 library + 486 binary + 19 integration = 973 executions**, dedicated Linux build and **84.51%** overall LLVM line coverage pass. The required fast script remains absent; Windows checking stops in GObject/GLib/GTK dependencies before client compilation because cross-compiling pkg-config/sysroots are unconfigured.

## UI-first continuation (current request; reuse this plan)

- [x] Reconcile Empire's full-screen reference geometry and non-overlapping summary columns with source-backed visual tests. Full typography/art/data acceptance remains separate.
- [x] Restore reference Graphics/Sound presentation (second graphics slider, twelve switches, native grooves/ticks/thumbs), without claiming unbound options functional.
- [x] Restore all five lower-panel bodies from source artwork/geometry; test empty/populated viewer-safe cases and inspect native/reference captures. Msgs population/popups, City controls and complete per-scenario parity remain unchecked above.
- [ ] Reconcile remaining manager/advisor/reference scenarios, inspect all 33 outputs and record precise visual gaps rather than certifying component tests as parity.
- [ ] Run full lint/tests/coverage/build and commit UI changes atomically before reporting progress.

Lower-panel milestone: the formerly placeholder Empire/City and empty Msgs/Tile bodies now use cached original control trees, native source fonts/portraits, single-caption dropdowns, parchment list/ranger and five category radios. Palette categories have real local accessible/pointer selection without an improvement mutation; the Farm is static palette artwork, not a claim of affordability. Viewer-only counts exclude foreign/stale cities; missing economic rates, pollution and turns remain unavailable. Independent SDL portrait tests and a failing native short-endcap regression cover the shared renderer; declared-height strip scaling and icon-layer offsets repair clipped radio artwork and oversized palette icons. The existing Unit panel's firepower/armor positions were reversed relative to `controlpanel.ldl`; restored them. Empty/populated reference-sized captures were inspected, not accepted as complete matches.

Visual milestone: three failing presentation tests reproduced the small Empire root, missing Graphics rows/slider and unpositioned Sound thumbs. Native SDL regressions now cover original option thumbs, paired C3 ticks, unobscured gold corners and original RGB565 resource-column bevels. Masked comparisons read the supplied Graphics/Sound PNGs in memory, normalize their window-decoration offsets, and reject deliberately displaced gold corners. Masks tolerate capture quantization and do **not** certify whole-screen equality. The retail localization loader also incorrectly registered uppercase/numeric translations as keys via overlapping token windows; failing regressions now verify key/value-only parsing. Actual native/reference captures were inspected; installed Empire side art, missing economic/physical-policy values, full Details content and font differences remain explicit.

Incoming government/Details work was already present at the start of this continuation. Its fileless tests now pass: exact eligible government identity/comparison, stale command rejection and domestic Summary/Details lifecycle. No additional feature work is prioritized ahead of visuals.

The unchecked items are deliberately still required. Do not mark the complete user request satisfied merely because the parser/framework, component screenshots, or a verified partial milestone is committed.
