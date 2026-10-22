# CTP2 client — main implementation and acceptance plan

**Single entry point.** This file owns priorities, remaining scope and the 33-screen acceptance ledger. The linked files below are detailed checklists/evidence, not competing plans or separate completion targets. Do not create another plan for a small slice; update this file and the relevant existing detail checklist. Completed component plans are retired into the ledger below and remain recoverable from Git.

## Execution order

1. **Current user priority:** source-accurate UI screens/workflows, original mouse-selection behavior and all33 reference screens, ahead of other outstanding work. Current context-popup correction finished in `fbeb093`; work is now on Science Manager.
2. Science24 -> Build/City03–07/14 -> Empire09–10 -> Library13 -> National18, Unit29/Army, Trade28, Diplomacy08, Ranking19 -> panels/options/messages/context and the remaining reference scenarios. Implement original ruin-exploration wording/popup23 within that UI wave.
3. Keep the explicitly lower-priority October22 live-play follow-ups below, including the repeated local resync investigation. Only a demonstrated blocking dependency should displace the UI wave. Reports do not substitute for implementations.
4. Then finish every remaining main/detail checklist and final platform/end-to-end validation. A component commit, successful launch, crash-free turns, or favorable screenshot region is not whole-game completion.

## Working contract

- Target CTP2 first; preserve the largely intact main menu. Use **local original `ctp2_source`**, not civctp/civctp2/Apolyton/community behavior. Follow called implementations before drawing conclusions; explicitly classify installed/reference variants.
- Read installed art/fonts/strings/source only; no copies, asset edits or shipping reference pixels. Server implementation/configuration stays untouched. The sole new server-repo write requested by the user is `$CTPSERVER/server-client-schisms.md`, diagnostic documentation for server developers.
- Server authority governs local-singleplayer and multiplayer. No fabricated hidden cells/IDs, private foreign facts, effective economy, culture, research planning, guessed AI activity or accepted command outcomes. Preserve missing/null versus false/zero/empty. Queued intent is not execution.
- Native and accessible interactions share exact identities/descriptors; never silently choose a different/first target. Current revisions, ownership, phase and advertised payload shapes gate commands. Preserve immutable retries, monotonic complete snapshots and actual discrepancy rollback/recovery.
- TDD red-green-refactor; new tests must not write files or contact live services. Use injected bytes/clocks/channels, read-only installed assets, in-memory SDL and accurate isolated gateways. Keep Rust modules below800 lines; no new allow attributes; no hook/rule/script/build-file changes.

## Current task: Science Manager UI

Detailed source/TDD checklist: [Science Manager reference24](plans/science-manager-reference-parity.md).

- [x] Inspect reference24 and original ScienceManagementDialog/layout/inherited progress controls; trace ClearScience, description/list/link and research entry callbacks.
- [x] Implement/verify the current four UI fixes: source radial/numeric turn text agreement; clear known no-target/already-known target controls/icon without turning unprojected data into zero; selected known discovery updates description/links without changing active research; constructor removes stray off-window Advice from painting/focus. Four REDs and1294 executions/85.65% LLVM/lint/native build/normal hooks pass; committed4b02dbe. Science24 itself remains unaccepted.
- [ ] Complete original Research chooser/Cancel/Close/ESC/parent return, explicit available-ID selection, current request/queued receipt/stale and removed selections, header sorting and source row geometry.
- [x] Resolve demonstrated selected-installation picture lookup: GOG `CM2_Upap012l.tga` exists loose but old two-case guesses fell through to missing archive `.rim`. Immutable startup localized/default case-folded inventory now uses exact installed filename before existing565/555 fallback; unsafe/colliding/error cases fail honestly. Nine fileless regressions pass. Read-only GOG production Science frame now renders, still nonidentical:111639/307200 over delta8/raw11700143. No copied/wrong art or per-picture directory scans.
- [ ] Freeze a source-backed authorized reference scenario, compare every viewport pixel at justified native/reference sizes and inspect; current bundled diagnostic is **not identical** (112457/307200 pixels above delta8, raw RGB error12109902, origin5,23). No reference acceptance claimed.

