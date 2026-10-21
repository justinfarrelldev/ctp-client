# Plan for Intro Video Skip Feature

## Objectives

- Ensure players can skip intro videos using keyboard, mouse, or controller input.
- Maintain accessibility by providing immediate, low-effort skip controls.
- Preserve compatibility with existing video playback pipeline and error handling.

## Checklist

- [x] Analyze current video playback implementation and process spawning behavior.
- [x] Design input handling architecture that works alongside external video player processes.
- [x] Define precise skip triggers for keyboard (Space/Enter/Escape), mouse (primary click), and controller (A/B/Start) interactions.
- [x] Establish clear communication channel between input handler and video process (e.g., signals or IPC) for clean termination.
- [x] Draft automated tests covering keyboard, mouse, and controller skip scenarios, plus non-skip completion path.
- [x] Implement mockable abstractions for process management and input polling to enable TDD.
- [x] Implement runtime input handling loop that monitors user input while video plays.
- [x] Integrate controller support via SDL2 with robust initialization/teardown and error reporting.
- [x] Ensure accessibility messaging/logging for skip functionality.
- [x] Run `cargo make lint` and address all findings.
- [x] Run `cargo make test` and confirm all tests pass.
- [x] Run `./scripts/check-coverage-fast.sh` to verify coverage threshold (script missing in repo; documented in status report).
- [x] Perform final code review for performance and maintainability.
- [x] Update documentation/comments reflecting new skip controls.
- [x] Prepare commit after running `cargo make lint-fix`.

## Post-review follow-up

- [x] Split `src/video.rs` into focused submodules to keep each file under 1000 LOC.
- [x] Expand overlay tests to cover controller instructions in the window title and bitmap rendering integrity.
- [x] Resolve outstanding review issues: controller skip pattern, missing error string, joystick subsystem documentation, startup controller removal handling, layout constant documentation, and error context naming consistency.

## 2025-10-05 Update

- [x] Pre-initialize the SDL skip overlay before spawning video players to eliminate `BadAccess` failures on fullscreen backends.
- [x] Enable keyboard and mouse grabs so Space/Esc/click/controller Guide skip shortcuts operate while the external player window has focus.
- [x] Verified intro playback orchestration cleanly hands control back to the main menu workflow after skips or completions.

## 2025-10-06 Global Input Tracking

- [x] Prototype replacement for overlay-dependent skip handling with a global input tracker abstraction.
- [x] Implement device-query backed provider and unit tests for keyboard, mouse, and steady-state behavior.
- [x] Validate SkipEventListener wiring with global tracker via focused tests.
- [x] Refresh accessibility and behavior documentation to note direct video window interaction.
- [x] Re-run formatting, lint, tests, coverage, and build targets before finalizing changes.

## 2025-10-07 Asset Extraction Pipeline

- [x] Audit existing installation and Play Game workflow to determine integration points for archive extraction.
- [x] Select cross-platform user data directory (AppData equivalent) and document rationale in code comments.
- [x] Enumerate all ZFS archives across bundled installations and verify availability before extraction.
- [x] Document archive purposes and hierarchy in `ZFS_ASSET_NOTES.md` alongside this plan.
- [x] Design and land initial test coverage for validation, extraction success, and missing archive errors.
- [x] Expand tests for manifest persistence, disk space failures, and progress callbacks.
- [x] Finalize extraction pipeline refactor (remove legacy backend trait usages, stabilise summary struct consumers).
- [x] Integrate extraction invocation into Play Game action with sensible progress reporting and error messaging.
- [x] Run lint, tests, coverage (or document missing script), and platform build targets.
  - `cargo make lint` passed.
  - `cargo make test` passed.
  - `./scripts/check-coverage-fast.sh` is not present in this repository.
  - `cargo build` passed; `cargo make build-all` was attempted and is blocked by missing i686 GLib/pkg-config cross-compilation sysroot configuration on this host.

## CTP2 Terrain Pixel Seam Repair

See [the detailed plan and red/green evidence](plans/terrain-pixel-seams.md).

