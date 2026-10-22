# Full-screen CTP2 reference parity

Continuation of [reference-interaction-fidelity.md](reference-interaction-fidelity.md), explicitly requested after contract-v2 integration `ea98904`. This is implementation/acceptance work, not a plan-only gate. Keep the main menu intact. Server, original assets/source, hooks, scripts, build rules and lint rules remain read-only. Tests use in-memory frames and accurate injected HTTP/SSE only.

## Completion rules

- Check off implementation only with source anchors, RED failure, GREEN regression and review.
- Every reference requires a deterministic authorized fixture, production SDL output at native/reference size, semantic interaction coverage, complete-frame comparison and visual inspection. Corner masks/framework tests are insufficient.
- Missing authoritative values remain unavailable; never invent private foreign scores, actual culture, optimizer results, hidden geography or accepted combat outcomes to match a picture.
- Record genuine environment/contract blockers specifically; do not mark blocked acceptance complete or redefine it as success.
- Commit coherent slices after lint-fix, strict lint, tests, native build and normal unchanged hooks. Enforce >=80% LLVM coverage; record the absent fast script and exact Windows validation outcome.

## Workstreams

- [ ] Re-audit all 33 references/current output and original source; build source-backed scenario/geometry/interaction manifests without copying reference pixels into production or shipping assets.
- [ ] Full-frame viewer-fixture rendering and in-memory comparison/oracle, decoration crop normalization, native/reference-size checks and negative controls.
- [ ] Source manager lifecycle and attached advisor windows; exact root/tab/column/control geometry and artwork, empty/populated/unknown values.
- [ ] Complete National resources/status/specialists; Unit statistics/tactical/readiness/advice/disband/army; Trade tabs/filters/goods count/accounting/advice/routes; Diplomacy contacts/proposals/intelligence.
- [ ] Complete Build/City/Empire/Science/Library controls and all source-created panes, dropdowns/rangers, progress/descriptions/goals, valid selections and stale-command guards.
- [ ] Complete all five panels, stack portraits/herald, source messages/date/eye/result windows and settlement radius.
- [ ] Complete context/map-info styling, observed goods/yields and adversarial remembered/hidden rendering.
- [ ] Finish accepted unit action/death/fortification presentation, moving-stack hit testing, terrain-cost movement/city/depletion previews and bounded camera feedback. Unsupported wrap never creates invented topology.
- [ ] Bind all Graphics/Sound controls to actual presentation/audio and verify handoff through injected effects; preserve source art/fonts and modal interactions.
- [ ] Statistics graph/legend/selector/zero-sum behavior within viewer privacy; explicitly unavailable foreign metrics cannot become zeros.
- [ ] Semantic end-to-end interactions cover every visible enabled control, return navigation, keyboard/focus/IME/clipboard, confirmations and disabled reasons.
- [ ] Audit each unavailable original operation against current server routes/capabilities and document exact remaining contract/environment limitations without changing the server.
- [ ] Final full-suite/lint/coverage/native build/Windows check, source/privacy/performance review and normal atomic commits; reconcile parent and component checklists only on actual evidence.

## Per-reference complete-frame acceptance

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

## Evidence ledger

Baseline: 514 library + 532 binary + 19 integration = 1065 executions; 84.27% LLVM lines. Native build/lints/hooks pass. Missing fast coverage script; Windows GNU target and GTK/GLib cross sysroot unavailable at the preceding checkpoint. No complete-reference acceptance is inherited from that checkpoint.

First slice: original attached Trade/Unit advisor lifecycle, source geometry and live own-only data. References 28/29 showed full left advice panes missing from the renderer. RED tests reproduced missing default panes, callbacks and collapsed list columns; GREEN source-native 224x350 panes dock at main.X-width/main.Y, toggle independently and close with their manager. Cached source styles/artwork and sequential header-width columns are shared with semantic/native interactions. Own-type counts exclude foreign units; outbound trade totals exclude inbound routes. Native readback compares all 224x30 roof pixels to independently loaded UPfg400 and verifies hiding erases the pane. Upkeep toggles never confuse gold maintenance with military production support; explicit zero support remains zero, not the original misleading 100%. Accurate missing planner/support fields are recorded in [reference-parity-contract-gaps.md](reference-parity-contract-gaps.md), not fabricated to match text. Full scenarios 28/29 remain unchecked pending their remaining main-pane controls and complete-frame acceptance.

Advisor checkpoint: strict lint, full suite (519 library + 537 binary + 19 integration = 1075 executions), native build and 84.38% LLVM line coverage pass. The fast script is still absent. Source/asset/server trees are unchanged; new tests retain frames in memory. Commit normally after lint-fix, then continue the full-frame scenario harness and remaining manager controls; this checkpoint is not task completion.

Unit/localization slice: RED exposed missing statistics/tactical columns, source readiness thumb/commands, independent type-selection invalidation, native icon offsets, unbounded total textbox and centered/clipped HyperTextBox content. Implemented source item columns, SmallIcon/native fallback, public base stats and own counts, actual readiness label/quantized thumb, per-actor tactical orders/location/health and explicit disband confirmation/target choice. Isolated HTTP verifies set-readiness's exact envelope and that a queued receipt keeps the received War state, not requested Peace. Source UnitManager::DrawHealthBar insets/partial yellow/black pixels have native readback. Public [unitmanager.ldl](https://raw.githubusercontent.com/civctp2/civctp2/master/ctp2_data/default/uidata/layouts/unitmanager.ldl) supplies 454x200 panes, header widths and native icon offsets; these are resolved-property adaptations, not copied assets.

The source-compatible two-settler Unit diagnostic is separate from Build's city/queue fixture. Native/reference-sized outputs were explicitly generated and inspected (`images/parity-unit-{native,reference}.png`, ignored). Inspection reproduced raw advisor title keys and text clipping. Actual `english/gamedata/Strings.txt` imports add_str and all other active string databases; the bounded injected import reader now restores them, with source `StringDB::InsertStr` first-base-definition behavior. The previous Library-title test incorrectly assumed the addon string was missing; it now verifies the actual canonical title/spacing. Dynamic recommendations and nonzero applied support remain unavailable for the precise reasons in the gap report. Remaining Unit work includes full type-wide batch/army composition interactions and complete-frame acceptance; no scenario is checked off from these component tests.

Unit/localization checkpoint: strict lint/full tests/native build pass; 530 library + 548 binary + 19 integration = 1097 executions; 84.53% LLVM lines with the >=80% threshold enforced. The fast script remains absent. All Rust modules remain below 800 lines. Commit after lint-fix with unchanged hooks, then continue complete-frame comparisons and remaining managers. No complete-reference acceptance is claimed.

After each coherent slice continue the next unchecked workstream and all remaining parent/component acceptance items; a partial commit is not task completion.