## UI-first remaining implementation

- [ ] Complete Build/City production categories/independent queue-entry and content selections, descriptions, Add/Insert/Remove/Clear/reorder/custom queues, Suggest/Load and affordable revision-bound rush/library/selector/parent workflows. Preserve source loading screen and options/setup handoff.
- [ ] Complete Empire policy/government/Details/Summary/lifecycle and source layout; implement Optimize only under actual server authorization/counterfactual contract, never a second client economy engine.
- [ ] Complete National resources/status/specialists, Unit statistics/tactical/advice/readiness/disband/Army, Trade market/export/import/accounting/filters/advice and Diplomacy contacts/details/full proposal authoring/category/history/response/war/embassy/embargo under supported authority.
- [ ] Complete Science/Library categories/search/articles/links/history/chart/editor/goals, source current/progress/description panes and every enabled control. Item-to-enabling-advance goal mapping is implemented; whole screens are not accepted.
- [ ] Complete all five control-panel tabs, stack portraits/herald/health/expanded exact member selection, city labels/art/radius, Msgs dates/read/eye/results and source context/map-info controls, including empty/populated/unknown/remembered states.
- [ ] Restore original ruin text for each **confirmed** gold/advance/unit/city/empty outcome using original localization/substitutions. New current-client advance popup is not the original268-gold scenario; never substitute a gold reward or raw synthetic ID as a localized name.
- [ ] Complete original Unit/City/None mouse selection, default LEFT right-click deselection/cancel, static/moving/garrison/popup/terrain transitions, hover/descriptions and alternate preference. Preserve exact current painted identities, revision/origin-canceled held gestures and owned/visible privacy.
- [ ] Complete full stack/army native, keyboard and accessible journeys using rendered IDs, not canonical-destination guesses. Attack/death/fortification/accepted-motion presentation must use received facts; unknown enemy disappearance is not proof of death.
- [ ] Complete Graphics/Sound/gameplay options and actual effects/audio/speed, not just persisted shell state; original frame/ticks/rangers/fonts/focus/modal behavior. Unsupported saving/loading/optimizer/battle/other operations must be honestly unavailable, never enabled no-ops or file-writing tests.
- [ ] Complete Statistics score/wonders/ranking/selectors/graphs/legend/zero-sum within privacy. Never fill unknown foreign or effective data with zero, local approximations or fake strategic advice.
- [ ] Re-audit historical community-layout assumptions against local original source and reference/installation variants; reconcile fonts/art/capture geometry and every visible source-created pane/advisor/header/ranger/dialog.
- [ ] All enabled controls need semantic end-to-end navigation, keyboard/focus/IME/clipboard/confirmation/disabled reasons, stale receipts and exact mocked request/queued-versus-executed tests; component coverage is not complete workflow acceptance.

## Reference-screen acceptance

**0/33 certified.** Each item requires an authorized deterministic scenario, production SDL composition at native/reference size, exact semantic workflows, independently justified decoration crop, every-pixel/dense-cell comparison and actual inspection. No gameplay masks, favorable crops, tolerances or lookalike assets/fonts can certify parity. The source/evidence ledger is [full-screen reference parity](plans/full-screen-reference-parity.md).