- [x] Reproduce tile-edge damage with actual SDL pixel readback on deterministic application-generated maps using installed original art (read-only).
- [x] Fix rectangular fog spill, zoomed edge culling, and undersized fallback ground; keep mountain/forest overhang transparency outside the flat-ground contract.
- [x] Test all four grassland/plains/water edges, full ground interiors, zoom 1-3, generated and mixed fog, and single-pixel negative controls. No snapshots, copied assets, live server, or file-mutating new tests.
- [x] Run strict lint, full tests, dedicated build, coverage measurement and final review. Terrain renderer line coverage: 93.88%; overall coverage: 76.56% (below the repository-wide 80% target, principally unrelated GTK/video paths). Required fast coverage script remains absent.
- [x] Conduct final review, update documentation, and prepare commit after running `cargo make lint-fix`.

## CTP2 Original-Art Gameplay Fidelity Pass

See [the detailed checklist and red/green evidence](plans/pixel-perfect-gameplay.md) and [snapshot usage and remaining limitations](GAMEPLAY_SNAPSHOTS.md).

- [x] Compose the original 800×600 logical frame, full terrain transitions/hats/rivers, native actors/reticles, layered controls, dropdowns and radar chrome; leave the main menu intact.
- [x] Provide offline previews and F12 capture with an isolated file sink; all new screenshot/PNG tests stay in memory and game-server calls remain mocked.
- [x] Fix stacked-unit identity/ownership, hidden order hit boxes, no-selection order appearance, empty authoritative projections and local fog revealing unknown geography.
- [x] Accept optional authoritative resource IDs and render original resource SPR actors with visibility-safe remembered shading; never invent missing server metadata.
- [x] Add source-verified river/resource/ownership omissions and visibility-safe recommendations only to the explicitly authorized server report; no server implementation changes.
- [x] Inspect 640×480, native 800×600 and larger 1280×960 captures. Document synthetic sightlines and remaining radar/dialog/stat/workflow limitations without claiming all-screen pixel parity.
- [x] `cargo make lint-fix`, strict lint, full tests and dedicated native build pass. LLVM line coverage: **80.24%**. Required fast coverage script is absent; Windows binary validation remains environment-dependent.
- [x] Review restricted paths, asset copyright, file sizes and authority boundaries; prepare normal commit with verification enabled.

## Missing Starting Settlers and Visibility

See [source evidence and red/green regressions](plans/starting-settlers-projection.md).

- [x] Verify server starter creation, owner/viewer mapping, fog refresh and SSE wire format read-only; confirm the running server uses its default synthetic rules.
- [x] Reproduce and fix client loss of the nested resync snapshot, Go null-slice rejection and misclassification of opaque settler-capable starters. Do not invent local units/terrain or modify the server.
- [x] Correct bootstrap mocks to match the actual envelope. Test both default synthetic and retail starter profiles through decoding, selection, command identity, centered camera, actual actor/terrain/radar rendering and semantic Move/Settle controls, all in memory.
- [x] Strict lint, full tests (308 library, 326 binary, 19 integration), native build and LLVM line coverage **80.35%** pass. Required fast coverage script remains absent; no Windows binary validation claimed.
- [x] Document fixes/review, run pre-commit lint-fix and commit with normal verification enabled.

## CTP2 Reference Interaction Pass (partial; full request remains open)

See [the 33-image context inventory, source evidence and unfinished checklist](plans/reference-interaction-fidelity.md).

- [x] View all references individually and preserve filename context; identify missing manager/context/city/message workflows rather than claiming parity.
- [x] Implement original-art gameplay cursors/hotspots and clock-driven edge scrolling, with focus/modal cancellation and stationary-pointer path refresh.
- [x] Present accepted submitted movement using directional SPR frames and one travel/frame clock for the currently supported Settler/Warrior/barbarian art. Preserve shell speed/animation settings and do not consume animations during network waits.
- [x] Reproduce/fix foreign actors rendered with missing tile facts; test hidden/stale/visible/missing cases using real SDL pixel readback.
- [x] Add fileless visual regressions for original cursor pixels and eight directions × four animation phases, plus deterministic opt-in motion previews. Keep the main-menu layout intact.
- [x] Strict lint, 326 library/344 binary/19 integration tests and native build pass; LLVM line coverage **80.88%**. Required coverage script is absent. Windows cross-check attempted, blocked by GIO/GLib cross-compilation pkg-config/sysroot.
- [ ] Complete all remaining manager/dialog/control-panel/context workflows, action animation and source/server metadata integration. The full installed unit-art catalog is now loaded. Optimize Sliders' Apolyton callback/evaluator has been located; the authoritative counterfactual evaluation contract is not established.
- [ ] Certify full-reference pixel parity and complete accessible click workflows only after that remaining work is implemented and validated.

