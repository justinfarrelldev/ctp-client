# Diplomacy Manager reference continuation

Parent: [full-screen-reference-parity.md](full-screen-reference-parity.md), after National `e14c0ef`. Execute client-only, source-driven TDD; no plan/approval gate. No source/asset/server/script/hook/lint-rule edits or copied assets. Tests use pure/in-memory fixtures, read-only installed art and accurate isolated gateway stubs.

## Audit and module boundary

- [x] Inspect the original Diplomacy reference, installed main/Intelligence item layouts and original selected-contact callback C++; cross-check public root/list/button reference geometry. Detailed panes remain to audit/implement.
- [x] Audit current viewer-safe contact/relation/proposal facts and exact server actions/envelopes. List missing public names/colors/portraits/foreign rankings/intelligence separately; no owner-ID-derived artwork or invented foreign history.
- [x] Split manager kind/action mapping out of the 798-line management root before further modules, preserving its stable public API and all current behavior. Keep the public enum declaration and a const forwarding function in place, with a single leaf implementation; no new pub-use lint exception is needed. Existing green tests protect this refactor; new behavior still starts RED.

## Implementation (red–green–refactor)

- [x] RED old 576x384/row-less main; GREEN source 601x558, 470x396 intelligence content, source header widths/six row columns and aligned y432 action row/Close. Cache intelligence.ldl item, preserving source 25px height. Newer dw_lb/dw_rb frames are absent: actual art wins when available; otherwise explicit reference-art-unavailable and installed original tiled columns keep the window usable, not parity-certified.
- [x] Current exact contact IDs and received regard/treaty/trade render in actual source columns; absent public nation name/flag/strength/embassy stay explicit ID/—. Receipt removes vanished contact selection, never picks its neighbor. Proposal/reply choices remain exact/explicit, not phantom execution.
- [ ] RED explicit known-contact proposal/response/war/embassy choices and revision/ownership/phase guards.
  - [x] Selected-contact proposal/reply workspace requires an actual current relation and filters exact counterparty actions, never the first contact.
  - [x] Current respond-agreement fields authorize either explicit boolean for the same proposal; reject malformed/null/extra/foreign IDs and fixed legacy mutations. Native source entry plus isolated exact HTTP verifies accept:false and no queued local treaty/proposal mutation.
  - [x] Distinguish Accept/Reject and trade/union labels; player IDs stay explicit IDs, not unrelated city-name substitutes.
  - [ ] Complete remaining original contact/detail/proposal panes and legal war/embassy/embargo controls; current metadata gaps remain explicit. Use exact current server payloads/targets, never the first candidate or a fabricated route.
- [ ] RED original relevant detail panes/navigation/return lifecycle, sorting, readable unavailable reasons, focus/keyboard and confirmation behavior; preserve selections and invalidate vanished/stale identities on snapshots.
- [ ] Accurate isolated HTTP/queued/rejection/resync regressions plus semantic/native interactions. Native frames stay in memory in tests; never call the live server.
- [x] Use explicit complete-v2 no-city/two-settler/uncontacted start rather than Build's queue; inspect native/reference-size diagnostic frames and dense full-frame report. Complete-reference acceptance remains unchecked (failed diagnostic), including missing assets/fonts/metadata.
- [ ] Document exact remaining server/assets/environment gaps in the active report and update PLAN/FIXES/parent/source evidence.

## Verification and atomic commits

- [ ] Run lint-fix before every commit; strict lint/full tests/dedicated native build; fix every failure and use unchanged normal hooks.
- [ ] Run the fast coverage script honestly (currently absent) and enforce >=80% LLVM full-target line coverage; all source files <800 lines.
- [ ] Review authority/privacy, immutable retry/revision boundaries, memory/CPU and no per-frame filesystem IO. Windows target/core and GTK/GLib cross sysroot remain unavailable until revalidated; original Times faces and National upic26/27 are missing, not accepted lookalikes.
- [ ] Commit coherent slices normally, never bypassing hooks. Keep server/original/assets trees unchanged.

Initial response slice evidence: server `visibility.diplomacyActions` advertises accept:true but includes the mutable typed accept field; engine `respondAgreement` explicitly supports false. Client fixed-payload equality had incorrectly blocked rejection. RED command/UI tests reproduced it; GREEN shape validation, same-proposal true/false choices, known-contact source-button routing, malformed/legacy/stale guards and actual isolated HTTP cover the fix. Public source main Diplomacy geometry is 601x558, list 470x396 and lower buttons y432; initially the installed old main still needed its geometry revision; the following geometry slice restores those bounded resolved properties. Do not present the response slice as completed Diplomacy/full-screen parity.

Response checkpoint: strict lint/full tests/dedicated native build pass; 573 library + 591 binary + 19 integration + 1 example = 1184 executions; 85.21% LLVM lines with >=80% enforced. Fast script absent (127). Original/server trees remain untouched/clean. Kind/action implementation is bounded in a leaf with unchanged public enum/API and no new lint exemptions. Committed response slice as 21d40df after lint-fix/normal hooks; continue the unchecked detail/proposal panes above. No screenshot acceptance is implied.

Tall-main/source-row continuation: RED root/list/row tests reproduced the old short (576x384,y108) dialog and missing Nation cell; GREEN uses current source geometry and six IntelligenceListItem columns. Main source-button contact workflows persist. New native opaque-window negative control caught lower HUD/map visible through chromakey side gaps; ctp2_Window owns an initially black opaque surface, not global SDL alpha transparency. Fill only its own clipped bounds (honor explicit transparent), then compose child art; native whole-root in-memory readback verifies zero magenta leaks and unchanged outside pixel. Installed reference extensions dw_lb/dw_rb are absent from both archives; source-specific tiled old-column fallback is marked, and injected present/missing-art tests preserve exact actual art when installed. Shared read-only index helper is construction-only; National keeps its existing optional icon handling. No server/original art changes or file-writing tests.

Diagnostics: images/parity-diplomacy-{native,reference}.png are explicit ignored manual captures, not test output. Decoration/menu-only origin calibration (5,23); full viewport comparison 97086/307200 pixels exceed channel delta8, raw RGB error10735316, identical:false. Dense worst cells identify unresolved side art; fonts/capture/footer and details/controls are not accepted. No interior masks/alternate favorable crop or screenshot certification.

Tall-main verification checkpoint: strict lint/full tests and independent dedicated native build pass; 578 library +596 binary +19 integration +1 example =1194 executions; 85.24% LLVM lines enforced >=80%. Combined verification shell timed out after full target tests/coverage summary; reran existing complete-profile line gate (exit0) and dedicated build independently (exit0), rather than claiming timeout success. Fast script remains absent(127); Windows target/core/sysroot gap unchanged. All Rust modules remain <800 lines and server remains clean. Run lint-fix/normal hooks before this coherent main-window commit; then continue unchecked detailed contact/proposal/navigation/confirmation work, not frame acceptance.

Then return to the parent and continue **all remaining** manager/panel/options/actor/normalization work and all 33 complete-frame acceptances. This plan is not a replacement definition of full-screen/game completion.