- [ ] 01 building-complete-message-in-msgs-clicked-to-show-popup
- [ ] 02 building-complete-message-in-msgs
- [ ] 03 build-manager-multiple-items-added-granary-selected
- [ ] 04 build-manager-multiple-items-added-settler-selected-on-left
- [ ] 05 city-founded-then-build-manager-pops-up
- [ ] 06 city-tab-no-city
- [ ] 07 city-tab-with-city
- [ ] 08 diplomacy-manager-opened
- [ ] 09 empire-manager-opened
- [ ] 10 empire-tab
- [ ] 11 fortified-warrior
- [ ] 12 graphics-overlay-ingame-from-dropdown
- [ ] 13 great-library-opened
- [ ] 14 hoplite-built-in-rome
- [ ] 15 messages-tab-no-messages
- [ ] 16 moving-unit-line
- [ ] 17 moving-unit-to-city-line-notice-the-numbers-for-num-of-turns
- [ ] 18 national-manager-opened
- [ ] 19 ranking-overlay-from-stats-dropdown-ingame
- [ ] 20 right-click-terrain-with-friendly-city-and-fortified-unit-in-city
- [ ] 21 right-click-terrain-with-friendly-unit-on
- [ ] 22 right-click-terrain-with-no-unit-on
- [ ] 23 ruin-information-popup
- [ ] 24 science-manager-opened
- [ ] 25 settler-shows-green-tile-outlines-around-where-city-bounds-will-be
- [ ] 26 sound-overlay-from-dropdown-ingame
- [ ] 27 tile-tab-not-enough-pw-for-farm
- [ ] 28 trade-manager-opened
- [ ] 29 unit-manager-opened
- [ ] 30 unit-tab-multiple-units-in-capitol-rome-clicked-on-arrow-to-expand-units
- [ ] 31 unit-tab-multiple-units-in-capitol-rome
- [ ] 32 unit-tab
- [ ] 33 yellow-select-archer-can-no-longer-move

## Lower-priority live-play follow-ups

Detailed explained checkboxes and all6 inspected current-client captures: [October22 follow-ups](plans/oct-22-2026-ui-followups.md). Current-client pictures are comparison/bug evidence; original-game references remain normative.

- [ ] Correct technical city names in defaults/map/selectors/City/Build/AI/ruin cities through authorized naming, not arbitrary local relabeling. Preserve IDs separately; original civilization naming needs appropriate authority.
- [ ] Fix massive fog-frontier river pools: unknown neighbor is not known no-river. Separate actual lake/source/mouth from incomplete connectivity; use viewer-safe observed/remembered geometry, never hidden-cell requests or invented continuation.
- [ ] Diagnose slow End Turn through enqueue, worker/budget wait, command HTTP, server planning/AI resolution, snapshot and presentation timings. Check optional-read head-of-line blocking without blaming the server from screenshots alone.
- [ ] Restore the original colored acting-nation strip below turn/date if authorized phase/progress facts exist. Simultaneous planning is not an invented sequential AI turn; missing progress must remain explicit.
- [ ] Right-click terrain cancels unsent movement gesture/preview as original. Canceling an already retained server order requires its exact authorized command; no local execution or accidental mouse-up submission.
- [ ] Make edge scrolling usable through letterboxes/DPI/window edges; current inner-content strips and discarded border motion can stop panning. Preserve no map clicks in borders, source timing/modal/focus/bounds and continuous-hover behavior.
- [ ] Fix impossible land/ocean/foreign-city previews: retained destination lines bypass intermediate checks and known cost is not traversal permission. Distinguish actual ocean from river artwork; no fake green route/turn1 or locally executed path. Verify transport/bridge/domain exceptions only from current facts.
- [ ] Diagnose oil-on-plains/resource-terrain mismatch through exact terrain/good/catalog/art and generation/terraform/observation lineage. Original oil is desert GOOD_TWO/GG016; modified installations and four-suffix generator assumptions require careful attribution. Never hide/relocate authoritative resources to disguise a server defect.
- [ ] Investigate game55 early-turn resync logs (19:07:02 rev10/turn2 through19:08:04 rev28/turn5; first50s then~3s). Distinguish ordinary full reads logged as resync from genuine recovery/rollback; audit actual SSE hints/idle/disconnect/rate scheduling and running build. Do not merely suppress logs or disable required recovery. [Detailed local resync investigation](plans/local-game-resync-investigation.md).
- [ ] Confirm retail restart plus **new game** and healthy live behavior separately. Existing games keep pinned synthetic rules; explicit synthetic labels are not real retail content. Multiple crash-free turns is user-confirmed progress, not latency or full-play acceptance. [Live-play checklist](plans/immediate-live-play-recovery.md).