## Reference Interaction Continuation (verified milestone, not completion)

The full, expanded remaining checklist is maintained in [reference-interaction-fidelity.md](plans/reference-interaction-fidelity.md). Source/parser/frame tests do not replace 33-scenario acceptance.

- [x] Preserve viewer-safe legal actions, movement planning, rules/research/reports and projected facts; separate unsupported gateway actions from executable orders.
- [x] Implement explicit city/unit context selection/cancel and terrain information; stop issuing a hidden first action on right-click.
- [x] Resolve all 72 installed unit records through retail `newsprite.txt`; cover Hoplite/Archer directional output, fortification overlays, Windows-1252 labels and original bad-frame fallback.
- [x] Add cost-aware routing/turn-marker accounting, expanded army-deduplicated portrait selection, authoritative partial-health colors and actual firepower values. Full retained multi-turn orders/actions remain unchecked in the detailed plan.
- [x] Parse installed LDL inheritance/draw order, localized labels, original centering and intrinsic image dimensions; compose fourteen original manager/dialog roots in native SDL.
- [x] Share manager control descriptors with accessible activation and exclusive input handling; preserve independent build/queue selections and reject stale confirmation payloads.
- [x] Bind/test exact Add, Rush Buy, research selection and server-recommended founding payloads using isolated mock HTTP receipts. Automatically open the accepted new city's Build Manager once; fix distinct National/Trade and Statistics tab entry points.
- [x] Validate strict lint, 364 library/382 binary/19 integration test executions, dedicated native build and **81.55%** LLVM line coverage. The fast script remains absent; Windows cross-check stops in GObject/GLib pkg-config sysroot setup before client compilation.
- [ ] Finish remaining client-side policy/government/specialist/army/trade/diplomacy/library/statistics/options workflows, lower-panel bodies, notifications, city rendering, event/action animation and all visual acceptance. These are not all server-blocked.
- [ ] Reconcile Apolyton-specific layout additions and runtime-created controls with references; verify complete semantic journeys and all 33 native/reference-sized captures.
- [ ] Resolve demonstrated server contract blockers through server-maintainer work; do not modify the server implementation or emulate destructive queue edits locally. See `$CTPSERVER/client-discovered-server-issues.md` (documentation commit `5ed6237`).

## Great Library browsing continuation (not full reference acceptance)

See [the component checklist](plans/great-library-workflows.md); continue afterward with [the full remaining plan](plans/reference-interaction-fidelity.md).

- [x] Read installed articles and active `uniticon.txt` metadata; preserve sections, safe internal links and non-executable variables. Fix shared scanner confusion between quoted braces and record delimiters with a failing-then-passing in-memory regression.
- [x] Implement explicit cached global search, local category indexes, bounded article history, Gameplay/Historical bodies and exact related-window return. Test production input/semantic dispatch without contacting a running server or altering the authoritative snapshot.
- [x] Keep buildable/queue selections independent while the last interaction controls source descriptions/library links. Use current projected numbers only for verified expressions; unknown mappings remain unavailable.
- [x] Reproduce/fix extreme scrolling, missing source portrait stretch and oversized title-art overdraw. Test original Library/Science/Build picture pixels, shared link rectangles/underlines, white edit fields, and SDL Unicode/IME state restoration entirely in memory.
- [x] Inspect native and reference-sized Library captures. Fix the obsolete negative-position caption, missing uppercase title key and source pattern subrectangles without copying installed assets or modifying the main menu/server.
- [x] Strict lint/lint-fix, 381 library + 399 binary + 19 integration test executions, native build and **82.09%** LLVM line coverage pass. Required fast coverage script is absent; repeated Windows MSVC validation remains blocked in GTK native dependencies by cross-compiling pkg-config/sysroots.
- [x] Implement projected advance dependency chart/recentering, source borders/connectors, overflow pagination and a real center-article callback.
- [x] Add source-created sort/index/rangers, UTF-8 editing/selection/word/delete/undo, injected clipboard shortcuts, AltGr safety, measured pointer/IME presentation and bounded range capture.
- [x] Add verified runtime expression mappings, session-owned source fonts/metrics, original LDL first-template/child precedence and tail-to-head painting, source justification/shadows and title overhang.
- [x] Inspect updated populated Build/Science and Library/chart diagnostics at native/reference sizes; keep empty fixtures empty and preserve read-only artwork/server boundaries.
- [ ] Complete exact Library/Build/Science art/text/layout acceptance, including font availability, picture framing, data/progress columns and source-created advisor/runtime details.
- [ ] Obtain authoritative persistent research-goal support before enabling Set Goal; immediate set-research is not equivalent and the server remains read-only.
- [ ] Complete the rest of the parent plan, every manager/panel/animation workflow and all 33 reference acceptances. Neither browsing nor chart/editor implementation is completion of the original full request.
- [x] Repeat lint-fix/strict lint, full tests (**410 library + 428 binary + 19 integration = 857 executions**), dedicated native build and LLVM coverage (**82.91%**); inspect read-only/source/authority boundaries and prepare a normal verified commit. Required fast script remains absent; Windows checking stops in GTK/GLib cross-compilation configuration before client compilation.

