# Diplomacy Manager reference continuation

Parent: [full-screen-reference-parity.md](full-screen-reference-parity.md), after National `e14c0ef`. Execute client-only, source-driven TDD; no plan/approval gate. No source/asset/server/script/hook/lint-rule edits or copied assets. Tests use pure/in-memory fixtures, read-only installed art and accurate isolated gateway stubs.

## Audit and module boundary

- [x] Inspect the original Diplomacy reference, installed main/Intelligence item layouts and original selected-contact callback C++; cross-check public root/list/button reference geometry. Detailed panes remain to audit/implement.
- [x] Audit current viewer-safe contact/relation/proposal facts and exact server actions/envelopes. List missing public names/colors/portraits/foreign rankings/intelligence separately; no owner-ID-derived artwork or invented foreign history.
- [x] Split manager kind/action mapping out of the 798-line management root before further modules, preserving its stable public API and all current behavior. Keep the public enum declaration and a const forwarding function in place, with a single leaf implementation; no new pub-use lint exception is needed. Existing green tests protect this refactor; new behavior still starts RED.

## Implementation (red–green–refactor)

- [ ] RED source geometry/tabs/native art/headers/row templates; restore reference revision from resolved installed assets, with honest missing-art status rather than wrong copies/substitutions.
- [ ] RED current contacts/relations/proposals rows and exact independent identities; render only received known names/relations/status, leaving foreign/private data unavailable.
- [ ] RED explicit known-contact proposal/response/war/embassy choices and revision/ownership/phase guards.
  - [x] Selected-contact proposal/reply workspace requires an actual current relation and filters exact counterparty actions, never the first contact.
  - [x] Current respond-agreement fields authorize either explicit boolean for the same proposal; reject malformed/null/extra/foreign IDs and fixed legacy mutations. Native source entry plus isolated exact HTTP verifies accept:false and no queued local treaty/proposal mutation.
  - [x] Distinguish Accept/Reject and trade/union labels; player IDs stay explicit IDs, not unrelated city-name substitutes.
  - [ ] Complete remaining original contact/detail/proposal panes and legal war/embassy/embargo controls; current metadata gaps remain explicit. Use exact current server payloads/targets, never the first candidate or a fabricated route.
- [ ] RED original relevant detail panes/navigation/return lifecycle, sorting, readable unavailable reasons, focus/keyboard and confirmation behavior; preserve selections and invalidate vanished/stale identities on snapshots.
- [ ] Accurate isolated HTTP/queued/rejection/resync regressions plus semantic/native interactions. Native frames stay in memory in tests; never call the live server.
- [ ] Explicit matching authorized diagnostic fixtures and native/reference-size inspection/full-frame report. Keep complete reference acceptance unchecked until actually matched, including unsupported/missing fields/art/font caveats.
- [ ] Document exact remaining server/assets/environment gaps in the active report and update PLAN/FIXES/parent/source evidence.

## Verification and atomic commits

- [ ] Run lint-fix before every commit; strict lint/full tests/dedicated native build; fix every failure and use unchanged normal hooks.
- [ ] Run the fast coverage script honestly (currently absent) and enforce >=80% LLVM full-target line coverage; all source files <800 lines.
- [ ] Review authority/privacy, immutable retry/revision boundaries, memory/CPU and no per-frame filesystem IO. Windows target/core and GTK/GLib cross sysroot remain unavailable until revalidated; original Times faces and National upic26/27 are missing, not accepted lookalikes.
- [ ] Commit coherent slices normally, never bypassing hooks. Keep server/original/assets trees unchanged.

Initial response slice evidence: server `visibility.diplomacyActions` advertises accept:true but includes the mutable typed accept field; engine `respondAgreement` explicitly supports false. Client fixed-payload equality had incorrectly blocked rejection. RED command/UI tests reproduced it; GREEN shape validation, same-proposal true/false choices, known-contact source-button routing, malformed/legacy/stale guards and actual isolated HTTP cover the fix. Public source main Diplomacy geometry is 601x558, list 470x396 and lower buttons y432; the installed old main still needs its geometry revision. Do not present the response slice as completed Diplomacy/full-screen parity.

Response checkpoint: strict lint/full tests/dedicated native build pass; 573 library + 591 binary + 19 integration + 1 example = 1184 executions; 85.21% LLVM lines with >=80% enforced. Fast script absent (127). Original/server trees remain untouched/clean. Kind/action implementation is bounded in a leaf with unchanged public enum/API and no new lint exemptions. Commit this coherent response fix after lint-fix/normal hooks, then continue the **unchecked original geometry/pane work** above. No screenshot acceptance is implied.

Then return to the parent and continue **all remaining** manager/panel/options/actor/normalization work and all 33 complete-frame acceptances. This plan is not a replacement definition of full-screen/game completion.