## Remaining detail checklists — complete index

Every remaining file under `plans/` must be linked directly here. Work below inherits the execution order above; read-only gap reports are not extra implementations or permission to stop.

| Detail/evidence | Scope |
|---|---|
| [Full-screen parity](plans/full-screen-reference-parity.md) | Source/capture methodology, cross-screen workstreams and comparison evidence; acceptance checkboxes live here in PLAN.md. |
| [Reference interaction fidelity](plans/reference-interaction-fidelity.md) | Detailed source map, managers/panels/mouse/workflow requirements and historical checkpoints. |
| [Science Manager](plans/science-manager-reference-parity.md) | Current UI screen, original callbacks/layout/chooser and reference24. |
| [Great Library](plans/great-library-workflows.md) | Remaining Library/Build/Science content/art/chart/editor/goal workflow and frame acceptance. |
| [Diplomacy Manager](plans/diplomacy-manager-reference-parity.md) | Remaining main/details/authoring/category/history/actions, source geometry and reference08. |
| [Original map input](plans/original-map-pointer-dispatch.md) | Original LEFT/RIGHT/Unit/City/None, contexts, garrisons and preferences. |
| [Original ruin text](plans/original-ruin-exploration-text.md) | Explicit user request, localized confirmed-reward variants and reference23. |
| [October22 follow-ups](plans/oct-22-2026-ui-followups.md) | All6 captures, city/river/performance/nation bar/cancel/panning/movement/goods requests. |
| [Local resync investigation](plans/local-game-resync-investigation.md) | Exact user logs, actual refresh/recovery ownership and cause-tagged diagnostics. |
| [Immediate live-play recovery](plans/immediate-live-play-recovery.md) | Retail/new-game and live stability confirmation still pending after tested client fixes. |
| [Remaining shell schisms](plans/fix-ui-schisms.md) | Still-unchecked original standard/grey/gold text and c3_PopupWindow chrome fidelity. |
| [Authority/art gap report](plans/reference-parity-contract-gaps.md) | Specific unprojected effective/private facts and asset/variant limits; no invented values or server edits. |

## Final correctness and delivery gates

- [ ] Route every remaining gameplay operation through exact current server commands and correct local/remote bootstrap/options; never regress to the historical client-generated world/AI/economy fallback. Multiplayer prerequisites/options and single-human local-server play remain authoritative.
- [ ] Complete reference-accurate city/goods/ruin/improvement/territory/labels and permitted action/event presentation. Original sprites are implemented, but visual/lifecycle acceptance and current reported bugs remain open.
- [ ] Obtain any demonstrated missing public/display/action contracts through server-maintainer work; keep unsupported controls and private/effective metrics unavailable. Diagnostic report: `$CTPSERVER/server-client-schisms.md` (documentation only).
- [ ] For every code slice: strict `cargo make lint`, full tests, >=80% full-target LLVM line coverage, dedicated native build, security/privacy/performance/source review and modules<800; `cargo make lint-fix` immediately before each normal commit; unchanged hooks, no bypasses.
- [ ] Report missing `./scripts/check-coverage-fast.sh` honestly (127); enforce the available LLVM80% gate instead, without creating/changing restricted scripts. Preserve native Linux support and complete Windows validation once Rust target/native GTK/GLib/GObject cross sysroot/pkg-config environment is available; no Windows success currently claimed.
- [ ] Final actual end-to-end play and all33 accepted reference scenarios, all accessible control journeys and remaining unchecked details; component tests alone do not satisfy this gate.

## Completed implementation ledger