## Main-plan runtime list/row checkpoint

The completion target remains [the entire reference interaction plan](plans/reference-interaction-fidelity.md), not a component checklist.

- [x] Implement original runtime list frame expansion with distinct content/header/ranger geometry, content-based row capacity and no phantom zero-sized source/SDL controls.
- [x] Cache/render installed Build/Science/Research item templates, real verified numeric columns and bounded cell clipping; retain exact parent identity for accessible/pointer activation.
- [x] Separate Science's viewer-known discovery table from the research candidate picker; sort by current projected costs, draw only verified own-player knowledge blocks and never infer foreign discoveries.
- [x] Inspect new populated native/reference-size diagnostic captures; run lint-fix/strict lint, **419 library + 437 binary + 19 integration = 875 executions**, native build and **83.15%** LLVM coverage. Windows/fast-script blockers are unchanged.
- [x] Integrate source Build/City city selectors, single captions, five-part dropdown art and explicit arrow layer flags; support exact-ID previous/next/popup/pointer/keyboard/scroll/cancellation and reject stale choices/click-through.
- [x] Verify new-city production metadata/session/revision/receipt handoff against an isolated HTTP stub. Inspect separate two-city open/closed diagnostic captures. Lint-fix/strict lint, **432 library + 450 binary + 19 integration = 901 executions**, native build and **83.43%** LLVM coverage pass.
- [x] Bind original five-level rations/workday/wages sliders and science/PW spinners to verified typed, bounded parameter schemas; preserve companion values, captured revisions, native source thumb/groove/arrow artwork, preview-only capture and receipt-owned displays. Repair the original workday pre-inversion comparison bug.
- [x] Reproduce stale scalar-intent rebasing and reject it before submission. Validate exact policy route/envelope/payloads with isolated HTTP receipts, including requested science60 versus received/displayed57. Inspect separate explicit `empire-populated` native/reference captures without accepting the mismatched retail/Apolyton layout.
- [x] Repeat lint-fix/strict lint, **445 library + 463 binary + 19 integration = 927 executions**, dedicated native build and **83.79%** LLVM line coverage (41,530 lines; 6,731 missed). Required fast script and Windows cross-compile environment blockers remain unchanged.
- [ ] Continue government comparison/enactment, Empire layout/lifecycle reconciliation, remaining authoritative manager/control-panel operations and every unchecked main-plan acceptance. Checkpoint commits are not a stopping condition.

## UI-first continuation and launch encoding blocker

Continue using [the existing reference plan](plans/reference-interaction-fidelity.md); no replacement plan was created.