Latest measured quality gate (Science picture lookup):1312 executions (637 library+655 binary+19 integration+1 example),85.66% LLVM; strict lint/full suite/dedicated native build pass. Previous Science source-control fixes are committed in4b02dbe (1294/85.65%); tracking consolidation is cac02df with unchanged hooks. These are component fixes, not whole Science or33-screen acceptance.

- [x] Launch/intro skip, menu/shell/setup/options/audio/input/viewport foundations, extraction/dialog fixes and related source-art regressions. Historical shell option state/persistence is not proof of all in-game effects.
- [x] Original terrain/SPR/RIM/resource/city decoding, fog/edge/seam fixes, actual viewer starter/projection decoding and menu-owned window handoff. No screenshot-completion claim.
- [x] Contractv2 integration: `04813e5`, `b12d801`, `5f3ebf1`, `31fcaa3`, `c557e58`, `ea98904`: typed unknown-safe facts, stable queues/commands, asynchronous immutable retries/receipts, retained destinations, forms and observed presentation/history.
- [x] Unit/Trade advisors and source controls (`cc8d812`, `c75912b`, `44dc654`); National (`e14c0ef`), Empire/details/government and footer (`bc1afcd`, `9a3f892`, `5533528`); complete-screen and missing effective/private facts remain open.
- [x] Diplomacy exact responses/main/details/review (`21d40df`, `1810eb1`, `614d3f6`, `9dbddc5`); Library item research-goal identity (`3c19f55`). Full authoring and screenshot workflows remain open.
- [x] Installed manifest marker launch repair `0b8841e` (user confirmed), selected-installation Times/Arial discovery `7fe2fe7` (no copies), revision/read-budget sync fixes `c1de63c`, historical-notice sync ownership `3df0793`; new local-resync/live symptoms remain open.
- [x] Presented-stack privacy/grouping `358c0ef`, source-pixel/current-revision picking and non-rebased gestures `a9c7dec`, source context popup correction `fbeb093`. **Correction retained:** ActivateSelectedInfo calls BuildList and opens a real popup; the earlier panel-only/no-popup inference in c2f748e was wrong. Current selected/eligible army rows precede known city/Map Info; unselected sheltered city fallback remains conditional.
- [x] Leader first-backspace fix is already committed in `229e3f5`; its old commit-only checkbox was stale. String-manifest final commit checkbox was likewise already satisfied in `0b8841e`.

### Consolidated open work (not marked completed)

The six redundant trackers were merged, not silently accepted: `full-game-parity-correction.md` (remaining server commands/art -> UI/final gates), `singleplayer-in-game.md` (remaining coverage/commit -> delivery gates), `maintainability-refactor.md` (coverage/environment -> delivery gates), `presented-unit-stack-identity.md` and `presented-actor-hit-testing.md` (remaining full stack/army/hover/right-click/keyboard -> UI checklist/original-map detail), `ui-first-reference-screen-priority.md` (entire priority order/reference/ruin/lower-follow-up/quality scope -> this file). Broader unaccepted frames, source behavior and all user requests remain unchecked above.

### Retired completed component plans

Files below had completed local checklists/commit evidence; implementation evidence is retained in Git, FIXES.txt, source/tests, and the ledger above. All51 quoted commits were independently verified to contain the exact retired file. Retrieve full prior text with `git show <commit>:plans/<filename>`. Historical plans may describe **superseded** approaches (generated frontier IDs, blocking/zero-resync policies, community layouts or synthetic fallback); current original-source/server-authority rules win. Removing these files does not certify the whole game or the33 screens.

| Retired filename | Evidence commit |
|---|---|
| authoritative-frontier-move-fix.md |60586d1; superseded by v2 public-coordinate intents, never reintroduce generated hidden IDs |
| authoritative-projection-move-fix.md |8169e26; superseded by v2 exploration/unknown-safe authority |
| client-contract-v2.md |3df0793; integration series above |
| ctp2-drag-move-input.md |d492634 |
| ctp2-movement-runtime-repair.md |2f189c0 |
| ctp2-movement-source-fidelity-pass.md |2c392a7 |
| ctp2-source-aligned-movement-repair.md |3de5355 |
| custom-map-new-game-fidelity.md |1c0e223 |
| dialog-continue-response.md |a71d2a0 |
| disable-options-and-prune-hub.md |b1d1248 |
| flat-world-original-description.md |a2fe577 |
| gameplay-viewport-reticle-repair.md |1319864 |
| historical-notice-sync-ownership.md |3df0793 |
| ingame-dropdown-parity.md |3927fb7 |
| in-game-interface-repair.md |20d7110 |
| installed-source-font-discovery.md |7fe2fe7 |
| intro-audio-options-completion.md |e6b75d8; shell foundations, not full game effects |
| intro-video-main-menu.md |0c03c5f |
| leader-backspace-single-character.md |229e3f5 |
| leader-name-editing-style-fix.md |5d17c9e |
| leader-textfield-native-edit-fidelity.md |59f0a47 |
| manager-runtime-geometry.md |b3fbbd8 |
| menu-resize-line-artifacts.md |5a3b8d6 |
| music-track-selection-and-repeat.md |0c2b3e6 |
| national-manager-reference-parity.md |e14c0ef; component only, frame18 still open |
| new-game-label-worldshape-alignment.md |0c2b3e6 |
| options-functionality-and-newgame-cleanup.md |4b69add |
| options-menu-fidelity.md |904e95a |
| options-right-side-disabled-only.md |689ca28 |
| options-runtime-effects.md |4b69add |
| original-actor-context-call-chain.md |fbeb093 |
| original-diplomatic-review.md |9dbddc5 |
| original-disabled-button-art.md |2649c3f |
| original-library-research-goals.md |3c19f55 |
| pixel-perfect-gameplay.md |53c77ec; component only, not33-frame parity |
| proportional-menu-viewport-scaling.md |22ee490 |
| random-music-click-latency-slider-sounds.md |35cad66 |
| resolution-scrollbar-warning-cleanup.md |ed5f1eb |
| resolution-selector-hitboxes.md |d81ef73 |
| server-startup-investigation.md |3e5877d; diagnosis only, no current startup guarantee |
| sharp-scaled-menu-viewport.md |fae2cbc |
| shell-fidelity-deep-pass.md |69e62d3 |
| shell-ui-fidelity-review.md |e3f5b14; remaining distinctions moved to UI/gaps |
| singleplayer-launch-crash-diagnosis.md |121083f |
| singleplayer-menu-system.md |7962e2b |
| starting-settlers-projection.md |4fa1e85 |
| string-manifest-launch-recovery.md |0b8841e |
| terrain-pixel-seams.md |644fe14 |
| ui-schisms-audit.md |8099f73; unresolved fixes still indexed above |
| world-shape-leader-input-fidelity.md |e039972 |
| zenity-dialog-fallback.md |4be7743 |

## Tracking cleanup and immediate continuation

- [x] Audit all69 client plan files, completed versus active versus redundant; preserve implementation evidence and all unresolved work, including stale commit-only closures. Do not remove server plans.
- [x] Retire51 implemented/completed component files, consolidate six redundant trackers into this main checklist, keep/index12 remaining detailed checklists/reports, migrate links and validate all33 unaccepted entries/no lost scope. Read-only validation reports12/12 indexed, zero broken Markdown links and33 unique unchecked references; historical evidence remains retrievable through the table above.
- [x] Run doc/link validation (12/12 indexed, no broken links, all33 unchecked), strict lint/full tests/LLVM80% gate/native build and pre-commit lint-fix; normal `cac02df` hooks passed. Keep requested server documentation separate from server implementation.
- [x] **Resume current Science Manager UI changes immediately** after cleanup; first four source-control fixes are verified and next unresolved work remains listed above. Continue its next source-backed RED test, UI wave and every unchecked detail; this checkbox is return-to-work, not screen/game completion.