- [x] Finish the incoming Empire government/Details lifecycle checkpoint without substituting National Manager for Details.
- [x] Restore full-screen Empire columns/header spinners and the supplied six-row Graphics/two-slider presentation; fix atomic Sound sliders, paired ticks, runtime Close and obsolete background overdraw. Native artwork tests and bounded reference-PNG corner masks pass; these are not whole-screen parity acceptance.
- [x] Restore source Empire/City/Msgs/Tile lower-panel bodies, five locally selectable tile categories, viewer-only labels and real progress portraits. Fix clipped short button caps, iconic layer offsets and reversed Unit firepower/armor positions. Inspect explicit empty/populated captures.
- [x] Reproduce the reported `Units.txt` launch failure using injected non-UTF-8 bytes. Fix the client reader, not original data; apply the shared Windows-1252/UTF-8 decoder to unit/resource/sound inputs and preserve real I/O failures.
- [x] Strict lint and full tests pass (**468 library + 486 binary + 19 integration = 973 executions**), dedicated native build passes, LLVM overall line coverage is **84.51%**. Required fast script remains absent; Windows MSVC checking is blocked before client compilation by native dependency cross-pkg-config/sysroot configuration.
- [ ] Complete remaining manager/advisor, message/popups, city/map UI and all 33 complete reference-scenario acceptances. Most/all UI has **not** been certified identical; follow the unchecked main-plan items rather than interpreting component passes as completion.

## Reported movement, city and cursor defects

- [x] Reproduce and fix the 24-native-pixel path-anchor offset, restore thin source path lines and zoom-scaled turn boxes, and verify real SDL line pixels.
- [x] Reproduce SDL 1.2x pointer450→449 truncation; share an integer window/drawable/native transform, preserve aspect-ratio placement and reassert hidden OS cursor state. The screenshot's cropped bottom was user-caused, not a layout defect.
- [x] Replace blue city rectangles with original cached GC sprites, native hotspots/alpha and remembered shading. Read installed style/age/size recipes; test every referenced city group and adversarial fog cases. Use projected names/queue heads, never machine IDs or invented Warriors, and inspect the native/reference-sized city capture.
- [x] Validate strict lint, **480 library + 498 binary + 19 integration = 997 test executions**, native build and **84.65%** LLVM line coverage. Windows checking is still blocked in GTK/GLib dependency cross-compilation setup, and the required fast coverage script remains absent.
- [x] Resolve authoritative terrain-details caching and phase/End Turn synchronization through the independently implemented server contract v2 below. The complete reference checklist remains open; the historical server-edit blocker is not the current feature status. This client integration changes no server files.

## Client reconciliation with server contract v2

See [the comprehensive child checklist and verification ledger](plans/client-contract-v2.md), continuing—not replacing—[the reference interaction plan](plans/reference-interaction-fidelity.md).

- [x] Preserve typed optional viewer observations, public bounds/calendar, quotes/forecasts, city art facts/governors, goals, retained movement, semantic notices and own history without inventing legacy zeroes or geography.
- [x] Use advertised phase-aware End Turn, current registered routes, parameter validation and atomic stable-entry queue operations; bind rename/disband, Sleep/Wake, governors, goals and explicit legal target forms.
- [x] Move gameplay commands/events off SDL, retain retry identity after lost/truncated responses, accept only monotonic complete safe snapshots, and distinguish Pending/Resynchronizing/queued intent from execution.
- [x] Restore and cancel server-retained journeys, submit public coordinates without fake tile IDs, keep provisional motion separate from canonical units, and reconcile only known submitted/retained paths. Constrain non-wrapped public camera bounds without discovery.
- [x] Consume cached inspection/ownership, original ruin/improvement/city age art, unaffordable quotes/forecasts/calendar and trade accounting. Bind bounded read/unread messages, source ranger/popups/eye navigation and actual own-history graphs; retain independent entry/content selections.
- [x] Full suite: **514 library + 532 binary + 19 integration = 1065 executions**; native build passes; LLVM line coverage **84.27%** with the 80% threshold enforced. The fast script is absent. Windows GNU checking was attempted but is blocked by its missing Rust target and GTK/GLib/GObject cross-pkg-config/sysroot before client compilation; no Windows success is claimed.
- [x] Finish strict lint/lint-fix and source/privacy/concurrency review; stage the presentation slice for a normal verified commit. Record evidence in the child ledger; unchanged commit hooks must pass.
- [ ] Continue the parent plan's remaining source/reference reconciliation. This integration does not certify complete-game deployment, exact missing culture/optimizer/foreign-history contracts, moving-stack/attack/death behavior or all 33 reference screenshots.
